//! Port of `src/main/services/mcinstall/forgeInstall.ts`.
//!
//! Forge/NeoForge ship their own Java-based installer (`--installClient <gameDir>`) that does
//! all the version-JSON generation, library patching, and processor steps itself — so unlike
//! Fabric/Quilt (which just need a generated version JSON), this only needs to run that jar and
//! detect which new `versions/<id>/` directory it produced.

use super::downloader::{download_file, DownloadOptions};
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

fn has_valid_zip_end(file_path: &Path) -> bool {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut file) = std::fs::File::open(file_path) else { return false };
    let Ok(len) = file.metadata().map(|m| m.len()) else { return false };
    if len < 22 {
        return false;
    }
    let buf_size = len.min(4096) as usize;
    if file.seek(SeekFrom::Start(len - buf_size as u64)).is_err() {
        return false;
    }
    let mut buf = vec![0u8; buf_size];
    if file.read_exact(&mut buf).is_err() {
        return false;
    }
    for i in (0..=buf.len().saturating_sub(22)).rev() {
        if buf[i] == 0x50 && buf[i + 1] == 0x4b && buf[i + 2] == 0x05 && buf[i + 3] == 0x06 {
            return true;
        }
    }
    false
}

fn installer_cache_dir(game_dir: &Path) -> PathBuf {
    let dir = game_dir.join(".bejaclient").join("installers");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn list_version_dirs(game_dir: &Path) -> HashSet<String> {
    let versions_dir = game_dir.join("versions");
    let Ok(entries) = std::fs::read_dir(&versions_dir) else { return HashSet::new() };
    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect()
}

/// The Forge/NeoForge installer's client-install mode refuses to run unless `game_dir` already
/// has a `launcher_profiles.json` — a leftover check from when it patched the vanilla launcher's
/// own profile list. Fails with "There is no minecraft launcher profile in ... you need to run
/// the launcher first!" and exits 0 new version folders. BejaClient never writes this file
/// itself (nor does the CurseForge App / Modrinth App / Lunar Client, whose instance folders get
/// reused in place for imported profiles), so every fresh `game_dir` hits this — write a minimal
/// stub the installer accepts if one isn't already there.
fn ensure_launcher_profiles_stub(game_dir: &Path) {
    let path = game_dir.join("launcher_profiles.json");
    if path.exists() {
        return;
    }
    let stub = serde_json::json!({
        "profiles": {},
        "selectedProfile": null,
        "authenticationDatabase": {},
        "clientToken": uuid::Uuid::new_v4().to_string(),
        "launcherVersion": { "name": "BejaClient", "format": 21 },
    });
    if let Ok(json) = serde_json::to_string_pretty(&stub) {
        let _ = std::fs::write(&path, json);
    }
}

async fn run_installer_jar(installer_jar_path: &Path, game_dir: &Path, java_path: &str, on_log: &impl Fn(String)) -> Result<String, String> {
    if !has_valid_zip_end(installer_jar_path) {
        let _ = std::fs::remove_file(installer_jar_path);
        return Err("Downloaded installer jar is corrupted (truncated download) — retry the install.".to_string());
    }

    ensure_launcher_profiles_stub(game_dir);

    let before = list_version_dirs(game_dir);

    let mut child = Command::new(java_path)
        .args(["-jar", &installer_jar_path.to_string_lossy(), "--installClient", &game_dir.to_string_lossy()])
        .current_dir(game_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    // Read stdout/stderr concurrently in this same task (no `tokio::spawn`, so `on_log`'s
    // borrow doesn't need to satisfy a `'static` bound) until the process exits.
    let mut stdout_lines = child.stdout.take().map(|s| BufReader::new(s).lines());
    let mut stderr_lines = child.stderr.take().map(|s| BufReader::new(s).lines());

    let status = loop {
        tokio::select! {
            line = async { stdout_lines.as_mut().unwrap().next_line().await }, if stdout_lines.is_some() => {
                match line {
                    Ok(Some(l)) => on_log(l),
                    _ => stdout_lines = None,
                }
            }
            line = async { stderr_lines.as_mut().unwrap().next_line().await }, if stderr_lines.is_some() => {
                match line {
                    Ok(Some(l)) => on_log(l),
                    _ => stderr_lines = None,
                }
            }
            status = child.wait(), if stdout_lines.is_none() && stderr_lines.is_none() => {
                break status.map_err(|e| e.to_string())?;
            }
        }
    };

    let after = list_version_dirs(game_dir);
    let new_versions: Vec<&String> = after.difference(&before).collect();

    if !status.success() || new_versions.is_empty() {
        return Err(format!(
            "Installer exited with code {} and produced {} new version folder(s). Check the log above for the underlying Java error.",
            status.code().unwrap_or(-1),
            new_versions.len()
        ));
    }

    Ok(new_versions[0].clone())
}

pub async fn install_forge(client: &reqwest::Client, mc_version: &str, forge_version: &str, game_dir: &Path, java_path: &str, on_log: &impl Fn(String)) -> Result<String, String> {
    let url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{mc_version}-{forge_version}/forge-{mc_version}-{forge_version}-installer.jar");
    let dest = installer_cache_dir(game_dir).join(format!("forge-{mc_version}-{forge_version}-installer.jar"));
    download_file(client, &url, &dest, DownloadOptions::default()).await?;
    run_installer_jar(&dest, game_dir, java_path, on_log).await
}

pub async fn install_neoforge(client: &reqwest::Client, neoforge_version: &str, game_dir: &Path, java_path: &str, on_log: &impl Fn(String)) -> Result<String, String> {
    let url = format!("https://maven.neoforged.net/releases/net/neoforged/neoforge/{neoforge_version}/neoforge-{neoforge_version}-installer.jar");
    let dest = installer_cache_dir(game_dir).join(format!("neoforge-{neoforge_version}-installer.jar"));
    download_file(client, &url, &dest, DownloadOptions::default()).await?;
    run_installer_jar(&dest, game_dir, java_path, on_log).await
}

/// NeoForge versions are bare (e.g. "21.1.235"), not MC-prefixed like Forge's
/// promotions_slim.json. Convention: version "X.Y.*" supports MC "1.X.Y". Empty result for an
/// unsupported MC version is a legitimate case, not an error.
pub async fn list_neoforge_versions(client: &reqwest::Client, mc_version: &str) -> Result<Vec<String>, String> {
    let re = Regex::new(r"^1\.(\d+)(?:\.(\d+))?$").unwrap();
    let Some(caps) = re.captures(mc_version) else { return Ok(Vec::new()) };
    let major = &caps[1];
    let minor = caps.get(2).map(|m| m.as_str()).unwrap_or("0");
    let prefix = format!("{major}.{minor}.");

    let xml = client
        .get("https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml")
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let version_re = Regex::new(r"<version>([^<]+)</version>").unwrap();
    Ok(version_re
        .captures_iter(&xml)
        .map(|c| c[1].to_string())
        .filter(|v| v.starts_with(&prefix))
        .collect())
}
