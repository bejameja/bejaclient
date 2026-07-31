//! Port of `src/main/services/mcinstall/rules.ts`.

use super::types::{Platform, Rule};
use regex::Regex;
use std::collections::HashMap;

fn os_matches(rule_os: &Option<super::types::RuleOs>, platform: &Platform) -> bool {
    let Some(os) = rule_os else { return true };
    if let Some(name) = &os.name {
        if name != &platform.name {
            return false;
        }
    }
    if let Some(arch) = &os.arch {
        if arch != &platform.arch {
            return false;
        }
    }
    if let Some(version) = &os.version {
        match Regex::new(version) {
            Ok(re) => {
                if !re.is_match(&platform.version) {
                    return false;
                }
            }
            Err(_) => return false,
        }
    }
    true
}

fn features_match(rule_features: &Option<HashMap<String, bool>>, features: &HashMap<String, bool>) -> bool {
    let Some(rule_features) = rule_features else { return true };
    rule_features
        .iter()
        .all(|(key, expected)| features.get(key).copied().unwrap_or(false) == *expected)
}

/// Evaluates a Mojang version-JSON rule list. No rules => allowed. Otherwise the LAST matching
/// rule's action wins (not "any disallow wins") — some libraries ship a base allow-all rule
/// followed by a narrower disallow for one OS/arch.
pub fn check_rules(rules: &Option<Vec<Rule>>, platform: &Platform, features: &HashMap<String, bool>) -> bool {
    let Some(rules) = rules else { return true };
    if rules.is_empty() {
        return true;
    }
    let mut allowed = false;
    for rule in rules {
        if !os_matches(&rule.os, platform) || !features_match(&rule.features, features) {
            continue;
        }
        allowed = rule.action == "allow";
    }
    allowed
}
