//! Port of `src/main/services/installTracker.ts`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::paths;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TrackerData {
    /// projectId → profileId[]
    pub mods: HashMap<String, Vec<String>>,
    /// "host:port" → profileId[]
    pub servers: HashMap<String, Vec<String>>,
}

fn tracker_path() -> std::path::PathBuf {
    paths::user_data_dir().join("install-tracker.json")
}

fn load() -> TrackerData {
    std::fs::read_to_string(tracker_path()).ok().and_then(|raw| serde_json::from_str(&raw).ok()).unwrap_or_default()
}

fn save(data: &TrackerData) {
    if let Ok(json) = serde_json::to_string_pretty(data) {
        let _ = std::fs::write(tracker_path(), json);
    }
}

pub fn record_mod_install(project_id: &str, profile_id: &str) {
    let mut data = load();
    let entry = data.mods.entry(project_id.to_string()).or_default();
    if !entry.contains(&profile_id.to_string()) {
        entry.push(profile_id.to_string());
        save(&data);
    }
}

pub fn record_server_add(host: &str, port: u16, profile_id: &str) {
    let key = format!("{host}:{port}");
    let mut data = load();
    let entry = data.servers.entry(key).or_default();
    if !entry.contains(&profile_id.to_string()) {
        entry.push(profile_id.to_string());
        save(&data);
    }
}

pub fn get_installs() -> TrackerData {
    load()
}
