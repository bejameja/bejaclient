//! Port of `src/main/services/modService.ts`.

use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use super::{profile_service, settings_service};

#[derive(Debug, Clone, Serialize)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub enabled: bool,
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "fileSize")]
    pub file_size: u64,
    #[serde(rename = "modifiedAt")]
    pub modified_at: String,
    #[serde(rename = "iconDataUrl", skip_serializing_if = "Option::is_none")]
    pub icon_data_url: Option<String>,
}

fn extract_mod_meta(file_path: &Path) -> (Option<String>, Option<String>) {
    let Ok(file) = fs::File::open(file_path) else { return (None, None) };
    let Ok(mut zip) = zip::ZipArchive::new(file) else { return (None, None) };

    let fabric_meta: Option<(Option<String>, Option<String>)> = {
        match zip.by_name("fabric.mod.json") {
            Ok(mut entry) => {
                let mut contents = String::new();
                if entry.read_to_string(&mut contents).is_ok() {
                    serde_json::from_str::<serde_json::Value>(&contents).ok().map(|meta| {
                        (meta["name"].as_str().map(|s| s.to_string()), meta["icon"].as_str().map(|s| s.to_string()))
                    })
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    };

    if let Some((name, icon_path)) = fabric_meta {
        if let Some(icon_path) = icon_path {
            if let Ok(mut icon_entry) = zip.by_name(&icon_path) {
                let mut buf = Vec::new();
                if icon_entry.read_to_end(&mut buf).is_ok() {
                    let ext = icon_path.rsplit('.').next().unwrap_or("png").to_lowercase();
                    let mime = if ext == "png" { "image/png" } else if ext == "jpg" || ext == "jpeg" { "image/jpeg" } else { "image/png" };
                    let icon_data_url = format!("data:{mime};base64,{}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &buf));
                    return (name, Some(icon_data_url));
                }
            }
        }
        return (name, None);
    }

    if let Ok(mut entry) = zip.by_name("pack.png") {
        let mut buf = Vec::new();
        if entry.read_to_end(&mut buf).is_ok() {
            let icon_data_url = format!("data:image/png;base64,{}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &buf));
            return (None, Some(icon_data_url));
        }
    }

    (None, None)
}

fn get_mods_dir(profile_id: &str) -> PathBuf {
    let profile = profile_service::get_profile(profile_id);
    let settings = settings_service::get_settings();
    let game_dir = profile
        .map(|p| p.game_dir)
        .filter(|d| !d.is_empty())
        .unwrap_or(settings.game.default_game_dir);
    Path::new(&game_dir).join("mods")
}

fn ensure_mods_dir(mods_dir: &Path) {
    if !mods_dir.exists() {
        let _ = fs::create_dir_all(mods_dir);
    }
}

fn title_case(s: &str) -> String {
    s.split(|c| c == '-' || c == '_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn list_mods(profile_id: &str) -> Vec<ModInfo> {
    let mods_dir = get_mods_dir(profile_id);
    ensure_mods_dir(&mods_dir);

    let mut mods: Vec<ModInfo> = Vec::new();
    let Ok(entries) = fs::read_dir(&mods_dir) else { return mods };

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else { continue };
        if !file_type.is_file() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(|s| s.to_string()) else { continue };
        if !(name.ends_with(".jar") || name.ends_with(".jar.disabled")) {
            continue;
        }

        let file_path = entry.path();
        let enabled = name.ends_with(".jar") && !name.ends_with(".jar.disabled");
        let fallback_name = title_case(name.trim_end_matches(".disabled").trim_end_matches(".jar"));
        let Ok(meta) = entry.metadata() else { continue };
        let modified_at = meta
            .modified()
            .ok()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
            .unwrap_or_default();
        let (meta_name, icon_data_url) = extract_mod_meta(&file_path);

        mods.push(ModInfo {
            id: name.clone(),
            name: meta_name.unwrap_or(fallback_name),
            file_name: name,
            enabled,
            file_path: file_path.to_string_lossy().to_string(),
            file_size: meta.len(),
            modified_at,
            icon_data_url,
        });
    }

    mods.sort_by(|a, b| a.name.cmp(&b.name));
    mods
}

pub fn toggle_mod(profile_id: &str, mod_id: &str) -> Result<Vec<ModInfo>, String> {
    let mods_dir = get_mods_dir(profile_id);
    let file_path = mods_dir.join(mod_id);
    if !file_path.exists() {
        return Err(format!("Mod file not found: {mod_id}"));
    }

    if mod_id.ends_with(".jar.disabled") {
        let new_path = mods_dir.join(mod_id.trim_end_matches(".disabled"));
        fs::rename(&file_path, new_path).map_err(|e| e.to_string())?;
    } else if mod_id.ends_with(".jar") {
        let new_path = mods_dir.join(format!("{mod_id}.disabled"));
        fs::rename(&file_path, new_path).map_err(|e| e.to_string())?;
    }

    Ok(list_mods(profile_id))
}

pub fn install_mod(profile_id: &str, source_path: &str) -> Result<Vec<ModInfo>, String> {
    let mods_dir = get_mods_dir(profile_id);
    ensure_mods_dir(&mods_dir);

    let source = Path::new(source_path);
    let file_name = source.file_name().ok_or("Invalid source path")?.to_string_lossy().to_string();
    if !file_name.ends_with(".jar") {
        return Err("Only .jar mod files are supported".to_string());
    }

    let dest_path = mods_dir.join(&file_name);
    fs::copy(source, dest_path).map_err(|e| e.to_string())?;
    Ok(list_mods(profile_id))
}

pub fn delete_mod(profile_id: &str, mod_id: &str) -> Vec<ModInfo> {
    let mods_dir = get_mods_dir(profile_id);
    let file_path = mods_dir.join(mod_id);
    if file_path.exists() {
        let _ = fs::remove_file(file_path);
    }
    list_mods(profile_id)
}

pub fn open_mods_folder(app: &tauri::AppHandle, profile_id: &str) {
    let mods_dir = get_mods_dir(profile_id);
    ensure_mods_dir(&mods_dir);
    let _ = tauri_plugin_opener::OpenerExt::opener(app).open_path(mods_dir.to_string_lossy().to_string(), None::<&str>);
}
