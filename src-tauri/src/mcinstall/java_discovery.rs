//! Port of `src/main/services/mcinstall/javaDiscovery.ts`.

use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[derive(Debug, Clone, serde::Serialize)]
pub struct JavaInfo {
    pub path: String,
    pub version: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
    #[serde(default)]
    pub arch: Option<String>,
}

fn glob_java_bin(base_dirs: &[&str], bin_relative: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for base in base_dirs {
        let Ok(entries) = std::fs::read_dir(base) else { continue };
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let candidate = entry.path().join(bin_relative);
            if candidate.exists() {
                found.push(candidate);
            }
        }
    }
    found
}

#[cfg(windows)]
async fn query_windows_registry() -> Vec<PathBuf> {
    let keys = [
        "HKLM\\SOFTWARE\\JavaSoft\\JDK",
        "HKLM\\SOFTWARE\\JavaSoft\\Java Development Kit",
        "HKLM\\SOFTWARE\\WOW6432Node\\JavaSoft\\JDK",
        "HKLM\\SOFTWARE\\WOW6432Node\\JavaSoft\\Java Development Kit",
        "HKCU\\SOFTWARE\\JavaSoft\\JDK",
    ];
    let mut found = Vec::new();
    let re = Regex::new(r"JavaHome\s+REG_SZ\s+(.+)").unwrap();
    for key in keys {
        let Ok(out) = Command::new("reg").args(["query", key, "/s", "/v", "JavaHome"]).output().await else { continue };
        let stdout = String::from_utf8_lossy(&out.stdout);
        for caps in re.captures_iter(&stdout) {
            let home = caps[1].trim();
            let candidate = Path::new(home).join("bin").join("java.exe");
            if candidate.exists() {
                found.push(candidate);
            }
        }
    }
    found
}

#[cfg(windows)]
async fn get_windows_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(out) = Command::new("where").arg("java").output().await {
        let stdout = String::from_utf8_lossy(&out.stdout);
        candidates.extend(stdout.lines().map(|l| PathBuf::from(l.trim())).filter(|p| !p.as_os_str().is_empty()));
    }

    candidates.extend(query_windows_registry().await);

    candidates.extend(glob_java_bin(
        &[
            "C:\\Program Files\\Java",
            "C:\\Program Files\\Eclipse Adoptium",
            "C:\\Program Files\\Microsoft",
            "C:\\Program Files\\Zulu",
            "C:\\Program Files (x86)\\Java",
        ],
        "bin/java.exe",
    ));

    if let Ok(home) = std::env::var("JAVA_HOME") {
        let candidate = Path::new(&home).join("bin").join("java.exe");
        if candidate.exists() {
            candidates.push(candidate);
        }
    }

    candidates
}

#[cfg(all(unix, not(target_os = "macos")))]
async fn get_linux_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(out) = Command::new("which").arg("java").output().await {
        let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !p.is_empty() {
            candidates.push(PathBuf::from(p));
        }
    }

    if let Ok(out) = Command::new("update-alternatives").args(["--list", "java"]).output().await {
        let stdout = String::from_utf8_lossy(&out.stdout);
        candidates.extend(stdout.lines().map(|l| PathBuf::from(l.trim())).filter(|p| !p.as_os_str().is_empty()));
    }

    candidates.extend(glob_java_bin(&["/usr/lib/jvm"], "bin/java"));

    if let Ok(home) = std::env::var("JAVA_HOME") {
        let candidate = Path::new(&home).join("bin").join("java");
        if candidate.exists() {
            candidates.push(candidate);
        }
    }

    candidates
}

#[cfg(target_os = "macos")]
async fn get_mac_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(out) = Command::new("/usr/libexec/java_home").arg("-V").output().await {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let re = Regex::new(r"(/Library/Java/JavaVirtualMachines/\S+/Contents/Home)").unwrap();
        for caps in re.captures_iter(&stderr) {
            let candidate = Path::new(&caps[1]).join("bin").join("java");
            if candidate.exists() {
                candidates.push(candidate);
            }
        }
    }
    candidates.extend(glob_java_bin(&["/Library/Java/JavaVirtualMachines"], "Contents/Home/bin/java"));
    candidates
}

fn parse_java_version_output(path: &str, output: &str) -> Option<JavaInfo> {
    let version_re = Regex::new(r#"version "([^"]+)""#).unwrap();
    let version = version_re.captures(output)?.get(1)?.as_str().to_string();

    let major_version = if let Some(rest) = version.strip_prefix("1.") {
        // Legacy Java 8 and earlier report "1.8.0_392" — major version is the 2nd component.
        rest.split('.').next().and_then(|s| s.parse().ok()).unwrap_or(0)
    } else {
        version.split('.').next().and_then(|s| s.parse().ok()).unwrap_or(0)
    };
    if major_version == 0 {
        return None;
    }

    let arch_re = Regex::new(r"(64|32)-Bit").unwrap();
    let arch = arch_re.captures(output).map(|c| format!("{}-bit", &c[1]));

    Some(JavaInfo { path: path.to_string(), version, major_version, arch })
}

async fn probe_java(path: &Path) -> Option<JavaInfo> {
    let out = Command::new(path).arg("-version").output().await.ok()?;
    // Java prints version info to stderr, not stdout.
    let text = if !out.stderr.is_empty() {
        String::from_utf8_lossy(&out.stderr).to_string()
    } else {
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    parse_java_version_output(&path.to_string_lossy(), &text)
}

/// Enumerates local Java installations. Zero-arg — does its own candidate discovery.
pub async fn scan_local_java() -> Vec<JavaInfo> {
    #[cfg(windows)]
    let candidates = get_windows_candidates().await;
    #[cfg(target_os = "macos")]
    let candidates = get_mac_candidates().await;
    #[cfg(all(unix, not(target_os = "macos")))]
    let candidates = get_linux_candidates().await;

    let mut seen = HashSet::new();
    let mut results = Vec::new();
    for candidate in candidates {
        let key = candidate.to_string_lossy().to_string();
        if key.trim().is_empty() || !seen.insert(key) {
            continue;
        }
        if let Some(info) = probe_java(&candidate).await {
            results.push(info);
        }
    }
    results
}
