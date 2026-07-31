//! Port of `src/main/services/optionsService.ts`.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Key-value pairs forced regardless of what options.txt currently says. `BTreeMap` (vs. the
/// TS `Record`) keeps insertion-independent, deterministic output — not load-bearing, just tidy.
fn perf_overrides() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::from([
        ("maxFps", "260"), // 260 = unlimited in MC 1.17+
        ("enableVsync", "false"),
        ("renderDistance", "8"),
        ("simulationDistance", "6"),
        ("particles", "2"), // minimal
        ("cloudRenderMode", "off"),
        ("entityShadows", "false"),
        ("ao", "false"),
        ("biomeBlendRadius", "0"),
    ])
}

pub fn patch_options_file(game_dir: &Path) {
    let opt_path = game_dir.join("options.txt");
    let existing = fs::read_to_string(&opt_path).unwrap_or_default();

    // Preserve insertion order of the existing file, only appending new keys at the end —
    // matches the TS `Map` behaviour (Map preserves first-insertion order on re-`set`).
    let mut order: Vec<String> = Vec::new();
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    let mut ordered_map: Vec<(String, String)> = Vec::new();

    for line in existing.lines() {
        if let Some(colon) = line.find(':') {
            let key = line[..colon].trim().to_string();
            let value = line[colon + 1..].trim().to_string();
            if !map.contains_key(&key) {
                order.push(key.clone());
            }
            map.insert(key, value);
        }
    }

    for (key, value) in perf_overrides() {
        if !map.contains_key(key) {
            order.push(key.to_string());
        }
        map.insert(key.to_string(), value.to_string());
    }

    for key in &order {
        if let Some(value) = map.get(key) {
            ordered_map.push((key.clone(), value.clone()));
        }
    }

    let out = ordered_map.iter().map(|(k, v)| format!("{k}:{v}")).collect::<Vec<_>>().join("\n") + "\n";
    let _ = fs::write(&opt_path, out);
}
