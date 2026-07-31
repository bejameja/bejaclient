//! Port of `src/main/services/mcinstall/libraries.ts`.

use super::downloader::{download_file, run_pool, summarize_failures, DownloadOptions};
use super::rules::check_rules;
use super::types::{Platform, RawLibrary, ResolvedLibrary};
use regex::Regex;
use std::path::Path;

fn classifier_arch_bucket(classifier: &str) -> &'static str {
    if classifier.ends_with("-arm64") {
        "arm64"
    } else if classifier.ends_with("-x86") {
        "x86"
    } else if classifier.ends_with("-arm32") || classifier.ends_with("-arm") {
        "arm"
    } else {
        "x64"
    }
}

fn platform_arch_bucket(platform: &Platform) -> &'static str {
    match platform.arch.as_str() {
        "arm64" => "arm64",
        "ia32" | "x86" => "x86",
        "arm" => "arm",
        _ => "x64",
    }
}

/// Converts a Maven coordinate ("group:artifact:version[:classifier][@ext]") to a relative jar path.
fn maven_name_to_path(name: &str) -> String {
    let (rest, ext) = match name.split_once('@') {
        Some((r, e)) => (r, e.to_string()),
        None => (name, "jar".to_string()),
    };
    let parts: Vec<&str> = rest.split(':').collect();
    let group = parts.first().copied().unwrap_or("");
    let artifact = parts.get(1).copied().unwrap_or("");
    let version = parts.get(2).copied().unwrap_or("");
    let classifier = parts.get(3).copied();
    let group_path = group.replace('.', "/");
    let file_name = match classifier {
        Some(c) => format!("{artifact}-{version}-{c}.{ext}"),
        None => format!("{artifact}-{version}.{ext}"),
    };
    format!("{group_path}/{artifact}/{version}/{file_name}")
}

pub fn resolve_libraries(raw_libraries: &[RawLibrary], platform: &Platform) -> Vec<ResolvedLibrary> {
    let mut result = Vec::new();
    let embedded_classifier_re = Regex::new(r":(natives-[\w-]+)$").unwrap();

    for lib in raw_libraries {
        if !check_rules(&lib.rules, platform, &Default::default()) {
            continue;
        }

        // Legacy format (pre-~1.19): one library entry carries a `natives` OS->classifier map
        // plus a `downloads.classifiers` bag holding every platform's jar.
        if let Some(legacy_classifier) = lib.natives.as_ref().and_then(|m| m.get(&platform.name)) {
            let artifact = lib.downloads.as_ref().and_then(|d| d.classifiers.as_ref()).and_then(|c| c.get(legacy_classifier));
            let path = artifact
                .and_then(|a| a.path.clone())
                .unwrap_or_else(|| maven_name_to_path(&format!("{}:{}", lib.name, legacy_classifier)));
            let url = artifact
                .map(|a| a.url.clone())
                .or_else(|| lib.url.as_ref().map(|u| format!("{}/{}", u.trim_end_matches('/'), path)));
            let Some(url) = url else { continue };
            result.push(ResolvedLibrary {
                name: lib.name.clone(),
                path,
                url,
                sha1: artifact.and_then(|a| a.sha1.clone()),
                size: artifact.and_then(|a| a.size),
                is_native: true,
                native_classifier: Some(legacy_classifier.clone()),
                extract_exclude: lib.extract.as_ref().and_then(|e| e.exclude.clone()),
            });
            continue;
        }

        // Modern format (LWJGL 3.x+): classifier baked into the Maven name itself.
        if let Some(caps) = embedded_classifier_re.captures(&lib.name) {
            let embedded_classifier = caps[1].to_string();
            if let Some(art) = lib.downloads.as_ref().and_then(|d| d.artifact.as_ref()) {
                if classifier_arch_bucket(&embedded_classifier) != platform_arch_bucket(platform) {
                    continue;
                }
                result.push(ResolvedLibrary {
                    name: lib.name.clone(),
                    path: art.path.clone().unwrap_or_else(|| maven_name_to_path(&lib.name)),
                    url: art.url.clone(),
                    sha1: art.sha1.clone(),
                    size: art.size,
                    is_native: true,
                    native_classifier: Some(embedded_classifier),
                    extract_exclude: lib.extract.as_ref().and_then(|e| e.exclude.clone()),
                });
                continue;
            }
        }

        if let Some(art) = lib.downloads.as_ref().and_then(|d| d.artifact.as_ref()) {
            result.push(ResolvedLibrary {
                name: lib.name.clone(),
                path: art.path.clone().unwrap_or_else(|| maven_name_to_path(&lib.name)),
                url: art.url.clone(),
                sha1: art.sha1.clone(),
                size: art.size,
                is_native: false,
                native_classifier: None,
                extract_exclude: None,
            });
        } else if let Some(base_url) = &lib.url {
            let path = maven_name_to_path(&lib.name);
            result.push(ResolvedLibrary {
                name: lib.name.clone(),
                path: path.clone(),
                url: format!("{}/{}", base_url.trim_end_matches('/'), path),
                sha1: None,
                size: None,
                is_native: false,
                native_classifier: None,
                extract_exclude: None,
            });
        }
    }

    result
}

pub fn build_classpath(libraries: &[ResolvedLibrary], version_jar_path: &Path, libraries_dir: &Path) -> String {
    let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
    let mut jar_paths: Vec<String> = libraries
        .iter()
        .filter(|l| !l.is_native)
        .map(|l| libraries_dir.join(l.path.replace('/', std::path::MAIN_SEPARATOR_STR)).to_string_lossy().to_string())
        .collect();
    jar_paths.push(version_jar_path.to_string_lossy().to_string());
    jar_paths.join(separator)
}

pub async fn download_libraries(
    client: &reqwest::Client,
    libraries: Vec<ResolvedLibrary>,
    libraries_dir: &Path,
    concurrency: usize,
) -> Result<(), String> {
    let libraries_dir = libraries_dir.to_path_buf();
    let client = client.clone();
    let failures = run_pool(libraries, concurrency, move |lib: ResolvedLibrary| {
        let dest = libraries_dir.join(lib.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        let client = client.clone();
        async move { download_file(&client, &lib.url, &dest, DownloadOptions { sha1: lib.sha1.clone() }).await }
    })
    .await;
    if let Some(summary) = summarize_failures(&failures, "libraries") {
        return Err(summary);
    }
    Ok(())
}
