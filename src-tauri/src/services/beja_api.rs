//! Shared HTTP helper for the many small REST-proxy IPC modules (`cosmetics.ts`, `capes.ts`,
//! `pass.ts`, `quests.ts`, `wallet.ts`, `profileShare.ts`, part of `friends.ts`) that all did the
//! same thing: a plain JSON request to `206.217.141.184:3093` with an optional bearer token.

use serde_json::Value;
use std::time::Duration;

use super::auth_service::{http_client, BEJA_API};

pub async fn request(method: reqwest::Method, path: &str, token: Option<&str>, body: Option<Value>) -> Value {
    let mut req = http_client().request(method, format!("{BEJA_API}{path}")).timeout(Duration::from_secs(8));
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    if let Some(b) = body {
        req = req.json(&b);
    } else {
        req = req.header("Content-Type", "application/json");
    }
    match req.send().await {
        Ok(res) => res.json::<Value>().await.unwrap_or(Value::Null),
        Err(e) => serde_json::json!({ "error": e.to_string() }),
    }
}

pub async fn get(path: &str, token: Option<&str>) -> Value {
    request(reqwest::Method::GET, path, token, None).await
}
pub async fn post(path: &str, token: Option<&str>, body: Value) -> Value {
    request(reqwest::Method::POST, path, token, Some(body)).await
}
pub async fn put(path: &str, token: Option<&str>, body: Value) -> Value {
    request(reqwest::Method::PUT, path, token, Some(body)).await
}
pub async fn delete(path: &str, token: Option<&str>) -> Value {
    request(reqwest::Method::DELETE, path, token, None).await
}
