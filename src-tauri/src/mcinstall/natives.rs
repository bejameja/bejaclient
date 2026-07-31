//! Port of `src/main/services/mcinstall/natives.ts`.

use super::types::ResolvedLibrary;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Bump whenever a change to native-library *selection* or *extraction* logic could change what
/// ends up on disk for an unchanged-looking resolved library set — see the TS original's comment
/// for the v2 history (wrong-arch classifier overwrite bug on 64-bit Windows).
const EXTRACTOR_VERSION: u32 = 2;

#[derive(Debug, Serialize, Deserialize)]
struct NativesManifest {
    #[serde(rename = "extractorVersion")]
    extractor_version: u32,
    jars: Vec<String>,
    files: Vec<String>,
}

fn manifest_path(natives_dir: &Path) -> PathBuf {
    natives_dir.join(".natives-manifest.json")
}

fn read_manifest(natives_dir: &Path) -> Option<NativesManifest> {
    let raw = fs::read_to_string(manifest_path(natives_dir)).ok()?;
    let manifest: NativesManifest = serde_json::from_str(&raw).ok()?;
    if manifest.extractor_version != EXTRACTOR_VERSION {
        return None;
    }
    Some(manifest)
}

const DEFAULT_EXCLUDE: &[&str] = &["META-INF/"];

fn is_excluded(entry_name: &str, excludes: &[String]) -> bool {
    DEFAULT_EXCLUDE.iter().any(|p| entry_name.starts_with(p)) || excludes.iter().any(|p| entry_name.starts_with(p.as_str()))
}

/// Newer vanilla version JSONs point `-Djava.library.path` at `${natives_directory}/java`
/// instead of the dir directly. Alias a `java` subdirectory back to the natives dir itself via a
/// directory junction (no admin rights needed on Windows) so both conventions resolve the same.
fn ensure_java_subdir_alias(natives_dir: &Path) {
    let java_subdir = natives_dir.join("java");
    if java_subdir.exists() {
        return;
    }
    #[cfg(windows)]
    {
        let _ = junction::create(natives_dir, &java_subdir);
    }
    #[cfg(not(windows))]
    {
        let _ = std::os::unix::fs::symlink(natives_dir, &java_subdir);
    }
}

/// Extracts native-classifier library jars into `natives_dir`. Skips re-extraction if the
/// resolved native library set is unchanged AND every previously-extracted file is still
/// present on disk.
pub async fn extract_natives(native_libraries: Vec<ResolvedLibrary>, libraries_dir: &Path, natives_dir: &Path) -> Result<(), String> {
    let libraries_dir = libraries_dir.to_path_buf();
    let natives_dir = natives_dir.to_path_buf();

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut current_set: Vec<String> = native_libraries.iter().map(|l| l.path.clone()).collect();
        current_set.sort();

        let previous = read_manifest(&natives_dir);
        let jars_unchanged = previous.as_ref().map(|p| p.jars == current_set).unwrap_or(false);
        let files_intact = previous
            .as_ref()
            .map(|p| !p.files.is_empty() && p.files.iter().all(|f| natives_dir.join(f).exists()))
            .unwrap_or(false);

        if jars_unchanged && files_intact {
            ensure_java_subdir_alias(&natives_dir);
            return Ok(());
        }

        fs::create_dir_all(&natives_dir).map_err(|e| e.to_string())?;

        let mut extracted_files = Vec::new();
        for lib in &native_libraries {
            let jar_path = libraries_dir.join(lib.path.replace('/', std::path::MAIN_SEPARATOR_STR));
            if !jar_path.exists() {
                continue;
            }
            let file = fs::File::open(&jar_path).map_err(|e| e.to_string())?;
            let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
            let excludes = lib.extract_exclude.clone().unwrap_or_default();

            for i in 0..zip.len() {
                let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
                if entry.is_dir() {
                    continue;
                }
                let name = entry.name().to_string();
                if is_excluded(&name, &excludes) {
                    continue;
                }
                let dest = natives_dir.join(&name);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
                fs::write(&dest, &buf).map_err(|e| e.to_string())?;
                extracted_files.push(name);
            }
        }

        let manifest = NativesManifest { extractor_version: EXTRACTOR_VERSION, jars: current_set, files: extracted_files };
        if let Ok(json) = serde_json::to_string(&manifest) {
            let _ = fs::write(manifest_path(&natives_dir), json);
        }
        ensure_java_subdir_alias(&natives_dir);
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
