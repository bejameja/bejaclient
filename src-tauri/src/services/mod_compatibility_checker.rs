//! Port of `src/main/services/modCompatibilityChecker.ts`.
//!
//! Distinct from `mod_service.rs`'s `.jar.disabled` toggle — this moves known-conflicting
//! manually-installed mods into a `.disabled/` *subdirectory* instead, since BejaClient bundles
//! native equivalents (Sodium/Lithium/FerriteCore/etc.) that would otherwise class-conflict.

use regex::Regex;
use serde::Serialize;
use std::path::Path;

struct BlacklistEntry {
    pattern: &'static str,
    mod_id: &'static str,
    reason: &'static str,
    suggestion: &'static str,
}

const BLACKLIST: &[BlacklistEntry] = &[
    BlacklistEntry { pattern: r"(?i)^optifine[-_]", mod_id: "OptiFine", reason: "Incompatible with Sodium renderer.", suggestion: "Remove OptiFine. BejaClient bundles Sodium for performance." },
    BlacklistEntry { pattern: r"(?i)^optifabric[-_]", mod_id: "OptiFabric", reason: "OptiFabric loads OptiFine which conflicts with Sodium.", suggestion: "Remove OptiFabric and OptiFine." },
    BlacklistEntry { pattern: r"(?i)^rubidium[-_]", mod_id: "Rubidium", reason: "Rubidium is a Sodium port — duplicates Sodium bundled by BejaClient.", suggestion: "Remove Rubidium. BejaClient bundles Sodium directly." },
    BlacklistEntry { pattern: r"(?i)^embeddium[-_]", mod_id: "Embeddium", reason: "Embeddium conflicts with Sodium renderer.", suggestion: "Remove Embeddium." },
    BlacklistEntry { pattern: r"(?i)^vulkanmod[-_]", mod_id: "VulkanMod", reason: "VulkanMod replaces the renderer — incompatible with Sodium.", suggestion: "Remove VulkanMod." },
    BlacklistEntry { pattern: r"(?i)^canvas[-_]", mod_id: "Canvas", reason: "Canvas renderer conflicts with Sodium.", suggestion: "Remove Canvas." },
    BlacklistEntry {
        pattern: r"(?i)^ferritecore[-_]|^ferrite-core[-_]",
        mod_id: "FerriteCore (manual)",
        reason: "BejaClient has built-in memory optimizations — this JAR duplicates that work and can cause class conflicts.",
        suggestion: "Remove this FerriteCore JAR; BejaClient optimizes memory usage natively.",
    },
    BlacklistEntry {
        pattern: r"(?i)^sodium-fabric[-_]|^sodium[-_]\d",
        mod_id: "Sodium (manual)",
        reason: "BejaClient has built-in rendering optimizations — this JAR duplicates that work and can cause class conflicts.",
        suggestion: "Remove this Sodium JAR; BejaClient optimizes rendering natively.",
    },
    BlacklistEntry {
        pattern: r"(?i)^lithium[-_]",
        mod_id: "Lithium (manual)",
        reason: "BejaClient has built-in game-logic optimizations — this JAR duplicates that work and can cause class conflicts.",
        suggestion: "Remove this Lithium JAR; BejaClient optimizes game logic natively.",
    },
    BlacklistEntry {
        pattern: r"(?i)^immediatelyfast[-_]",
        mod_id: "ImmediatelyFast (manual)",
        reason: "BejaClient has built-in rendering optimizations — this JAR duplicates that work and can cause class conflicts.",
        suggestion: "Remove this ImmediatelyFast JAR; BejaClient optimizes rendering natively.",
    },
    BlacklistEntry {
        pattern: r"(?i)^entityculling[-_]",
        mod_id: "EntityCulling (manual)",
        reason: "BejaClient has built-in entity culling — this JAR duplicates that work and can cause class conflicts.",
        suggestion: "Remove this EntityCulling JAR; BejaClient culls entities natively.",
    },
    BlacklistEntry {
        pattern: r"(?i)^moreculling[-_]",
        mod_id: "More Culling (manual)",
        reason: "BejaClient has built-in culling optimizations — this JAR duplicates that work and can cause class conflicts.",
        suggestion: "Remove this More Culling JAR; BejaClient culls natively.",
    },
    BlacklistEntry {
        pattern: r"(?i)^badoptimizations[-_]",
        mod_id: "BadOptimizations (manual)",
        reason: "BejaClient has built-in micro-optimizations — this JAR duplicates that work and can cause class conflicts.",
        suggestion: "Remove this BadOptimizations JAR; BejaClient applies these natively.",
    },
    BlacklistEntry {
        pattern: r"(?i)^dynamic[-_]?fps[-_]",
        mod_id: "Dynamic FPS (manual)",
        reason: "BejaClient has a built-in idle/unfocused FPS throttle — this JAR duplicates that work and can cause class conflicts.",
        suggestion: "Remove this Dynamic FPS JAR; BejaClient throttles idle FPS natively.",
    },
    BlacklistEntry {
        pattern: r"(?i)^krypton[-_]",
        mod_id: "Krypton (manual)",
        reason: "BejaClient has built-in network optimizations — this JAR duplicates that work and can cause class conflicts.",
        suggestion: "Remove this Krypton JAR; BejaClient optimizes networking natively.",
    },
];

#[derive(Debug, Clone, Serialize)]
pub struct MovedMod {
    pub file: String,
    #[serde(rename = "modId")]
    pub mod_id: String,
    pub reason: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompatibilityResult {
    pub moved: Vec<MovedMod>,
    pub clean: bool,
}

pub fn enforce_mod_compatibility(mods_dir: &Path, on_log: &impl Fn(String)) -> CompatibilityResult {
    if !mods_dir.exists() {
        return CompatibilityResult { moved: Vec::new(), clean: true };
    }

    let disabled_dir = mods_dir.join(".disabled");
    let mut moved = Vec::new();

    let Ok(entries) = std::fs::read_dir(mods_dir) else {
        return CompatibilityResult { moved, clean: true };
    };

    let jars: Vec<String> = entries
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|f| f.to_lowercase().ends_with(".jar") && !f.starts_with("beja-"))
        .collect();

    for jar in jars {
        let Some(entry) = BLACKLIST.iter().find(|e| Regex::new(e.pattern).unwrap().is_match(&jar)) else { continue };

        if !disabled_dir.exists() {
            let _ = std::fs::create_dir_all(&disabled_dir);
        }

        let src = mods_dir.join(&jar);
        let dest = disabled_dir.join(&jar);

        match std::fs::rename(&src, &dest) {
            Ok(_) => {
                on_log(format!("[ModChecker] Disabled {jar} → .disabled/  |  Reason: {}", entry.reason));
                moved.push(MovedMod { file: jar, mod_id: entry.mod_id.to_string(), reason: entry.reason.to_string(), suggestion: entry.suggestion.to_string() });
            }
            Err(e) => on_log(format!("[ModChecker] WARN: Could not move {jar}: {e}")),
        }
    }

    if moved.is_empty() {
        on_log("[ModChecker] No conflicting mods found.".to_string());
    }
    let clean = moved.is_empty();
    CompatibilityResult { moved, clean }
}
