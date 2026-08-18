//! Port of `src/main/services/modrinthService.ts`. CurseForge/"explore" aggregation
//! (`curseforgeService.ts`, `explore:search`) is not ported — those commands return a clear
//! "not supported yet" error instead of a real result.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::{profile_service, settings_service};

const API: &str = "https://api.modrinth.com/v2";
const UA: &str = "BejaClient/2.0 (bejaclient.xyz) Tauri-edition";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModrinthVersionFile {
    pub url: String,
    pub filename: String,
    #[serde(default)]
    pub primary: bool,
    #[serde(default)]
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModrinthVersion {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "version_number")]
    pub version_number: String,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    pub files: Vec<ModrinthVersionFile>,
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().user_agent(UA).build().unwrap()
}

async fn get_json(url: &str) -> Result<Value, String> {
    let res = client().get(url).send().await.map_err(|e| e.to_string())?;
    let status = res.status();
    let text = res.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("Modrinth API error {status}: {}", text.chars().take(200).collect::<String>()));
    }
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub async fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    let mut current_url = url.to_string();
    for _ in 0..8 {
        let res = client().get(&current_url).send().await.map_err(|e| e.to_string())?;
        if res.status().is_redirection() {
            if let Some(loc) = res.headers().get("location").and_then(|v| v.to_str().ok()) {
                current_url = loc.to_string();
                continue;
            }
        }
        if !res.status().is_success() {
            return Err(format!("HTTP {} for {current_url}", res.status()));
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
        }
        let bytes = res.bytes().await.map_err(|e| e.to_string())?;
        let mut file = fs::File::create(dest).await.map_err(|e| e.to_string())?;
        file.write_all(&bytes).await.map_err(|e| e.to_string())?;
        return Ok(());
    }
    Err("Too many redirects".to_string())
}

pub async fn get_categories() -> Result<Value, String> {
    get_json(&format!("{API}/tag/category")).await
}

pub async fn search_modrinth(
    query: &str,
    project_type: &str,
    game_version: Option<&str>,
    loader: Option<&str>,
    offset: u32,
    categories: Option<Vec<String>>,
    sort: Option<&str>,
) -> Result<Value, String> {
    let mut facets: Vec<Vec<String>> = vec![vec![format!("project_type:{project_type}")]];
    if let Some(gv) = game_version {
        facets.push(vec![format!("versions:{gv}")]);
    }
    if let Some(l) = loader {
        if l != "vanilla" {
            facets.push(vec![format!("categories:{l}")]);
        }
    }
    if let Some(cats) = categories {
        if !cats.is_empty() {
            facets.push(cats.into_iter().map(|c| format!("categories:{c}")).collect());
        }
    }

    // Modrinth doesn't expose a raw "views" metric — `index` only accepts
    // relevance/downloads/follows/newest/updated.
    let index = match sort.unwrap_or("relevance") {
        "downloads" => "downloads",
        "newest" => "newest",
        "updated" => "updated",
        _ => "relevance",
    };

    let facets_str = urlencoding::encode(&serde_json::to_string(&facets).unwrap()).to_string();
    let query_str = urlencoding::encode(query).to_string();
    let url = format!("{API}/search?query={query_str}&facets={facets_str}&limit=20&offset={offset}&index={index}");

    let data = get_json(&url).await?;
    if !data["hits"].is_array() {
        return Err(data["description"].as_str().or(data["error"].as_str()).unwrap_or("Unexpected Modrinth response").to_string());
    }
    Ok(data)
}

pub async fn get_project(project_id: &str) -> Result<Value, String> {
    get_json(&format!("{API}/project/{project_id}")).await
}

pub async fn get_project_versions(project_id: &str, game_version: Option<&str>, loader: Option<&str>) -> Result<Vec<ModrinthVersion>, String> {
    let mut params = Vec::new();
    if let Some(gv) = game_version {
        params.push(format!("game_versions={}", urlencoding::encode(&serde_json::to_string(&vec![gv]).unwrap())));
    }
    if let Some(l) = loader {
        if l != "vanilla" {
            params.push(format!("loaders={}", urlencoding::encode(&serde_json::to_string(&vec![l]).unwrap())));
        }
    }
    let suffix = if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) };
    let data = get_json(&format!("{API}/project/{project_id}/version{suffix}")).await?;
    serde_json::from_value(data).map_err(|e| e.to_string())
}

async fn download_to_dir(project_id: &str, dest_dir: &Path, game_version: Option<&str>, loader: Option<&str>) -> Result<(), String> {
    let versions = get_project_versions(project_id, game_version, loader).await?;
    let version = versions.first().ok_or("No compatible version found for this Minecraft version / loader")?;
    let file = version.files.iter().find(|f| f.primary).or_else(|| version.files.first()).ok_or("No downloadable file found")?;
    download_file(&file.url, &dest_dir.join(&file.filename)).await
}

fn profile_game_dir(profile_id: &str) -> Result<(profile_service::LaunchProfile, PathBuf), String> {
    let profile = profile_service::get_profile(profile_id).ok_or("Profile not found")?;
    let settings = settings_service::get_settings();
    let game_dir = if !profile.game_dir.is_empty() { profile.game_dir.clone() } else { settings.game.default_game_dir };
    Ok((profile, PathBuf::from(game_dir)))
}

pub async fn download_mod(project_id: &str, profile_id: &str) -> Result<(), String> {
    let (profile, game_dir) = profile_game_dir(profile_id)?;
    download_to_dir(project_id, &game_dir.join("mods"), Some(&profile.version), Some(&profile.loader)).await
}

pub async fn download_mod_version(project_id: &str, version_id: &str, profile_id: &str) -> Result<(), String> {
    let (_, game_dir) = profile_game_dir(profile_id)?;
    let version: ModrinthVersion = serde_json::from_value(get_json(&format!("{API}/version/{version_id}")).await?).map_err(|e| e.to_string())?;
    let _ = project_id;
    let file = version.files.iter().find(|f| f.primary).or_else(|| version.files.first()).ok_or("No downloadable file found")?;
    download_file(&file.url, &game_dir.join("mods").join(&file.filename)).await
}

pub async fn download_resourcepack(project_id: &str, profile_id: &str) -> Result<(), String> {
    let (profile, game_dir) = profile_game_dir(profile_id)?;
    download_to_dir(project_id, &game_dir.join("resourcepacks"), Some(&profile.version), None).await
}

pub async fn download_shader(project_id: &str, profile_id: &str) -> Result<(), String> {
    let (profile, game_dir) = profile_game_dir(profile_id)?;
    download_to_dir(project_id, &game_dir.join("shaderpacks"), Some(&profile.version), None).await
}

pub async fn download_datapack(project_id: &str, profile_id: &str) -> Result<(), String> {
    let (profile, game_dir) = profile_game_dir(profile_id)?;
    download_to_dir(project_id, &game_dir.join("datapacks"), Some(&profile.version), None).await
}

pub struct ModpackInstallResult {
    pub profile_id: String,
    pub name: String,
}

pub async fn install_modpack(project_id: &str, version_id: Option<&str>) -> Result<ModpackInstallResult, String> {
    let project = get_json(&format!("{API}/project/{project_id}")).await?;

    let version: ModrinthVersion = if let Some(vid) = version_id {
        serde_json::from_value(get_json(&format!("{API}/version/{vid}")).await?).map_err(|e| e.to_string())?
    } else {
        get_project_versions(project_id, None, None).await?.into_iter().next().ok_or("No versions found")?
    };

    let mrpack_file = version.files.iter().find(|f| f.primary).or_else(|| version.files.first()).ok_or("No modpack file found")?;

    let tmp_path = std::env::temp_dir().join(format!("{project_id}-{}.mrpack", chrono::Utc::now().timestamp_millis()));
    download_file(&mrpack_file.url, &tmp_path).await?;

    let settings = settings_service::get_settings();
    let profile_name = project["title"].as_str().unwrap_or("Modpack").to_string();
    let safe_name: String = profile_name.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == ' ').collect();
    let game_dir = PathBuf::from(&settings.game.default_game_dir).join("modpacks").join(&safe_name);
    fs::create_dir_all(&game_dir).await.map_err(|e| e.to_string())?;

    // Re-installing/updating a pack that's already installed here (same deterministic
    // game_dir) previously left the PREVIOUS version's mod jars sitting alongside the new
    // ones — install_modpack only skips a download when the exact filename already exists,
    // and different mod versions almost always have different filenames, so nothing ever got
    // removed. That produced a mods/ folder with two Minecraft-version-incompatible copies of
    // nearly every mod (e.g. both a 1.21.11 and a 26.2 build of Sodium/Iris/etc. installed
    // together), which Fabric Loader then refuses to boot. Clear stale mods before writing the
    // new set. Only mods/ is wiped — saves/config/resourcepacks/screenshots are untouched.
    // NOT gated on an existing profile being found: game_dir is a deterministic path derived
    // purely from the pack's title, so any mods/ already sitting there — even predating any
    // profile row (e.g. a stray manual test install) — is guaranteed stale once we're about to
    // repopulate it fresh from modrinth.index.json below. Confirmed hit in practice: a Fabulously
    // Optimized profile whose FIRST-ever install (no prior profile row) still failed to boot
    // because 1.16.5-era jars were already sitting in mods/ from an earlier manual drop.
    let game_dir_str = game_dir.to_string_lossy().to_string();
    let existing_profile = profile_service::list_profiles().into_iter().find(|p| p.game_dir == game_dir_str);
    {
        let mods_dir = game_dir.join("mods");
        if fs::metadata(&mods_dir).await.is_ok() {
            fs::remove_dir_all(&mods_dir).await.map_err(|e| e.to_string())?;
        }
    }

    let tmp_path_blocking = tmp_path.clone();
    let game_dir_blocking = game_dir.clone();
    let (mc_version, loader, loader_version) = tokio::task::spawn_blocking(move || -> Result<(String, String, String), String> {
        let file = std::fs::File::open(&tmp_path_blocking).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

        let index: Value = {
            let mut entry = zip.by_name("modrinth.index.json").map_err(|_| "Invalid mrpack: missing modrinth.index.json".to_string())?;
            let mut s = String::new();
            entry.read_to_string(&mut s).map_err(|e| e.to_string())?;
            serde_json::from_str(&s).map_err(|e| e.to_string())?
        };

        let mc_version = index["dependencies"]["minecraft"].as_str().unwrap_or_default().to_string();
        let fabric_ver = index["dependencies"]["fabric-loader"].as_str().map(|s| s.to_string());
        let forge_ver = index["dependencies"]["forge"].as_str().map(|s| s.to_string());
        let quilt_ver = index["dependencies"]["quilt-loader"].as_str().map(|s| s.to_string());
        let (loader, loader_version) = if let Some(v) = &fabric_ver {
            ("fabric".to_string(), v.clone())
        } else if let Some(v) = &forge_ver {
            ("forge".to_string(), v.clone())
        } else if let Some(v) = &quilt_ver {
            ("quilt".to_string(), v.clone())
        } else {
            ("vanilla".to_string(), String::new())
        };

        // Apply overrides/ — mod downloads themselves are fetched async by the caller below;
        // this blocking closure only unpacks the zip's static override files.
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
            if entry.is_dir() {
                continue;
            }
            let name = entry.name().to_string();
            if let Some(rel) = name.strip_prefix("overrides/") {
                let dest = game_dir_blocking.join(rel);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
                std::fs::write(&dest, buf).map_err(|e| e.to_string())?;
            }
        }

        Ok((mc_version, loader, loader_version))
    })
    .await
    .map_err(|e| e.to_string())??;

    // Download mod files listed in modrinth.index.json (re-read the JSON, this time async).
    let index_files: Vec<Value> = tokio::task::spawn_blocking({
        let tmp_path = tmp_path.clone();
        move || -> Result<Vec<Value>, String> {
            let file = std::fs::File::open(&tmp_path).map_err(|e| e.to_string())?;
            let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
            let mut entry = zip.by_name("modrinth.index.json").map_err(|e| e.to_string())?;
            let mut s = String::new();
            entry.read_to_string(&mut s).map_err(|e| e.to_string())?;
            let index: Value = serde_json::from_str(&s).map_err(|e| e.to_string())?;
            Ok(index["files"].as_array().cloned().unwrap_or_default())
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    for f in index_files {
        let Some(rel_path) = f["path"].as_str() else { continue };
        let dest = game_dir.join(rel_path);
        if fs::metadata(&dest).await.is_ok() {
            continue;
        }
        if let Some(url) = f["downloads"][0].as_str() {
            let _ = download_file(url, &dest).await; // best-effort, skip on error
        }
    }

    let _ = fs::remove_file(&tmp_path).await;

    let icon_url = project["icon_url"].as_str().map(|s| s.to_string());
    let gallery = project["gallery"].as_array().cloned().unwrap_or_default();
    let banner_url = gallery
        .iter()
        .find(|g| g["featured"].as_bool().unwrap_or(false))
        .or_else(|| gallery.first())
        .and_then(|g| g["url"].as_str())
        .map(|s| s.to_string())
        .or_else(|| icon_url.clone());

    // Update the existing profile in place if this project was already installed here, rather
    // than inserting a second profile that points at the very same game_dir (create_profile
    // always mints a fresh id/row — it has no dedup of its own).
    let profile_id = if let Some(existing) = existing_profile {
        profile_service::update_profile(&existing.id, serde_json::json!({
            "version": mc_version,
            "loader": loader,
            "loaderVersion": loader_version,
            "imageUrl": icon_url,
            "backgroundUrl": banner_url,
        }))
        .ok_or("Failed to update existing modpack profile")?
        .id
    } else {
        profile_service::create_profile(profile_service::NewProfile {
            name: profile_name.clone(),
            description: None,
            version: mc_version,
            loader,
            loader_version,
            game_dir: game_dir.to_string_lossy().to_string(),
            min_ram: 1024,
            max_ram: settings.game.max_ram,
            java_path: settings.game.default_java_path,
            jvm_args: String::new(),
            resolution: settings.game.resolution,
            use_beja_client: false,
            image_url: icon_url,
            background_url: banner_url,
            isolate_profile: None,
        })
        .id
    };

    Ok(ModpackInstallResult { profile_id, name: profile_name })
}
