//! Port of `src/main/services/clientUpdateService.ts`.

use regex::Regex;
use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::paths;

const VERSION_JSON_BASE: &str = "http://206.217.141.184:3093/api/jars/current";

/// Checks that a file ends with a ZIP end-of-central-directory signature (PK\x05\x06). Cheap
/// way to detect truncated/corrupt jar downloads without a full zip parser.
pub fn has_valid_zip_end(file_path: &Path) -> bool {
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

fn compute_sha256(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Some(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

pub fn get_downloaded_libs_dir() -> PathBuf {
    paths::beja_libs_dir()
}

fn find_jar(dirs: &[PathBuf], matches: impl Fn(&str) -> bool) -> Option<PathBuf> {
    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(dir) else { continue };
        let mut jars: Vec<String> = entries.flatten().filter_map(|e| e.file_name().into_string().ok()).filter(|f| matches(f)).collect();
        jars.sort();
        jars.reverse();
        if let Some(f) = jars.into_iter().next() {
            return Some(dir.join(f));
        }
    }
    None
}

fn candidate_dirs() -> Vec<PathBuf> {
    vec![get_downloaded_libs_dir(), bundled_beja_libs_dir()]
}

fn bundled_beja_libs_dir() -> PathBuf {
    // Packaged builds carry `resources/beja-libs/` next to the executable (see
    // tauri.conf.json's `bundle.resources`); dev runs fall back to the project's own copy.
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("beja-libs")))
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("resources/beja-libs"))
}

/// Finds the bootstrap JAR (the Java agent). Checks the downloaded (auto-update) dir first, then
/// the bundled resources dir.
pub fn resolve_bootstrap_jar() -> Option<PathBuf> {
    find_jar(&candidate_dirs(), |f| f.starts_with("beja-bootstrap-") && f.ends_with(".jar"))
}

/// Maps a Minecraft version string to the correct adapter JAR prefix. The 1.20 line is split
/// across two adapter modules built against different Yarn mappings.
pub fn adapter_jar_prefix(mc_version: &str) -> String {
    let year_re = Regex::new(r"^(\d+)\.").unwrap();
    if let Some(caps) = year_re.captures(mc_version) {
        if caps[1].parse::<u32>().unwrap_or(0) >= 26 {
            return "beja-v1_26".to_string();
        }
    }

    let re = Regex::new(r"^1\.(\d+)(?:\.(\d+))?").unwrap();
    let Some(caps) = re.captures(mc_version) else { return "beja-v1_21".to_string() };
    let minor: u32 = caps.get(1).map(|m| m.as_str().parse().unwrap_or(99)).unwrap_or(99);
    let patch: u32 = caps.get(2).map(|m| m.as_str().parse().unwrap_or(0)).unwrap_or(0);
    if minor >= 21 {
        "beja-v1_21".to_string()
    } else if minor == 20 {
        if patch >= 5 { "beja-v1_20".to_string() } else { "beja-v1_20_pre".to_string() }
    } else if minor >= 19 {
        "beja-v1_19".to_string()
    } else if minor >= 18 {
        "beja-v1_18".to_string()
    } else if minor >= 16 {
        "beja-v1_16".to_string()
    } else {
        "beja-v1_21".to_string()
    }
}

/// Finds the adapter JAR for a given MC version (beja-v1_21-*.jar, beja-v1_20-*.jar, …). Falls
/// back to any beja-v* JAR if no version-specific one is found.
pub fn resolve_adapter_jar(mc_version: Option<&str>) -> Option<PathBuf> {
    let prefix = mc_version.map(|v| adapter_jar_prefix(v));
    let is_adapter = |f: &str, p: &Option<String>| {
        let starts = p.as_deref().map(|p| f.starts_with(p)).unwrap_or_else(|| f.starts_with("beja-v"));
        starts && f.ends_with(".jar") && !f.contains("-sources") && !f.contains("-dev")
    };

    if let Some(found) = find_jar(&candidate_dirs(), |f| is_adapter(f, &prefix)) {
        return Some(found);
    }
    // Fallback: any adapter JAR when version-specific one is missing.
    if prefix.is_some() {
        return resolve_adapter_jar(None);
    }
    None
}

fn get_local_version(dir: &Path) -> Option<String> {
    if !dir.exists() {
        return None;
    }
    let re = Regex::new(r"beja-bootstrap-(.+)\.jar$").unwrap();
    let entries = std::fs::read_dir(dir).ok()?;
    let mut jars: Vec<String> = entries.flatten().filter_map(|e| e.file_name().into_string().ok()).filter(|f| f.starts_with("beja-bootstrap-") && f.ends_with(".jar")).collect();
    jars.sort();
    jars.reverse();
    let first = jars.first()?;
    re.captures(first).map(|c| c[1].to_string())
}

fn semver_gt(remote: &str, local: &str) -> bool {
    let r: Vec<i64> = remote.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    let l: Vec<i64> = local.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    for i in 0..r.len().max(l.len()) {
        let rv = r.get(i).copied().unwrap_or(0);
        let lv = l.get(i).copied().unwrap_or(0);
        if rv > lv {
            return true;
        }
        if rv < lv {
            return false;
        }
    }
    false
}

fn family_for(mc_version: Option<&str>) -> String {
    let prefix = mc_version.map(|v| adapter_jar_prefix(v)).unwrap_or_else(|| "beja-v1_21".to_string());
    prefix.trim_start_matches("beja-").to_string()
}

/// Fetches client-version.json from the Beja API, compares with the locally installed bootstrap
/// JAR, and downloads the new version if outdated. Non-fatal — if anything fails the launcher
/// continues with whatever JAR is present.
pub async fn check_and_update_client_jar(client: &reqwest::Client, on_log: &impl Fn(String), on_status: &impl Fn(&str), mc_version: Option<&str>) {
    if let Err(e) = check_and_update_client_jar_inner(client, on_log, on_status, mc_version).await {
        on_log(format!("[BejaClient] Update check skipped: {e}"));
    }
}

async fn check_and_update_client_jar_inner(client: &reqwest::Client, on_log: &impl Fn(String), on_status: &impl Fn(&str), mc_version: Option<&str>) -> Result<(), String> {
    let family = family_for(mc_version);
    on_log(format!("[BejaClient] Checking for client updates… (family: {family})"));

    let remote: serde_json::Value = client
        .get(format!("{VERSION_JSON_BASE}?family={family}"))
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let remote_version = remote["version"].as_str().unwrap_or_default();
    let remote_url = remote["url"].as_str().unwrap_or_default();
    if remote_version.is_empty() || remote_url.is_empty() || remote_version == "0.0.0" {
        on_log(format!("[BejaClient] No released client version yet for {family} — skipping update."));
        return Ok(());
    }

    let dl_dir = get_downloaded_libs_dir();
    let local_ver = get_local_version(&dl_dir);
    on_log(format!("[BejaClient] Local: {}  |  Remote: {remote_version}", local_ver.as_deref().unwrap_or("none")));

    let local_bootstrap_valid = local_ver.as_ref().map(|v| has_valid_zip_end(&dl_dir.join(format!("beja-bootstrap-{v}.jar")))).unwrap_or(false);
    let local_adapter_jar = resolve_adapter_jar(mc_version);
    let local_adapter_valid = local_adapter_jar.as_ref().map(|p| has_valid_zip_end(p)).unwrap_or(false);

    if let Some(local_ver) = &local_ver {
        if !semver_gt(remote_version, local_ver) {
            if local_bootstrap_valid && local_adapter_valid {
                on_log("[BejaClient] Client JAR is up to date.".to_string());
                return Ok(());
            }
            on_log("[BejaClient] Local client JAR appears corrupted — re-downloading current version.".to_string());
        }
    }

    let _ = std::fs::create_dir_all(&dl_dir);

    on_status(&format!("Downloading BejaClient {remote_version}…"));
    on_log(format!("[BejaClient] Downloading {}…", remote["filename"].as_str().unwrap_or_default()));

    if let Ok(entries) = std::fs::read_dir(&dl_dir) {
        for f in entries.flatten().filter_map(|e| e.file_name().into_string().ok()).filter(|f| f.starts_with("beja-bootstrap-") && f.ends_with(".jar")) {
            let _ = std::fs::remove_file(dl_dir.join(f));
        }
    }

    // Use canonical version-based filename regardless of what the server reports — the
    // server's filename field can lag behind the version field.
    let dest = dl_dir.join(format!("beja-bootstrap-{remote_version}.jar"));
    crate::mcinstall::downloader::download_file(client, remote_url, &dest, crate::mcinstall::downloader::DownloadOptions::default()).await?;

    if !has_valid_zip_end(&dest) {
        let _ = std::fs::remove_file(&dest);
        return Err("Downloaded bootstrap JAR is not a valid zip (truncated download?)".to_string());
    }

    if let Some(expected_sha256) = remote["sha256"].as_str() {
        on_log("[BejaClient] Verifying bootstrap checksum…".to_string());
        let actual = compute_sha256(&dest).unwrap_or_default();
        if !actual.eq_ignore_ascii_case(expected_sha256) {
            let _ = std::fs::remove_file(&dest);
            return Err(format!("Bootstrap checksum mismatch — expected {expected_sha256} got {actual}"));
        }
        on_log("[BejaClient] Bootstrap checksum OK.".to_string());
    }

    // Adapter JAR — must live next to bootstrap so AdapterLocator finds it.
    if let (Some(adapter_url), Some(adapter_filename)) = (remote["adapterUrl"].as_str(), remote["adapterFilename"].as_str()) {
        on_log(format!("[BejaClient] Downloading {adapter_filename}…"));

        if let Ok(entries) = std::fs::read_dir(&dl_dir) {
            for f in entries.flatten().filter_map(|e| e.file_name().into_string().ok()).filter(|f| f.starts_with("beja-v") && f.ends_with(".jar") && !f.contains("-sources") && !f.contains("-dev")) {
                let _ = std::fs::remove_file(dl_dir.join(f));
            }
        }

        let adapter_dest = dl_dir.join(adapter_filename);
        crate::mcinstall::downloader::download_file(client, adapter_url, &adapter_dest, crate::mcinstall::downloader::DownloadOptions::default()).await?;

        if !has_valid_zip_end(&adapter_dest) {
            let _ = std::fs::remove_file(&adapter_dest);
            return Err("Downloaded adapter JAR is not a valid zip (truncated download?)".to_string());
        }

        if let Some(expected_sha256) = remote["adapterSha256"].as_str() {
            on_log("[BejaClient] Verifying adapter checksum…".to_string());
            let actual = compute_sha256(&adapter_dest).unwrap_or_default();
            if !actual.eq_ignore_ascii_case(expected_sha256) {
                let _ = std::fs::remove_file(&adapter_dest);
                return Err(format!("Adapter checksum mismatch — expected {expected_sha256} got {actual}"));
            }
            on_log("[BejaClient] Adapter checksum OK.".to_string());
        }

        on_log(format!("[BejaClient] Adapter downloaded → {}", adapter_dest.to_string_lossy()));
    } else {
        on_log("[BejaClient] No adapter URL in version manifest — adapter not updated.".to_string());
    }

    on_log(format!("[BejaClient] Update complete → {}", dest.to_string_lossy()));
    Ok(())
}
