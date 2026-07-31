//! Port of `src/main/ipc/video.ts`.
//!
//! The Electron version ran a tiny local HTTP server on 127.0.0.1:49217 to serve scene.mp4 /
//! custom background files with Range support (needed for `<video>` seeking). Tauri's asset
//! protocol (enabled in `tauri.conf.json`'s `app.security.assetProtocol`) already does exactly
//! this — Range requests included — for any path in its `scope`, so no custom server is needed
//! here: these commands just resolve/copy the file and hand back an absolute path, and the
//! frontend wraps it with `convertFileSrc()` (see `src/api-bridge.ts`) to get a playable URL.

use std::fs;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

fn customization_media_dir(app: &AppHandle) -> std::path::PathBuf {
    let dir = crate::paths::user_data_dir().join("customization-media");
    let _ = fs::create_dir_all(&dir);
    let _ = app; // kept for parity with call sites that may need app-relative resolution later
    dir
}

pub fn get_scene_url(app: &AppHandle) -> Result<String, String> {
    // Packaged builds bundle scene.mp4 via `bundle.resources` (tauri.conf.json) so it never has
    // to be downloaded on first launch.
    let resource_path = app
        .path()
        .resolve("scene.mp4", tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;
    if resource_path.exists() {
        return Ok(resource_path.to_string_lossy().to_string());
    }

    // Dev fallback: the resources/ dir sitting next to the project during `tauri dev`.
    let dev_path = std::path::Path::new("resources/scene.mp4");
    if dev_path.exists() {
        return Ok(dev_path.canonicalize().unwrap_or_else(|_| dev_path.to_path_buf()).to_string_lossy().to_string());
    }

    Err("scene.mp4 not found — expected it bundled as a resource (see tauri.conf.json bundle.resources).".to_string())
}

pub fn pick_custom_bg(app: &AppHandle, kind: &str) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;

    let dialog = app.dialog().file();
    let picked = if kind == "video" {
        dialog.add_filter("Videos", &["mp4", "webm", "mov"]).blocking_pick_file()
    } else {
        dialog.add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"]).blocking_pick_file()
    }?;

    let src = std::path::PathBuf::from(picked.to_string());
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("");
    let filename = format!("{}.{ext}", Uuid::new_v4());
    let dest = customization_media_dir(app).join(&filename);
    fs::copy(&src, &dest).ok()?;

    Some(dest.to_string_lossy().to_string())
}
