//! OptiFine has no official download API — versions are scraped off optifine.net's downloads
//! page. Confirmed by hand against the live site: the "(Mirror)" link on each row goes straight
//! to `adloadx?f=<filename>` (no ad-network redirect, no captcha), whose HTML embeds the real
//! `downloadx?f=<filename>&x=<token>` link inline — no session cookie required, the token alone
//! is enough. This is the same two-hop approach used by other third-party OptiFine downloaders.
//!
//! Installing is deliberately NOT reimplemented here. Two attempts at doing it ourselves both
//! failed against a real 1.21.11 jar: (1) treating the jar as a launchwrapper tweaker mod — that
//! only works on Java 8, which can't run a modern client at all; (2) calling `optifine.Patcher`
//! (its own internal patch-apply class) directly — it ran without error but produced an
//! intermediate build artifact (still notch/srg-namespaced), not a runnable client jar. The real
//! entry point, `optifine.Installer`, branches between several install strategies per MC
//! version/edition (plain patch, library+launchwrapper, and what looks like Forge-coupled
//! handling — its jar embeds a full `net.minecraftforge.*` tree) that would take real
//! reverse-engineering to reproduce correctly. So instead: download the real installer jar and
//! launch it (`java -jar OptiFine.jar`) — the same GUI a normal user runs by hand — pointed at the
//! profile's game dir. The user clicks "Install" once; OptiFine's own code does the rest
//! correctly. `resolve_version_id` (launch_service.rs) already finds whatever version folder that
//! produces by name-matching (contains the mc_version + "optifine"), so no extra wiring is needed
//! for the resulting profile to launch normally afterward.
//!
//! Every profile created from the pinned OptiFine shortcut runs `use_beja_client: false` (see
//! ProfilesSettings.vue / OptifineWizard.vue) — OptiFine's own renderer patches conflict with the
//! bundled Sodium (already blacklisted in mod_compatibility_checker.rs for exactly this reason),
//! so these profiles get a plain vanilla+OptiFine install with none of BejaClient's bundled
//! performance mods.

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use regex::Regex;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const DOWNLOADS_PAGE: &str = "https://optifine.net/downloads";
const INDEX_TTL: Duration = Duration::from_secs(600);

#[derive(Debug, Clone, Serialize)]
pub struct OptifineEntry {
    #[serde(rename = "mcVersion")]
    pub mc_version: String,
    /// e.g. "HD_U_J9" — everything after the mc_version segment of the filename.
    #[serde(rename = "buildTag")]
    pub build_tag: String,
    pub filename: String,
    pub preview: bool,
}

static INDEX_CACHE: Lazy<Mutex<Option<(Instant, Vec<OptifineEntry>)>>> = Lazy::new(|| Mutex::new(None));

async fn https_get_text(client: &reqwest::Client, url: &str) -> Result<String, String> {
    client
        .get(url)
        .header("User-Agent", "Mozilla/5.0")
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}

async fn https_get_bytes(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    client
        .get(url)
        .header("User-Agent", "Mozilla/5.0")
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| e.to_string())
}

/// Parses the downloads page: `<h2>Minecraft X.Y.Z</h2>` headers, each followed by a table of
/// `colMirror` rows linking to `adloadx?f=<filename>.jar`. Sectioning is done by h2 byte-offset
/// rather than a stateful line walk, since a version's build table can span many lines.
fn parse_downloads_page(html: &str) -> Vec<OptifineEntry> {
    let h2_re = Regex::new(r"<h2>Minecraft ([^<]+)</h2>").unwrap();
    let mirror_re = Regex::new(r#"colMirror'>\s*<a href="[^"]*[?&]f=([^"&]+\.jar)""#).unwrap();

    let headers: Vec<(usize, String)> = h2_re
        .captures_iter(html)
        .map(|c| (c.get(0).unwrap().start(), c[1].trim().to_string()))
        .collect();

    let mut entries = Vec::new();
    for (i, (start, mc_version)) in headers.iter().enumerate() {
        let end = headers.get(i + 1).map(|(s, _)| *s).unwrap_or(html.len());
        let section = &html[*start..end];
        for cap in mirror_re.captures_iter(section) {
            let filename = cap[1].to_string();
            let preview = filename.starts_with("preview_");
            let stem = filename.trim_start_matches("preview_").trim_end_matches(".jar");
            // stem looks like "OptiFine_1.21.11_HD_U_J9" — strip the "OptiFine_" prefix, then
            // the mc_version segment, to isolate the build tag ("HD_U_J9").
            let after_prefix = stem.strip_prefix("OptiFine_").unwrap_or(stem);
            let build_tag = after_prefix.strip_prefix(&format!("{mc_version}_")).unwrap_or(after_prefix).to_string();
            entries.push(OptifineEntry { mc_version: mc_version.clone(), build_tag, filename, preview });
        }
    }
    entries
}

async fn get_index(client: &reqwest::Client) -> Result<Vec<OptifineEntry>, String> {
    {
        let cache = INDEX_CACHE.lock();
        if let Some((fetched_at, entries)) = cache.as_ref() {
            if fetched_at.elapsed() < INDEX_TTL {
                return Ok(entries.clone());
            }
        }
    }
    let html = https_get_text(client, DOWNLOADS_PAGE).await?;
    let entries = parse_downloads_page(&html);
    if entries.is_empty() {
        return Err("Could not find any OptiFine builds - optifine.net may have changed its page layout.".to_string());
    }
    *INDEX_CACHE.lock() = Some((Instant::now(), entries.clone()));
    Ok(entries)
}

/// Distinct MC versions that have at least one OptiFine build, newest first (matches the site's
/// own ordering — it already lists newest-first, so this just dedupes rather than re-sorting).
pub async fn list_mc_versions(client: &reqwest::Client) -> Result<Vec<String>, String> {
    let entries = get_index(client).await?;
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for e in entries {
        if seen.insert(e.mc_version.clone()) {
            out.push(e.mc_version);
        }
    }
    Ok(out)
}

/// Builds for one MC version, stable first (then previews) — some versions (e.g. right after a
/// new MC release) only ever get preview builds on optifine.net, so previews can't just be
/// dropped or those versions would show as having no OptiFine build at all.
pub async fn list_builds(client: &reqwest::Client, mc_version: &str) -> Result<Vec<OptifineEntry>, String> {
    let entries = get_index(client).await?;
    let mut matching: Vec<OptifineEntry> = entries.into_iter().filter(|e| e.mc_version == mc_version).collect();
    matching.sort_by_key(|e| e.preview);
    Ok(matching)
}

/// `adloadx?f=<filename>` embeds the real, already-tokenized `downloadx?f=...&x=...` link in its
/// HTML — no session state needed, confirmed against the live site.
async fn fetch_jar_bytes(client: &reqwest::Client, filename: &str) -> Result<Vec<u8>, String> {
    let interstitial = https_get_text(client, &format!("https://optifine.net/adloadx?f={filename}")).await?;
    let link_re = Regex::new(r"downloadx\?f=[^'\x22]+&x=[0-9a-f]+").unwrap();
    let relative = link_re
        .find(&interstitial)
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| format!("Could not find a download link for {filename} on optifine.net - the page layout may have changed."))?;
    https_get_bytes(client, &format!("https://optifine.net/{relative}")).await
}

/// Downloads the real installer jar (cached by filename — re-downloading would burn another
/// single-use optifine.net token for no reason) and launches it detached, exactly as if the user
/// had double-clicked it. `game_dir` isn't passed as a CLI arg — OptiFine's own installer GUI
/// auto-detects `.minecraft` and has its own "Edit" field to point elsewhere, so the caller shows
/// `game_dir` to the user to paste in there if it differs from the default.
pub async fn launch_gui_installer(client: &reqwest::Client, filename: &str, java_path: &str, cache_dir: &Path) -> Result<(), String> {
    let dest = cache_dir.join(filename);
    if !dest.exists() {
        let bytes = fetch_jar_bytes(client, filename).await?;
        if let Some(dir) = dest.parent() {
            tokio::fs::create_dir_all(dir).await.map_err(|e| e.to_string())?;
        }
        tokio::fs::write(&dest, &bytes).await.map_err(|e| e.to_string())?;
    }

    tokio::process::Command::new(java_path)
        .arg("-jar")
        .arg(&dest)
        .current_dir(cache_dir)
        .spawn()
        .map_err(|e| format!("Failed to launch the OptiFine installer: {e}"))?;
    Ok(())
}

pub fn installer_cache_dir() -> PathBuf {
    crate::paths::user_data_dir().join("optifine-installers")
}
