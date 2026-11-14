//! Crash-safe mod updates: before `modrinth_swap_mod` replaces a mod jar, the old jar is copied
//! here first and logged, so a bad update (one that turns out to crash the game) can be
//! reverted — the missing piece a plain "update mod" button doesn't have on its own.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const MAX_LOG_ENTRIES: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLogEntry {
    pub timestamp: String,
    #[serde(rename = "modName")]
    pub mod_name: String,
    #[serde(rename = "oldFileName")]
    pub old_file_name: String,
    #[serde(rename = "newFileName")]
    pub new_file_name: String,
}

fn backups_dir(game_dir: &str) -> PathBuf {
    Path::new(game_dir).join(".beja-mod-update-backups")
}

fn log_path(game_dir: &str) -> PathBuf {
    backups_dir(game_dir).join("log.json")
}

fn read_log(game_dir: &str) -> Vec<UpdateLogEntry> {
    std::fs::read_to_string(log_path(game_dir)).ok().and_then(|raw| serde_json::from_str(&raw).ok()).unwrap_or_default()
}

fn write_log(game_dir: &str, entries: &[UpdateLogEntry]) {
    if let Ok(json) = serde_json::to_string_pretty(entries) {
        let _ = std::fs::write(log_path(game_dir), json);
    }
}

/// Call before deleting the old mod file. Best-effort: a failed backup just means rollback won't
/// be available for this one update, not a reason to fail the update itself.
pub fn record_update(game_dir: &str, old_file_path: &Path, old_file_name: &str, new_file_name: &str, mod_name: &str) {
    let dir = backups_dir(game_dir);
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let stamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S%.3f").to_string();
    let backup_path = dir.join(format!("{stamp}__{old_file_name}"));
    if std::fs::copy(old_file_path, &backup_path).is_err() {
        return;
    }

    let mut entries = read_log(game_dir);
    entries.push(UpdateLogEntry {
        timestamp: stamp,
        mod_name: mod_name.to_string(),
        old_file_name: old_file_name.to_string(),
        new_file_name: new_file_name.to_string(),
    });
    // Bound both the log and the backup files it references — an old backup whose log entry
    // gets dropped here is simply orphaned on disk, not reachable via rollback anymore.
    while entries.len() > MAX_LOG_ENTRIES {
        let dropped = entries.remove(0);
        let _ = std::fs::remove_file(dir.join(format!("{}__{}", dropped.timestamp, dropped.old_file_name)));
    }
    write_log(game_dir, &entries);
}

pub fn list_recent(game_dir: &str) -> Vec<UpdateLogEntry> {
    let mut entries = read_log(game_dir);
    entries.reverse();
    entries
}

/// Restores the most recent update's old jar and removes whatever replaced it. Returns the
/// rolled-back mod's display name for the confirmation toast.
pub fn rollback_last(game_dir: &str) -> Result<String, String> {
    let mut entries = read_log(game_dir);
    let entry = entries.pop().ok_or("No recent update to roll back")?;

    let backup_path = backups_dir(game_dir).join(format!("{}__{}", entry.timestamp, entry.old_file_name));
    let mods_dir = Path::new(game_dir).join("mods");
    let restored_path = mods_dir.join(&entry.old_file_name);
    std::fs::copy(&backup_path, &restored_path).map_err(|e| format!("Could not restore backup: {e}"))?;
    let _ = std::fs::remove_file(mods_dir.join(&entry.new_file_name));
    let _ = std::fs::remove_file(&backup_path);

    write_log(game_dir, &entries);
    Ok(entry.mod_name)
}
