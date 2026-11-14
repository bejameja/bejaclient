//! All `#[tauri::command]` entry points — the Rust-side counterpart of every channel in the old
//! `src/main/preload.ts` / `src/main/ipc/*.ts`. Registered in `lib.rs`'s
//! `tauri::generate_handler![...]` list. Command names are the snake_case form of the old IPC
//! channel name (see `src/api-bridge.ts` on the frontend for the exact mapping table).

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

use crate::mcinstall::java_discovery;
use crate::paths;
use crate::services::{
    auth_service, auto_backup_service, beja_api, beja_auth, beja_socket_service, client_update_service, curseforge_service, giphy_service,
    install_tracker, instance_doctor_service, launch_service, launcher_import_service, mod_service, mod_update_backup_service,
    modrinth_service, profile_service, screenshots_service, server_ping_service, settings_service, version_service, worlds_service,
};

// ── Auth ──────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn auth_login(app: AppHandle) -> Result<auth_service::StoredAccount, String> {
    auth_service::login_with_microsoft(app).await
}

#[tauri::command]
pub fn auth_get_login_url() -> String {
    auth_service::build_microsoft_login_url()
}

#[tauri::command]
pub async fn auth_login_with_code(input: String) -> Result<auth_service::StoredAccount, String> {
    auth_service::login_with_microsoft_code(input).await
}

#[tauri::command]
pub fn auth_logout(id: String) -> bool {
    auth_service::logout_account(&id);
    true
}

#[tauri::command]
pub fn auth_list_accounts() -> Vec<auth_service::StoredAccount> {
    auth_service::load_accounts()
}

#[tauri::command]
pub fn auth_select_account(id: String) -> Vec<auth_service::StoredAccount> {
    auth_service::select_account(&id)
}

#[tauri::command]
pub async fn auth_refresh(id: String) -> Result<Option<auth_service::StoredAccount>, String> {
    auth_service::refresh_account(&id).await
}

#[tauri::command]
pub async fn auth_import_launcher() -> Result<Vec<auth_service::StoredAccount>, String> {
    auth_service::import_from_official_launcher().await
}

// ── Versions ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn versions_list_remote() -> Result<version_service::VersionManifest, String> {
    version_service::fetch_version_manifest(auth_service::http_client()).await
}

#[tauri::command]
pub fn versions_list_installed() -> Vec<String> {
    let game_dir = std::path::PathBuf::from(settings_service::get_settings().game.default_game_dir);
    version_service::get_installed_versions(&game_dir)
}

#[tauri::command]
pub async fn versions_install(app: AppHandle, version_id: String, loader_type: String, loader_version: Option<String>) -> Result<Vec<String>, String> {
    let settings = settings_service::get_settings();
    let game_dir = std::path::PathBuf::from(&settings.game.default_game_dir);
    let java_path = if settings.game.default_java_path.is_empty() { "java".to_string() } else { settings.game.default_java_path.clone() };
    let app2 = app.clone();
    version_service::install_version(
        auth_service::http_client(),
        &version_id,
        &loader_type,
        loader_version.as_deref(),
        move |task, progress, total| {
            let _ = app2.emit("versions:progress", json!({ "task": task, "progress": progress, "total": total }));
        },
        &game_dir,
        settings.launcher.concurrent_downloads as usize,
        &java_path,
    )
    .await?;
    Ok(version_service::get_installed_versions(&game_dir))
}

#[tauri::command]
pub async fn versions_delete(version_id: String) -> Result<Vec<String>, String> {
    let game_dir = std::path::PathBuf::from(settings_service::get_settings().game.default_game_dir);
    version_service::delete_version(&version_id, &game_dir).await?;
    Ok(version_service::get_installed_versions(&game_dir))
}

#[tauri::command]
pub async fn versions_list_fabric(mc_version: String) -> Result<Vec<version_service::FabricLoaderVersionEntry>, String> {
    version_service::list_fabric_versions(auth_service::http_client(), &mc_version).await
}

#[tauri::command]
pub async fn versions_list_forge(mc_version: String) -> Vec<String> {
    version_service::list_forge_versions(auth_service::http_client(), &mc_version).await
}

#[tauri::command]
pub async fn optifine_list_mc_versions() -> Result<Vec<String>, String> {
    crate::mcinstall::optifine::list_mc_versions(auth_service::http_client()).await
}

#[tauri::command]
pub async fn optifine_list_builds(mc_version: String) -> Result<Vec<crate::mcinstall::optifine::OptifineEntry>, String> {
    crate::mcinstall::optifine::list_builds(auth_service::http_client(), &mc_version).await
}

/// Downloads the real OptiFine installer jar and opens it — same GUI a user gets double-clicking
/// it themselves. Returns the game dir a freshly-created OptiFine profile will actually use (its
/// `gameDir` is empty, so launch_service.rs falls back to this same default), so the caller can
/// show the user where to point the installer if it doesn't auto-detect it.
#[tauri::command]
pub async fn optifine_launch_installer(mc_version: String, filename: String) -> Result<String, String> {
    let settings = settings_service::get_settings();
    let client = auth_service::http_client();

    let java_path = if !settings.game.default_java_path.is_empty() {
        settings.game.default_java_path.clone()
    } else {
        let component = crate::mcinstall::java_runtime::guess_runtime_component(&mc_version);
        match crate::mcinstall::java_runtime::ensure_java_runtime(client, &component, &paths::runtimes_dir(), |_| {}).await {
            Ok(Some(bin)) => bin.to_string_lossy().to_string(),
            _ => "java".to_string(),
        }
    };

    crate::mcinstall::optifine::launch_gui_installer(client, &filename, &java_path, &crate::mcinstall::optifine::installer_cache_dir()).await?;
    Ok(paths::default_game_dir().to_string_lossy().to_string())
}

// ── Launch ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn launch_start(app: AppHandle, profile_id: String) -> Result<(), String> {
    launch_service::launch_game(app, &profile_id).await
}

#[tauri::command]
pub fn launch_kill() -> bool {
    launch_service::kill_game()
}

#[tauri::command]
pub fn launch_open_console(app: AppHandle) {
    crate::console_window::open_console_window(&app);
}

#[tauri::command]
pub fn launch_save_logs(lines: Vec<String>) -> Result<String, String> {
    let dest = paths::logs_dir().join(format!("session-{}.log", chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S")));
    std::fs::write(&dest, lines.join("\n")).map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}

// ── Profiles ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn profiles_list() -> Vec<profile_service::LaunchProfile> {
    profile_service::list_profiles()
}

#[tauri::command]
pub fn profiles_create(profile: profile_service::NewProfile) -> profile_service::LaunchProfile {
    profile_service::create_profile(profile)
}

#[tauri::command]
pub fn detect_external_profiles() -> Vec<launcher_import_service::DetectedProfile> {
    launcher_import_service::detect_all()
}

#[tauri::command]
pub fn import_external_profile(id: String) -> Result<profile_service::LaunchProfile, String> {
    launcher_import_service::import_profile(&id)
}

#[tauri::command]
pub fn profiles_update(id: String, profile: Value) -> Option<profile_service::LaunchProfile> {
    profile_service::update_profile(&id, profile)
}

#[tauri::command]
pub fn profiles_delete(id: String) -> bool {
    profile_service::delete_profile(&id)
}

#[tauri::command]
pub fn profiles_get_active() -> Option<profile_service::LaunchProfile> {
    settings_service::get_settings().active_profile_id.and_then(|id| profile_service::get_profile(&id))
}

#[tauri::command]
pub fn profiles_set_active(id: String) -> Option<profile_service::LaunchProfile> {
    let profile = profile_service::get_profile(&id)?;
    let mut settings = settings_service::get_settings();
    settings.active_profile_id = Some(id);
    settings_service::save_settings(&settings);
    Some(profile)
}

/// Port of `src/main/ipc/launcher.ts`'s `profiles:export` — writes a `.beja` pack file (profile +
/// list of installed mod filenames) via a save dialog. Mirrors `profiles_share`'s pack shape
/// (`bejaPackVersion: 1`) but written to disk instead of uploaded to the Beja API.
#[tauri::command]
pub fn profiles_export(app: AppHandle, id: String, without_beja_client: bool) -> Value {
    let Some(mut profile) = profile_service::get_profile(&id) else { return json!({ "ok": false, "error": "Profile not found" }) };
    if without_beja_client {
        profile.use_beja_client = false;
    }
    let settings = settings_service::get_settings();
    let game_dir = if !profile.game_dir.is_empty() { profile.game_dir.clone() } else { settings.game.default_game_dir };
    let mods_dir = std::path::Path::new(&game_dir).join("mods");
    let mods: Vec<String> = std::fs::read_dir(&mods_dir)
        .map(|it| it.flatten().filter_map(|e| e.file_name().into_string().ok()).filter(|f| f.ends_with(".jar") || f.ends_with(".jar.disabled")).collect())
        .unwrap_or_default();

    let default_name = {
        let safe: String = profile.name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
        format!("{safe}.beja")
    };

    let Some(save_path) = app
        .dialog()
        .file()
        .set_title("Export Profile Pack")
        .add_filter("BejaClient Pack", &["beja"])
        .set_file_name(&default_name)
        .blocking_save_file()
    else {
        return json!({ "ok": false, "error": "cancelled" });
    };

    let pack = json!({
        "bejaPackVersion": 1,
        "exportedAt": chrono::Utc::now().timestamp_millis(),
        "profile": profile,
        "mods": mods,
    });
    let Ok(text) = serde_json::to_string_pretty(&pack) else { return json!({ "ok": false, "error": "Failed to serialize pack" }) };
    match std::fs::write(save_path.to_string(), text) {
        Ok(_) => json!({ "ok": true, "mods": mods.len() }),
        Err(e) => json!({ "ok": false, "error": e.to_string() }),
    }
}

/// Self-contained ZIP export: `profile.json` (portable subset — no local paths/account fields)
/// plus the actual mod jars, so a recipient without a BejaClient account can just unpack it.
#[tauri::command]
pub fn profiles_export_zip(app: AppHandle, id: String, without_beja_client: bool) -> Value {
    let Some(profile) = profile_service::get_profile(&id) else { return json!({ "ok": false, "error": "Profile not found" }) };
    let settings = settings_service::get_settings();
    let game_dir = if !profile.game_dir.is_empty() { profile.game_dir.clone() } else { settings.game.default_game_dir };
    let mods_dir = std::path::Path::new(&game_dir).join("mods");

    let default_name = {
        let safe: String = profile.name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
        format!("{safe}.zip")
    };

    let Some(save_path) = app
        .dialog()
        .file()
        .set_title("Export Profile as ZIP")
        .add_filter("ZIP Archive", &["zip"])
        .set_file_name(&default_name)
        .blocking_save_file()
    else {
        return json!({ "ok": false, "error": "cancelled" });
    };

    // Portable subset only — `gameDir`/`javaPath` are absolute local paths that mean nothing on
    // another machine, and `id`/`createdAt`/`lastPlayed`/`playtimeMs` are this install's own.
    let manifest = json!({
        "bejaPackVersion": 1,
        "exportedAt": chrono::Utc::now().timestamp_millis(),
        "name": profile.name,
        "description": profile.description,
        "version": profile.version,
        "loader": profile.loader,
        "loaderVersion": profile.loader_version,
        "minRam": profile.min_ram,
        "maxRam": profile.max_ram,
        "jvmArgs": profile.jvm_args,
        "resolution": profile.resolution,
        "useBejaClient": if without_beja_client { false } else { profile.use_beja_client },
    });
    let manifest_text = match serde_json::to_string_pretty(&manifest) {
        Ok(t) => t,
        Err(e) => return json!({ "ok": false, "error": e.to_string() }),
    };

    let file = match std::fs::File::create(save_path.to_string()) {
        Ok(f) => f,
        Err(e) => return json!({ "ok": false, "error": e.to_string() }),
    };
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let write_result = (|| -> std::io::Result<usize> {
        zip.start_file("profile.json", options)?;
        std::io::Write::write_all(&mut zip, manifest_text.as_bytes())?;

        let mut mod_count = 0usize;
        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let Some(name) = name.to_str() else { continue };
                if !(name.ends_with(".jar") || name.ends_with(".jar.disabled")) {
                    continue;
                }
                let Ok(bytes) = std::fs::read(entry.path()) else { continue };
                zip.start_file(format!("mods/{name}"), options)?;
                std::io::Write::write_all(&mut zip, &bytes)?;
                mod_count += 1;
            }
        }
        zip.finish()?;
        Ok(mod_count)
    })();

    match write_result {
        Ok(mod_count) => json!({ "ok": true, "mods": mod_count }),
        Err(e) => json!({ "ok": false, "error": e.to_string() }),
    }
}

/// Exports a profile as a genuine CurseForge-modpack-shaped ZIP: `manifest.json` +
/// `overrides/mods/*.jar`. All mods ship inside `overrides/` rather than as `files` entries
/// resolved by project/file ID — an "overrides-only" pack is a well-established pattern for
/// packs containing mods that aren't (all) on CurseForge, and any importer that reads the
/// manifest spec (Lunar Client's modpack import included) treats it exactly like a real one.
#[tauri::command]
pub fn profiles_export_curseforge(app: AppHandle, id: String) -> Value {
    let Some(profile) = profile_service::get_profile(&id) else { return json!({ "ok": false, "error": "Profile not found" }) };
    let settings = settings_service::get_settings();
    let game_dir = if !profile.game_dir.is_empty() { profile.game_dir.clone() } else { settings.game.default_game_dir };
    let mods_dir = std::path::Path::new(&game_dir).join("mods");

    let default_name = {
        let safe: String = profile.name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
        format!("{safe}.zip")
    };

    let Some(save_path) = app
        .dialog()
        .file()
        .set_title("Export as CurseForge Modpack")
        .add_filter("ZIP Archive", &["zip"])
        .set_file_name(&default_name)
        .blocking_save_file()
    else {
        return json!({ "ok": false, "error": "cancelled" });
    };

    let loader_id = match profile.loader.as_str() {
        "fabric" | "forge" | "neoforge" | "quilt" if !profile.loader_version.is_empty() => format!("{}-{}", profile.loader, profile.loader_version),
        other => other.to_string(),
    };

    let manifest = json!({
        "minecraft": {
            "version": profile.version,
            "modLoaders": [ { "id": loader_id, "primary": true } ],
        },
        "manifestType": "minecraftModpack",
        "manifestVersion": 1,
        "name": profile.name,
        "version": "1.0.0",
        "author": "BejaClient",
        "files": [],
        "overrides": "overrides",
    });
    let manifest_text = match serde_json::to_string_pretty(&manifest) {
        Ok(t) => t,
        Err(e) => return json!({ "ok": false, "error": e.to_string() }),
    };

    let file = match std::fs::File::create(save_path.to_string()) {
        Ok(f) => f,
        Err(e) => return json!({ "ok": false, "error": e.to_string() }),
    };
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let write_result = (|| -> std::io::Result<usize> {
        zip.start_file("manifest.json", options)?;
        std::io::Write::write_all(&mut zip, manifest_text.as_bytes())?;

        let mut mod_count = 0usize;
        let mut mod_names: Vec<String> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&mods_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let Some(name) = name.to_str() else { continue };
                // Only enabled mods (bare .jar) — a modpack meant to be played shouldn't ship
                // disabled ones, unlike the plain folder-mirror ZIP export.
                if !name.ends_with(".jar") {
                    continue;
                }
                let Ok(bytes) = std::fs::read(entry.path()) else { continue };
                zip.start_file(format!("overrides/mods/{name}"), options)?;
                std::io::Write::write_all(&mut zip, &bytes)?;
                mod_count += 1;
                mod_names.push(name.trim_end_matches(".jar").to_string());
            }
        }

        let escape = |s: &str| s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
        let modlist_html = format!(
            "<ul>\n{}\n</ul>\n",
            mod_names.iter().map(|n| format!("<li>{}</li>", escape(n))).collect::<Vec<_>>().join("\n")
        );
        zip.start_file("modlist.html", options)?;
        std::io::Write::write_all(&mut zip, modlist_html.as_bytes())?;

        zip.finish()?;
        Ok(mod_count)
    })();

    match write_result {
        Ok(mod_count) => json!({ "ok": true, "mods": mod_count }),
        Err(e) => json!({ "ok": false, "error": e.to_string() }),
    }
}

/// Port of `src/main/ipc/launcher.ts`'s `profiles:import`.
#[tauri::command]
pub fn profiles_import(app: AppHandle) -> Value {
    let Some(picked) = app.dialog().file().set_title("Import Profile Pack").add_filter("BejaClient Pack", &["beja"]).blocking_pick_file() else {
        return Value::Null;
    };
    let Ok(raw) = std::fs::read_to_string(picked.to_string()) else {
        return json!({ "error": "Could not read file" });
    };
    let Ok(pack) = serde_json::from_str::<Value>(&raw) else {
        return json!({ "error": "Invalid pack file" });
    };
    if pack["bejaPackVersion"].as_i64() != Some(1) {
        return json!({ "error": "Unsupported pack version" });
    }
    let Ok(new_profile_data) = serde_json::from_value::<profile_service::NewProfile>(pack["profile"].clone()) else {
        return json!({ "error": "Malformed profile in pack" });
    };
    let new_profile = profile_service::create_profile(new_profile_data);
    json!({ "profile": new_profile, "mods": pack["mods"] })
}

#[tauri::command]
pub async fn profiles_share(id: String, without_beja_client: bool) -> Value {
    let Some(mut profile) = profile_service::get_profile(&id) else { return json!({ "error": "Profile not found" }) };
    if without_beja_client {
        profile.use_beja_client = false;
    }
    let Some(token) = beja_auth::get_beja_token().await else {
        return json!({ "error": "You need to be signed in to share a profile." });
    };
    let settings = settings_service::get_settings();
    let game_dir = if !profile.game_dir.is_empty() { profile.game_dir.clone() } else { settings.game.default_game_dir };
    let mods_dir = std::path::Path::new(&game_dir).join("mods");
    let mods: Vec<String> = std::fs::read_dir(&mods_dir)
        .map(|it| it.flatten().filter_map(|e| e.file_name().into_string().ok()).filter(|f| f.ends_with(".jar") || f.ends_with(".jar.disabled")).collect())
        .unwrap_or_default();

    let pack = json!({ "bejaPackVersion": 1, "exportedAt": chrono::Utc::now().timestamp_millis(), "profile": profile, "mods": mods });
    let res = beja_api::post("/api/profiles/share", Some(&token), pack).await;
    if res["id"].is_string() {
        json!({ "id": res["id"], "protocolUrl": res["protocolUrl"], "webUrl": res["webUrl"] })
    } else {
        json!({ "error": res["error"].as_str().unwrap_or("Failed to create share link") })
    }
}

#[tauri::command]
pub async fn profiles_peek_shared(share_id: String) -> Value {
    let res = beja_api::get(&format!("/api/profiles/share/{share_id}"), None).await;
    if res["error"].is_string() && res["pack"].is_null() {
        return json!({ "error": res["error"] });
    }
    json!({
        "ownerUsername": res["ownerUsername"],
        "profileName": res["pack"]["profile"]["name"].as_str().unwrap_or("Unknown profile"),
        "version": res["pack"]["profile"]["version"],
        "loader": res["pack"]["profile"]["loader"],
        "modCount": res["pack"]["mods"].as_array().map(|a| a.len()).unwrap_or(0),
    })
}

#[tauri::command]
pub async fn profiles_import_shared(share_id: String) -> Value {
    let res = beja_api::get(&format!("/api/profiles/share/{share_id}"), None).await;
    let pack = &res["pack"];
    if pack["bejaPackVersion"].as_i64() != Some(1) {
        return json!({ "error": "Unsupported pack version" });
    }
    let Ok(new_profile_data) = serde_json::from_value::<profile_service::NewProfile>(pack["profile"].clone()) else {
        return json!({ "error": "Malformed shared profile" });
    };
    let new_profile = profile_service::create_profile(new_profile_data);
    json!({ "profile": new_profile, "mods": pack["mods"], "ownerUsername": res["ownerUsername"] })
}

// ── Mods ──────────────────────────────────────────────────────────────────────

// mods_* below all go through `spawn_blocking` — `list_mods` opens and reads the zip central
// directory of every installed mod jar (to pull `fabric.mod.json`/`pack.png`), which for a
// modpack-sized `mods/` folder is real disk + CPU work. Tauri's IPC dispatch shares the main
// thread with the WebView2 message loop, so running that synchronously here would hang the
// whole window ("BejaClient (Not Responding)") instead of just being slow.

#[tauri::command]
pub async fn mods_list(profile_id: String) -> Vec<mod_service::ModInfo> {
    tokio::task::spawn_blocking(move || mod_service::list_mods(&profile_id)).await.unwrap_or_default()
}

#[tauri::command]
pub async fn mods_count_enabled(profile_id: String) -> usize {
    tokio::task::spawn_blocking(move || mod_service::count_enabled_mods(&profile_id)).await.unwrap_or(0)
}

#[tauri::command]
pub async fn mods_check_conflicts(profile_id: String) -> Vec<String> {
    const RULES: &[(&str, &str, &str)] = &[
        ("optifine", "sodium", "OptiFine + Sodium are incompatible. Use Iris+Sodium instead of OptiFine."),
        ("optifine", "iris", "OptiFine + Iris are incompatible. Remove one of them."),
        ("optifine", "rubidium", "OptiFine + Rubidium are incompatible."),
        ("optifabric", "sodium", "OptiFabric + Sodium are incompatible."),
        ("sodium", "embeddium", "Sodium + Embeddium conflict - both are rendering overhauls."),
        ("sodium", "rubidium", "Sodium + Rubidium conflict - both are rendering overhauls."),
        ("lithium", "canary", "Lithium + Canary are incompatible optimization mods."),
    ];
    let mods = tokio::task::spawn_blocking({
        let profile_id = profile_id.clone();
        move || mod_service::list_mods(&profile_id)
    })
    .await
    .unwrap_or_default();
    let names: Vec<String> = mods.iter().filter(|m| m.enabled).map(|m| m.file_name.to_lowercase()).collect();
    let mut conflicts = Vec::new();
    for (a, b, msg) in RULES {
        if names.iter().any(|n| n.contains(a)) && names.iter().any(|n| n.contains(b)) {
            conflicts.push(msg.to_string());
        }
    }
    conflicts
}

#[tauri::command]
pub async fn mods_install(app: AppHandle, profile_id: String, file_path: Option<String>) -> Result<Vec<mod_service::ModInfo>, String> {
    let path = match file_path {
        Some(p) => p,
        None => {
            let picked = app
                .dialog()
                .file()
                .add_filter("Minecraft Mods", &["jar"])
                .blocking_pick_file();
            let Some(picked) = picked else { return Ok(mods_list(profile_id).await) };
            picked.to_string()
        }
    };
    tokio::task::spawn_blocking(move || mod_service::install_mod(&profile_id, &path)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn mods_toggle(profile_id: String, mod_id: String) -> Result<Vec<mod_service::ModInfo>, String> {
    tokio::task::spawn_blocking(move || mod_service::toggle_mod(&profile_id, &mod_id)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn mods_delete(profile_id: String, mod_id: String) -> Vec<mod_service::ModInfo> {
    tokio::task::spawn_blocking(move || mod_service::delete_mod(&profile_id, &mod_id)).await.unwrap_or_default()
}

#[tauri::command]
pub fn mods_open_folder(app: AppHandle, profile_id: String) {
    mod_service::open_mods_folder(&app, &profile_id);
}

// ── Instance Health Doctor ──────────────────────────────────────────────────

#[tauri::command]
pub async fn doctor_run(profile_id: String) -> Result<Vec<instance_doctor_service::HealthCheck>, String> {
    let profile = profile_service::get_profile(&profile_id).ok_or("Profile not found")?;
    let mut checks = instance_doctor_service::run_health_check(auth_service::http_client(), &profile).await;
    let mod_conflicts = mods_check_conflicts(profile_id).await;
    checks.push(instance_doctor_service::HealthCheck {
        id: "mod_conflicts".to_string(),
        label: "Mod conflicts".to_string(),
        status: if mod_conflicts.is_empty() { "pass".to_string() } else { "warn".to_string() },
        message: if mod_conflicts.is_empty() { "No known conflicting mod pairs found".to_string() } else { mod_conflicts.join("; ") },
        fixable: false,
    });
    Ok(checks)
}

/// Re-runs the normal install pipeline — `download_file` already skips anything whose sha1
/// already matches, so this only actually re-fetches what the Doctor found missing/corrupt.
#[tauri::command]
pub async fn doctor_repair(app: AppHandle, profile_id: String) -> Value {
    let Some(profile) = profile_service::get_profile(&profile_id) else { return json!({ "ok": false, "error": "Profile not found" }) };
    let settings = settings_service::get_settings();
    let game_dir = std::path::PathBuf::from(if !profile.game_dir.is_empty() { profile.game_dir.clone() } else { settings.game.default_game_dir });
    let java_path = if profile.java_path.is_empty() { "java".to_string() } else { profile.java_path.clone() };
    let app2 = app.clone();
    let result = version_service::install_version(
        auth_service::http_client(),
        &profile.version,
        &profile.loader,
        if profile.loader_version.is_empty() { None } else { Some(profile.loader_version.as_str()) },
        move |task, progress, total| {
            let _ = app2.emit("doctor:progress", json!({ "task": task, "progress": progress, "total": total }));
        },
        &game_dir,
        settings.launcher.concurrent_downloads as usize,
        &java_path,
    )
    .await;
    match result {
        Ok(()) => json!({ "ok": true }),
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

// ── Hardware / auto-tune ────────────────────────────────────────────────────────

/// System RAM + core count for the "Auto-tune" button in Game Settings. `sysinfo` is queried
/// fresh each call (memory-only refresh, not a whole-system snapshot) — this is a one-shot
/// settings action, not something polled on a timer.
#[tauri::command]
pub fn system_hardware_info() -> Value {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    sys.refresh_cpu_all();
    json!({
        "totalRamMb": sys.total_memory() / 1024 / 1024,
        "cpuCores": sys.cpus().len().max(1),
    })
}

// ── Worlds ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn worlds_list(profile_id: String) -> Vec<worlds_service::WorldInfo> {
    worlds_service::list_worlds(&profile_service::resolve_game_dir(&profile_id))
}

#[tauri::command]
pub fn worlds_open_folder(app: AppHandle, profile_id: String, world_id: String) {
    let path = worlds_service::world_folder(&profile_service::resolve_game_dir(&profile_id), &world_id);
    let _ = tauri_plugin_opener::OpenerExt::opener(&app).open_path(path.to_string_lossy().to_string(), None::<&str>);
}

#[tauri::command]
pub fn worlds_rename(profile_id: String, world_id: String, new_name: String) -> Value {
    match worlds_service::rename_world(&profile_service::resolve_game_dir(&profile_id), &world_id, new_name) {
        Ok(world) => json!({ "ok": true, "world": world }),
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

#[tauri::command]
pub fn worlds_duplicate(profile_id: String, world_id: String, new_name: String) -> Value {
    match worlds_service::duplicate_world(&profile_service::resolve_game_dir(&profile_id), &world_id, new_name) {
        Ok(world) => json!({ "ok": true, "world": world }),
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

#[tauri::command]
pub fn worlds_delete(profile_id: String, world_id: String) -> Value {
    match worlds_service::delete_world(&profile_service::resolve_game_dir(&profile_id), &world_id) {
        Ok(()) => json!({ "ok": true }),
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

// ── Screenshots ───────────────────────────────────────────────────────────────

#[tauri::command]
pub fn screenshots_list(profile_id: String) -> Vec<screenshots_service::ScreenshotInfo> {
    screenshots_service::list_screenshots(&profile_service::resolve_game_dir(&profile_id))
}

#[tauri::command]
pub fn screenshots_thumbnail(profile_id: String, id: String) -> Option<String> {
    screenshots_service::thumbnail(&profile_service::resolve_game_dir(&profile_id), &id)
}

#[tauri::command]
pub fn screenshots_delete(profile_id: String, id: String) -> Value {
    match screenshots_service::delete_screenshot(&profile_service::resolve_game_dir(&profile_id), &id) {
        Ok(()) => json!({ "ok": true }),
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

#[tauri::command]
pub fn screenshots_open(app: AppHandle, profile_id: String, id: String) {
    let Some(path) = screenshots_service::path_for(&profile_service::resolve_game_dir(&profile_id), &id) else { return };
    let _ = tauri_plugin_opener::OpenerExt::opener(&app).open_path(path.to_string_lossy().to_string(), None::<&str>);
}

#[tauri::command]
pub fn screenshots_open_folder(app: AppHandle, profile_id: String) {
    let path = screenshots_service::folder(&profile_service::resolve_game_dir(&profile_id));
    let _ = tauri_plugin_opener::OpenerExt::opener(&app).open_path(path.to_string_lossy().to_string(), None::<&str>);
}

#[tauri::command]
pub fn screenshots_export(app: AppHandle, profile_id: String, id: String) -> Value {
    let Some(src) = screenshots_service::path_for(&profile_service::resolve_game_dir(&profile_id), &id) else {
        return json!({ "ok": false, "error": "Screenshot not found" });
    };
    let Some(save_path) = app.dialog().file().set_title("Save Screenshot As").add_filter("PNG Image", &["png"]).set_file_name(&id).blocking_save_file()
    else {
        return json!({ "ok": false, "error": "cancelled" });
    };
    match std::fs::copy(&src, save_path.to_string()) {
        Ok(_) => json!({ "ok": true }),
        Err(e) => json!({ "ok": false, "error": e.to_string() }),
    }
}

#[tauri::command]
pub fn worlds_list_auto_backups(profile_id: String, world_id: String) -> Vec<String> {
    auto_backup_service::list_backups(&profile_id, &world_id)
}

#[tauri::command]
pub fn worlds_restore_auto_backup(profile_id: String, world_id: String, backup_name: String) -> Value {
    let game_dir = profile_service::resolve_game_dir(&profile_id);
    match auto_backup_service::restore_backup(&profile_id, &world_id, &backup_name, &game_dir) {
        Ok(world) => json!({ "ok": true, "world": world }),
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

/// Zips the world and lets the user pick where to save it (a manual "keep this world safe"
/// backup — separate from any auto-backup policy, there isn't one yet).
#[tauri::command]
pub fn worlds_backup(app: AppHandle, profile_id: String, world_id: String) -> Value {
    let default_name = format!("{world_id}.zip");
    let Some(save_path) = app
        .dialog()
        .file()
        .set_title("Backup World")
        .add_filter("ZIP Archive", &["zip"])
        .set_file_name(&default_name)
        .blocking_save_file()
    else {
        return json!({ "ok": false, "error": "cancelled" });
    };
    let dest = std::path::PathBuf::from(save_path.to_string());
    match worlds_service::backup_world(&profile_service::resolve_game_dir(&profile_id), &world_id, &dest) {
        Ok(()) => json!({ "ok": true }),
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

/// Restores a world from a `worlds_backup` zip (or any zip shaped like one — a single top-level
/// folder containing `level.dat`) into this profile's `saves/`, auto-renaming on collision.
#[tauri::command]
pub fn worlds_restore(app: AppHandle, profile_id: String) -> Value {
    let Some(pick) = app.dialog().file().set_title("Restore World").add_filter("ZIP Archive", &["zip"]).blocking_pick_file() else {
        return json!({ "ok": false, "error": "cancelled" });
    };
    let zip_path = std::path::PathBuf::from(pick.to_string());
    match worlds_service::restore_world(&profile_service::resolve_game_dir(&profile_id), &zip_path) {
        Ok(world) => json!({ "ok": true, "world": world }),
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

#[tauri::command]
pub async fn mods_auto_fix(profile_id: String) -> Value {
    let fixed = tokio::task::spawn_blocking(move || {
        let mods = mod_service::list_mods(&profile_id);
        let enabled: Vec<_> = mods.iter().filter(|m| m.enabled).collect();
        let slug_of = |name: &str| -> String {
            let base = name.trim_end_matches(".disabled").trim_end_matches(".jar");
            let re = regex::Regex::new(r"[-_]\d.*$").unwrap();
            re.replace(base, "").to_lowercase()
        };
        let mut groups: std::collections::HashMap<String, Vec<&mod_service::ModInfo>> = std::collections::HashMap::new();
        for m in &enabled {
            groups.entry(slug_of(&m.file_name)).or_default().push(m);
        }
        let mut fixed = Vec::new();
        for (_, mut group) in groups {
            if group.len() < 2 {
                continue;
            }
            group.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
            for m in &group[1..] {
                mod_service::delete_mod(&profile_id, &m.id);
                fixed.push(m.file_name.clone());
            }
        }
        fixed
    })
    .await
    .unwrap_or_default();
    json!({ "fixed": fixed })
}

// ── Settings ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn settings_get() -> settings_service::AppSettings {
    settings_service::get_settings()
}

#[tauri::command]
pub fn settings_set(settings: settings_service::AppSettings) -> bool {
    settings_service::save_settings(&settings);
    true
}

// ── BejaClient adapter versions ──────────────────────────────────────────────

#[tauri::command]
pub async fn client_versions_list(family: String) -> Result<Vec<client_update_service::JarVersionEntry>, String> {
    let client = reqwest::Client::new();
    client_update_service::list_versions(&client, &family).await
}

#[tauri::command]
pub fn client_version_pinned(family: String) -> Option<String> {
    settings_service::get_settings().launcher.pinned_client_versions.get(&family).cloned()
}

#[tauri::command]
pub async fn client_version_install(family: String, version: String) -> Result<(), String> {
    let client = reqwest::Client::new();
    let versions = client_update_service::list_versions(&client, &family).await?;
    let entry = versions.into_iter().find(|v| v.version == version).ok_or_else(|| format!("Version {version} not found for {family}"))?;
    client_update_service::install_specific_version(&client, &|_| {}, &|_| {}, &entry).await?;

    let mut settings = settings_service::get_settings();
    settings.launcher.pinned_client_versions.insert(family, version);
    settings_service::save_settings(&settings);
    Ok(())
}

#[tauri::command]
pub async fn client_version_use_latest(family: String) -> Result<(), String> {
    let mut settings = settings_service::get_settings();
    settings.launcher.pinned_client_versions.remove(&family);
    settings_service::save_settings(&settings);

    let client = reqwest::Client::new();
    client_update_service::update_family_to_latest(&client, &|_| {}, &|_| {}, &family).await
}

#[tauri::command]
pub fn settings_game_dir() -> String {
    settings_service::get_settings().game.default_game_dir
}

#[tauri::command]
pub fn settings_set_game_dir(dir: String) -> bool {
    let mut settings = settings_service::get_settings();
    settings.game.default_game_dir = dir;
    settings_service::save_settings(&settings);
    true
}

#[tauri::command]
pub fn settings_choose_java(app: AppHandle) -> Option<String> {
    app.dialog().file().add_filter("Java executable", if cfg!(windows) { &["exe"] } else { &[""] }).blocking_pick_file().map(|p| p.to_string())
}

#[tauri::command]
pub fn settings_choose_dir(app: AppHandle) -> Option<String> {
    app.dialog().file().blocking_pick_folder().map(|p| p.to_string())
}

// ── Modrinth / explore ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn modrinth_search(
    query: String,
    r#type: String,
    game_version: Option<String>,
    loader: Option<String>,
    offset: Option<u32>,
    categories: Option<Vec<String>>,
) -> Result<Value, String> {
    modrinth_service::search_modrinth(&query, &r#type, game_version.as_deref(), loader.as_deref(), offset.unwrap_or(0), categories, None).await
}

#[tauri::command]
pub async fn modrinth_categories() -> Result<Value, String> {
    modrinth_service::get_categories().await
}

/// Port of `src/main/ipc/modrinth.ts`'s `explore:search` handler — normalizes both Modrinth's and
/// CurseForge's differently-shaped search hits into one `ExploreHit` shape the frontend renders
/// uniformly, and interleaves mr/cf/mr/cf... when `source === 'both'`.
#[tauri::command]
pub async fn explore_search(
    query: String,
    r#type: String,
    source: String,
    game_version: Option<String>,
    loader: Option<String>,
    offset: Option<u32>,
    categories: Option<Vec<String>>,
    sort: Option<String>,
) -> Result<Value, String> {
    let offset = offset.unwrap_or(0);
    let mut mr_hits: Vec<Value> = Vec::new();
    let mut cf_hits: Vec<Value> = Vec::new();
    let mut total: u64 = 0;

    if source == "modrinth" || source == "both" {
        match modrinth_service::search_modrinth(&query, &r#type, game_version.as_deref(), loader.as_deref(), offset, categories.clone(), sort.as_deref()).await {
            Ok(res) => {
                if let Some(hits) = res["hits"].as_array() {
                    for h in hits {
                        mr_hits.push(json!({
                            "id": h["project_id"],
                            "title": h["title"],
                            "description": h["description"],
                            "iconUrl": h["icon_url"],
                            "downloads": h["downloads"],
                            "categories": h["categories"],
                            "source": "modrinth",
                            "projectType": h["project_type"],
                            "slug": h["slug"],
                            "author": h["author"],
                        }));
                    }
                }
                total += res["total_hits"].as_u64().unwrap_or(0);
            }
            Err(e) => {
                if source == "modrinth" {
                    return Err(e);
                }
            }
        }
    }

    if source == "curseforge" || source == "both" {
        match curseforge_service::search_curseforge(&query, &r#type, game_version.as_deref(), loader.as_deref(), offset, sort.as_deref()).await {
            Ok(res) => {
                if let Some(hits) = res["hits"].as_array() {
                    for h in hits {
                        cf_hits.push(json!({
                            "id": h["id"].as_u64().map(|n| n.to_string()).unwrap_or_default(),
                            "title": h["name"],
                            "description": h["summary"],
                            "iconUrl": h["logo"]["thumbnailUrl"],
                            "downloads": h["downloadCount"],
                            "categories": h["categories"].as_array().map(|a| a.iter().map(|c| c["name"].clone()).collect::<Vec<_>>()).unwrap_or_default(),
                            "source": "curseforge",
                            "projectType": r#type,
                            "slug": h["slug"],
                            "author": h["authors"].as_array().and_then(|a| a.first()).map(|a| a["name"].clone()).unwrap_or(Value::Null),
                        }));
                    }
                }
                total += res["total"].as_u64().unwrap_or(0);
            }
            Err(e) => {
                if source == "curseforge" {
                    return Err(e);
                }
            }
        }
    }

    let hits = if source == "both" {
        let max_len = mr_hits.len().max(cf_hits.len());
        let mut interleaved = Vec::with_capacity(mr_hits.len() + cf_hits.len());
        for i in 0..max_len {
            if let Some(h) = mr_hits.get(i) {
                interleaved.push(h.clone());
            }
            if let Some(h) = cf_hits.get(i) {
                interleaved.push(h.clone());
            }
        }
        interleaved
    } else if source == "modrinth" {
        mr_hits
    } else {
        cf_hits
    };

    Ok(json!({ "hits": hits, "total": total }))
}

#[tauri::command]
pub async fn curseforge_install(mod_id: String, project_type: String, profile_id: String) -> Result<bool, String> {
    curseforge_service::install_curseforge_mod(&mod_id, &project_type, &profile_id, |_msg| {}).await?;
    install_tracker::record_mod_install(&mod_id, &profile_id);
    Ok(true)
}

/// Fetches everything the "view mod" detail panel needs and normalizes Modrinth's and
/// CurseForge's very differently-shaped project payloads into one common shape.
#[tauri::command]
pub async fn mod_details(id: String, source: String) -> Result<Value, String> {
    if source == "curseforge" {
        let m = curseforge_service::get_mod_details(&id).await?;
        Ok(json!({
            "id": m["id"].as_u64().map(|n| n.to_string()).unwrap_or_default(),
            "title": m["name"],
            "author": m["authors"].as_array().and_then(|a| a.first()).map(|a| a["name"].clone()).unwrap_or(Value::Null),
            "iconUrl": m["logo"]["url"],
            "downloads": m["downloadCount"],
            "categories": m["categories"].as_array().map(|a| a.iter().map(|c| c["name"].clone()).collect::<Vec<_>>()).unwrap_or_default(),
            "description": m["fullDescription"],
            "descriptionFormat": "html",
            "gallery": m["screenshots"].as_array().map(|a| a.iter().filter_map(|s| s["url"].as_str()).map(Value::from).collect::<Vec<_>>()).unwrap_or_default(),
            "sourceUrl": m["links"]["websiteUrl"],
            "license": Value::Null,
            "createdAt": m["dateCreated"],
            "updatedAt": m["dateModified"],
            "source": "curseforge",
        }))
    } else {
        let p = modrinth_service::get_project(&id).await?;
        Ok(json!({
            "id": p["id"],
            "title": p["title"],
            "author": Value::Null,
            "iconUrl": p["icon_url"],
            "downloads": p["downloads"],
            "categories": p["categories"],
            "description": p["body"],
            "descriptionFormat": "markdown",
            "gallery": p["gallery"].as_array().map(|a| a.iter().filter_map(|g| g["url"].as_str()).map(Value::from).collect::<Vec<_>>()).unwrap_or_default(),
            "sourceUrl": Value::String(format!("https://modrinth.com/mod/{}", p["slug"].as_str().unwrap_or(&id))),
            "license": p["license"]["name"],
            "createdAt": p["published"],
            "updatedAt": p["updated"],
            "source": "modrinth",
        }))
    }
}

#[tauri::command]
pub async fn modrinth_versions(project_id: String, game_version: Option<String>, loader: Option<String>) -> Result<Vec<modrinth_service::ModrinthVersion>, String> {
    modrinth_service::get_project_versions(&project_id, game_version.as_deref(), loader.as_deref()).await
}

#[tauri::command]
pub async fn modrinth_install_mod(project_id: String, profile_id: String) -> Result<bool, String> {
    modrinth_service::download_mod(&project_id, &profile_id).await?;
    install_tracker::record_mod_install(&project_id, &profile_id);
    Ok(true)
}

#[tauri::command]
pub async fn modrinth_install_modpack(project_id: String, version_id: Option<String>) -> Result<Value, String> {
    let r = modrinth_service::install_modpack(&project_id, version_id.as_deref()).await?;
    Ok(json!({ "profileId": r.profile_id, "name": r.name }))
}

#[tauri::command]
pub async fn modrinth_install_resourcepack(project_id: String, profile_id: String) -> Result<bool, String> {
    modrinth_service::download_resourcepack(&project_id, &profile_id).await?;
    install_tracker::record_mod_install(&project_id, &profile_id);
    Ok(true)
}

#[tauri::command]
pub async fn modrinth_install_shader(project_id: String, profile_id: String) -> Result<bool, String> {
    modrinth_service::download_shader(&project_id, &profile_id).await?;
    install_tracker::record_mod_install(&project_id, &profile_id);
    Ok(true)
}

#[tauri::command]
pub async fn modrinth_install_datapack(project_id: String, profile_id: String) -> Result<bool, String> {
    modrinth_service::download_datapack(&project_id, &profile_id).await?;
    install_tracker::record_mod_install(&project_id, &profile_id);
    Ok(true)
}

/// Best-effort update check for one installed mod — mods aren't installed with a stored
/// Modrinth project id, so this guesses one from the filename (same slug heuristic as
/// `mods_auto_fix`) and asks Modrinth for that project's latest version. A miss just means
/// "couldn't check" (mod not on Modrinth, or the guessed slug didn't resolve to a real
/// project), not "no update" — the frontend should say so rather than claiming up-to-date.
/// This is explicitly a manual fallback for when a mod's own auto-updater doesn't run, not a
/// system meant to be authoritative.
#[derive(serde::Serialize)]
pub struct ModUpdateInfo {
    #[serde(rename = "projectId")]
    pub project_id: String,
    #[serde(rename = "versionId")]
    pub version_id: String,
    #[serde(rename = "versionNumber")]
    pub version_number: String,
}

fn mod_slug_from_filename(file_name: &str) -> String {
    let base = file_name.trim_end_matches(".disabled").trim_end_matches(".jar");
    let re = regex::Regex::new(r"[-_]\d.*$").unwrap();
    re.replace(base, "").to_lowercase()
}

#[tauri::command]
pub async fn check_mod_update(profile_id: String, mod_id: String) -> Option<ModUpdateInfo> {
    let profile = profile_service::get_profile(&profile_id)?;
    let mods = mod_service::list_mods(&profile_id);
    let current = mods.iter().find(|m| m.id == mod_id)?;
    let slug = mod_slug_from_filename(&current.file_name);
    if slug.is_empty() {
        return None;
    }

    let loader = if profile.loader == "vanilla" { None } else { Some(profile.loader.as_str()) };
    let versions = modrinth_service::get_project_versions(&slug, Some(&profile.version), loader).await.ok()?;
    let latest = versions.first()?;
    let latest_file = latest.files.iter().find(|f| f.primary).or_else(|| latest.files.first())?;

    let current_base = current.file_name.trim_end_matches(".disabled");
    if latest_file.filename == current_base {
        return None;
    }

    Some(ModUpdateInfo { project_id: slug, version_id: latest.id.clone(), version_number: latest.version_number.clone() })
}

#[tauri::command]
pub async fn modrinth_swap_mod(profile_id: String, old_mod_id: String, project_id: String, version_id: String) -> Result<Vec<mod_service::ModInfo>, String> {
    let game_dir = profile_service::resolve_game_dir(&profile_id);
    let old_mod = mod_service::list_mods(&profile_id).into_iter().find(|m| m.id == old_mod_id);

    let new_file_name = modrinth_service::download_mod_version(&project_id, &version_id, &profile_id).await?;

    if let Some(old) = old_mod {
        let old_path = std::path::PathBuf::from(&old.file_path);
        mod_update_backup_service::record_update(&game_dir, &old_path, &old.id, &new_file_name, &old.name);
    }
    tokio::task::spawn_blocking({
        let profile_id = profile_id.clone();
        move || mod_service::delete_mod(&profile_id, &old_mod_id)
    })
    .await
    .ok();
    Ok(mods_list(profile_id).await)
}

#[tauri::command]
pub fn mods_recent_updates(profile_id: String) -> Vec<mod_update_backup_service::UpdateLogEntry> {
    mod_update_backup_service::list_recent(&profile_service::resolve_game_dir(&profile_id))
}

#[tauri::command]
pub fn mods_rollback_last_update(profile_id: String) -> Value {
    match mod_update_backup_service::rollback_last(&profile_service::resolve_game_dir(&profile_id)) {
        Ok(mod_name) => json!({ "ok": true, "modName": mod_name }),
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

/// Cross-profile update scan for the Mod Update Center — checks every mod in every profile
/// (sequentially; Modrinth's API doesn't have a batch-lookup-by-filename endpoint) and returns
/// only the ones that actually have an update, so the UI never has to filter a huge "up to date"
/// list itself.
#[tauri::command]
pub async fn mods_check_all_profiles_updates() -> Vec<Value> {
    let mut results = Vec::new();
    for profile in profile_service::list_profiles() {
        let mods = mod_service::list_mods(&profile.id);
        for m in mods {
            if let Some(update) = check_mod_update(profile.id.clone(), m.id.clone()).await {
                results.push(json!({
                    "profileId": profile.id,
                    "profileName": profile.name,
                    "modId": m.id,
                    "modName": m.name,
                    "update": update,
                }));
            }
        }
    }
    results
}

// ── System ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn system_get_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub async fn java_runtimes_list() -> Vec<crate::mcinstall::java_runtime::ManagedRuntime> {
    crate::mcinstall::java_runtime::list_managed(&paths::runtimes_dir()).await
}

/// Downloads (or verifies) one managed runtime and returns the path to its `java` binary.
#[tauri::command]
pub async fn java_runtime_install(component: String) -> Result<String, String> {
    use crate::mcinstall::java_runtime as jr;
    if !jr::MANAGED_COMPONENTS.iter().any(|(c, _, _)| *c == component) {
        return Err(format!("unknown runtime component: {component}"));
    }
    let client = auth_service::http_client();
    match jr::ensure_java_runtime(client, &component, &paths::runtimes_dir(), |_| {}).await? {
        Some(bin) => Ok(bin.to_string_lossy().to_string()),
        None => Err("Mojang does not offer this runtime for your platform".to_string()),
    }
}

#[tauri::command]
pub async fn java_runtime_remove(component: String) -> Result<(), String> {
    use crate::mcinstall::java_runtime as jr;
    // Refuse to delete the runtime the user pinned as their Java, or the next launch would fail.
    let pinned = settings_service::get_settings().game.default_java_path;
    let dir = paths::runtimes_dir().join(&component);
    if !pinned.is_empty() && std::path::Path::new(&pinned).starts_with(&dir) {
        return Err("This runtime is selected as your Java executable. Switch to Auto-detect first.".to_string());
    }
    jr::remove_managed(&paths::runtimes_dir(), &component).await
}

#[tauri::command]
pub async fn system_java_versions() -> Vec<java_discovery::JavaInfo> {
    java_discovery::scan_local_java().await
}

#[tauri::command]
pub fn system_open_external(app: AppHandle, url: String) -> Result<(), String> {
    // this gets called with chat links / mod metadata, so block anything that isn't a plain
    // http(s) link — a UNC path here would make Windows do an SMB auth handshake against it
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("refusing to open non-http(s) url".to_string());
    }
    tauri_plugin_opener::OpenerExt::opener(&app).open_url(url, None::<&str>).map_err(|e| e.to_string())
}

// ── Players ───────────────────────────────────────────────────────────────────

fn add_uuid_dashes(id: &str) -> String {
    if id.len() != 32 {
        return id.to_string();
    }
    format!("{}-{}-{}-{}-{}", &id[0..8], &id[8..12], &id[12..16], &id[16..20], &id[20..32])
}

async fn mojang_lookup(username: &str) -> Option<Value> {
    let res = auth_service::http_client()
        .get(format!("https://api.mojang.com/users/profiles/minecraft/{}", urlencoding::encode(username)))
        .send()
        .await
        .ok()?;
    if res.status() == 204 || res.status() == 404 {
        return None;
    }
    res.json::<Value>().await.ok()
}

#[tauri::command]
pub async fn players_lookup(username: String) -> Option<Value> {
    let mojang = mojang_lookup(&username).await?;
    let id = mojang["id"].as_str()?.to_string();
    let uuid = add_uuid_dashes(&id);
    let textures = fetch_player_textures(&id).await;
    Some(json!({ "uuid": uuid, "username": mojang["name"], "skinUrl": textures.0, "capeUrl": textures.1, "skinModel": textures.2 }))
}

async fn fetch_player_textures(uuid_no_dashes: &str) -> (Option<String>, Option<String>, String) {
    let Ok(res) = auth_service::http_client().get(format!("https://sessionserver.mojang.com/session/minecraft/profile/{uuid_no_dashes}")).send().await else {
        return (None, None, "default".to_string());
    };
    let Ok(profile) = res.json::<Value>().await else { return (None, None, "default".to_string()) };
    let Some(props) = profile["properties"].as_array() else { return (None, None, "default".to_string()) };
    let Some(tex_prop) = props.iter().find(|p| p["name"] == "textures") else { return (None, None, "default".to_string()) };
    let Some(b64) = tex_prop["value"].as_str() else { return (None, None, "default".to_string()) };
    let Ok(decoded) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64) else {
        return (None, None, "default".to_string());
    };
    let Ok(tex_json) = serde_json::from_slice::<Value>(&decoded) else { return (None, None, "default".to_string()) };
    let skin = &tex_json["textures"]["SKIN"];
    let cape = &tex_json["textures"]["CAPE"];
    let model = if skin["metadata"]["model"].as_str() == Some("slim") { "slim" } else { "default" };
    (skin["url"].as_str().map(|s| s.to_string()), cape["url"].as_str().map(|s| s.to_string()), model.to_string())
}

#[tauri::command]
pub async fn players_save_skin(skin_url: String, username: String) -> Result<String, String> {
    let dir = dirs::picture_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_default()).join("BejaClient");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = dir.join(format!("{username}_skin.png"));
    let bytes = auth_service::http_client().get(&skin_url).send().await.map_err(|e| e.to_string())?.bytes().await.map_err(|e| e.to_string())?;
    std::fs::write(&dest, bytes).map_err(|e| e.to_string())?;
    Ok(dest.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn players_fetch_image(url: String) -> String {
    let Ok(res) = auth_service::http_client().get(&url).header("User-Agent", "Mozilla/5.0 BejaClient").send().await else { return String::new() };
    if !res.status().is_success() {
        return String::new();
    }
    let mime = res.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("image/png").to_string();
    let Ok(bytes) = res.bytes().await else { return String::new() };
    format!("data:{mime};base64,{}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes))
}

#[tauri::command]
pub async fn players_beja_profile(uuid: String) -> Option<Value> {
    let token = beja_auth::get_beja_token().await?;
    let res = beja_api::get(&format!("/api/users/profile/{}", urlencoding::encode(&uuid)), Some(&token)).await;
    if res["error"].is_string() { None } else { Some(res) }
}

#[tauri::command]
pub async fn players_mc_created(uuid: String) -> Option<String> {
    let res = auth_service::http_client()
        .get(format!("https://api.ashcon.app/mojang/v2/user/{}", urlencoding::encode(&uuid)))
        .header("User-Agent", "BejaClient")
        .send()
        .await
        .ok()?;
    if !res.status().is_success() {
        return None;
    }
    let json: Value = res.json().await.ok()?;
    json["created_at"].as_str().map(|s| s.to_string())
}

#[tauri::command]
pub async fn players_search(query: String) -> Vec<Value> {
    let q = query.trim();
    if q.len() < 2 {
        return Vec::new();
    }
    let token = beja_auth::get_beja_token().await;
    let mojang_fut = mojang_lookup(q);
    let beja_fut = async {
        match &token {
            Some(t) => beja_api::get(&format!("/api/users/search?q={}", urlencoding::encode(q)), Some(t)).await,
            None => Value::Array(vec![]),
        }
    };
    let (mojang, beja_rows) = tokio::join!(mojang_fut, beja_fut);

    let mut results = Vec::new();
    if let Some(rows) = beja_rows.as_array() {
        for r in rows {
            results.push(json!({ "uuid": r["uuid"], "username": r["username"], "source": "beja" }));
        }
    }
    if let Some(m) = mojang {
        let id = m["id"].as_str().unwrap_or_default();
        if !results.iter().any(|r| r["uuid"].as_str().map(|u| u.replace('-', "")) == Some(id.to_string())) {
            results.push(json!({ "uuid": add_uuid_dashes(id), "username": m["name"], "source": "mojang" }));
        }
    }
    results
}

#[tauri::command]
pub async fn players_capes(uuid: String) -> Vec<Value> {
    const LABELS: &[(&str, &str)] = &[
        ("minecraft", "Vanilla"),
        ("optifine", "OptiFine"),
        ("minecraftcapes", "MinecraftCapes"),
        ("labymod", "LabyMod"),
        ("5zig", "5zig"),
        ("tlauncher", "TLauncher"),
        ("skinmc", "SkinMC"),
    ];
    let Ok(res) = auth_service::http_client().get(format!("https://api.capes.dev/load/{}", urlencoding::encode(&uuid))).header("User-Agent", "BejaClient").send().await else {
        return Vec::new();
    };
    if !res.status().is_success() {
        return Vec::new();
    }
    let Ok(json) = res.json::<Value>().await else { return Vec::new() };
    let Some(obj) = json.as_object() else { return Vec::new() };
    let mut capes = Vec::new();
    for (service, cape) in obj {
        if cape["exists"].as_bool().unwrap_or(false) {
            if let Some(image_url) = cape["imageUrl"].as_str() {
                let label = LABELS.iter().find(|(k, _)| k == service).map(|(_, v)| *v).unwrap_or(service.as_str());
                capes.push(json!({ "service": label, "capeUrl": image_url }));
            }
        }
    }
    capes
}

async fn mc_services_request(access_token: &str, method: reqwest::Method, path: &str, body: Option<Value>) -> Result<Value, String> {
    let mut req = auth_service::http_client().request(method, format!("https://api.minecraftservices.com{path}")).bearer_auth(access_token);
    if let Some(b) = &body {
        req = req.json(b);
    }
    let res = req.send().await.map_err(|e| e.to_string())?;
    let status = res.status();
    let text = res.text().await.map_err(|e| e.to_string())?;
    if status.as_u16() >= 400 {
        return Err(format!("Mojang API {path} → {status}: {}", text.chars().take(300).collect::<String>()));
    }
    Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)))
}

#[tauri::command]
pub async fn players_mc_profile(access_token: String) -> Option<Value> {
    mc_services_request(&access_token, reqwest::Method::GET, "/minecraft/profile", None).await.ok()
}

#[tauri::command]
pub async fn players_set_skin(access_token: String, url: String, variant: String) -> Result<Value, String> {
    mc_services_request(&access_token, reqwest::Method::POST, "/minecraft/profile/skins", Some(json!({ "variant": variant, "url": url }))).await
}

#[tauri::command]
pub async fn players_set_skin_file(access_token: String, base64_png: String, variant: String) -> Result<Value, String> {
    let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &base64_png).map_err(|e| e.to_string())?;
    let part = reqwest::multipart::Part::bytes(bytes).file_name("skin.png").mime_str("image/png").map_err(|e| e.to_string())?;
    let form = reqwest::multipart::Form::new().text("variant", variant).part("file", part);
    let res = auth_service::http_client()
        .post("https://api.minecraftservices.com/minecraft/profile/skins")
        .bearer_auth(&access_token)
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = res.status();
    let text = res.text().await.map_err(|e| e.to_string())?;
    if status.as_u16() >= 400 {
        return Err(format!("Mojang API POST /minecraft/profile/skins → {status}: {}", text.chars().take(300).collect::<String>()));
    }
    Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)))
}

#[tauri::command]
pub async fn players_set_cape(access_token: String, cape_id: String) -> Result<Value, String> {
    mc_services_request(&access_token, reqwest::Method::PUT, "/minecraft/profile/capes/active", Some(json!({ "capeId": cape_id }))).await
}

#[tauri::command]
pub async fn players_clear_cape(access_token: String) -> Result<Value, String> {
    mc_services_request(&access_token, reqwest::Method::DELETE, "/minecraft/profile/capes/active", None).await
}

// ── Friends ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn friends_connect(app: AppHandle) -> bool {
    let Some(token) = beja_auth::get_beja_token().await else { return false };
    beja_socket_service::connect_beja_socket(app, token).await;
    true
}

#[tauri::command]
pub async fn friends_disconnect() {
    beja_socket_service::disconnect_beja_socket().await;
}

#[tauri::command]
pub async fn friends_list() -> Vec<Value> {
    let Some(account) = auth_service::get_selected_account() else { return Vec::new() };
    let Some(token) = beja_auth::get_beja_token().await else { return Vec::new() };
    let res = beja_api::get(&format!("/api/friends/{}", account.uuid), Some(&token)).await;
    res.as_array().cloned().unwrap_or_default()
}

#[tauri::command]
pub async fn friends_request(username: String) -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "error": "not_logged_in" }) };
    let lookup = beja_api::get(&format!("/api/users/lookup/{}", urlencoding::encode(&username)), Some(&token)).await;
    if let Some(uuid) = lookup["uuid"].as_str() {
        return beja_api::post("/api/friends/request", Some(&token), json!({ "targetUuid": uuid })).await;
    }
    let Some(profile) = mojang_lookup(&username).await else { return json!({ "error": "not_found" }) };
    beja_api::post("/api/friends/request", Some(&token), json!({ "targetUuid": profile["id"], "targetUsername": profile["name"] })).await
}

#[tauri::command]
pub async fn friends_accept(uuid: String) -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "error": "not_logged_in" }) };
    beja_api::post("/api/friends/accept", Some(&token), json!({ "requesterUuid": uuid })).await
}

#[tauri::command]
pub async fn friends_remove(uuid: String) -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "error": "not_logged_in" }) };
    beja_api::delete(&format!("/api/friends/{uuid}"), Some(&token)).await
}

#[tauri::command]
pub async fn friends_block(uuid: String, username: String) -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "error": "not_logged_in" }) };
    beja_api::post("/api/friends/block", Some(&token), json!({ "targetUuid": uuid, "targetUsername": username })).await
}

#[tauri::command]
pub async fn friends_unblock(uuid: String) -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "error": "not_logged_in" }) };
    beja_api::delete(&format!("/api/friends/block/{uuid}"), Some(&token)).await
}

// ── Stats ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn stats_online() -> u64 {
    beja_api::get("/api/stats/online", None).await["count"].as_u64().unwrap_or(0)
}

// ── Lobby / party ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn lobby_emit(event: String, data: Value) {
    beja_socket_service::emit_lobby_event(&event, data);
}

#[tauri::command]
pub async fn launch_start_server(app: AppHandle, profile_id: String, server: String, port: u16) -> Result<(), String> {
    crate::console_window::send_console_clear(&app);
    crate::console_window::open_console_window(&app);
    // NOTE: extra `--server`/`--port` CLI args for direct-connect and the Discord-RPC /
    // presence-broadcast hookup around this launch aren't wired yet (tracked in the final
    // status report) — this currently launches the profile the same as a normal `launch_start`.
    let _ = (server, port);
    launch_service::launch_game(app, &profile_id).await
}

// ── Cosmetics ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn cosmetics_get(uuid: String) -> Value {
    beja_api::get(&format!("/api/cosmetics/{uuid}"), None).await
}

#[tauri::command]
pub async fn cosmetics_update(data: Value) -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "error": "not_logged_in" }) };
    beja_api::put("/api/cosmetics/", Some(&token), data).await
}

#[tauri::command]
pub async fn cosmetics_inventory(uuid: String) -> Value {
    beja_api::get(&format!("/api/cosmetics/inventory/{uuid}"), None).await
}

// ── Crates / crafting (still server-side mocked, same as the Electron build) ─

#[tauri::command]
pub async fn crates_list() -> Value {
    json!([{ "id": "test-crate", "name": "Test Crate", "description": "UI testing crate" }])
}

#[tauri::command]
pub async fn crates_open() -> Value {
    use rand::Rng;
    const POOL: &[(&str, &[(&str, &str, &str)])] = &[
        ("leather", &[("antlers", "Antlers", "hat"), ("leather-acc-1", "???", "accessory")]),
        ("iron", &[("wings", "Wings", "wings"), ("iron-acc-1", "???", "accessory")]),
        ("gold", &[("gold-acc-1", "???", "accessory")]),
        ("diamond", &[("diamond-acc-1", "???", "accessory")]),
        ("netherite", &[("netherite-acc-1", "???", "accessory")]),
        ("enchanted", &[("enchanted-acc-1", "???", "accessory")]),
    ];
    const WEIGHTS: &[(&str, u32)] = &[("leather", 50), ("iron", 26), ("gold", 13), ("diamond", 7), ("netherite", 3), ("enchanted", 1)];
    let mut rng = rand::thread_rng();
    let total: u32 = WEIGHTS.iter().map(|(_, w)| w).sum();
    let mut r = rng.gen_range(0..total) as i64;
    let mut rarity = "leather";
    for (name, w) in WEIGHTS {
        r -= *w as i64;
        if r <= 0 {
            rarity = name;
            break;
        }
    }
    let pool = POOL.iter().find(|(r, _)| *r == rarity).map(|(_, p)| *p).unwrap_or(&[]);
    let (id, name, ty) = pool[rng.gen_range(0..pool.len().max(1))];
    json!({ "cosmetic": { "id": id, "name": name, "type": ty, "rarity": rarity }, "is_new": rng.gen_bool(0.5) })
}

#[tauri::command]
pub async fn crates_keys() -> Value {
    json!({ "count": 99 })
}

#[tauri::command]
pub async fn crafting_inventory() -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!([]) };
    beja_api::get("/api/cosmetics/inventory", Some(&token)).await
}

#[tauri::command]
pub async fn crafting_combine(rarity: String) -> Value {
    if beja_auth::get_beja_token().await.is_none() {
        return json!({ "error": "not_logged_in" });
    }
    let chain = [("leather", "iron"), ("iron", "gold"), ("gold", "diamond"), ("diamond", "netherite")];
    let Some((_, next)) = chain.iter().find(|(r, _)| *r == rarity) else { return json!({ "error": "cannot_craft_enchanted" }) };
    json!({ "cosmetic": { "id": format!("forged-{}", chrono::Utc::now().timestamp_millis()), "name": "???", "type": "accessory", "rarity": next }, "is_new": true })
}

// ── Community capes ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn capes_list(offset: Option<u32>) -> Value {
    beja_api::get(&format!("/api/capes?limit=50&offset={}", offset.unwrap_or(0)), None).await
}

#[tauri::command]
pub async fn capes_upload(base64_image: String, filename: String, name: String) -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "error": "not_logged_in" }) };
    let Ok(bytes) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &base64_image) else {
        return json!({ "error": "invalid_image" });
    };
    let mime = if filename.to_lowercase().ends_with(".jpg") || filename.to_lowercase().ends_with(".jpeg") { "image/jpeg" } else { "image/png" };
    let Ok(part) = reqwest::multipart::Part::bytes(bytes).file_name(filename).mime_str(mime) else {
        return json!({ "error": "invalid_image" });
    };
    let form = reqwest::multipart::Form::new().text("name", name).part("image", part);
    match auth_service::http_client()
        .post(format!("{}/api/capes", auth_service::BEJA_API))
        .bearer_auth(&token)
        .multipart(form)
        .send()
        .await
    {
        Ok(res) => res.json().await.unwrap_or(Value::Null),
        Err(e) => json!({ "error": e.to_string() }),
    }
}


#[tauri::command]
pub async fn capes_report(id: u64) -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "error": "not_logged_in" }) };
    beja_api::post(&format!("/api/capes/{id}/report"), Some(&token), Value::Null).await
}

// ── Client Pass ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn pass_get() -> Value {
    beja_api::get("/api/pass", None).await
}

#[tauri::command]
pub async fn pass_progress() -> Value {
    let Some(token) = beja_auth::get_beja_token().await else {
        return json!({ "xp": 0, "current_tier": 0, "unlocked_cosmetics": [], "daily_available": false });
    };
    beja_api::get("/api/pass/progress", Some(&token)).await
}

#[tauri::command]
pub async fn pass_daily() -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "awarded": false }) };
    beja_api::post("/api/pass/daily", Some(&token), Value::Null).await
}

// ── Quests ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn quests_list() -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "week": "", "quests": [] }) };
    beja_api::get("/api/quests", Some(&token)).await
}

#[tauri::command]
pub async fn quests_progress(quest_id: String, amount: u32) -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return Value::Null };
    beja_api::post("/api/quests/progress", Some(&token), json!({ "questId": quest_id, "amount": amount })).await
}

#[tauri::command]
pub async fn quests_claim(quest_id: String) -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "awarded": false }) };
    beja_api::post("/api/quests/claim", Some(&token), json!({ "questId": quest_id })).await
}

#[tauri::command]
pub async fn quests_leaderboard() -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "entries": [], "myRank": null }) };
    beja_api::get("/api/quests/leaderboard", Some(&token)).await
}

// ── Wallet / shop ─────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn wallet_balance() -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "balance": 0 }) };
    beja_api::get("/api/wallet", Some(&token)).await
}

#[tauri::command]
pub async fn shop_list() -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "items": [], "owned": [] }) };
    beja_api::get("/api/shop", Some(&token)).await
}

#[tauri::command]
pub async fn shop_purchase(item_id: String) -> Value {
    let Some(token) = beja_auth::get_beja_token().await else { return json!({ "purchased": false, "reason": "no_token" }) };
    beja_api::post("/api/shop/purchase", Some(&token), json!({ "itemId": item_id })).await
}

// ── Install tracking ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn installs_get() -> install_tracker::TrackerData {
    install_tracker::get_installs()
}

// ── Servers ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn servers_list(app: AppHandle) -> Vec<server_ping_service::ServerStatus> {
    let list = server_ping_service::get_static_list();
    for s in list.clone() {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Ok(status) = server_ping_service::ping_server(&s.host, s.port).await {
                let _ = app.emit(
                    "servers:ping-result",
                    json!({
                        "id": s.id, "online": true, "favicon": status.favicon, "version": status.version,
                        "playersOnline": status.players_online, "playersMax": status.players_max,
                        "motd": status.motd, "ping": status.ping,
                    }),
                );
            }
        });
    }
    list
}

#[tauri::command]
pub async fn servers_ping(host: String, port: u16) -> Option<Value> {
    let status = server_ping_service::ping_server(&host, port).await.ok()?;
    Some(json!({
        "favicon": status.favicon, "version": status.version, "playersOnline": status.players_online,
        "playersMax": status.players_max, "motd": status.motd, "ping": status.ping,
    }))
}

#[tauri::command]
pub fn servers_add(host: String, port: u16, name: String) -> String {
    server_ping_service::add_server(&host, port, &name)
}

#[tauri::command]
pub fn servers_remove(id: String) -> bool {
    server_ping_service::remove_server(&id);
    true
}

#[tauri::command]
pub fn servers_add_to_profile(host: String, port: u16, name: String, favicon: Option<String>, profile_id: String) -> bool {
    let settings = settings_service::get_settings();
    let game_dir = profile_service::get_profile(&profile_id).map(|p| p.game_dir).filter(|d| !d.is_empty()).unwrap_or(settings.game.default_game_dir);
    server_ping_service::add_server_to_profile(&host, port, &name, favicon.as_deref(), std::path::Path::new(&game_dir));
    install_tracker::record_server_add(&host, port, &profile_id);
    true
}

// ── Chat ──────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn chat_send(to_uuid: String, content: String) {
    beja_socket_service::emit_lobby_event("chat:send", json!({ "toUuid": to_uuid, "content": content }));
}

#[tauri::command]
pub fn chat_typing(to_uuid: String) {
    beja_socket_service::emit_lobby_event("chat:typing", json!({ "toUuid": to_uuid }));
}

#[tauri::command]
pub async fn chat_history(target_uuid: String) -> Vec<Value> {
    let Some(token) = beja_auth::get_beja_token().await else { return Vec::new() };
    beja_api::get(&format!("/api/chat/history/{target_uuid}"), Some(&token)).await.as_array().cloned().unwrap_or_default()
}

#[tauri::command]
pub async fn giphy_search(query: Option<String>) -> Result<Vec<giphy_service::GifResult>, String> {
    giphy_service::search(query).await
}

// ── Discord Rich Presence ───────────────────────────────────────────────────────

#[tauri::command]
pub fn discord_set_presence(details: String, state: String) {
    crate::discord_rpc::set_context_presence(&details, &state);
}

// ── Video ─────────────────────────────────────────────────────────────────────

// ── Updater ───────────────────────────────────────────────────────────────────
// Thin wrappers around `tauri-plugin-updater` — see `lib.rs`'s `updater_check` helper for the
// actual check/download/install flow using the plugin's `Update` handle.

#[tauri::command]
pub async fn updater_check(app: AppHandle) -> Result<(), String> {
    crate::updater::check_for_update(app).await
}

#[tauri::command]
pub async fn updater_download(app: AppHandle) -> Result<(), String> {
    crate::updater::download_update(app).await
}

#[tauri::command]
pub fn updater_install(app: AppHandle) -> Result<(), String> {
    crate::updater::install_update(app)
}

use tauri_plugin_dialog::DialogExt;
