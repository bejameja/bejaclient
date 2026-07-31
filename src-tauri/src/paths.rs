//! Filesystem path helpers.
//!
//! Electron's `app.getPath('userData')` resolves to `%APPDATA%\BejaClient` on Windows and
//! `~/.config/BejaClient` on Linux. `dirs::config_dir()` maps to the same OS folder on both
//! platforms (on Windows `config_dir()` == `data_dir()` == `%APPDATA%`), so existing installs
//! from the Electron build keep working unchanged — accounts.json, settings.json, profiles.json,
//! the beja-libs cache, etc. are all found in the same place after this migration.

use std::path::PathBuf;

pub fn user_data_dir() -> PathBuf {
    let dir = dirs::config_dir()
        .expect("no config dir available on this platform")
        .join("BejaClient");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

pub fn logs_dir() -> PathBuf {
    let dir = user_data_dir().join("logs");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

/// Default isolated instance dir (post 1.1.18 migration in the Electron app).
pub fn default_game_dir() -> PathBuf {
    user_data_dir().join("instance")
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
    user_data_dir().join("settings.json")
}

pub fn profiles_path() -> PathBuf {
    user_data_dir().join("profiles.json")
}

pub fn accounts_path() -> PathBuf {
    user_data_dir().join("accounts.json")
}

pub fn session_path() -> PathBuf {
    user_data_dir().join("session.json")
}

pub fn servers_path() -> PathBuf {
    user_data_dir().join("servers.json")
}
