//! Port of `src/main/services/settingsService.ts`.
//! JSON-backed app settings with an in-memory cache and one-time legacy-dir migration.

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

use crate::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    #[serde(rename = "defaultGameDir")]
    pub default_game_dir: String,
    #[serde(rename = "defaultJavaPath")]
    pub default_java_path: String,
    #[serde(rename = "minRam")]
    pub min_ram: u32,
    #[serde(rename = "maxRam")]
    pub max_ram: u32,
    #[serde(rename = "jvmArgs")]
    pub jvm_args: String,
    pub resolution: Resolution,
    pub fullscreen: bool,
    #[serde(rename = "extraGameArgs")]
    pub extra_game_args: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherSettings {
    #[serde(rename = "closeOnLaunch")]
    pub close_on_launch: bool,
    #[serde(rename = "keepLauncherOpen")]
    pub keep_launcher_open: bool,
    #[serde(rename = "autoUpdate")]
    pub auto_update: bool,
    #[serde(rename = "concurrentDownloads")]
    pub concurrent_downloads: u32,
    #[serde(rename = "soundEnabled")]
    pub sound_enabled: bool,
    #[serde(rename = "soundVolume")]
    pub sound_volume: u32,
    #[serde(rename = "soundStyle")]
    pub sound_style: String,
    #[serde(rename = "curseforgeApiKey")]
    pub curseforge_api_key: String,
    #[serde(rename = "advancedMode")]
    pub advanced_mode: bool,
    #[serde(rename = "debugLogging")]
    pub debug_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub language: String,
    #[serde(rename = "accentColor")]
    pub accent_color: String,
    #[serde(rename = "reduceMotion")]
    pub reduce_motion: String,
    #[serde(rename = "pageTransition")]
    pub page_transition: String,
    #[serde(rename = "disableHoverEffects")]
    pub disable_hover_effects: bool,
    #[serde(rename = "disableSplashScreen")]
    pub disable_splash_screen: bool,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomizationSettings {
    #[serde(rename = "editorMode")]
    pub editor_mode: bool,
    #[serde(rename = "wiggleEnabled")]
    pub wiggle_enabled: bool,
    /// `Record<string, ElementOverride>` on the TS side — kept as raw JSON since the override
    /// shape has ~20 optional fields the Rust side never needs to inspect, only round-trip.
    pub elements: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub game: GameSettings,
    pub launcher: LauncherSettings,
    pub appearance: AppearanceSettings,
    pub customization: CustomizationSettings,
    #[serde(rename = "activeProfileId")]
    pub active_profile_id: Option<String>,
}

fn default_settings() -> AppSettings {
    AppSettings {
        game: GameSettings {
            default_game_dir: paths::default_game_dir().to_string_lossy().to_string(),
            default_java_path: String::new(),
            min_ram: 512,
            max_ram: 2048,
            jvm_args: String::new(),
            resolution: Resolution { width: 854, height: 480 },
            fullscreen: false,
            extra_game_args: String::new(),
        },
        launcher: LauncherSettings {
            close_on_launch: false,
            keep_launcher_open: true,
            auto_update: true,
            concurrent_downloads: 16,
            sound_enabled: true,
            sound_volume: 50,
            sound_style: "soft".into(),
            curseforge_api_key: String::new(),
            advanced_mode: false,
            debug_logging: false,
        },
        appearance: AppearanceSettings {
            language: "en".into(),
            accent_color: "#27ade0".into(),
            reduce_motion: "system".into(),
            page_transition: "fade".into(),
            disable_hover_effects: false,
            disable_splash_screen: false,
            theme: "default".into(),
        },
        customization: CustomizationSettings {
            editor_mode: false,
            wiggle_enabled: true,
            elements: Value::Object(Default::default()),
        },
        active_profile_id: None,
    }
}

/// Merges `patch` (parsed from disk, may be a partial/older shape) onto `default_settings()`.
/// Missing keys in `patch` fall back to defaults instead of erroring — mirrors the TS
/// `deepMerge` behaviour so settings.json written by an older version still loads.
fn merge_from_disk(raw: Value) -> AppSettings {
    let mut base = serde_json::to_value(default_settings()).unwrap();
    deep_merge_json(&mut base, raw);
    serde_json::from_value(base).unwrap_or_else(|_| default_settings())
}

fn deep_merge_json(base: &mut Value, patch: Value) {
    match (base, patch) {
        (Value::Object(base_map), Value::Object(patch_map)) => {
            for (k, v) in patch_map {
                if let Some(existing) = base_map.get_mut(&k) {
                    if existing.is_object() && v.is_object() {
                        deep_merge_json(existing, v);
                        continue;
                    }
                }
                base_map.insert(k, v);
            }
        }
        (base_slot, patch_val) => {
            *base_slot = patch_val;
        }
    }
}

static SETTINGS_CACHE: Lazy<Mutex<Option<AppSettings>>> = Lazy::new(|| Mutex::new(None));

pub fn get_settings() -> AppSettings {
    {
        let cache = SETTINGS_CACHE.lock();
        if let Some(s) = cache.as_ref() {
            return s.clone();
        }
    }
    let path = paths::settings_path();
    let loaded = if path.exists() {
        match fs::read_to_string(&path).ok().and_then(|raw| serde_json::from_str::<Value>(&raw).ok()) {
            Some(raw) => merge_from_disk(raw),
            None => default_settings(),
        }
    } else {
        default_settings()
    };

    let mut loaded = loaded;
    // One-time migration: users who never customized their game directory were silently
    // pinned to the shared vanilla .minecraft folder. Move them to the isolated instance dir.
    let legacy = paths::legacy_default_game_dir().to_string_lossy().to_string();
    if loaded.game.default_game_dir == legacy {
        loaded.game.default_game_dir = paths::default_game_dir().to_string_lossy().to_string();
        save_settings(&loaded);
    }

    *SETTINGS_CACHE.lock() = Some(loaded.clone());
    loaded
}

pub fn save_settings(settings: &AppSettings) {
    *SETTINGS_CACHE.lock() = Some(settings.clone());
    let path = paths::settings_path();
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(path, json);
    }
}
