//! Detects existing modded instances from other Minecraft launchers (Lunar Client's custom
//! mod profiles, the CurseForge App, the Modrinth App) already sitting on this machine, so the
//! Profiles tab can offer to import them as BejaClient profiles instead of the user rebuilding
//! a modpack from scratch. Every source here reuses the external launcher's instance folder
//! in place as the new profile's `gameDir` (mods/saves/config/etc. already sit there) — only
//! the Minecraft version + loader get installed fresh by BejaClient's own launch pipeline, the
//! same way any newly created profile resolves its version on first launch.

use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use std::path::Path;

use crate::services::profile_service;

#[derive(Debug, Clone, Serialize)]
pub struct DetectedProfile {
    pub id: String,
    pub source: String,
    pub name: String,
    pub version: String,
    pub loader: String,
    #[serde(rename = "loaderVersion")]
    pub loader_version: String,
    #[serde(rename = "gameDir")]
    pub game_dir: String,
    #[serde(rename = "modCount")]
    pub mod_count: u32,
    #[serde(rename = "iconPath")]
    pub icon_path: Option<String>,
}

fn count_jars(dir: &Path) -> u32 {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x.eq_ignore_ascii_case("jar")).unwrap_or(false))
                .count() as u32
        })
        .unwrap_or(0)
}

fn loader_from_curseforge_type(t: u64) -> &'static str {
    match t {
        1 => "forge",
        4 => "fabric",
        5 => "quilt",
        6 => "neoforge",
        _ => "vanilla",
    }
}

fn detect_curseforge() -> Vec<DetectedProfile> {
    let Some(base) = dirs::home_dir().map(|h| h.join("curseforge").join("minecraft").join("Instances")) else { return Vec::new() };
    let Ok(entries) = std::fs::read_dir(&base) else { return Vec::new() };

    entries
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let dir = entry.path();
            let manifest = dir.join("minecraftinstance.json");
            let raw = std::fs::read_to_string(&manifest).ok()?;
            let j: serde_json::Value = serde_json::from_str(&raw).ok()?;

            let mods_dir = dir.join("mods");
            let mod_count = count_jars(&mods_dir);
            if mod_count == 0 {
                return None;
            }

            let version = j["baseModLoader"]["minecraftVersion"].as_str().or_else(|| j["gameVersion"].as_str())?.to_string();
            let loader = loader_from_curseforge_type(j["baseModLoader"]["type"].as_u64().unwrap_or(0)).to_string();
            let loader_version = j["baseModLoader"]["forgeVersion"].as_str().unwrap_or("").to_string();
            let fallback_name = entry.file_name().to_string_lossy().to_string();
            let name = j["name"].as_str().map(|s| s.to_string()).unwrap_or(fallback_name);

            Some(DetectedProfile {
                id: format!("curseforge:{}", entry.file_name().to_string_lossy()),
                source: "curseforge".to_string(),
                name,
                version,
                loader,
                loader_version,
                game_dir: dir.to_string_lossy().to_string(),
                mod_count,
                icon_path: None,
            })
        })
        .collect()
}

/// Lunar Client's own client is Fabric-based and its custom "mod profiles" UI only accepts
/// Fabric-format mods, so the loader is always Fabric — there is no per-profile loader field to
/// read. `loaderVersion` is left empty; BejaClient auto-resolves the latest stable Fabric loader
/// for the target version on first launch (same fallback every other new profile relies on).
fn detect_lunar() -> Vec<DetectedProfile> {
    let Some(base) = dirs::home_dir().map(|h| h.join(".lunarclient").join("profiles")) else { return Vec::new() };
    let Ok(entries) = std::fs::read_dir(&base) else { return Vec::new() };

    entries
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let dir = entry.path();
            if !dir.is_dir() {
                return None;
            }
            let mod_count = count_jars(&dir.join("mods"));
            if mod_count == 0 {
                return None;
            }

            // A profile can have run under several MC versions over time (one subfolder each
            // under game-versions/); take whichever was played most recently.
            let version = std::fs::read_dir(dir.join("game-versions")).ok().and_then(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .max_by_key(|e| e.metadata().and_then(|m| m.modified()).ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
            })?;

            let name = entry.file_name().to_string_lossy().to_string();
            let icon = ["icon.png", "icon.webp"].iter().map(|f| dir.join(f)).find(|p| p.exists()).map(|p| p.to_string_lossy().to_string());

            Some(DetectedProfile {
                id: format!("lunar:{name}"),
                source: "lunar".to_string(),
                name: format!("{name} (Lunar Client)"),
                version,
                loader: "fabric".to_string(),
                loader_version: String::new(),
                game_dir: dir.to_string_lossy().to_string(),
                mod_count,
                icon_path: icon,
            })
        })
        .collect()
}

/// The Modrinth App (Theseus) keeps profile metadata in a SQLite DB rather than a per-instance
/// manifest file — opened read-only so we never race the app's own WAL writer if it's running.
fn detect_modrinth_app() -> Vec<DetectedProfile> {
    let Some(app_dir) = dirs::config_dir().map(|d| d.join("ModrinthApp")) else { return Vec::new() };
    let db_path = app_dir.join("app.db");
    if !db_path.exists() {
        return Vec::new();
    }

    let conn = match Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut stmt = match conn.prepare("SELECT path, name, game_version, mod_loader, mod_loader_version, icon_path FROM profiles") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
        ))
    });

    let Ok(rows) = rows else { return Vec::new() };

    rows.filter_map(|r| r.ok())
        .filter_map(|(path, name, version, loader, loader_version, icon_path)| {
            let dir = app_dir.join("profiles").join(&path);
            let mod_count = count_jars(&dir.join("mods"));
            if mod_count == 0 {
                return None;
            }
            Some(DetectedProfile {
                id: format!("modrinth_app:{path}"),
                source: "modrinth_app".to_string(),
                name,
                version,
                loader,
                loader_version,
                game_dir: dir.to_string_lossy().to_string(),
                mod_count,
                icon_path,
            })
        })
        .collect()
}

/// Drops anything whose gameDir already matches an existing BejaClient profile, so a profile
/// stays "detected" only until it's actually been imported (or the user made an unrelated
/// BejaClient profile that happens to point at the same folder — same effect either way, it's
/// already usable from BejaClient).
pub fn detect_all() -> Vec<DetectedProfile> {
    let existing: Vec<String> = profile_service::list_profiles().into_iter().map(|p| p.game_dir).collect();
    let mut all = detect_curseforge();
    all.extend(detect_lunar());
    all.extend(detect_modrinth_app());
    all.retain(|p| !existing.contains(&p.game_dir));
    all
}

pub fn import_profile(id: &str) -> Result<profile_service::LaunchProfile, String> {
    let detected = detect_all();
    let hit = detected.into_iter().find(|p| p.id == id).ok_or("Profile no longer detected — it may already be imported")?;

    let settings = crate::services::settings_service::get_settings();
    Ok(profile_service::create_profile(profile_service::NewProfile {
        name: hit.name,
        description: None,
        version: hit.version,
        loader: hit.loader,
        loader_version: hit.loader_version,
        game_dir: hit.game_dir,
        min_ram: 1024,
        max_ram: settings.game.max_ram,
        java_path: settings.game.default_java_path,
        jvm_args: String::new(),
        resolution: settings.game.resolution,
        use_beja_client: false,
        image_url: None,
        background_url: None,
        isolate_profile: Some(false),
    }))
}
