//! In-launcher screenshot gallery: lists `<gameDir>/screenshots/*.png`, generates small preview
//! thumbnails on demand, and forwards open/export/delete to the OS.

use serde::Serialize;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

const THUMB_MAX_SIDE: u32 = 360;

#[derive(Debug, Clone, Serialize)]
pub struct ScreenshotInfo {
    pub id: String,
    #[serde(rename = "takenAt")]
    pub taken_at: i64,
    #[serde(rename = "sizeBytes")]
    pub size_bytes: u64,
}

fn screenshots_dir(game_dir: &str) -> PathBuf {
    Path::new(game_dir).join("screenshots")
}

fn file_for(game_dir: &str, id: &str) -> Option<PathBuf> {
    // `id` is always a filename straight from `list_screenshots`, but never trust that blindly —
    // reject anything that could escape the screenshots dir before joining.
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return None;
    }
    let path = screenshots_dir(game_dir).join(id);
    path.is_file().then_some(path)
}

pub fn list_screenshots(game_dir: &str) -> Vec<ScreenshotInfo> {
    let Ok(entries) = fs::read_dir(screenshots_dir(game_dir)) else { return Vec::new() };
    let mut shots: Vec<ScreenshotInfo> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let ext = path.extension()?.to_str()?.to_lowercase();
            if ext != "png" {
                return None;
            }
            let meta = entry.metadata().ok()?;
            let taken_at = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            Some(ScreenshotInfo {
                id: entry.file_name().to_str()?.to_string(),
                taken_at,
                size_bytes: meta.len(),
            })
        })
        .collect();
    shots.sort_by(|a, b| b.taken_at.cmp(&a.taken_at));
    shots
}

/// Downscaled JPEG data URL for the grid — full PNGs (often several MB at modern resolutions)
/// would make a gallery of dozens of shots slow to render and heavy to pipe over Tauri's IPC.
pub fn thumbnail(game_dir: &str, id: &str) -> Option<String> {
    let path = file_for(game_dir, id)?;
    let img = image::open(&path).ok()?;
    let thumb = img.thumbnail(THUMB_MAX_SIDE, THUMB_MAX_SIDE);
    let mut buf = Cursor::new(Vec::new());
    thumb.to_rgb8().write_to(&mut buf, image::ImageFormat::Jpeg).ok()?;
    Some(format!(
        "data:image/jpeg;base64,{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, buf.get_ref())
    ))
}

pub fn delete_screenshot(game_dir: &str, id: &str) -> Result<(), String> {
    let path = file_for(game_dir, id).ok_or("Screenshot not found")?;
    fs::remove_file(path).map_err(|e| e.to_string())
}

pub fn path_for(game_dir: &str, id: &str) -> Option<PathBuf> {
    file_for(game_dir, id)
}

pub fn folder(game_dir: &str) -> PathBuf {
    screenshots_dir(game_dir)
}
