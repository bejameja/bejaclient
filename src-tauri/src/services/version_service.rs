//! Port of `src/main/services/versionService.ts`.

use crate::mcinstall::{assets, fabric, forge_install, libraries, platform, quilt, version_resolve};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoteVersion {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionManifestLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionManifest {
    pub latest: VersionManifestLatest,
    pub versions: Vec<RemoteVersion>,
}

pub async fn fetch_version_manifest(client: &reqwest::Client) -> Result<VersionManifest, String> {
    let text = client
        .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|_| {
        format!("Failed to fetch Minecraft version list: {}", text.chars().take(200).collect::<String>())
    })
}

pub async fn list_forge_versions(client: &reqwest::Client, mc_version: &str) -> Vec<String> {
    let Ok(res) = client.get("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json").send().await else {
        return Vec::new();
    };
    let Ok(data) = res.json::<serde_json::Value>().await else { return Vec::new() };
    let Some(promos) = data["promos"].as_object() else { return Vec::new() };
    let prefix = format!("{mc_version}-");
    promos
        .iter()
        .filter(|(k, _)| k.starts_with(&prefix))
        .filter_map(|(_, v)| v.as_str().map(|s| s.to_string()))
        .collect()
}

pub fn get_installed_versions(game_dir: &Path) -> Vec<String> {
    let versions_dir = game_dir.join("versions");
    let Ok(entries) = std::fs::read_dir(&versions_dir) else { return Vec::new() };
    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect()
}

pub async fn install_version(
    client: &reqwest::Client,
    version_id: &str,
    loader_type: &str,
    loader_version: Option<&str>,
    on_progress: impl Fn(String, u32, u32),
    game_dir: &Path,
    concurrency: usize,
    java_path: &str,
) -> Result<(), String> {
    on_progress(format!("Installing Minecraft {version_id}..."), 0, 100);

    let manifest = fetch_version_manifest(client).await?;
    let entry = manifest
        .versions
        .iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| format!("Version {version_id} not found in manifest"))?;

    version_resolve::download_version_json_and_jar(client, version_id, &entry.url, game_dir).await?;
    let resolved = version_resolve::resolve_version(game_dir, version_id).await?;

    on_progress("Installing assets and libraries...".to_string(), 50, 100);
    let plat = platform::get_current_platform();
    let libs = libraries::resolve_libraries(&resolved.libraries, &plat);
    libraries::download_libraries(client, libs, &game_dir.join("libraries"), concurrency).await?;

    let asset_index = assets::download_asset_index(client, &resolved, game_dir).await?;
    assets::download_assets(client, &asset_index, game_dir, concurrency).await?;
    assets::link_legacy_assets(&asset_index, &resolved, game_dir).await?;

    match (loader_type, loader_version) {
        ("fabric", Some(lv)) => {
            on_progress(format!("Installing Fabric {lv}..."), 80, 100);
            let artifact = fabric::fetch_fabric_loader_artifact(client, version_id, lv).await?;
            fabric::install_fabric_version_json(&artifact, version_id, game_dir).await?;
        }
        ("quilt", Some(lv)) => {
            on_progress(format!("Installing Quilt {lv}..."), 80, 100);
            let artifact = quilt::fetch_quilt_loader_artifact(client, version_id, lv).await?;
            quilt::install_quilt_version_json(&artifact, version_id, game_dir).await?;
        }
        ("forge", Some(lv)) => {
            on_progress(format!("Installing Forge {lv}..."), 80, 100);
            let on_progress_log = |line: String| on_progress(line, 85, 100);
            forge_install::install_forge(client, version_id, lv, game_dir, java_path, &on_progress_log).await?;
        }
        ("neoforge", Some(lv)) => {
            on_progress(format!("Installing NeoForge {lv}..."), 80, 100);
            let on_progress_log = |line: String| on_progress(line, 85, 100);
            forge_install::install_neoforge(client, lv, game_dir, java_path, &on_progress_log).await?;
        }
        _ => {}
    }

    on_progress("Done".to_string(), 100, 100);
    Ok(())
}

pub async fn delete_version(version_id: &str, game_dir: &Path) -> Result<(), String> {
    let dir = game_dir.join("versions").join(version_id);
    if fs::metadata(&dir).await.is_ok() {
        fs::remove_dir_all(&dir).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FabricLoaderVersionEntry {
    pub loader: FabricLoaderVersionInfo,
    pub intermediary: FabricIntermediaryInfo,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FabricLoaderVersionInfo {
    pub version: String,
    pub stable: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FabricIntermediaryInfo {
    pub version: String,
}

pub async fn list_fabric_versions(client: &reqwest::Client, mc_version: &str) -> Result<Vec<FabricLoaderVersionEntry>, String> {
    let text = client
        .get(format!("https://meta.fabricmc.net/v2/versions/loader/{mc_version}"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|_| format!("No Fabric loader available for Minecraft {mc_version}"))
}
