//! Port of `src/main/services/mcinstall/javaRuntime.ts`.

use super::downloader::{download_file, DownloadOptions};
use super::platform::get_current_platform;
use super::types::Platform;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::fs;

const RUNTIME_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

#[derive(Debug, Serialize, Deserialize)]
struct RuntimeInfo {
    #[serde(rename = "javaBinRelPath")]
    java_bin_rel_path: String,
}

async fn https_get_json(client: &reqwest::Client, url: &str) -> Result<Value, String> {
    let res = client
        .get(url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    res.json::<Value>().await.map_err(|e| e.to_string())
}

fn runtime_platform_key(platform: &Platform) -> &'static str {
    match platform.name.as_str() {
        "windows" => match platform.arch.as_str() {
            "arm64" => "windows-arm64",
            "x86" | "ia32" => "windows-x86",
            _ => "windows-x64",
        },
        "osx" => {
            if platform.arch == "arm64" {
                "mac-os-arm64"
            } else {
                "mac-os"
            }
        }
        _ => {
            if platform.arch == "x86" || platform.arch == "ia32" {
                "linux-i386"
            } else {
                "linux"
            }
        }
    }
}

/// Maps a Minecraft version id to the Mojang java-runtime component to use when the version JSON
/// doesn't carry `javaVersion.component`. Cutoffs: 1.20.5+ needs Java 21, 1.18-1.20.4 needs 17,
/// 1.17 needs 16, everything earlier needs 8. Non "1.x" ids are newer than any 1.x release.
pub fn guess_runtime_component(mc_version: &str) -> String {
    let re = Regex::new(r"^1\.(\d+)(?:\.(\d+))?").unwrap();
    let Some(caps) = re.captures(mc_version) else { return "java-runtime-delta".to_string() };
    let minor: u32 = caps.get(1).map(|m| m.as_str().parse().unwrap_or(0)).unwrap_or(0);
    let patch: u32 = caps.get(2).map(|m| m.as_str().parse().unwrap_or(0)).unwrap_or(0);
    if minor > 20 || (minor == 20 && patch >= 5) {
        "java-runtime-delta".to_string()
    } else if minor >= 18 {
        "java-runtime-gamma".to_string()
    } else if minor == 17 {
        "java-runtime-alpha".to_string()
    } else {
        "jre-legacy".to_string()
    }
}

fn info_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join(".runtime-info.json")
}

async fn read_cached_java_bin(runtime_dir: &Path) -> Option<PathBuf> {
    let raw = fs::read_to_string(info_path(runtime_dir)).await.ok()?;
    let info: RuntimeInfo = serde_json::from_str(&raw).ok()?;
    let java_bin = runtime_dir.join(info.java_bin_rel_path.replace('/', std::path::MAIN_SEPARATOR_STR));
    if fs::metadata(&java_bin).await.is_ok() {
        Some(java_bin)
    } else {
        None
    }
}

/// Ensures a Mojang-provided Java runtime for `component` is present under
/// `runtimes_dir/<component>/`, downloading it on first use. Returns the path to the `java`
/// executable, or `None` if this component/platform combo isn't offered (caller falls back to
/// system Java).
pub async fn ensure_java_runtime(
    client: &reqwest::Client,
    component: &str,
    runtimes_dir: &Path,
    on_log: impl Fn(String),
) -> Result<Option<PathBuf>, String> {
    let runtime_dir = runtimes_dir.join(component);

    if let Some(cached) = read_cached_java_bin(&runtime_dir).await {
        return Ok(Some(cached));
    }

    let platform = get_current_platform();
    let plat_key = runtime_platform_key(&platform);

    let manifest = https_get_json(client, RUNTIME_MANIFEST_URL).await?;
    let entry = &manifest[plat_key][component][0];
    if entry.is_null() {
        return Ok(None);
    }
    let manifest_url = entry["manifest"]["url"].as_str().ok_or("malformed runtime manifest entry")?;
    let version_name = entry["version"]["name"].as_str().unwrap_or("?");

    on_log(format!("[Java] Downloading managed runtime {component} ({version_name})…"));
    let files_manifest = https_get_json(client, manifest_url).await?;
    let files = files_manifest["files"].as_object().ok_or("malformed runtime files manifest")?;

    let java_rel_path = files
        .keys()
        .find(|p| p.ends_with("bin/java") || p.ends_with("bin/java.exe"))
        .cloned();
    let Some(java_rel_path) = java_rel_path else { return Ok(None) };

    for (rel_path, file) in files {
        let dest = runtime_dir.join(rel_path.replace('/', std::path::MAIN_SEPARATOR_STR));
        let file_type = file["type"].as_str().unwrap_or("");
        match file_type {
            "directory" => {
                fs::create_dir_all(&dest).await.map_err(|e| e.to_string())?;
            }
            "link" => {
                let Some(target) = file["target"].as_str() else { continue };
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
                }
                #[cfg(windows)]
                {
                    let _ = std::os::windows::fs::symlink_file(target, &dest);
                }
                #[cfg(not(windows))]
                {
                    let _ = std::os::unix::fs::symlink(target, &dest);
                }
            }
            _ => {
                let Some(raw) = file["downloads"]["raw"].as_object() else { continue };
                let url = raw["url"].as_str().unwrap_or_default();
                let sha1 = raw["sha1"].as_str().map(|s| s.to_string());
                download_file(client, url, &dest, DownloadOptions { sha1 }).await?;
                #[cfg(not(windows))]
                {
                    if file["executable"].as_bool().unwrap_or(false) {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(meta) = fs::metadata(&dest).await {
                            let mut perms = meta.permissions();
                            perms.set_mode(0o755);
                            let _ = fs::set_permissions(&dest, perms).await;
                        }
                    }
                }
            }
        }
    }

    let java_bin = runtime_dir.join(java_rel_path.replace('/', std::path::MAIN_SEPARATOR_STR));
    if fs::metadata(&java_bin).await.is_err() {
        return Ok(None);
    }

    let info = RuntimeInfo { java_bin_rel_path: java_rel_path };
    if let Ok(json) = serde_json::to_string(&info) {
        let _ = fs::write(info_path(&runtime_dir), json).await;
    }
    on_log(format!("[Java] Runtime {component} ready."));
    Ok(Some(java_bin))
}
