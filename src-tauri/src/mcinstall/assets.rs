//! Port of `src/main/services/mcinstall/assets.ts`.

use super::downloader::{download_file, run_pool, summarize_failures, DownloadOptions};
use super::types::{AssetIndex, ResolvedVersion};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

const RESOURCES_BASE_URL: &str = "https://resources.download.minecraft.net";

pub async fn download_asset_index(client: &reqwest::Client, resolved_version: &ResolvedVersion, game_dir: &Path) -> Result<AssetIndex, String> {
    let dest = game_dir.join("assets").join("indexes").join(format!("{}.json", resolved_version.asset_index.id));
    download_file(
        client,
        &resolved_version.asset_index.url,
        &dest,
        DownloadOptions { sha1: Some(resolved_version.asset_index.sha1.clone()) },
    )
    .await?;
    let raw = fs::read_to_string(&dest).await.map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub async fn download_assets(
    client: &reqwest::Client,
    asset_index: &AssetIndex,
    game_dir: &Path,
    concurrency: usize,
) -> Result<(), String> {
    // assetIndex.objects is keyed by asset *name* — many names legitimately share the same
    // content hash. Dedupe by hash first so two names never race on the same <hash>.part temp path.
    let mut unique_objects: HashMap<String, u64> = HashMap::new();
    for obj in asset_index.objects.values() {
        unique_objects.insert(obj.hash.clone(), obj.size);
    }

    let game_dir = game_dir.to_path_buf();
    let client = client.clone();
    let objects: Vec<String> = unique_objects.into_keys().collect();
    let failures = run_pool(objects, concurrency, move |hash: String| {
        let prefix = hash[0..2].to_string();
        let dest = game_dir.join("assets").join("objects").join(&prefix).join(&hash);
        let url = format!("{RESOURCES_BASE_URL}/{prefix}/{hash}");
        let client = client.clone();
        async move { download_file(&client, &url, &dest, DownloadOptions { sha1: Some(hash.clone()) }).await }
    })
    .await;

    if let Some(summary) = summarize_failures(&failures, "assets") {
        return Err(summary);
    }
    Ok(())
}

/// Pre-1.7.10 clients read assets by their logical filename under assets/virtual/legacy/, not by
/// content hash — without this, those versions boot with sounds/textures silently missing.
pub async fn link_legacy_assets(asset_index: &AssetIndex, resolved_version: &ResolvedVersion, game_dir: &Path) -> Result<(), String> {
    if resolved_version.assets != "legacy" && resolved_version.assets != "pre-1.6" {
        return Ok(());
    }
    let virtual_dir = game_dir.join("assets").join("virtual").join("legacy");
    for (name, obj) in &asset_index.objects {
        let src = game_dir.join("assets").join("objects").join(&obj.hash[0..2]).join(&obj.hash);
        let dest = virtual_dir.join(name.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !src.exists() || dest.exists() {
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
        }
        fs::copy(&src, &dest).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}
