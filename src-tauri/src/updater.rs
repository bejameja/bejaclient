//! Port of `src/main/services/updaterService.ts` + `src/main/ipc/updater.ts`, backed by
//! `tauri-plugin-updater` instead of `electron-updater`. The frontend calls `check()`,
//! `download()`, and `install()` as three separate round-trips (see `UpdateNotification.vue`'s
//! state machine), so the in-progress `Update` handle + downloaded bytes are kept in a static
//! between those calls.

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

static PENDING: Lazy<Mutex<Option<tauri_plugin_updater::Update>>> = Lazy::new(|| Mutex::new(None));
static DOWNLOADED_BYTES: Lazy<Mutex<Option<Vec<u8>>>> = Lazy::new(|| Mutex::new(None));

pub async fn check_for_update(app: AppHandle) -> Result<(), String> {
    let _ = app.emit("updater:checking", ());
    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await {
        Ok(Some(update)) => {
            let _ = app.emit("updater:available", serde_json::json!({ "version": update.version, "releaseNotes": update.body }));
            *PENDING.lock() = Some(update);
        }
        Ok(None) => {
            let _ = app.emit("updater:not-available", ());
        }
        Err(e) => {
            // A failed *check* almost always just means the update backend/manifest isn't
            // reachable or doesn't exist yet (no release published, endpoint not deployed,
            // transient network hiccup on an unattended startup check) — not something an end
            // user should ever see as a red error toast. Only failures *after* an update was
            // already found (download/install, handled separately below) are real errors worth
            // surfacing. Still logged, just not emitted to the frontend.
            log::warn!("[Updater] check failed (treated as not-available): {e}");
            let _ = app.emit("updater:not-available", ());
        }
    }
    Ok(())
}

pub async fn download_update(app: AppHandle) -> Result<(), String> {
    let update = PENDING.lock().clone();
    let Some(update) = update else { return Err("No update available to download — call check() first.".to_string()) };

    let mut transferred: u64 = 0;
    let mut total: u64 = 0;
    let app_progress = app.clone();
    let result = update
        .download(
            move |chunk_len, content_len| {
                transferred += chunk_len as u64;
                if let Some(t) = content_len {
                    total = t;
                }
                let percent = if total > 0 { (transferred * 100 / total) as u32 } else { 0 };
                let _ = app_progress.emit(
                    "updater:progress",
                    serde_json::json!({ "percent": percent, "transferred": transferred, "total": total, "bytesPerSecond": 0 }),
                );
            },
            || {},
        )
        .await;

    match result {
        Ok(bytes) => {
            let version = update.version.clone();
            *DOWNLOADED_BYTES.lock() = Some(bytes);
            let _ = app.emit("updater:downloaded", serde_json::json!({ "version": version }));
            Ok(())
        }
        Err(e) => {
            let _ = app.emit("updater:error", e.to_string());
            Err(e.to_string())
        }
    }
}

pub fn install_update(app: AppHandle) -> Result<(), String> {
    let update = PENDING.lock().clone().ok_or("No update available to install — call check() and download() first.")?;
    let bytes = DOWNLOADED_BYTES.lock().take().ok_or("Update hasn't finished downloading yet.")?;
    update.install(bytes).map_err(|e| e.to_string())?;
    app.restart();
}
