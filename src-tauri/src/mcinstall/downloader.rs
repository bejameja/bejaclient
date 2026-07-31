//! Port of `src/main/services/mcinstall/downloader.ts`.

use futures_util::{stream, StreamExt};
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;

async fn sha1_file(path: &Path) -> Result<String, std::io::Error> {
    let bytes = fs::read(path).await?;
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

#[derive(Default, Clone)]
pub struct DownloadOptions {
    pub sha1: Option<String>,
}

/// Downloads to a `.part` sibling of `dest`, verifying checksum (when provided) before
/// atomically renaming into place. If `dest` already exists and matches `sha1`, returns
/// immediately with no network call — this is what makes installs resumable/cheap to retry.
pub async fn download_file(client: &reqwest::Client, url: &str, dest: &Path, options: DownloadOptions) -> Result<(), String> {
    if let Some(expected) = &options.sha1 {
        if fs::metadata(dest).await.is_ok() {
            if let Ok(actual) = sha1_file(dest).await {
                if actual.eq_ignore_ascii_case(expected) {
                    return Ok(());
                }
            }
        }
    }

    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir).await.map_err(|e| e.to_string())?;
    }

    let part_path = part_path(dest);
    download_to_temp(client, url, &part_path, 5).await?;

    if let Some(expected) = &options.sha1 {
        let actual = sha1_file(&part_path).await.map_err(|e| e.to_string())?;
        if !actual.eq_ignore_ascii_case(expected) {
            let _ = fs::remove_file(&part_path).await;
            return Err(format!("Checksum mismatch for {url}: expected {expected}, got {actual}"));
        }
    }

    fs::rename(&part_path, dest).await.map_err(|e| e.to_string())?;
    Ok(())
}

fn part_path(dest: &Path) -> PathBuf {
    let mut s = dest.as_os_str().to_os_string();
    s.push(".part");
    PathBuf::from(s)
}

async fn download_to_temp(client: &reqwest::Client, url: &str, temp_dest: &Path, redirects: u8) -> Result<(), String> {
    let fail = |e: String| async move {
        let _ = fs::remove_file(temp_dest).await;
        Err::<(), String>(e)
    };

    let res = match client.get(url).timeout(Duration::from_secs(60)).send().await {
        Ok(r) => r,
        Err(e) => return fail(e.to_string()).await,
    };

    if res.status().is_redirection() {
        if redirects == 0 {
            return fail("Too many redirects".to_string()).await;
        }
        if let Some(loc) = res.headers().get("location").and_then(|v| v.to_str().ok()) {
            let loc = loc.to_string();
            return Box::pin(download_to_temp(client, &loc, temp_dest, redirects - 1)).await;
        }
    }
    if !res.status().is_success() {
        return fail(format!("HTTP {} for {url}", res.status())).await;
    }

    let mut file = match fs::File::create(temp_dest).await {
        Ok(f) => f,
        Err(e) => return fail(e.to_string()).await,
    };
    let mut stream = res.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => return fail(e.to_string()).await,
        };
        if let Err(e) = file.write_all(&chunk).await {
            return fail(e.to_string()).await;
        }
    }
    let _ = file.flush().await;
    Ok(())
}

pub struct PoolFailure<T> {
    pub item: T,
    pub error: String,
}

/// Runs `worker` over `items` with at most `concurrency` in flight at once. Per-item failures
/// are collected rather than aborting the whole batch.
pub async fn run_pool<T, F, Fut>(items: Vec<T>, concurrency: usize, worker: F) -> Vec<PoolFailure<T>>
where
    T: Clone + Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), String>> + Send,
{
    let limit = concurrency.clamp(1, 64);
    let worker = std::sync::Arc::new(worker);
    let results: Vec<(T, Result<(), String>)> = stream::iter(items.into_iter())
        .map(|item| {
            let worker = worker.clone();
            let item_clone = item.clone();
            async move {
                let r = worker(item_clone).await;
                (item, r)
            }
        })
        .buffer_unordered(limit)
        .collect()
        .await;

    results
        .into_iter()
        .filter_map(|(item, r)| r.err().map(|error| PoolFailure { item, error }))
        .collect()
}

pub fn summarize_failures<T>(failures: &[PoolFailure<T>], label: &str) -> Option<String> {
    if failures.is_empty() {
        return None;
    }
    let sample: Vec<&str> = failures.iter().take(3).map(|f| f.error.as_str()).collect();
    Some(format!(
        "{} {label} failed: {}{}",
        failures.len(),
        sample.join("; "),
        if failures.len() > 3 { "…" } else { "" }
    ))
}

mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
    }
}
