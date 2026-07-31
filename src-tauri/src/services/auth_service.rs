//! Port of `src/main/services/authService.ts`.
//!
//! The interactive MSA login used to be an Electron `BrowserWindow` whose `will-navigate` /
//! `will-redirect` events were intercepted to grab the `code=` query param off the
//! `oauth20_desktop.srf` redirect. Tauri's `WebviewWindowBuilder::on_navigation` callback is the
//! direct equivalent — it fires synchronously for every navigation and can veto it (return
//! `false`) before the webview actually loads the URL, so we can grab the code and close the
//! window ourselves instead of letting it render Microsoft's blank redirect page.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::oneshot;

use crate::paths;

const AZURE_CLIENT_ID: &str = "00000000402b5328";
const AZURE_SCOPE: &str = "XboxLive.signin offline_access";
const AZURE_AUTH_ENDPOINT: &str = "https://login.live.com/oauth20_authorize.srf";
const AZURE_TOKEN_ENDPOINT: &str = "https://login.live.com/oauth20_token.srf";
const LIVE_REDIRECT_URI: &str = "https://login.live.com/oauth20_desktop.srf";

pub const BEJA_API: &str = "http://206.217.141.184:3093";

static HTTP: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .expect("failed to build reqwest client")
});

pub fn http_client() -> &'static reqwest::Client {
    &HTTP
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoredAccount {
    pub id: String,
    pub username: String,
    pub uuid: String,
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    #[serde(rename = "tokenExpiry")]
    pub token_expiry: i64,
    #[serde(rename = "skinUrl")]
    pub skin_url: Option<String>,
    #[serde(rename = "capeUrl")]
    pub cape_url: Option<String>,
    #[serde(rename = "skinModel")]
    pub skin_model: String,
    pub selected: bool,
    #[serde(rename = "bejaToken", default, skip_serializing_if = "Option::is_none")]
    pub beja_token: Option<String>,
    #[serde(rename = "countryCode", default, skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
}

pub fn load_accounts() -> Vec<StoredAccount> {
    let path = paths::accounts_path();
    if !path.exists() {
        return Vec::new();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save_accounts(accounts: &[StoredAccount]) {
    if let Ok(json) = serde_json::to_string_pretty(accounts) {
        let _ = fs::write(paths::accounts_path(), json);
    }
}

pub fn get_selected_account() -> Option<StoredAccount> {
    load_accounts().into_iter().find(|a| a.selected)
}

pub fn select_account(id: &str) -> Vec<StoredAccount> {
    let accounts: Vec<StoredAccount> = load_accounts()
        .into_iter()
        .map(|mut a| {
            a.selected = a.id == id;
            a
        })
        .collect();
    save_accounts(&accounts);
    accounts
}

pub fn logout_account(id: &str) {
    let accounts: Vec<StoredAccount> = load_accounts().into_iter().filter(|a| a.id != id).collect();
    save_accounts(&accounts);
}

// ── Live (MSA) token exchange ──────────────────────────────────────────────────

struct LiveTokens {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

async fn exchange_live_code(code: &str) -> Result<LiveTokens, String> {
    let params = [
        ("client_id", AZURE_CLIENT_ID),
        ("code", code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", LIVE_REDIRECT_URI),
    ];
    let res = HTTP
        .post(AZURE_TOKEN_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let data: Value = res.json().await.map_err(|e| e.to_string())?;
    let access_token = data["access_token"].as_str().ok_or_else(|| {
        data["error_description"].as_str().unwrap_or("Failed to get token").to_string()
    })?;
    Ok(LiveTokens {
        access_token: access_token.to_string(),
        refresh_token: data["refresh_token"].as_str().unwrap_or_default().to_string(),
        expires_in: data["expires_in"].as_i64().unwrap_or(3600),
    })
}

async fn refresh_live_token(refresh_token: &str) -> Result<LiveTokens, String> {
    let params = [
        ("client_id", AZURE_CLIENT_ID),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
        ("redirect_uri", LIVE_REDIRECT_URI),
    ];
    let res = HTTP
        .post(AZURE_TOKEN_ENDPOINT)
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let data: Value = res.json().await.map_err(|e| e.to_string())?;
    let access_token = data["access_token"].as_str().ok_or_else(|| {
        data["error_description"].as_str().unwrap_or("Failed to refresh token").to_string()
    })?;
    Ok(LiveTokens {
        access_token: access_token.to_string(),
        refresh_token: data["refresh_token"].as_str().unwrap_or(refresh_token).to_string(),
        expires_in: data["expires_in"].as_i64().unwrap_or(3600),
    })
}

struct XblToken {
    token: String,
    user_hash: String,
}

async fn auth_xbox_live(live_token: &str) -> Result<XblToken, String> {
    let body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={live_token}"),
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT",
    });
    let res = HTTP
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let data: Value = res.json().await.map_err(|e| e.to_string())?;
    let token = data["Token"].as_str().ok_or("Xbox Live authentication failed")?;
    let user_hash = data["DisplayClaims"]["xui"][0]["uhs"].as_str().ok_or("Xbox Live authentication failed")?;
    Ok(XblToken { token: token.to_string(), user_hash: user_hash.to_string() })
}

async fn get_xsts_token(xbl_token: &str) -> Result<XblToken, String> {
    let body = serde_json::json!({
        "Properties": { "SandboxId": "RETAIL", "UserTokens": [xbl_token] },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT",
    });
    let res = HTTP
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let data: Value = res.json().await.map_err(|e| e.to_string())?;
    if let Some(token) = data["Token"].as_str() {
        let user_hash = data["DisplayClaims"]["xui"][0]["uhs"].as_str().unwrap_or_default();
        return Ok(XblToken { token: token.to_string(), user_hash: user_hash.to_string() });
    }
    let x_err = data["XErr"].as_i64().unwrap_or(0);
    Err(match x_err {
        2148916233 => "This Microsoft account has no Xbox profile. Set one up at xbox.com.".to_string(),
        2148916235 => "Xbox Live is not available in your country.".to_string(),
        _ => format!("XSTS error: {x_err}"),
    })
}

async fn get_minecraft_token(xsts_token: &str, user_hash: &str) -> Result<String, String> {
    let body = serde_json::json!({ "identityToken": format!("XBL3.0 x={user_hash};{xsts_token}") });
    let res = HTTP
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let data: Value = res.json().await.map_err(|e| e.to_string())?;
    data["access_token"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Failed to get Minecraft token: {data}"))
}

#[derive(Debug, Deserialize)]
struct McSkin {
    url: String,
    state: String,
    variant: Option<String>,
}
#[derive(Debug, Deserialize)]
struct McCape {
    url: String,
    state: String,
}
#[derive(Debug, Deserialize)]
struct McProfile {
    id: String,
    name: String,
    #[serde(default)]
    skins: Vec<McSkin>,
    #[serde(default)]
    capes: Vec<McCape>,
}

async fn get_minecraft_profile(mc_token: &str) -> Result<McProfile, String> {
    let res = HTTP
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(mc_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let data: Value = res.json().await.map_err(|e| e.to_string())?;
    if data.get("error").is_some() {
        return Err("This account does not own Minecraft Java Edition.".to_string());
    }
    serde_json::from_value(data).map_err(|e| e.to_string())
}

async fn fetch_beja_token(uuid: &str, username: &str) -> Option<String> {
    let res = HTTP
        .post(format!("{BEJA_API}/api/auth/login"))
        .json(&serde_json::json!({ "uuid": uuid, "username": username }))
        .send()
        .await
        .ok()?;
    let data: Value = res.json().await.ok()?;
    data["token"].as_str().map(|s| s.to_string())
}

async fn finalize_login(live_access_token: &str, live_refresh_token: &str, live_expires_in: i64) -> Result<StoredAccount, String> {
    let xbl = auth_xbox_live(live_access_token).await?;
    let xsts = get_xsts_token(&xbl.token).await?;
    let mc_token = get_minecraft_token(&xsts.token, &xsts.user_hash).await?;
    let profile = get_minecraft_profile(&mc_token).await?;

    let active_skin = profile.skins.iter().find(|s| s.state == "ACTIVE");
    let active_cape = profile.capes.iter().find(|c| c.state == "ACTIVE");
    let beja_token = fetch_beja_token(&profile.id, &profile.name).await;

    let now_ms = chrono::Utc::now().timestamp_millis();
    let account = StoredAccount {
        id: profile.id.clone(),
        username: profile.name,
        uuid: profile.id,
        access_token: mc_token,
        refresh_token: live_refresh_token.to_string(),
        token_expiry: now_ms + live_expires_in * 1000,
        skin_url: active_skin.map(|s| s.url.replace("http://", "https://")),
        cape_url: active_cape.map(|c| c.url.replace("http://", "https://")),
        skin_model: match active_skin.and_then(|s| s.variant.as_deref()) {
            Some("SLIM") => "slim".to_string(),
            _ => "default".to_string(),
        },
        selected: false,
        beja_token,
        country_code: None,
    };

    let mut accounts = load_accounts();
    if let Some(existing) = accounts.iter_mut().find(|a| a.id == account.id) {
        let selected = existing.selected;
        *existing = account.clone();
        existing.selected = selected;
    } else {
        accounts.push(account.clone());
    }
    save_accounts(&accounts);
    Ok(account)
}

/// Opens the Microsoft sign-in webview, intercepts the `oauth20_desktop.srf` redirect to pull
/// the auth `code` out of the URL, then runs the XBL → XSTS → MC token chain.
pub async fn login_with_microsoft(app: AppHandle) -> Result<StoredAccount, String> {
    let auth_url = format!(
        "{AZURE_AUTH_ENDPOINT}?client_id={AZURE_CLIENT_ID}&response_type=code&scope={}&redirect_uri={}&prompt=select_account",
        urlencoding::encode(AZURE_SCOPE),
        urlencoding::encode(LIVE_REDIRECT_URI),
    );

    let (tx, rx) = oneshot::channel::<Result<String, String>>();
    let tx: Arc<Mutex<Option<oneshot::Sender<Result<String, String>>>>> = Arc::new(Mutex::new(Some(tx)));
    let tx_nav = tx.clone();

    let label = "msa-login";
    // Close any stale login window left over from a previous cancelled attempt.
    if let Some(existing) = app.get_webview_window(label) {
        let _ = existing.close();
    }

    let _ = app.emit("auth:browser-opened", ());

    let window = WebviewWindowBuilder::new(&app, label, WebviewUrl::External(auth_url.parse().map_err(|e: url::ParseError| e.to_string())?))
        .title("Sign in with Microsoft")
        .inner_size(480.0, 640.0)
        .resizable(true)
        .on_navigation(move |url| {
            if url.as_str().contains("oauth20_desktop.srf") {
                let mut code = None;
                let mut error = None;
                // Check both the query string (the normal case) and the fragment — some
                // Microsoft account flows (e.g. a silent SSO probe before the interactive
                // form even renders) bounce through this redirect URI with neither, or with
                // params after a `#` instead of a `?`. Only ever treat this as final once we
                // actually have a code or an error; an empty bounce just keeps waiting instead
                // of being misreported as "authorisation denied".
                for (k, v) in url.query_pairs() {
                    if k == "code" {
                        code = Some(v.to_string());
                    } else if k == "error" {
                        error = Some(v.to_string());
                    }
                }
                if code.is_none() && error.is_none() {
                    if let Some(fragment) = url.fragment() {
                        for pair in fragment.split('&') {
                            if let Some(v) = pair.strip_prefix("code=") {
                                code = Some(v.to_string());
                            } else if let Some(v) = pair.strip_prefix("error=") {
                                error = Some(v.to_string());
                            }
                        }
                    }
                }

                if code.is_none() && error.is_none() {
                    // No params at all yet — an intermediate bounce, not the final redirect.
                    // Let it load and keep waiting for the real one.
                    return true;
                }

                if let Some(sender) = tx_nav.lock().unwrap().take() {
                    let result = match (code, error) {
                        (Some(c), _) => Ok(c),
                        (None, Some(e)) => Err(e),
                        _ => Err("No auth code received".to_string()),
                    };
                    let _ = sender.send(result);
                }
                return false; // veto navigation — we already have what we need
            }
            true
        })
        .build()
        .map_err(|e| e.to_string())?;

    // The user closing the window manually (no code captured) must reject the promise
    // instead of leaving the frontend's "signing in…" state hanging forever.
    let tx_close = tx.clone();
    window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Destroyed) {
            if let Some(sender) = tx_close.lock().unwrap().take() {
                let _ = sender.send(Err("Sign-in cancelled".to_string()));
            }
        }
    });

    let code_result = match tokio::time::timeout(Duration::from_secs(5 * 60), rx).await {
        Ok(Ok(r)) => r,
        Ok(Err(_)) => Err("Sign-in cancelled".to_string()),
        Err(_) => Err("Sign-in timed out".to_string()),
    };

    if let Some(w) = app.get_webview_window(label) {
        let _ = w.close();
    }

    let code = code_result?;
    let tokens = exchange_live_code(&code).await?;
    finalize_login(&tokens.access_token, &tokens.refresh_token, tokens.expires_in).await
}

pub async fn refresh_account(id: &str) -> Result<Option<StoredAccount>, String> {
    let accounts = load_accounts();
    let Some(account) = accounts.iter().find(|a| a.id == id) else { return Ok(None) };
    if account.refresh_token.is_empty() {
        return Ok(None);
    }
    let tokens = refresh_live_token(&account.refresh_token).await?;
    let mut updated = finalize_login(&tokens.access_token, &tokens.refresh_token, tokens.expires_in).await?;
    updated.id = account.id.clone();
    Ok(Some(updated))
}

// ── Automatic token refresh ────────────────────────────────────────────────────
// Live tokens are only valid for ~1h, so a launcher left open in the background would
// otherwise silently end up with a stale access token until the user hit "Refresh".
const AUTO_REFRESH_INTERVAL_SECS: u64 = 5 * 60;
const AUTO_REFRESH_MARGIN_MS: i64 = 10 * 60 * 1000;

pub fn start_token_auto_refresh(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let now = chrono::Utc::now().timestamp_millis();
            let due: Vec<StoredAccount> = load_accounts()
                .into_iter()
                .filter(|a| !a.refresh_token.is_empty() && a.token_expiry < now + AUTO_REFRESH_MARGIN_MS)
                .collect();
            let mut changed = false;
            for account in due {
                match refresh_account(&account.id).await {
                    Ok(Some(_)) => changed = true,
                    Ok(None) => {}
                    Err(e) => log::error!("[Auth] Auto-refresh failed for {}: {e}", account.username),
                }
            }
            if changed {
                let _ = app.emit("auth:accounts-updated", load_accounts());
            }
            tokio::time::sleep(Duration::from_secs(AUTO_REFRESH_INTERVAL_SECS)).await;
        }
    });
}

pub async fn import_from_official_launcher() -> Result<Vec<StoredAccount>, String> {
    let launcher_path = dirs::config_dir()
        .ok_or("no config dir")?
        .join(".minecraft")
        .join("launcher_accounts.json");
    if !launcher_path.exists() {
        return Err("Official Minecraft Launcher not found or not logged in".to_string());
    }
    let raw: Value = serde_json::from_str(&fs::read_to_string(&launcher_path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    let mut imported = Vec::new();
    let empty = serde_json::Map::new();
    let entries = raw["accounts"].as_object().unwrap_or(&empty);

    for acc in entries.values() {
        let Some(profile_id) = acc["minecraftProfile"]["id"].as_str() else { continue };
        let profile_name = acc["minecraftProfile"]["name"].as_str().unwrap_or_default().to_string();
        let access_token = acc["accessToken"].as_str().unwrap_or_default().to_string();
        let token_expiry = acc["accessTokenExpiresAt"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.timestamp_millis())
            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis() + 86_400_000);

        let (skin_url, cape_url, skin_model) = match get_minecraft_profile(&access_token).await {
            Ok(mc_profile) => {
                let skin = mc_profile.skins.iter().find(|s| s.state == "ACTIVE");
                let cape = mc_profile.capes.iter().find(|c| c.state == "ACTIVE");
                (
                    skin.map(|s| s.url.replace("http://", "https://")),
                    cape.map(|c| c.url.replace("http://", "https://")),
                    if skin.and_then(|s| s.variant.as_deref()) == Some("SLIM") { "slim".to_string() } else { "default".to_string() },
                )
            }
            Err(_) => (None, None, "default".to_string()), // expired token — proceed without skin
        };

        let beja_token = fetch_beja_token(profile_id, &profile_name).await;

        let account = StoredAccount {
            id: profile_id.to_string(),
            username: profile_name,
            uuid: profile_id.to_string(),
            access_token,
            refresh_token: String::new(),
            token_expiry,
            skin_url,
            cape_url,
            skin_model,
            selected: false,
            beja_token,
            country_code: None,
        };

        let mut accounts = load_accounts();
        if let Some(existing) = accounts.iter_mut().find(|a| a.id == account.id) {
            let selected = existing.selected;
            *existing = account.clone();
            existing.selected = selected;
        } else {
            accounts.push(account.clone());
        }
        save_accounts(&accounts);
        imported.push(account);
    }

    if imported.is_empty() {
        return Err("No accounts found in official launcher".to_string());
    }
    Ok(imported)
}
