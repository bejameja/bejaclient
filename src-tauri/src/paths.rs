//! Filesystem path helpers.
//!
//! Electron's `app.getPath('userData')` resolves to `%APPDATA%\BejaClient` on Windows and
//! `~/.config/BejaClient` on Linux. `dirs::config_dir()` maps to the same OS folder on both
//! platforms (on Windows `config_dir()` == `data_dir()` == `%APPDATA%`), so existing installs
//! from the Electron build keep working unchanged — accounts.json, settings.json, profiles.json,
//! the beja-libs cache, etc. are all found in the same place after this migration.

use std::path::{Component, Path, PathBuf};

pub fn user_data_dir() -> PathBuf {
    let dir = dirs::config_dir()
        .expect("no config dir available on this platform")
        .join("BejaClient");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

/// Name of a secondary launcher instance (`BEJA_INSTANCE=<name>`), for testing with a second account side by
/// side. `None` for the normal launcher.
pub fn instance_name() -> Option<String> {
    let raw = std::env::var("BEJA_INSTANCE").ok()?;
    let name: String = raw.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
    if name.is_empty() { None } else { Some(name) }
}

/// Per-instance data: accounts, session, settings, profiles, servers, logs, the default game folder. The normal
/// launcher uses `user_data_dir()` itself; a secondary instance gets `<user_data_dir>/launcher-instances/<name>`.
/// Big downloaded caches (beja-libs, runtimes) stay shared in `user_data_dir()`.
pub fn instance_dir() -> PathBuf {
    let Some(name) = instance_name() else { return user_data_dir() };
    let dir = user_data_dir().join("launcher-instances").join(name);
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

pub fn logs_dir() -> PathBuf {
    let dir = instance_dir().join("logs");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

/// Default isolated instance dir (post 1.1.18 migration in the Electron app).
pub fn default_game_dir() -> PathBuf {
    instance_dir().join("instance")
}

/// Pre-1.1.18 default — BejaClient used to share the vanilla .minecraft folder.
/// Kept only so `settings.rs`'s one-time migration check can still detect it.
pub fn legacy_default_game_dir() -> PathBuf {
    if cfg!(target_os = "linux") {
        dirs::home_dir().expect("no home dir").join(".minecraft")
    } else {
        dirs::config_dir().expect("no config dir").join(".minecraft")
    }
}

pub fn beja_libs_dir() -> PathBuf {
    let dir = user_data_dir().join("beja-libs");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

pub fn runtimes_dir() -> PathBuf {
    user_data_dir().join("runtimes")
}

pub fn settings_path() -> PathBuf {
    instance_dir().join("settings.json")
}

pub fn profiles_path() -> PathBuf {
    instance_dir().join("profiles.json")
}

pub fn accounts_path() -> PathBuf {
    instance_dir().join("accounts.json")
}

pub fn session_path() -> PathBuf {
    instance_dir().join("session.json")
}

pub fn servers_path() -> PathBuf {
    instance_dir().join("servers.json")
}

/// Joins `base` with a zip-entry-style path, refusing anything that tries to `..` its way out
/// (zip-slip) — mrpack overrides and native jars are untrusted archives even when the download
/// itself came from a trusted host.
pub fn safe_join(base: &Path, rel: &str) -> Result<PathBuf, String> {
    let mut joined = base.to_path_buf();
    for part in Path::new(rel).components() {
        match part {
            Component::Normal(p) => joined.push(p),
            Component::CurDir => {}
            _ => return Err(format!("unsafe archive entry path: {rel}")),
        }
    }
    if !joined.starts_with(base) {
        return Err(format!("unsafe archive entry path: {rel}"));
    }
    Ok(joined)
}
