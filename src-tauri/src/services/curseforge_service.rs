//! Port of `src/main/services/curseforgeService.ts`.

use serde_json::Value;
use std::path::PathBuf;

use super::{profile_service, settings_service};

const CF_API: &str = "https://api.curseforge.com/v1";
const UA: &str = "BejaClient/2.0 (bejaclient.xyz) Tauri-edition";
const MC_GAME_ID: u32 = 432;

fn class_id(project_type: &str) -> Option<u32> {
    match project_type {
        "mod" => Some(6),
        "modpack" => Some(4471),
        "shader" => Some(6552),
        "resourcepack" => Some(12),
        "datapack" => Some(6945),
        _ => None,
    }
}

fn loader_id(loader: &str) -> Option<u32> {
    match loader {
        "forge" => Some(1),
        "fabric" => Some(4),
        "quilt" => Some(5),
        "neoforge" => Some(6),
        _ => None,
    }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().user_agent(UA).build().unwrap()
}

async fn cf_get(url_path: &str, api_key: &str) -> Result<Value, String> {
    let res = client()
        .get(format!("{CF_API}{url_path}"))
        .header("x-api-key", api_key)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = res.status();
    let text = res.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("CurseForge API error {status}: {}", text.chars().take(200).collect::<String>()));
    }
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

async fn download_file(url: &str, dest: &std::path::Path) -> Result<(), String> {
    let mut current_url = url.to_string();
    for _ in 0..8 {
        let res = client().get(&current_url).send().await.map_err(|e| e.to_string())?;
        if res.status().is_redirection() {
            if let Some(loc) = res.headers().get("location").and_then(|v| v.to_str().ok()) {
                current_url = loc.to_string();
                continue;
            }
        }
        if let Some(parent) = dest.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let bytes = res.bytes().await.map_err(|e| e.to_string())?;
        std::fs::write(dest, bytes).map_err(|e| e.to_string())?;
        return Ok(());
    }
    Err("Too many redirects".to_string())
}

pub async fn install_curseforge_mod(mod_id: &str, project_type: &str, profile_id: &str, on_progress: impl Fn(String)) -> Result<(), String> {
    let settings = settings_service::get_settings();
    let api_key = settings.launcher.curseforge_api_key.clone();
    if api_key.is_empty() {
        return Err("CurseForge API key not configured in Settings".to_string());
    }

    let profile = profile_service::get_profile(profile_id).ok_or("Profile not found")?;
    let game_dir = if !profile.game_dir.is_empty() { PathBuf::from(&profile.game_dir) } else { PathBuf::from(&settings.game.default_game_dir) };

    let dest_dir = match project_type {
        "mod" => game_dir.join("mods"),
        "shader" => game_dir.join("shaderpacks"),
        "resourcepack" => game_dir.join("resourcepacks"),
        "datapack" => game_dir.join("datapacks"),
        _ => game_dir.join("mods"),
    };

    on_progress("Fetching file info…".to_string());
    let data = cf_get(&format!("/mods/{mod_id}/files?pageSize=20&sortOrder=desc&index=0"), &api_key).await?;
    let file = data["data"].as_array().and_then(|a| a.first()).ok_or("No files found")?;

    let download_url = file["downloadUrl"].as_str().ok_or("Download restricted — check mod page on CurseForge")?;
    let file_name = file["fileName"].as_str().unwrap_or("download.jar");

    on_progress(format!("Downloading {file_name}…"));
    download_file(download_url, &dest_dir.join(file_name)).await?;
    on_progress("Done".to_string());
    Ok(())
}

/// Combines `/mods/{id}` (stats, links, screenshots) with `/mods/{id}/description`
/// (the full HTML body — not included in the base mod object) into one JSON blob.
pub async fn get_mod_details(mod_id: &str) -> Result<Value, String> {
    let settings = settings_service::get_settings();
    let api_key = settings.launcher.curseforge_api_key.clone();
    if api_key.is_empty() {
        return Err("CurseForge API key not configured".to_string());
    }

    let mod_data = cf_get(&format!("/mods/{mod_id}"), &api_key).await?;
    let description = cf_get(&format!("/mods/{mod_id}/description"), &api_key).await
        .ok()
        .and_then(|d| d["data"].as_str().map(|s| s.to_string()))
        .unwrap_or_default();

    let mut mod_obj = mod_data["data"].clone();
    if let Value::Object(ref mut map) = mod_obj {
        map.insert("fullDescription".to_string(), Value::String(description));
    }
    Ok(mod_obj)
}

pub async fn search_curseforge(query: &str, project_type: &str, game_version: Option<&str>, loader: Option<&str>, offset: u32, sort: Option<&str>) -> Result<Value, String> {
    let settings = settings_service::get_settings();
    let api_key = settings.launcher.curseforge_api_key.clone();
    if api_key.is_empty() {
        return Err("CurseForge API key not configured".to_string());
    }

    let Some(class_id) = class_id(project_type) else { return Ok(serde_json::json!({ "hits": [], "total": 0 })) };

    // CurseForge doesn't expose a raw "views" metric either — map the shared sort
    // options onto the closest sortField it does have.
    let sort_field = match sort.unwrap_or("relevance") {
        "downloads" => "6", // TotalDownloads
        "newest" => "11",   // ReleasedDate
        "updated" => "3",   // LastUpdated
        _ => "2",           // Popularity
    };

    let mut query_params = vec![
        ("gameId".to_string(), MC_GAME_ID.to_string()),
        ("classId".to_string(), class_id.to_string()),
        ("searchFilter".to_string(), query.to_string()),
        ("pageSize".to_string(), "20".to_string()),
        ("index".to_string(), offset.to_string()),
        ("sortField".to_string(), sort_field.to_string()),
        ("sortOrder".to_string(), "desc".to_string()),
    ];
    if let Some(gv) = game_version {
        query_params.push(("gameVersion".to_string(), gv.to_string()));
    }
    if let Some(l) = loader {
        if l != "vanilla" {
            if let Some(lid) = loader_id(l) {
                query_params.push(("modLoaderType".to_string(), lid.to_string()));
            }
        }
    }
    let qs = query_params.iter().map(|(k, v)| format!("{k}={}", urlencoding::encode(v))).collect::<Vec<_>>().join("&");

    let data = cf_get(&format!("/mods/search?{qs}"), &api_key).await?;
    Ok(serde_json::json!({
        "hits": data["data"].as_array().cloned().unwrap_or_default(),
        "total": data["pagination"]["totalCount"].as_u64().unwrap_or(0),
    }))
}
