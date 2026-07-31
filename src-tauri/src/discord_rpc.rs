//! Port of `src/main/services/discordRPC.ts`, backed by the `discord-rich-presence` crate
//! instead of the Node `discord-rpc` package.

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use once_cell::sync::Lazy;
use parking_lot::Mutex;

const CLIENT_ID: &str = "1491721511168639016";

static CLIENT: Lazy<Mutex<Option<DiscordIpcClient>>> = Lazy::new(|| Mutex::new(None));
static START_TIMESTAMP: Lazy<Mutex<i64>> = Lazy::new(|| Mutex::new(chrono::Utc::now().timestamp()));

/// Discord not running or RPC unavailable is expected and non-fatal — every call here is
/// best-effort, same as the TS version's swallowed `.catch(() => {})`.
pub fn init_discord_rpc() {
    let mut client = match DiscordIpcClient::new(CLIENT_ID) {
        Ok(c) => c,
        Err(_) => return,
    };
    if client.connect().is_err() {
        return;
    }
    *START_TIMESTAMP.lock() = chrono::Utc::now().timestamp();
    *CLIENT.lock() = Some(client);
    set_idle_presence();
}

pub fn set_idle_presence() {
    let mut guard = CLIENT.lock();
    let Some(client) = guard.as_mut() else { return };
    let ts = *START_TIMESTAMP.lock();
    let _ = client.set_activity(
        activity::Activity::new()
            .details("Browsing the launcher")
            .state("Idle")
            .assets(activity::Assets::new().large_image("logo").large_text("BejaClient"))
            .timestamps(activity::Timestamps::new().start(ts)),
    );
}

pub fn set_playing_presence(version: Option<&str>) {
    let mut guard = CLIENT.lock();
    let Some(client) = guard.as_mut() else { return };
    let details = match version {
        Some(v) => format!("Playing Minecraft {v}"),
        None => "Playing Minecraft".to_string(),
    };
    let now = chrono::Utc::now().timestamp();
    let _ = client.set_activity(
        activity::Activity::new()
            .details(&details)
            .state("In Game")
            .assets(activity::Assets::new().large_image("logo").large_text("BejaClient"))
            .timestamps(activity::Timestamps::new().start(now)),
    );
}

pub fn destroy_discord_rpc() {
    if let Some(mut client) = CLIENT.lock().take() {
        let _ = client.close();
    }
}
