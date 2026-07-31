//! Port of `src/main/services/bejaAuth.ts`.
//!
//! Writes the `session.json` bridge file `SessionBridge.java` (BejaClient-MC) reads directly, so
//! in-game features (emote ownership, friends) can trust the launcher's login without a separate
//! in-game auth flow.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde_json::Value;
use std::fs;

use super::auth_service::{self, StoredAccount};

/// Mojang profile UUIDs (what `StoredAccount.uuid` holds) come back undashed, but Java's
/// `UUID.toString()` on the mod side is always dashed (8-4-4-4-12) — `SessionBridge.java`
/// compares against that dashed form, so the bridge file must dash it too.
fn to_dashed_uuid(uuid: &str) -> String {
    let hex: String = uuid.chars().filter(|c| *c != '-').collect();
    if hex.len() != 32 {
        return uuid.to_string();
    }
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

fn decode_jwt_expiry_ms(jwt: &str) -> i64 {
    let default_expiry = chrono::Utc::now().timestamp_millis() + 24 * 60 * 60 * 1000;
    let Some(payload_b64) = jwt.split('.').nth(1) else { return default_expiry };
    let Ok(bytes) = URL_SAFE_NO_PAD.decode(payload_b64) else { return default_expiry };
    let Ok(payload) = serde_json::from_slice::<Value>(&bytes) else { return default_expiry };
    payload["exp"].as_i64().map(|exp| exp * 1000).unwrap_or(default_expiry)
}

fn write_session_file(uuid: &str, jwt: &str) {
    let expires_at = decode_jwt_expiry_ms(jwt);
    let path = crate::paths::session_path();
    let body = serde_json::json!({
        "jwt": jwt,
        "expiresAt": expires_at,
        "accountUuid": to_dashed_uuid(uuid),
    });
    if let Ok(json) = serde_json::to_string_pretty(&body) {
        if let Err(e) = fs::write(&path, json) {
            log::error!("[BejaAuth] Failed to write session bridge file: {e}");
        }
    }
}

/// Returns the cached Beja JWT for the selected account, fetching + persisting one if the
/// account predates `bejaToken` (or never got one due to a transient login failure).
pub async fn get_beja_token() -> Option<String> {
    let account = auth_service::get_selected_account()?;
    if let Some(token) = &account.beja_token {
        write_session_file(&account.uuid, token);
        return Some(token.clone());
    }

    let token = fetch_beja_token(&account.uuid, &account.username).await?;
    let mut accounts = auth_service::load_accounts();
    if let Some(a) = accounts.iter_mut().find(|a| a.id == account.id) {
        a.beja_token = Some(token.clone());
    }
    auth_service::save_accounts(&accounts);
    write_session_file(&account.uuid, &token);
    Some(token)
}

async fn fetch_beja_token(uuid: &str, username: &str) -> Option<String> {
    let res = auth_service::http_client()
        .post(format!("{}/api/auth/login", auth_service::BEJA_API))
        .json(&serde_json::json!({ "uuid": uuid, "username": username }))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .ok()?;
    let data: Value = res.json().await.ok()?;
    data["token"].as_str().map(|s| s.to_string())
}

#[allow(dead_code)]
pub fn selected_account() -> Option<StoredAccount> {
    auth_service::get_selected_account()
}
