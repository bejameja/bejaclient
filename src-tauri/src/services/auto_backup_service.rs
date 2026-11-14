//! Scheduled world backups: zips every singleplayer world for a profile right after a play
//! session ends, pruning older backups beyond the configured retention count. Builds entirely on
//! `worlds_service`'s existing (manual) backup/list primitives — this is just the "when" and
//! "how many to keep" layer on top.

use std::path::PathBuf;

use crate::paths;
use crate::services::{profile_service, settings_service, worlds_service};

fn backups_root(profile_id: &str, world_id: &str) -> PathBuf {
    paths::user_data_dir().join("world-backups").join(profile_id).join(world_id)
}

/// Fire-and-forget: spawns its own background task so the caller (the launch-exit handler)
/// never blocks on zipping potentially several worlds. Safe to call unconditionally — it's a
/// no-op unless the setting is on.
pub fn run_auto_backup_if_enabled(profile_id: &str) {
    let settings = settings_service::get_settings();
    if !settings.launcher.auto_backup_worlds {
        return;
    }
    let Some(profile) = profile_service::get_profile(profile_id) else { return };
    let profile_id = profile_id.to_string();
    let retention = settings.launcher.auto_backup_retention.max(1) as usize;

    tauri::async_runtime::spawn_blocking(move || {
        let game_dir = if !profile.game_dir.is_empty() { profile.game_dir } else { settings.game.default_game_dir };
        let worlds = worlds_service::list_worlds(&game_dir);
        for world in worlds {
            let dir = backups_root(&profile_id, &world.id);
            if std::fs::create_dir_all(&dir).is_err() {
                continue;
            }
            // Sortable filename (ISO-ish, colons stripped for Windows) doubles as the "newest
            // first" ordering `prune` relies on — no need to read file mtimes separately.
            let stamp = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
            let dest = dir.join(format!("{stamp}.zip"));
            if let Err(e) = worlds_service::backup_world(&game_dir, &world.id, &dest) {
                log::warn!("[AutoBackup] Failed to back up world '{}': {e}", world.id);
                continue;
            }
            prune(&dir, retention);
        }
    });
}

fn prune(dir: &std::path::Path, retention: usize) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut zips: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("zip"))
        .collect();
    // Filenames are the zero-padded timestamp itself, so lexicographic order is chronological.
    zips.sort();
    if zips.len() > retention {
        for old in &zips[..zips.len() - retention] {
            let _ = std::fs::remove_file(old);
        }
    }
}

pub fn list_backups(profile_id: &str, world_id: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(backups_root(profile_id, world_id)) else { return Vec::new() };
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".zip"))
        .collect();
    names.sort();
    names.reverse();
    names
}

pub fn restore_backup(profile_id: &str, world_id: &str, backup_name: &str, game_dir: &str) -> Result<worlds_service::WorldInfo, String> {
    if backup_name.contains('/') || backup_name.contains('\\') || backup_name.contains("..") {
        return Err("Invalid backup name".to_string());
    }
    let zip_path = backups_root(profile_id, world_id).join(backup_name);
    if !zip_path.is_file() {
        return Err("Backup not found".to_string());
    }
    worlds_service::restore_world(game_dir, &zip_path)
}
