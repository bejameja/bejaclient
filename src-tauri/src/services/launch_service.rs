//! Port of `src/main/services/launchService.ts`.

use crate::mcinstall::{
    java_runtime, launch_args::{self, ArgContext, AuthCtx, ResolutionCtx},
    libraries, natives, platform, version_resolve,
};
use crate::paths;
use crate::services::{auth_service, options_service, profile_service, settings_service, version_service};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use regex::Regex;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

const G1GC_FLAGS: &[&str] = &[
    "-XX:+UseG1GC",
    "-XX:+ParallelRefProcEnabled",
    "-XX:MaxGCPauseMillis=200",
    "-XX:+UnlockExperimentalVMOptions",
    "-XX:+DisableExplicitGC",
    "-XX:+AlwaysPreTouch",
    "-XX:G1NewSizePercent=30",
    "-XX:G1MaxNewSizePercent=40",
    "-XX:G1HeapRegionSize=8M",
    "-XX:G1ReservePercent=20",
    "-XX:G1HeapWastePercent=5",
    "-XX:G1MixedGCCountTarget=4",
    "-XX:InitiatingHeapOccupancyPercent=15",
    "-XX:G1MixedGCLiveThresholdPercent=90",
    "-XX:G1RSetUpdatingPauseTimePercent=5",
    "-XX:SurvivorRatio=32",
    "-XX:+PerfDisableSharedMem",
    "-XX:MaxTenuringThreshold=1",
];

const ZGC_FLAGS: &[&str] = &["-XX:+UseZGC", "-XX:+DisableExplicitGC", "-XX:+AlwaysPreTouch", "-XX:+UnlockExperimentalVMOptions"];

const COMMON_FLAGS: &[&str] = &[
    "-XX:+OptimizeStringConcat",
    "-XX:+UseStringDeduplication",
    "-Xshare:off", // suppress OpenJDK CDS bootstrap warning
    "-Djava.rmi.server.useCodebaseOnly=true",
];

fn has_gc_flag(args: &[String]) -> bool {
    let re = Regex::new(r"Use(ZGC|G1GC|ShenandoahGC|SerialGC|ParallelGC|CMS)").unwrap();
    args.iter().any(|a| re.is_match(a))
}

fn mc_minor_version(version: &str) -> u32 {
    version.split('.').nth(1).and_then(|s| s.parse().ok()).unwrap_or(0)
}


fn build_jvm_args_for_profile(profile: &profile_service::LaunchProfile) -> Vec<String> {
    let user_args: Vec<String> = profile.jvm_args.split(' ').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
    let mc_minor = mc_minor_version(&profile.version);

    let gc_flags: Vec<String> = if has_gc_flag(&user_args) {
        vec![]
    } else if mc_minor >= 17 {
        ZGC_FLAGS.iter().map(|s| s.to_string()).collect()
    } else {
        G1GC_FLAGS.iter().map(|s| s.to_string()).collect()
    };

    let mut javaagent = Vec::new();
    if profile.use_beja_client {
        if let Some(jar) = crate::services::client_update_service::resolve_bootstrap_jar() {
            javaagent.push(format!("-javaagent:{}={}", jar.to_string_lossy(), profile.version));
            log::info!("[BejaBootstrap] Injecting: {} (MC {})", jar.to_string_lossy(), profile.version);
        } else {
            log::warn!("[BejaBootstrap] Bootstrap JAR not found — client will launch without BejaClient.");
        }
    }

    // Order: agent → GC → common → user (user args can override anything)
    let mut out = javaagent;
    out.extend(gc_flags);
    out.extend(COMMON_FLAGS.iter().map(|s| s.to_string()));
    out.extend(user_args);
    out
}

fn resolve_version_id(mc_version: &str, loader: &str, game_dir: &Path) -> String {
    if loader == "vanilla" {
        return mc_version.to_string();
    }
    let versions_dir = game_dir.join("versions");
    let Ok(entries) = std::fs::read_dir(&versions_dir) else { return mc_version.to_string() };
    let loader_key = if loader == "neoforge" { "neoforge" } else { loader }.to_lowercase();
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str() {
            if name.contains(mc_version) && name.to_lowercase().contains(&loader_key) {
                return name.to_string();
            }
        }
    }
    mc_version.to_string()
}

async fn resolve_game_java_path(
    client: &reqwest::Client,
    profile: &profile_service::LaunchProfile,
    on_log: &(impl Fn(String) + Sync),
    known_component: Option<&str>,
) -> String {
    let settings = settings_service::get_settings();
    let override_path = if !profile.java_path.is_empty() {
        profile.java_path.clone()
    } else {
        settings.game.default_java_path.clone()
    };
    if !override_path.is_empty() {
        return override_path;
    }

    let component = known_component.map(|s| s.to_string()).unwrap_or_else(|| java_runtime::guess_runtime_component(&profile.version));
    let runtimes_dir = paths::runtimes_dir();
    match java_runtime::ensure_java_runtime(client, &component, &runtimes_dir, |l| on_log(l)).await {
        Ok(Some(bin)) => return bin.to_string_lossy().to_string(),
        Ok(None) => on_log(format!("[Java] No managed runtime available for {component} on this platform — falling back to system Java.")),
        Err(e) => on_log(format!("[Java] Failed to fetch managed runtime ({component}): {e} — falling back to system Java.")),
    }
    "java".to_string()
}

struct RunningGame {
    child: Child,
    started_at: Instant,
}

static ACTIVE: Lazy<Mutex<Option<RunningGame>>> = Lazy::new(|| Mutex::new(None));
static KILL_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn is_running() -> bool {
    ACTIVE.lock().is_some()
}

pub fn kill_game() -> bool {
    let mut guard = ACTIVE.lock();
    if let Some(running) = guard.as_mut() {
        KILL_REQUESTED.store(true, Ordering::SeqCst);
        let _ = running.child.start_kill();
        *guard = None;
        return true;
    }
    false
}

pub async fn launch_game(app: AppHandle, profile_id: &str) -> Result<(), String> {
    if is_running() {
        return Err("Game is already running".to_string());
    }

    let profile = profile_service::get_profile(profile_id).ok_or_else(|| format!("Profile {profile_id} not found"))?;

    let client = auth_service::http_client();
    let app_log = app.clone();
    let on_log = move |line: String| {
        let _ = app_log.emit("launch:log", line.clone());
        crate::console_window::send_console_log(&app_log, &line);
    };
    let app_status = app.clone();
    let on_status = move |status: &str| {
        let _ = app_status.emit("launch:status", status);
        crate::console_window::send_console_status(&app_status, status);
    };

    let mut account = auth_service::get_selected_account().ok_or("No account selected. Please log in first.")?;
    if account.token_expiry < chrono::Utc::now().timestamp_millis() + 60_000 {
        on_status("Refreshing authentication...");
        match auth_service::refresh_account(&account.id).await {
            Ok(Some(refreshed)) => account = refreshed,
            Ok(None) => {}
            Err(e) => return Err(format!("Token refresh failed — please log out and sign in again. ({e})")),
        }
    }

    let settings = settings_service::get_settings();
    let game_dir = if !profile.game_dir.is_empty() { Path::new(&profile.game_dir).to_path_buf() } else { paths::default_game_dir() };
    let mods_dir = game_dir.join("mods");

    on_log(format!(
        "[Launcher] Profile: {} | {} | {} | BejaClient: {}",
        profile.name, profile.version, profile.loader, profile.use_beja_client
    ));
    on_log(format!("[Launcher] Game dir: {}", game_dir.to_string_lossy()));

    on_status("Resolving Java runtime...");
    let mut java_path = resolve_game_java_path(client, &profile, &on_log, None).await;
    on_log(format!("[Launcher] Java: {java_path}"));

    let base_installed = game_dir.join("versions").join(&profile.version).join(format!("{}.json", profile.version)).exists()
        && game_dir.join("versions").join(&profile.version).join(format!("{}.jar", profile.version)).exists();
    let tentative_version_id = resolve_version_id(&profile.version, &profile.loader, &game_dir);
    let loader_already_installed = profile.loader == "vanilla" || tentative_version_id != profile.version;

    if !base_installed || !loader_already_installed {
        on_log(format!(
            "[Launcher] Version not installed — downloading {}{}…",
            profile.version,
            if profile.loader != "vanilla" { format!(" + {}", profile.loader) } else { String::new() }
        ));
        let mut loader_ver = if profile.loader_version.is_empty() { None } else { Some(profile.loader_version.clone()) };
        if loader_ver.is_none() {
            loader_ver = match profile.loader.as_str() {
                "fabric" => version_service::list_fabric_versions(client, &profile.version)
                    .await
                    .ok()
                    .and_then(|versions| versions.iter().find(|v| v.loader.stable).or_else(|| versions.first()).map(|v| v.loader.version.clone())),
                "quilt" => crate::mcinstall::quilt::list_quilt_versions(client, &profile.version)
                    .await
                    .ok()
                    .and_then(|versions| versions.first().map(|v| v.loader.version.clone())),
                "forge" => version_service::list_forge_versions(client, &profile.version).await.into_iter().next(),
                "neoforge" => crate::mcinstall::forge_install::list_neoforge_versions(client, &profile.version)
                    .await
                    .ok()
                    .and_then(|versions| versions.into_iter().last()),
                _ => None,
            };
        }
        let on_progress = {
            let on_log = on_log.clone_fn();
            move |task: String, progress: u32, total: u32| {
                let pct = if total > 0 { progress * 100 / total } else { 0 };
                on_log(format!("[Download] {task} ({pct}%)"));
            }
        };
        version_service::install_version(
            client,
            &profile.version,
            &profile.loader,
            loader_ver.as_deref(),
            on_progress,
            &game_dir,
            settings.launcher.concurrent_downloads as usize,
            &java_path,
        )
        .await?;
        on_log("[Launcher] Download complete.".to_string());
    }

    crate::services::client_update_service::check_and_update_client_jar(client, &on_log, &on_status, Some(&profile.version)).await;

    // Force unlimited FPS, disable VSync, apply optimized defaults in options.txt.
    options_service::patch_options_file(&game_dir);

    // Remove conflicting manually-installed mods before launching (BejaClient bundles native
    // equivalents of Sodium/Lithium/FerriteCore/etc.).
    crate::services::mod_compatibility_checker::enforce_mod_compatibility(&mods_dir, &on_log);

    on_status("starting");

    let version_id = resolve_version_id(&profile.version, &profile.loader, &game_dir);
    on_log(format!("[Launcher] Version ID: {version_id}"));

    // Purge any leftover bejaclient mod JARs — BejaClient runs as a Java agent, not a mod.
    remove_beja_mod_jars(&game_dir, &on_log);

    if !mods_dir.exists() {
        let _ = std::fs::create_dir_all(&mods_dir);
    }
    let adapter_temp_path = mods_dir.join("beja-adapter-loader.jar");
    let _ = std::fs::remove_file(&adapter_temp_path);
    let mut adapter_was_staged = false;
    if profile.use_beja_client {
        if let Some(adapter_jar) = crate::services::client_update_service::resolve_adapter_jar(Some(&profile.version)) {
            match std::fs::copy(&adapter_jar, &adapter_temp_path) {
                Ok(_) => {
                    adapter_was_staged = true;
                    on_log("[BejaClient] Staged adapter JAR → mods/beja-adapter-loader.jar".to_string());
                }
                Err(e) => return Err(format!("Failed to stage adapter JAR: {e}")),
            }
        } else {
            on_log("[BejaClient] Adapter JAR not found — BejaHooks calls may fail.".to_string());
        }
    }

    let resolved_version = version_resolve::resolve_version(&game_dir, &version_id).await?;

    if let Some(jv) = &resolved_version.java_version {
        if let Some(component) = &jv.component {
            java_path = resolve_game_java_path(client, &profile, &on_log, Some(component)).await;
            on_log(format!("[Launcher] Java: {java_path}"));
        }
    }

    let chain = version_resolve::read_version_json_chain(&game_dir, &version_id).await?;
    let base_version_id = &chain[0].id;
    let version_jar_path = game_dir.join("versions").join(base_version_id).join(format!("{base_version_id}.jar"));
    let libraries_dir = game_dir.join("libraries");
    let natives_dir = game_dir.join("versions").join(&version_id).join(format!("{version_id}-natives"));

    let plat = platform::get_current_platform();
    let resolved_libs = libraries::resolve_libraries(&resolved_version.libraries, &plat);
    libraries::download_libraries(client, resolved_libs.clone(), &libraries_dir, settings.launcher.concurrent_downloads as usize).await?;
    let native_libs: Vec<_> = resolved_libs.iter().filter(|l| l.is_native).cloned().collect();
    natives::extract_natives(native_libs, &libraries_dir, &natives_dir).await?;
    let classpath = libraries::build_classpath(&resolved_libs, &version_jar_path, &libraries_dir);

    let uuid_no_dashes: String = account.uuid.chars().filter(|c| *c != '-').collect();
    let ctx = ArgContext {
        version: &resolved_version,
        game_dir: game_dir.to_string_lossy().to_string(),
        resource_dir: game_dir.to_string_lossy().to_string(),
        natives_dir: natives_dir.to_string_lossy().to_string(),
        classpath,
        java_path: java_path.clone(),
        auth: AuthCtx { uuid: uuid_no_dashes, username: account.username.clone(), access_token: account.access_token.clone(), user_type: "msa".to_string() },
        resolution: ResolutionCtx { width: profile.resolution.width, height: profile.resolution.height, fullscreen: false },
        min_memory: Some(profile.min_ram),
        max_memory: Some(profile.max_ram),
        launcher_version: app.package_info().version.to_string(),
    };

    let user_extra_args: Vec<String> = settings.game.extra_game_args.split(' ').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
    let args = launch_args::assemble_command(&ctx, &build_jvm_args_for_profile(&profile), &user_extra_args);

    if settings.launcher.debug_logging {
        on_log(format!("[Debug] Java: {java_path}"));
        on_log(format!("[Debug] Libraries: {} resolved", resolved_libs.len()));
        on_log(format!("[Debug] Full command: {java_path} {}", args.join(" ")));
    }

    let mut command = Command::new(&java_path);
    command.args(&args).current_dir(&game_dir).stdout(Stdio::piped()).stderr(Stdio::piped());
    apply_game_env(&mut command);

    let mut child = command.spawn().map_err(|e| format!("Failed to start Java process at '{java_path}': {e}"))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    KILL_REQUESTED.store(false, Ordering::SeqCst);
    *ACTIVE.lock() = Some(RunningGame { child, started_at: Instant::now() });
    let _ = app.emit("launch:status", "running");
    crate::console_window::send_console_status(&app, "running");
    crate::discord_rpc::set_playing_presence(Some(&profile.version));
    crate::services::beja_socket_service::emit_lobby_event(
        "presence:update",
        serde_json::json!({ "playing": { "game": "Minecraft", "version": profile.version }, "countryCode": account.country_code }),
    );
    if settings.launcher.close_on_launch {
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.hide();
        }
    }

    if let Some(stdout) = stdout {
        let app_out = app.clone();
        tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            let mut log4j = crate::services::log4j_parser::Log4jParser::new();
            while let Ok(Some(raw_line)) = lines.next_line().await {
                // Feed with the newline `.lines()` stripped off put back — the parser's
                // buffering logic (plain-text passthrough vs. XML event extraction) depends on
                // `\n` boundaries being present, same as the raw chunk stream the original TS
                // version fed it before any line-splitting happened.
                log4j.feed(&format!("{raw_line}\n"), |line| {
                    for finding in crate::services::crash_analyzer::analyze_crash_log(line) {
                        let crash_line = format!("[CRASH] [{}] {}: {}", finding.severity.to_uppercase(), finding.category, finding.human_readable);
                        let _ = app_out.emit("launch:log", crash_line.clone());
                        crate::console_window::send_console_log(&app_out, &crash_line);
                        let suggestion_line = format!("[CRASH] Suggestion: {}", finding.suggestion);
                        let _ = app_out.emit("launch:log", suggestion_line.clone());
                        crate::console_window::send_console_log(&app_out, &suggestion_line);
                    }
                    let _ = app_out.emit("launch:log", line.to_string());
                    crate::console_window::send_console_log(&app_out, line);
                });
            }
        });
    }
    if let Some(stderr) = stderr {
        let app_err = app.clone();
        tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let err_line = format!("[ERR] {line}");
                let _ = app_err.emit("launch:log", err_line.clone());
                crate::console_window::send_console_log(&app_err, &err_line);
            }
        });
    }

    let profile_id = profile_id.to_string();
    let app_exit = app.clone();
    tauri::async_runtime::spawn(async move {
        let exit_code = {
            let mut guard = ACTIVE.lock();
            let Some(running) = guard.as_mut() else { return };
            // We can't `.wait()` while holding the lock across an await point with a
            // std Mutex-guarded child easily, so re-borrow after releasing below.
            let _ = running;
            None::<i32>
        };
        // Re-fetch the child to await it outside the lock (parking_lot::Mutex is not held
        // across .await; the block above only checked presence).
        let wait_result = {
            let mut guard = ACTIVE.lock();
            guard.as_mut().map(|r| r.started_at)
        };
        let Some(started_at) = wait_result else { return };
        // Poll the process to completion.
        let code = loop {
            let done = {
                let mut guard = ACTIVE.lock();
                match guard.as_mut() {
                    Some(running) => running.child.try_wait().ok().flatten(),
                    None => break exit_code.unwrap_or(0), // killed externally
                }
            };
            if let Some(status) = done {
                break status.code().unwrap_or(0);
            }
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        };

        if adapter_was_staged {
            let _ = std::fs::remove_file(&adapter_temp_path);
        }
        let session_ms = started_at.elapsed().as_millis() as u64;
        if let Some(p) = profile_service::get_profile(&profile_id) {
            let patch = serde_json::json!({
                "lastPlayed": chrono::Utc::now().to_rfc3339(),
                "playtimeMs": p.playtime_ms + session_ms,
            });
            profile_service::update_profile(&profile_id, patch);
        }
        *ACTIVE.lock() = None;
        let _ = app_exit.emit("launch:status", format!("stopped:{code}"));
        crate::console_window::send_console_status(&app_exit, &format!("stopped:{code}"));
        crate::discord_rpc::set_idle_presence();
        crate::services::beja_socket_service::emit_lobby_event(
            "presence:update",
            serde_json::json!({ "playing": null, "countryCode": auth_service::get_selected_account().and_then(|a| a.country_code) }),
        );
        if settings_service::get_settings().launcher.keep_launcher_open {
            if let Some(win) = app_exit.get_webview_window("main") {
                let _ = win.show();
            }
        }
    });

    Ok(())
}

fn remove_beja_mod_jars(game_dir: &Path, on_log: &impl Fn(String)) {
    let mods_dir = game_dir.join("mods");
    let Ok(entries) = std::fs::read_dir(&mods_dir) else { return };
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(|s| s.to_string()) else { continue };
        if name.ends_with(".jar") && (name.starts_with("bejaclient-") || name.starts_with("beja-core-")) {
            if std::fs::remove_file(entry.path()).is_ok() {
                on_log(format!("[BejaClient] Removed stale mod JAR: {name}"));
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn apply_game_env(command: &mut Command) {
    // Force dedicated GPU on Linux (NVIDIA PRIME / AMD DRI_PRIME).
    command.env("__NV_PRIME_RENDER_OFFLOAD", "1");
    command.env("__GLX_VENDOR_LIBRARY_NAME", "nvidia");
    command.env("__VK_LAYER_NV_optimus", "NVIDIA_only");
    command.env("DRI_PRIME", "1");
    command.env("__GL_SYNC_TO_VBLANK", "0");
    command.env("vblank_mode", "0");
}

#[cfg(not(target_os = "linux"))]
fn apply_game_env(command: &mut Command) {
    command.env("__GL_SYNC_TO_VBLANK", "0");
    command.env("vblank_mode", "0");
}

/// Small helper so the `on_log` closure (which borrows `app` by move) can be cheaply reused
/// inside the `on_progress` closure passed to `install_version` without fighting the borrow
/// checker over FnMut vs Fn — both call sites only ever need to re-emit a Tauri event.
trait ClonableLogFn: Fn(String) + Send + Sync + 'static {
    fn clone_fn(&self) -> Box<dyn Fn(String) + Send + Sync>;
}
impl<T: Fn(String) + Clone + Send + Sync + 'static> ClonableLogFn for T {
    fn clone_fn(&self) -> Box<dyn Fn(String) + Send + Sync> {
        Box::new(self.clone())
    }
}

