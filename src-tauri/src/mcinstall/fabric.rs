//! Port of `src/main/services/mcinstall/fabric.ts`.

use super::types::{RawLibrary, RawVersionJson, VersionArguments};
use super::version_resolve::write_version_json;
use serde::Deserialize;
use std::path::Path;

const FABRIC_META: &str = "https://meta.fabricmc.net";

#[derive(Debug, Clone, Deserialize)]
pub struct FabricMavenLib {
    pub name: String,
    pub url: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FabricLauncherLibs {
    pub client: Vec<FabricMavenLib>,
    pub common: Vec<FabricMavenLib>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FabricMainClass {
    pub client: String,
    #[allow(dead_code)]
    pub server: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FabricLauncherMeta {
    pub libraries: FabricLauncherLibs,
    #[serde(rename = "mainClass")]
    pub main_class: FabricMainClass,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FabricMavenRef {
    pub maven: String,
    #[allow(dead_code)]
    pub version: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FabricLoaderArtifact {
    pub loader: FabricLoaderRef,
    pub intermediary: FabricMavenRef,
    #[serde(rename = "launcherMeta")]
    pub launcher_meta: FabricLauncherMeta,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FabricLoaderRef {
    pub maven: String,
    pub version: String,
    #[allow(dead_code)]
    pub stable: bool,
    #[allow(dead_code)]
    pub build: u64,
}

async fn https_get_text(client: &reqwest::Client, url: &str) -> Result<String, String> {
    client
        .get(url)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}

pub async fn fetch_fabric_loader_artifact(client: &reqwest::Client, mc_version: &str, loader_version: &str) -> Result<FabricLoaderArtifact, String> {
    let raw = https_get_text(client, &format!("{FABRIC_META}/v2/versions/loader/{mc_version}/{loader_version}")).await?;
    // Fabric's meta API returns plain-text error bodies for invalid mc/loader combos instead of JSON.
    serde_json::from_str(&raw).map_err(|_| {
        let snippet: String = raw.chars().take(200).collect();
        format!(
            "Fabric loader {loader_version} is not available for Minecraft {mc_version}: {}",
            if snippet.trim().is_empty() { "(empty response)".to_string() } else { snippet.trim().to_string() }
        )
    })
}

pub fn generate_fabric_version_json(artifact: &FabricLoaderArtifact, mc_version: &str) -> RawVersionJson {
    let mut libraries = vec![
        RawLibrary { name: artifact.loader.maven.clone(), url: Some("https://maven.fabricmc.net/".to_string()), ..Default::default() },
        RawLibrary { name: artifact.intermediary.maven.clone(), url: Some("https://maven.fabricmc.net/".to_string()), ..Default::default() },
    ];
    // launcherMeta.libraries.development (e.g. mixinextras-fabric) is intentionally excluded —
    // it's for Fabric Loom's dev/test environment; mods needing it shade it into their own jar.
    for l in &artifact.launcher_meta.libraries.client {
        libraries.push(RawLibrary { name: l.name.clone(), url: Some(l.url.clone()), ..Default::default() });
    }
    for l in &artifact.launcher_meta.libraries.common {
        libraries.push(RawLibrary { name: l.name.clone(), url: Some(l.url.clone()), ..Default::default() });
    }

    let now = chrono::Utc::now().to_rfc3339();
    RawVersionJson {
        id: format!("{mc_version}-fabric{}", artifact.loader.version),
        inherits_from: Some(mc_version.to_string()),
        r#type: "release".to_string(),
        main_class: artifact.launcher_meta.main_class.client.clone(),
        libraries,
        arguments: Some(VersionArguments { game: vec![], jvm: vec![] }),
        release_time: Some(now.clone()),
        time: Some(now),
        ..Default::default()
    }
}

/// Pure local write — no network call beyond the metadata fetch already performed.
pub async fn install_fabric_version_json(artifact: &FabricLoaderArtifact, mc_version: &str, game_dir: &Path) -> Result<String, String> {
    let json = generate_fabric_version_json(artifact, mc_version);
    write_version_json(game_dir, &json).await?;
    Ok(json.id)
}
