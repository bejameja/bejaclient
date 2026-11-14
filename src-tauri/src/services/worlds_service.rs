//! Singleplayer world (`<gameDir>/saves/*`) management for the Worlds tab: list, backup to a
//! zip, restore from a zip, duplicate, rename, delete, reveal in file manager.

use serde::Serialize;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::paths::safe_join;
use crate::services::nbt_lite::{self, NbtValue};

#[derive(Debug, Clone, Serialize)]
pub struct WorldInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "lastPlayed")]
    pub last_played: Option<i64>,
    #[serde(rename = "sizeBytes")]
    pub size_bytes: u64,
    #[serde(rename = "iconDataUrl")]
    pub icon_data_url: Option<String>,
    pub hardcore: bool,
    #[serde(rename = "gameMode")]
    pub game_mode: Option<i32>,
    pub difficulty: Option<i32>,
}

fn saves_dir(game_dir: &str) -> PathBuf {
    Path::new(game_dir).join("saves")
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    let Ok(entries) = fs::read_dir(path) else { return 0 };
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if meta.is_dir() {
            total += dir_size(&entry.path());
        } else {
            total += meta.len();
        }
    }
    total
}

/// Recursively copies `src` into `dst` (which must not yet exist), skipping `session.lock`
/// (an OS file lock vanilla Minecraft holds on the currently-open world — copying it is
/// meaningless and on Windows a still-open world can keep it exclusively locked).
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == "session.lock" {
            continue;
        }
        let from = entry.path();
        let to = dst.join(&name);
        if entry.metadata()?.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn read_level_dat(world_dir: &Path) -> Option<NbtValue> {
    let bytes = fs::read(world_dir.join("level.dat")).ok()?;
    nbt_lite::parse_gzipped(&bytes)
}

fn world_info_from_dir(id: String, path: &Path) -> WorldInfo {
    let level = read_level_dat(path);
    let data = level.as_ref().and_then(|root| nbt_lite::dig(root, &["Data"]));

    let name = data
        .and_then(|d| nbt_lite::dig(d, &["LevelName"]))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| id.clone());

    let last_played = data
        .and_then(|d| nbt_lite::dig(d, &["LastPlayed"]))
        .and_then(|v| v.as_i64())
        .or_else(|| {
            fs::metadata(path.join("level.dat"))
                .ok()?
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_millis() as i64)
        });

    let hardcore = data.and_then(|d| nbt_lite::dig(d, &["hardcore"])).and_then(|v| v.as_bool()).unwrap_or(false);
    let game_mode = data.and_then(|d| nbt_lite::dig(d, &["GameType"])).and_then(|v| v.as_i32());
    let difficulty = data.and_then(|d| nbt_lite::dig(d, &["Difficulty"])).and_then(|v| v.as_i32());

    let icon_data_url = fs::read(path.join("icon.png"))
        .ok()
        .map(|bytes| format!("data:image/png;base64,{}", base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes)));

    WorldInfo {
        id,
        name,
        last_played,
        size_bytes: dir_size(path),
        icon_data_url,
        hardcore,
        game_mode,
        difficulty,
    }
}

pub fn list_worlds(game_dir: &str) -> Vec<WorldInfo> {
    let dir = saves_dir(game_dir);
    let Ok(entries) = fs::read_dir(&dir) else { return Vec::new() };
    let mut worlds: Vec<WorldInfo> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let id = e.file_name().to_str()?.to_string();
            Some(world_info_from_dir(id, &e.path()))
        })
        .collect();
    worlds.sort_by(|a, b| b.last_played.unwrap_or(0).cmp(&a.last_played.unwrap_or(0)));
    worlds
}

/// Sanitizes a user-provided string into a filesystem-safe folder name — mirrors vanilla's own
/// "Copy of <name>" folder-naming behavior (strip anything that isn't safe cross-platform).
fn sanitize_folder_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_control() || "\\/:*?\"<>|".contains(c) { '_' } else { c })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.');
    if trimmed.is_empty() { "World".to_string() } else { trimmed.chars().take(64).collect() }
}

fn unique_folder(base_dir: &Path, desired: &str) -> PathBuf {
    let mut candidate = base_dir.join(desired);
    let mut n = 2;
    while candidate.exists() {
        candidate = base_dir.join(format!("{desired} ({n})"));
        n += 1;
    }
    candidate
}

pub fn backup_world(game_dir: &str, world_id: &str, dest_zip: &Path) -> Result<(), String> {
    let world_dir = saves_dir(game_dir).join(world_id);
    if !world_dir.is_dir() {
        return Err("World not found".to_string());
    }
    let file = fs::File::create(dest_zip).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    fn add_dir(zip: &mut zip::ZipWriter<fs::File>, options: zip::write::SimpleFileOptions, base: &Path, current: &Path) -> std::io::Result<()> {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_name() == "session.lock" {
                continue;
            }
            let rel = path.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/");
            if entry.metadata()?.is_dir() {
                add_dir(zip, options, base, &path)?;
            } else {
                zip.start_file(rel, options)?;
                let bytes = fs::read(&path)?;
                zip.write_all(&bytes)?;
            }
        }
        Ok(())
    }

    // The archive's own root folder is the world id, so restoring elsewhere doesn't need the
    // caller to already know what the world was called.
    (|| -> std::io::Result<()> {
        add_dir(&mut zip, options, &world_dir.parent().unwrap().to_path_buf(), &world_dir)?;
        zip.finish()?;
        Ok(())
    })()
    .map_err(|e| e.to_string())
}

/// Restores a world backup made by `backup_world` (or any zip whose entries are rooted under a
/// single top-level folder) into `<gameDir>/saves/`, auto-renaming on a folder-name collision
/// rather than overwriting an existing world.
pub fn restore_world(game_dir: &str, zip_path: &Path) -> Result<WorldInfo, String> {
    let file = fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    // Determine the common root folder name inside the archive (backup_world always writes one).
    let root_name = {
        let first = zip.by_index(0).map_err(|e| e.to_string())?;
        let name = first.name().to_string();
        name.split('/').next().unwrap_or("World").to_string()
    };

    let saves = saves_dir(game_dir);
    fs::create_dir_all(&saves).map_err(|e| e.to_string())?;
    let dest_dir = unique_folder(&saves, &sanitize_folder_name(&root_name));
    let dest_id = dest_dir.file_name().unwrap().to_string_lossy().to_string();
    fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        if entry.is_dir() {
            continue;
        }
        let full_name = entry.name().to_string();
        // Strip the archive's own root-folder prefix so it lands directly under dest_dir.
        let rel = full_name.splitn(2, '/').nth(1).unwrap_or(&full_name);
        if rel.is_empty() {
            continue;
        }
        let dest = safe_join(&dest_dir, rel)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        fs::write(&dest, &buf).map_err(|e| e.to_string())?;
    }

    Ok(world_info_from_dir(dest_id, &dest_dir))
}

pub fn duplicate_world(game_dir: &str, world_id: &str, new_name: String) -> Result<WorldInfo, String> {
    let saves = saves_dir(game_dir);
    let src = saves.join(world_id);
    if !src.is_dir() {
        return Err("World not found".to_string());
    }
    let dest = unique_folder(&saves, &sanitize_folder_name(&new_name));
    copy_dir_recursive(&src, &dest).map_err(|e| e.to_string())?;
    let dest_id = dest.file_name().unwrap().to_string_lossy().to_string();
    apply_level_name(&dest, &new_name);
    Ok(world_info_from_dir(dest_id, &dest))
}

/// Patches `Data.LevelName` inside `level.dat` in place (full parse → mutate → re-serialize —
/// see `nbt_lite::write_gzipped`'s doc comment on why this is safe for a field-level edit).
fn apply_level_name(world_dir: &Path, new_name: &str) -> bool {
    let level_path = world_dir.join("level.dat");
    let Some(mut root) = read_level_dat(world_dir) else { return false };
    let NbtValue::Compound(root_map) = &mut root else { return false };
    let Some(NbtValue::Compound(data_map)) = root_map.get_mut("Data") else { return false };
    data_map.insert("LevelName".to_string(), NbtValue::String(new_name.to_string()));
    let Some(bytes) = nbt_lite::write_gzipped(&root) else { return false };
    fs::write(level_path, bytes).is_ok()
}

pub fn rename_world(game_dir: &str, world_id: &str, new_name: String) -> Result<WorldInfo, String> {
    let dir = saves_dir(game_dir).join(world_id);
    if !dir.is_dir() {
        return Err("World not found".to_string());
    }
    if !apply_level_name(&dir, &new_name) {
        return Err("Could not read or patch this world's level.dat (unsupported/corrupt format)".to_string());
    }
    Ok(world_info_from_dir(world_id.to_string(), &dir))
}

pub fn delete_world(game_dir: &str, world_id: &str) -> Result<(), String> {
    let dir = saves_dir(game_dir).join(world_id);
    if !dir.is_dir() {
        return Err("World not found".to_string());
    }
    fs::remove_dir_all(&dir).map_err(|e| e.to_string())
}

pub fn world_folder(game_dir: &str, world_id: &str) -> PathBuf {
    saves_dir(game_dir).join(world_id)
}
