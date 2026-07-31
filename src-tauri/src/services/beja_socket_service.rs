//! Port of `src/main/services/bejaSocketService.ts`.

use futures_util::FutureExt;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rust_socketio::{
    asynchronous::{Client, ClientBuilder},
    Payload,
};
use serde_json::json;
use tauri::{AppHandle, Emitter};

use super::auth_service::BEJA_API;

static SOCKET: Lazy<Mutex<Option<Client>>> = Lazy::new(|| Mutex::new(None));

/// Events that are just relayed 1:1 from the Beja server's socket.io connection to the
/// frontend, under the same event name, with no server-side processing needed.
const RELAYED_EVENTS: &[&str] = &[
    "friend:online",
    "friend:offline",
    "friend:request",
    "friend:accepted",
    "friend:removed",
    "chat:message",
    "chat:typing",
    "stats:online",
    "party:state",
    "party:member_joined",
    "party:member_left",
    "party:ready_update",
    "party:skin_update",
    "party:launched",
    "party:disbanded",
    "party:invite_received",
    "party:emote",
    "party:error",
    "voice:speaking",
    "voice:offer",
    "voice:answer",
    "voice:ice",
];

#[allow(deprecated)]
fn payload_to_json(payload: Payload) -> serde_json::Value {
    match payload {
        Payload::Text(values) => values.into_iter().next().unwrap_or(serde_json::Value::Null),
        // `Payload::String` is deprecated in favor of `Text`, but the match must stay exhaustive.
        Payload::String(s) => serde_json::from_str(&s).unwrap_or(serde_json::Value::String(s)),
        Payload::Binary(_) => serde_json::Value::Null,
    }
}

pub async fn connect_beja_socket(app: AppHandle, token: String) {
    disconnect_beja_socket().await;

    let mut builder = ClientBuilder::new(BEJA_API).auth(json!({ "token": token })).reconnect_on_disconnect(true);

    {
        let app = app.clone();
        builder = builder.on("connect", move |_, _| {
            let app = app.clone();
            async move {
                log::info!("[Socket] connected");
                let _ = app.emit("socket:status", "connected");
            }
            .boxed()
        });
    }
    {
        let app = app.clone();
        builder = builder.on("disconnect", move |payload, _| {
            let app = app.clone();
            async move {
                log::info!("[Socket] disconnected: {:?}", payload_to_json(payload));
                let _ = app.emit("socket:status", "disconnected");
            }
            .boxed()
        });
    }
    {
        let app = app.clone();
        builder = builder.on("error", move |payload, _| {
            let app = app.clone();
            async move {
                log::error!("[Socket] connect_error: {:?}", payload_to_json(payload));
                let _ = app.emit("socket:status", "error");
            }
            .boxed()
        });
    }

    for event in RELAYED_EVENTS {
        let app = app.clone();
        let event_name = event.to_string();
        builder = builder.on(*event, move |payload, _| {
            let app = app.clone();
            let event_name = event_name.clone();
            async move {
                let _ = app.emit(&event_name, payload_to_json(payload));
            }
            .boxed()
        });
    }

    match builder.connect().await {
        Ok(client) => {
            *SOCKET.lock() = Some(client);
        }
        Err(e) => {
            log::error!("[Socket] connect failed: {e}");
            let _ = app.emit("socket:status", "error");
        }
    }
}

pub async fn disconnect_beja_socket() {
    let client = SOCKET.lock().take();
    if let Some(client) = client {
        let _ = client.disconnect().await;
    }
}

pub fn emit_lobby_event(event: &str, data: serde_json::Value) {
    let client = SOCKET.lock().clone();
    if let Some(client) = client {
        let event = event.to_string();
        tauri::async_runtime::spawn(async move {
            let _ = client.emit(event, data).await;
        });
    }
}
