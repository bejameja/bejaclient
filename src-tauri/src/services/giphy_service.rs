//! GIF search for the friends chat's GIF picker. The request happens here in
//! Rust rather than the frontend so the API key never ships inside the
//! webview's JS bundle (the compiled Rust binary isn't readable the way the
//! shipped JS bundle is).

use serde::Serialize;
use serde_json::Value;

const GIPHY_API: &str = "https://api.giphy.com/v1/gifs";
const GIPHY_API_KEY: &str = "nFUTyqrVEbC9LKgmFFnFyQoJYXmULIJC";
const UA: &str = "BejaClient/2.0 (bejaclient.xyz) Tauri-edition";

#[derive(Debug, Clone, Serialize)]
pub struct GifResult {
    id: String,
    thumb: String,
    url: String,
    title: String,
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().user_agent(UA).build().unwrap()
}

fn map_result(g: &Value) -> Option<GifResult> {
    let id = g["id"].as_str()?.to_string();
    let images = &g["images"];
    let thumb = images["fixed_width_small"]["url"]
        .as_str()
        .or_else(|| images["fixed_width"]["url"].as_str())?
        .to_string();
    let url = images["fixed_height"]["url"]
        .as_str()
        .or_else(|| images["original"]["url"].as_str())?
        .to_string();
    let title = g["title"].as_str().filter(|s| !s.is_empty()).unwrap_or("GIF").to_string();
    Some(GifResult { id, thumb, url, title })
}

pub async fn search(query: Option<String>) -> Result<Vec<GifResult>, String> {
    let url = match query.as_deref().map(str::trim).filter(|q| !q.is_empty()) {
        Some(q) => format!(
            "{GIPHY_API}/search?api_key={GIPHY_API_KEY}&q={}&limit=15&rating=pg-13",
            urlencoding::encode(q)
        ),
        None => format!("{GIPHY_API}/trending?api_key={GIPHY_API_KEY}&limit=15&rating=pg-13"),
    };

    let res = client().get(&url).send().await.map_err(|e| e.to_string())?;
    let status = res.status();
    let text = res.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("GIPHY API error {status}: {}", text.chars().take(200).collect::<String>()));
    }
    let json: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let data = json["data"].as_array().cloned().unwrap_or_default();
    Ok(data.iter().filter_map(map_result).collect())
}
