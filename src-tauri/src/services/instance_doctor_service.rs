//! Instance Health Doctor: proactive, on-demand diagnostics for a profile's install — distinct
//! from `crash_analyzer` (which is reactive, parsing a crash log after the fact). Reuses the
//! same resolution/download pipeline the installer itself uses (`version_resolve`, `libraries`,
//! `assets`) so "is this file where it should be" always agrees with what a real install run
//! would produce.

use serde::Serialize;
use std::path::Path;

use crate::mcinstall::{assets, libraries, platform, version_resolve};
use crate::services::profile_service::LaunchProfile;

#[derive(Debug, Clone, Serialize)]
pub struct HealthCheck {
    pub id: String,
    pub label: String,
    /// "pass" | "warn" | "fail"
    pub status: String,
    pub message: String,
    pub fixable: bool,
}

fn check(id: &str, label: &str, status: &str, message: String, fixable: bool) -> HealthCheck {
    HealthCheck { id: id.to_string(), label: label.to_string(), status: status.to_string(), message, fixable }
}

pub async fn run_health_check(client: &reqwest::Client, profile: &LaunchProfile) -> Vec<HealthCheck> {
    let game_dir = Path::new(&profile.game_dir);
    let mut checks = Vec::new();

    // ── Version files present ────────────────────────────────────────────────
    let version_dir = game_dir.join("versions").join(&profile.version);
    let jar_ok = version_dir.join(format!("{}.jar", profile.version)).is_file();
    let json_ok = version_dir.join(format!("{}.json", profile.version)).is_file();
    checks.push(if jar_ok && json_ok {
        check("version_files", "Game version files", "pass", format!("Minecraft {} is installed", profile.version), false)
    } else {
        check("version_files", "Game version files", "fail", "The client jar or version manifest is missing".to_string(), true)
    });

    // Everything past this point needs a resolvable version — bail out with one clear
    // "run Repair" result instead of a wall of confusing secondary failures.
    let Ok(resolved) = version_resolve::resolve_version(game_dir, &profile.version).await else {
        checks.push(check(
            "resolve",
            "Version resolution",
            "fail",
            "Could not read this version's install metadata - run Repair to reinstall it".to_string(),
            true,
        ));
        return checks;
    };

    // ── Libraries ─────────────────────────────────────────────────────────────
    let plat = platform::get_current_platform();
    let resolved_libs = libraries::resolve_libraries(&resolved.libraries, &plat);
    let libraries_dir = game_dir.join("libraries");
    let missing_libs = resolved_libs.iter().filter(|l| !libraries_dir.join(&l.path).is_file()).count();
    checks.push(if missing_libs == 0 {
        check("libraries", "Libraries", "pass", format!("{} libraries present", resolved_libs.len()), false)
    } else {
        check("libraries", "Libraries", "fail", format!("{missing_libs} of {} libraries are missing", resolved_libs.len()), true)
    });

    // ── Assets ────────────────────────────────────────────────────────────────
    match assets::download_asset_index(client, &resolved, game_dir).await {
        Ok(index) => {
            // Full per-object verification can be tens of thousands of files — sample instead,
            // same tradeoff a package manager's "verify" command makes for large trees.
            let objects_dir = game_dir.join("assets").join("objects");
            let sample: Vec<_> = index.objects.values().take(200).collect();
            let missing = sample.iter().filter(|o| !objects_dir.join(&o.hash[0..2]).join(&o.hash).is_file()).count();
            if sample.is_empty() {
                checks.push(check("assets", "Assets", "pass", "No assets required for this version".to_string(), false));
            } else if missing == 0 {
                checks.push(check("assets", "Assets", "pass", format!("Sampled {} assets, all present", sample.len()), false));
            } else {
                let pct = (missing * 100) / sample.len();
                checks.push(check(
                    "assets",
                    "Assets",
                    if pct > 20 { "fail" } else { "warn" },
                    format!("{missing} of {} sampled assets are missing (~{pct}% of the pack)", sample.len()),
                    true,
                ));
            }
        }
        Err(e) => checks.push(check("assets", "Assets", "warn", format!("Could not check assets: {e}"), false)),
    }

    // ── Natives ───────────────────────────────────────────────────────────────
    let natives_dir = game_dir.join("versions").join(&profile.version).join("natives");
    let has_native_libs = resolved_libs.iter().any(|l| l.is_native);
    if has_native_libs {
        let natives_present = std::fs::read_dir(&natives_dir).map(|mut it| it.next().is_some()).unwrap_or(false);
        checks.push(if natives_present {
            check("natives", "Native libraries", "pass", "Extracted".to_string(), false)
        } else {
            check("natives", "Native libraries", "fail", "Native libraries (LWJGL etc.) were never extracted".to_string(), true)
        });
    }

    // ── Java compatibility ────────────────────────────────────────────────────
    if let Some(required) = &resolved.java_version {
        let configured = if profile.java_path.is_empty() { None } else { Some(profile.java_path.clone()) };
        let scanned = crate::mcinstall::java_discovery::scan_local_java().await;
        let matched = match &configured {
            Some(path) => scanned.iter().find(|j| j.path.eq_ignore_ascii_case(path)),
            // Auto-detect: same "pick the newest compatible" idea launch itself would apply —
            // good enough for a diagnostic even if it isn't launch's exact selection algorithm.
            None => scanned.iter().filter(|j| j.major_version >= required.major_version).max_by_key(|j| j.major_version),
        };
        match matched {
            Some(j) if j.major_version >= required.major_version => {
                checks.push(check("java", "Java version", "pass", format!("Java {} satisfies the required {}+", j.major_version, required.major_version), false));
            }
            Some(j) => {
                checks.push(check(
                    "java",
                    "Java version",
                    "fail",
                    format!("Configured Java is version {}, but Minecraft {} needs {}+", j.major_version, profile.version, required.major_version),
                    false,
                ));
            }
            None if configured.is_some() => {
                checks.push(check("java", "Java version", "warn", "Configured Java executable could not be identified - verify the path in Game Settings".to_string(), false));
            }
            None => {
                checks.push(check(
                    "java",
                    "Java version",
                    "fail",
                    format!("No installed Java {}+ found for Minecraft {} - install one and set it in Game Settings", required.major_version, profile.version),
                    false,
                ));
            }
        }
    }

    // ── Disk space ────────────────────────────────────────────────────────────
    {
        let disks = sysinfo::Disks::new_with_refreshed_list();
        let mut best: Option<u64> = None;
        for disk in disks.list() {
            if game_dir.starts_with(disk.mount_point()) {
                best = Some(best.map_or(disk.available_space(), |b| b.max(disk.available_space())));
            }
        }
        if let Some(free) = best {
            let free_gb = free as f64 / 1_073_741_824.0;
            checks.push(if free_gb < 1.0 {
                check("disk_space", "Disk space", "fail", format!("Only {free_gb:.1} GB free - installs and world saves can fail", ), false)
            } else if free_gb < 5.0 {
                check("disk_space", "Disk space", "warn", format!("{free_gb:.1} GB free on this drive"), false)
            } else {
                check("disk_space", "Disk space", "pass", format!("{free_gb:.0} GB free"), false)
            });
        }
    }

    checks
}
