mod commands;
mod console_window;
mod discord_rpc;
pub mod mcinstall;
pub mod paths;
pub mod services;
mod updater;
mod video;

use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_deep_link::DeepLinkExt;

static PENDING_DEEP_LINK: Mutex<Option<String>> = Mutex::new(None);

fn handle_deep_link(app: &tauri::AppHandle, url: &str) {
    // bejaclient://install/<shareId>
    let Some(rest) = url.strip_prefix("bejaclient://install/") else { return };
    let share_id: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect();
    if share_id.is_empty() {
        return;
    }
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.emit("profiles:shared-link", &share_id);
    } else {
        *PENDING_DEEP_LINK.lock().unwrap() = Some(share_id);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            // Windows/Linux: a second launch while already running arrives here instead of
            // spawning a new window.
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.unminimize();
                let _ = win.set_focus();
            }
            if let Some(url) = argv.iter().find(|a| a.starts_with("bejaclient://")) {
                handle_deep_link(app, url);
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(tauri_plugin_log::Builder::default().level(log::LevelFilter::Info).build())?;
            }

            // ── Deep links (bejaclient://install/<shareId>) ──────────────────────────
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            {
                let _ = app.deep_link().register("bejaclient");
            }
            {
                let handle = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                    if let Some(url) = event.urls().first() {
                        handle_deep_link(&handle, url.as_str());
                    }
                });
            }
            // First launch (app wasn't already running) can also carry the deep link as an argv.
            if let Some(url) = std::env::args().find(|a| a.starts_with("bejaclient://")) {
                handle_deep_link(&app.handle().clone(), &url);
            }

            // ── Main window ───────────────────────────────────────────────────────────
            // The Electron build started hidden and showed on `ready-to-show` to avoid an
            // unstyled flash. Tauri v2 has no reliably-firing equivalent webview-ready event
            // for this (relying on one left the window invisible forever), so the window is
            // just visible from the start (`visible: true` in tauri.conf.json) — maximize it
            // immediately instead.
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.maximize();

                if let Some(share_id) = PENDING_DEEP_LINK.lock().unwrap().take() {
                    let _ = win.emit("profiles:shared-link", share_id);
                }

                let win_events = win.clone();
                win.on_window_event(move |event| match event {
                    tauri::WindowEvent::Resized(_) => {
                        let maximized = win_events.is_maximized().unwrap_or(false);
                        let _ = win_events.emit("window:maximized", maximized);
                    }
                    _ => {}
                });
            }

            // ── Background services ───────────────────────────────────────────────────
            discord_rpc::init_discord_rpc();
            services::auth_service::start_token_auto_refresh(app.handle().clone());

            let settings = services::settings_service::get_settings();
            if settings.launcher.auto_update {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    let _ = updater::check_for_update(app_handle).await;
                });
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" && matches!(event, tauri::WindowEvent::Destroyed) {
                discord_rpc::destroy_discord_rpc();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // auth
            commands::auth_login,
            commands::auth_logout,
            commands::auth_list_accounts,
            commands::auth_select_account,
            commands::auth_refresh,
            commands::auth_import_launcher,
            // versions
            commands::versions_list_remote,
            commands::versions_list_installed,
            commands::versions_install,
            commands::versions_delete,
            commands::versions_list_fabric,
            commands::versions_list_forge,
            // launch
            commands::launch_start,
            commands::launch_kill,
            commands::launch_open_console,
            commands::launch_save_logs,
            commands::launch_start_server,
            // profiles
            commands::profiles_list,
            commands::profiles_create,
            commands::profiles_update,
            commands::profiles_delete,
            commands::profiles_get_active,
            commands::profiles_set_active,
            commands::profiles_export,
            commands::profiles_import,
            commands::profiles_share,
            commands::profiles_peek_shared,
            commands::profiles_import_shared,
            // mods
            commands::mods_list,
            commands::mods_check_conflicts,
            commands::mods_install,
            commands::mods_toggle,
            commands::mods_delete,
            commands::mods_open_folder,
            commands::mods_auto_fix,
            // settings
            commands::settings_get,
            commands::settings_set,
            commands::settings_game_dir,
            commands::settings_set_game_dir,
            commands::settings_choose_java,
            commands::settings_choose_dir,
            // modrinth / explore
            commands::modrinth_search,
            commands::modrinth_categories,
            commands::explore_search,
            commands::curseforge_install,
            commands::modrinth_versions,
            commands::modrinth_install_mod,
            commands::modrinth_install_modpack,
            commands::modrinth_install_resourcepack,
            commands::modrinth_install_shader,
            commands::modrinth_install_datapack,
            commands::modrinth_swap_mod,
            // system
            commands::system_get_version,
            commands::system_java_versions,
            commands::system_open_external,
            // console
            console_window::console_ready,
            // players
            commands::players_lookup,
            commands::players_save_skin,
            commands::players_fetch_image,
            commands::players_mc_profile,
            commands::players_beja_profile,
            commands::players_mc_created,
            commands::players_capes,
            commands::players_search,
            commands::players_set_skin,
            commands::players_set_skin_file,
            commands::players_set_cape,
            commands::players_clear_cape,
            // friends
            commands::friends_connect,
            commands::friends_disconnect,
            commands::friends_list,
            commands::friends_request,
            commands::friends_accept,
            commands::friends_remove,
            // stats
            commands::stats_online,
            // lobby
            commands::lobby_emit,
            // cosmetics
            commands::cosmetics_get,
            commands::cosmetics_update,
            commands::cosmetics_inventory,
            // crates / crafting
            commands::crates_list,
            commands::crates_open,
            commands::crates_keys,
            commands::crafting_inventory,
            commands::crafting_combine,
            // capes
            commands::capes_list,
            commands::capes_upload,
            commands::capes_report,
            // pass
            commands::pass_get,
            commands::pass_progress,
            commands::pass_daily,
            // quests
            commands::quests_list,
            commands::quests_progress,
            commands::quests_claim,
            commands::quests_leaderboard,
            // wallet / shop
            commands::wallet_balance,
            commands::shop_list,
            commands::shop_purchase,
            // installs
            commands::installs_get,
            // servers
            commands::servers_list,
            commands::servers_ping,
            commands::servers_add,
            commands::servers_remove,
            commands::servers_add_to_profile,
            // chat
            commands::chat_send,
            commands::chat_typing,
            commands::chat_history,
            // video
            commands::video_get_scene,
            commands::video_pick_custom_bg,
            // updater
            commands::updater_check,
            commands::updater_download,
            commands::updater_install,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
