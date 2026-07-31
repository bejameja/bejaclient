//! Port of `src/main/services/mcinstall/platform.ts`.

use super::types::Platform;
use once_cell::sync::OnceCell;

static CACHED: OnceCell<Platform> = OnceCell::new();

pub fn get_current_platform() -> Platform {
    CACHED
        .get_or_init(|| {
            let name = if cfg!(target_os = "windows") {
                "windows"
            } else if cfg!(target_os = "macos") {
                "osx"
            } else {
                "linux"
            };
            let arch = match std::env::consts::ARCH {
                "x86_64" => "x64",
                "aarch64" => "arm64",
                other => other,
            };
            Platform {
                name: name.to_string(),
                version: os_version_string(),
                arch: arch.to_string(),
            }
        })
        .clone()
}

fn os_version_string() -> String {
    // Best-effort — only ever matched against version regexes in a handful of legacy
    // library rules (e.g. old LWJGL Windows-XP exclusions), so an approximate release
    // string is fine; unlike Node's `os.release()` there's no single stdlib call for this.
    std::env::var("OS_VERSION_HINT").unwrap_or_else(|_| "0.0.0".to_string())
}
