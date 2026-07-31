//! Port of `src/main/services/consoleWindowService.ts` — a second always-there window mirroring
//! launch logs, for when the main window is hidden (`closeOnLaunch`).

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

static LOG_BUFFER: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
static READY: Mutex<bool> = Mutex::new(false);

const LABEL: &str = "console";

pub fn open_console_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(LABEL) {
        let _ = win.set_focus();
        return;
    }

    *LOG_BUFFER.lock() = Vec::new();
    *READY.lock() = false;

    let _ = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("BejaConsole")
        .inner_size(860.0, 520.0)
        .min_inner_size(580.0, 300.0)
        .background_color(tauri::window::Color(6, 8, 9, 255))
        .build();
    // The frontend router reads `window.location.hash` on startup to decide it should render
    // the console view instead of the main shell — see `src/router/index.ts`'s `#/console`
    // route (unchanged from the Electron build's `loadFile(..., { hash: '/console' })`).
}

/// Renderer signals when its IPC listeners are registered (after Vue `onMounted`).
#[tauri::command]
pub fn console_ready(app: AppHandle) {
    *READY.lock() = true;
    let buffered = std::mem::take(&mut *LOG_BUFFER.lock());
    if let Some(win) = app.get_webview_window(LABEL) {
        for line in buffered {
            let _ = win.emit("console:log", line);
        }
    }
}

pub fn send_console_log(app: &AppHandle, line: &str) {
    if !*READY.lock() {
        LOG_BUFFER.lock().push(line.to_string());
        return;
    }
    if let Some(win) = app.get_webview_window(LABEL) {
        let _ = win.emit("console:log", line);
    }
}

pub fn send_console_status(app: &AppHandle, status: &str) {
    if let Some(win) = app.get_webview_window(LABEL) {
        let _ = win.emit("console:status", status);
    }
}

pub fn send_console_clear(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(LABEL) {
        let _ = win.emit("console:clear", ());
    }
}
