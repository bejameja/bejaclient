//! Port of `src/main/services/mcinstall/versionResolve.ts`.

use super::downloader::{download_file, DownloadOptions};
use super::types::{RawLibrary, RawVersionJson, ResolvedVersion, VersionArguments};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;

fn version_json_path(game_dir: &Path, version_id: &str) -> PathBuf {
    game_dir.join("versions").join(version_id).join(format!("{version_id}.json"))
}

fn version_jar_path(game_dir: &Path, version_id: &str) -> PathBuf {
    game_dir.join("versions").join(version_id).join(format!("{version_id}.jar"))
}

async fn https_get_json(client: &reqwest::Client, url: &str) -> Result<Value, String> {
    let res = client
        .get(url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    res.json::<Value>().await.map_err(|e| e.to_string())
}

/// Downloads the version JSON + client jar for a base (vanilla) version manifest entry. Fetches
/// the JSON before creating the version directory — if the network request fails, no
/// `versions/<id>/` directory is left behind (its existence is used as a cheap "already
/// installed" check elsewhere).
pub async fn download_version_json_and_jar(
    client: &reqwest::Client,
    version_id: &str,
    manifest_url: &str,
    game_dir: &Path,
) -> Result<RawVersionJson, String> {
    let raw = https_get_json(client, manifest_url).await?;
    let json: RawVersionJson = serde_json::from_value(raw.clone()).map_err(|e| e.to_string())?;

    let dir = game_dir.join("versions").join(version_id);
    fs::create_dir_all(&dir).await.map_err(|e| e.to_string())?;
    fs::write(version_json_path(game_dir, version_id), raw.to_string())
        .await
        .map_err(|e| e.to_string())?;

    if let Some(downloads) = &json.downloads {
        if let Some(client_artifact) = &downloads.client {
            download_file(
                client,
                &client_artifact.url,
                &version_jar_path(game_dir, version_id),
                DownloadOptions { sha1: client_artifact.sha1.clone() },
            )
            .await?;
        }
    }

    Ok(json)
}

pub async fn write_version_json(game_dir: &Path, json: &RawVersionJson) -> Result<(), String> {
    let dir = game_dir.join("versions").join(&json.id);
    fs::create_dir_all(&dir).await.map_err(|e| e.to_string())?;
    let raw = serde_json::to_string(json).map_err(|e| e.to_string())?;
    fs::write(version_json_path(game_dir, &json.id), raw).await.map_err(|e| e.to_string())
}

/// Walks the inheritsFrom chain starting at version_id, returns [root, ..., mostDerived].
pub async fn read_version_json_chain(game_dir: &Path, version_id: &str) -> Result<Vec<RawVersionJson>, String> {
    let mut chain = Vec::new();
    let mut current_id = Some(version_id.to_string());
    let mut seen = HashSet::new();

    while let Some(id) = current_id {
        if seen.contains(&id) {
            return Err(format!("Circular inheritsFrom chain detected at {id}"));
        }
        seen.insert(id.clone());

        let path = version_json_path(game_dir, &id);
        if !path.exists() {
            return Err(format!(
                "Version JSON missing for \"{id}\" (needed by {version_id}'s inheritsFrom chain). Install {id} before launching {version_id}."
            ));
        }
        let raw = fs::read_to_string(&path).await.map_err(|e| e.to_string())?;
        let json: RawVersionJson = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        current_id = json.inherits_from.clone();
        chain.insert(0, json);
    }

    Ok(chain)
}

/// Includes the classifier (4th Maven coordinate segment) in the key — modern LWJGL libraries
/// ship native variants as sibling entries with the same group:artifact but a different
/// classifier; keying on group:artifact alone would collapse them.
fn library_key(lib: &RawLibrary) -> String {
    let parts: Vec<&str> = lib.name.split(':').collect();
    format!(
        "{}:{}:{}",
        parts.first().unwrap_or(&""),
        parts.get(1).unwrap_or(&""),
        parts.get(3).unwrap_or(&"")
    )
}

/// Merges a version JSON inheritance chain (root/vanilla first, most-derived last) into one
/// fully-resolved version. Libraries concat child-first, de-duped by group:artifact keeping the
/// most-derived occurrence. Arguments concat child-then-parent. Scalars take the most-derived
/// defined value.
pub async fn resolve_version(game_dir: &Path, version_id: &str) -> Result<ResolvedVersion, String> {
    let chain = read_version_json_chain(game_dir, version_id).await?;
    let most_derived_first: Vec<&RawVersionJson> = chain.iter().rev().collect();

    let mut main_class = String::new();
    let mut asset_index = None;
    let mut assets = String::new();
    let mut r#type = "release".to_string();
    let mut java_version = None;
    let mut minecraft_arguments = None;
    let mut seen_structured = false;
    let mut seen_flat = false;

    let mut libraries = Vec::new();
    let mut seen_lib_keys = HashSet::new();
    let mut game_args = Vec::new();
    let mut jvm_args = Vec::new();

    for (i, version) in most_derived_first.iter().enumerate() {
        if main_class.is_empty() && !version.main_class.is_empty() {
            main_class = version.main_class.clone();
        }
        if asset_index.is_none() {
            asset_index = version.asset_index.clone();
        }
        if assets.is_empty() {
            if let Some(a) = &version.assets {
                assets = a.clone();
            }
        }
        if java_version.is_none() {
            java_version = version.java_version.clone();
        }
        if i == 0 && !version.r#type.is_empty() {
            r#type = version.r#type.clone();
        }

        if let Some(args) = &version.arguments {
            seen_structured = true;
            game_args.extend(args.game.iter().cloned());
            jvm_args.extend(args.jvm.iter().cloned());
        } else if let Some(flat) = &version.minecraft_arguments {
            if minecraft_arguments.is_none() {
                seen_flat = true;
                minecraft_arguments = Some(flat.clone());
            }
        }

        for lib in &version.libraries {
            let key = library_key(lib);
            if seen_lib_keys.contains(&key) {
                continue;
            }
            seen_lib_keys.insert(key);
            libraries.push(lib.clone());
        }
    }

    if main_class.is_empty() {
        return Err(format!("No mainClass resolved for version {version_id}"));
    }
    let Some(asset_index) = asset_index else {
        return Err(format!("No assetIndex resolved for version {version_id}"));
    };
    if assets.is_empty() {
        return Err(format!("No assetIndex resolved for version {version_id}"));
    }

    let arguments = if seen_structured {
        Some(VersionArguments { game: game_args, jvm: jvm_args })
    } else {
        None
    };
    let minecraft_arguments = if seen_flat { minecraft_arguments } else { None };

    Ok(ResolvedVersion {
        id: version_id.to_string(),
        r#type,
        main_class,
        minecraft_arguments,
        arguments,
        libraries,
        asset_index,
        assets,
        java_version,
    })
}
