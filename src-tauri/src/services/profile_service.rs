//! Port of `src/main/services/profileService.ts`.

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::paths;
use crate::services::settings_service::Resolution;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchProfile {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub version: String,
    /// 'vanilla' | 'fabric' | 'forge' | 'quilt' | 'neoforge'
    pub loader: String,
    #[serde(rename = "loaderVersion")]
    pub loader_version: String,
    #[serde(rename = "gameDir")]
    pub game_dir: String,
    #[serde(rename = "minRam")]
    pub min_ram: u32,
    #[serde(rename = "maxRam")]
    pub max_ram: u32,
    #[serde(rename = "javaPath")]
    pub java_path: String,
    #[serde(rename = "jvmArgs")]
    pub jvm_args: String,
    pub resolution: Resolution,
    #[serde(rename = "useBejaClient")]
    pub use_beja_client: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "lastPlayed")]
    pub last_played: Option<String>,
    #[serde(rename = "playtimeMs")]
    pub playtime_ms: u64,
    #[serde(rename = "imageUrl", default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(rename = "backgroundUrl", default, skip_serializing_if = "Option::is_none")]
    pub background_url: Option<String>,
    #[serde(rename = "isolateProfile", default, skip_serializing_if = "Option::is_none")]
    pub isolate_profile: Option<bool>,
}

/// Everything in `LaunchProfile` except the fields the backend fills in on create
/// (`id`, `createdAt`, `lastPlayed`, `playtimeMs`) — mirrors the TS `Omit<...>` param type.
#[derive(Debug, Clone, Deserialize)]
pub struct NewProfile {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub version: String,
    pub loader: String,
    #[serde(rename = "loaderVersion")]
    pub loader_version: String,
    #[serde(rename = "gameDir")]
    pub game_dir: String,
    #[serde(rename = "minRam")]
    pub min_ram: u32,
    #[serde(rename = "maxRam")]
    pub max_ram: u32,
    #[serde(rename = "javaPath")]
    pub java_path: String,
    #[serde(rename = "jvmArgs")]
    pub jvm_args: String,
    pub resolution: Resolution,
    #[serde(rename = "useBejaClient")]
    pub use_beja_client: bool,
    #[serde(rename = "imageUrl", default)]
    pub image_url: Option<String>,
    #[serde(rename = "backgroundUrl", default)]
    pub background_url: Option<String>,
    #[serde(rename = "isolateProfile", default)]
    pub isolate_profile: Option<bool>,
}

static PROFILES_CACHE: Lazy<Mutex<Option<Vec<LaunchProfile>>>> = Lazy::new(|| Mutex::new(None));

pub fn list_profiles() -> Vec<LaunchProfile> {
    {
        let cache = PROFILES_CACHE.lock();
        if let Some(p) = cache.as_ref() {
            return p.clone();
        }
    }
    let path = paths::profiles_path();
    let loaded = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<Vec<LaunchProfile>>(&raw).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    *PROFILES_CACHE.lock() = Some(loaded.clone());
    loaded
}

pub fn save_profiles(profiles: &[LaunchProfile]) {
    *PROFILES_CACHE.lock() = Some(profiles.to_vec());
    if let Ok(json) = serde_json::to_string_pretty(profiles) {
        let _ = fs::write(paths::profiles_path(), json);
    }
}

pub fn create_profile(data: NewProfile) -> LaunchProfile {
    let id = Uuid::new_v4().to_string();
    let mut game_dir = data.game_dir;
    // Isolated profiles get their own game directory so worlds/mods/settings don't leak
    // between profiles. launchService falls back to the shared default dir when empty.
    if data.isolate_profile.unwrap_or(false) && game_dir.is_empty() {
        game_dir = paths::user_data_dir()
            .join("instances")
            .join(&id)
            .to_string_lossy()
            .to_string();
    }
    let profile = LaunchProfile {
        id,
        name: data.name,
        description: data.description,
        version: data.version,
        loader: data.loader,
        loader_version: data.loader_version,
        game_dir,
        min_ram: data.min_ram,
        max_ram: data.max_ram,
        java_path: data.java_path,
        jvm_args: data.jvm_args,
        resolution: data.resolution,
        use_beja_client: data.use_beja_client,
        created_at: chrono::Utc::now().to_rfc3339(),
        last_played: None,
        playtime_ms: 0,
        image_url: data.image_url,
        background_url: data.background_url,
        isolate_profile: data.isolate_profile,
    };
    let mut profiles = list_profiles();
    profiles.push(profile.clone());
    save_profiles(&profiles);
    profile
}

pub fn update_profile(id: &str, patch: serde_json::Value) -> Option<LaunchProfile> {
    let mut profiles = list_profiles();
    let idx = profiles.iter().position(|p| p.id == id)?;
    let mut current = serde_json::to_value(&profiles[idx]).ok()?;
    if let (Some(cur_map), serde_json::Value::Object(patch_map)) = (current.as_object_mut(), patch) {
        for (k, v) in patch_map {
            cur_map.insert(k, v);
        }
    }
    let updated: LaunchProfile = serde_json::from_value(current).ok()?;
    profiles[idx] = updated.clone();
    save_profiles(&profiles);
    Some(updated)
}

pub fn delete_profile(id: &str) -> bool {
    let profiles = list_profiles();
    let filtered: Vec<_> = profiles.iter().filter(|p| p.id != id).cloned().collect();
    if filtered.len() == profiles.len() {
        return false;
    }
    save_profiles(&filtered);
    true
}

pub fn get_profile(id: &str) -> Option<LaunchProfile> {
    list_profiles().into_iter().find(|p| p.id == id)
}
