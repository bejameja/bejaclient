//! Port of `src/main/services/mcinstall/types.ts`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Platform {
    /// "windows" | "osx" | "linux"
    pub name: String,
    pub version: String,
    pub arch: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RuleOs {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    /// "allow" | "disallow"
    pub action: String,
    #[serde(default)]
    pub os: Option<RuleOs>,
    #[serde(default)]
    pub features: Option<HashMap<String, bool>>,
}

/// `ArgumentEntry = string | { rules: Rule[]; value: string | string[] }` — kept as raw JSON
/// since serde untagged enums with this shape are a common footgun; parsed ad hoc where used.
pub type ArgumentEntry = Value;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DownloadArtifact {
    #[serde(default)]
    pub path: Option<String>,
    pub url: String,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<DownloadArtifact>,
    #[serde(default)]
    pub classifiers: Option<HashMap<String, DownloadArtifact>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LibraryExtract {
    #[serde(default)]
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RawLibrary {
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub rules: Option<Vec<Rule>>,
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
    #[serde(default)]
    pub extract: Option<LibraryExtract>,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetIndexRef {
    pub id: String,
    pub url: String,
    pub sha1: String,
    pub size: u64,
    #[serde(default, rename = "totalSize")]
    pub total_size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct VersionArguments {
    #[serde(default)]
    pub game: Vec<ArgumentEntry>,
    #[serde(default)]
    pub jvm: Vec<ArgumentEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct VersionDownloads {
    #[serde(default)]
    pub client: Option<DownloadArtifact>,
    #[serde(default)]
    pub server: Option<DownloadArtifact>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct JavaVersionRef {
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
    #[serde(default)]
    pub component: Option<String>,
}

/// A version JSON as read straight off disk/network — may be a partial "child" link in an
/// inheritsFrom chain.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RawVersionJson {
    pub id: String,
    #[serde(default, rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
    #[serde(default)]
    pub r#type: String,
    #[serde(default, rename = "mainClass")]
    pub main_class: String,
    #[serde(default, rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    #[serde(default)]
    pub arguments: Option<VersionArguments>,
    #[serde(default)]
    pub libraries: Vec<RawLibrary>,
    #[serde(default, rename = "assetIndex")]
    pub asset_index: Option<AssetIndexRef>,
    #[serde(default)]
    pub assets: Option<String>,
    #[serde(default)]
    pub downloads: Option<VersionDownloads>,
    #[serde(default, rename = "javaVersion")]
    pub java_version: Option<JavaVersionRef>,
    #[serde(default, rename = "releaseTime")]
    pub release_time: Option<String>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default, rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: Option<u32>,
}

/// Fully merged version — every field guaranteed present after walking the inheritsFrom chain.
#[derive(Debug, Clone)]
pub struct ResolvedVersion {
    pub id: String,
    pub r#type: String,
    pub main_class: String,
    pub minecraft_arguments: Option<String>,
    pub arguments: Option<VersionArguments>,
    pub libraries: Vec<RawLibrary>,
    pub asset_index: AssetIndexRef,
    pub assets: String,
    pub java_version: Option<JavaVersionRef>,
}

#[derive(Debug, Clone)]
pub struct ResolvedLibrary {
    pub name: String,
    pub path: String,
    pub url: String,
    pub sha1: Option<String>,
    pub size: Option<u64>,
    pub is_native: bool,
    pub native_classifier: Option<String>,
    pub extract_exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
    #[serde(default)]
    pub r#virtual: Option<bool>,
    #[serde(default)]
    pub map_to_resources: Option<bool>,
}
