//! Port of `src/main/services/mcinstall/quilt.ts`.

use super::types::{RawLibrary, RawVersionJson, VersionArguments};
use super::version_resolve::write_version_json;
use serde::Deserialize;
use std::path::Path;

const QUILT_META: &str = "https://meta.quiltmc.org";

#[derive(Debug, Clone, Deserialize)]
pub struct QuiltMavenLib {
    pub name: String,
    pub url: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct QuiltLauncherLibs {
    pub client: Vec<QuiltMavenLib>,
    pub common: Vec<QuiltMavenLib>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct QuiltMainClass {
    pub client: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct QuiltLauncherMeta {
    pub libraries: QuiltLauncherLibs,
    #[serde(rename = "mainClass")]
    pub main_class: QuiltMainClass,
}
#[derive(Debug, Clone, Deserialize)]
pub struct QuiltMavenRef {
    pub maven: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct QuiltLoaderRef {
    pub maven: String,
    pub version: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct QuiltLoaderArtifact {
    pub loader: QuiltLoaderRef,
    pub hashed: QuiltMavenRef,
    pub intermediary: QuiltMavenRef,
    #[serde(rename = "launcherMeta")]
    pub launcher_meta: QuiltLauncherMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuiltVersionListEntry {
    pub loader: QuiltVersionInfo,
}
#[derive(Debug, Clone, Deserialize)]
pub struct QuiltVersionInfo {
    pub version: String,
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

pub async fn fetch_quilt_loader_artifact(client: &reqwest::Client, mc_version: &str, loader_version: &str) -> Result<QuiltLoaderArtifact, String> {
    let raw = https_get_text(client, &format!("{QUILT_META}/v3/versions/loader/{mc_version}/{loader_version}")).await?;
    serde_json::from_str(&raw).map_err(|_| format!("Quilt loader {loader_version} is not available for Minecraft {mc_version}"))
}

pub async fn list_quilt_versions(client: &reqwest::Client, mc_version: &str) -> Result<Vec<QuiltVersionListEntry>, String> {
    let raw = https_get_text(client, &format!("{QUILT_META}/v3/versions/loader/{mc_version}")).await?;
    serde_json::from_str(&raw).map_err(|_| format!("No Quilt loader available for Minecraft {mc_version}"))
}

pub fn generate_quilt_version_json(artifact: &QuiltLoaderArtifact, mc_version: &str) -> RawVersionJson {
    let mut libraries = vec![
        RawLibrary { name: artifact.loader.maven.clone(), url: Some("https://maven.quiltmc.org/repository/release/".to_string()), ..Default::default() },
        RawLibrary { name: artifact.hashed.maven.clone(), url: Some("https://maven.quiltmc.org/repository/release/".to_string()), ..Default::default() },
        RawLibrary { name: artifact.intermediary.maven.clone(), url: Some("https://maven.fabricmc.net/".to_string()), ..Default::default() },
    ];
    // launcherMeta.libraries.development is intentionally excluded — it's Quilt's own
    // dev-environment tooling, not needed (or wanted) on a real launch.
    for l in &artifact.launcher_meta.libraries.client {
        libraries.push(RawLibrary { name: l.name.clone(), url: Some(l.url.clone()), ..Default::default() });
    }
    for l in &artifact.launcher_meta.libraries.common {
        libraries.push(RawLibrary { name: l.name.clone(), url: Some(l.url.clone()), ..Default::default() });
    }

    let now = chrono::Utc::now().to_rfc3339();
    RawVersionJson {
        id: format!("{mc_version}-quilt{}", artifact.loader.version),
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

pub async fn install_quilt_version_json(artifact: &QuiltLoaderArtifact, mc_version: &str, game_dir: &Path) -> Result<String, String> {
    let json = generate_quilt_version_json(artifact, mc_version);
    write_version_json(game_dir, &json).await?;
    Ok(json.id)
}
