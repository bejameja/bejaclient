//! Port of `src/main/services/crashAnalyzer.ts`.

use regex::{Regex, RegexBuilder};
use std::collections::HashSet;
use std::sync::LazyLock;

#[derive(Debug, Clone, serde::Serialize)]
pub struct CrashFinding {
    /// "fatal" | "error" | "warn"
    pub severity: String,
    pub category: String,
    pub raw: String,
    #[serde(rename = "humanReadable")]
    pub human_readable: String,
    pub suggestion: String,
}

struct CrashPattern {
    regex: LazyLock<Regex>,
    severity: &'static str,
    category: &'static str,
    explain: fn(&regex::Captures) -> (String, String),
}

macro_rules! pattern {
    ($re:expr, $severity:expr, $category:expr, $explain:expr) => {
        CrashPattern { regex: LazyLock::new(|| Regex::new($re).unwrap()), severity: $severity, category: $category, explain: $explain }
    };
    (i: $re:expr, $severity:expr, $category:expr, $explain:expr) => {
        CrashPattern {
            regex: LazyLock::new(|| RegexBuilder::new($re).case_insensitive(true).build().unwrap()),
            severity: $severity,
            category: $category,
            explain: $explain,
        }
    };
}

fn patterns() -> Vec<CrashPattern> {
    vec![
        pattern!(
            r"Mixin apply failed (.+?) -> (.+?)(?:\n|:)",
            "fatal",
            "Mixin Conflict",
            |m| {
                (
                    format!("Mixin failed to apply to class \"{}\" from config \"{}\".", &m[2], &m[1]),
                    "A mod is patching the same class as BejaClient or Sodium. Check for conflicting mods (OptiFine, Rubidium, Embeddium) and remove them.".to_string(),
                )
            }
        ),
        pattern!(
            r"(?:Exception|Error|FATAL|Caused by)[^\n]*class_(\d+)",
            "error",
            "Mapping Mismatch",
            |m| {
                (
                    format!("Obfuscated class reference \"class_{}\" in an exception — likely a yarn/intermediary mapping mismatch.", &m[1]),
                    "Ensure the Fabric Loader version matches the yarn mappings version in build.gradle. Run `./gradlew --stop` then rebuild.".to_string(),
                )
            }
        ),
        pattern!(
            r"(?s)NullPointerException.{0,200}?GameOptions",
            "fatal",
            "GameOptions NPE",
            |_| {
                (
                    "Minecraft's GameOptions failed to initialize — options.txt is likely corrupt.".to_string(),
                    "Delete %APPDATA%\\.minecraft\\options.txt and let Minecraft recreate it.".to_string(),
                )
            }
        ),
        pattern!(
            r"Cannot find .+? in the target class (.+?) for injection",
            "fatal",
            "Mixin Target Missing",
            |m| {
                (
                    format!("Mixin injection target missing in class \"{}\".", &m[1]),
                    "The Minecraft version changed and a method was renamed. Update yarn mappings and re-check Mixin @At selectors.".to_string(),
                )
            }
        ),
        pattern!(
            r"net\.fabricmc\.loader requires .+ version .+ of (.+?),",
            "fatal",
            "Dependency Version",
            |m| {
                (
                    format!("Required dependency \"{}\" has an incompatible version.", &m[1]),
                    format!("Update or remove \"{}\" to match the version required by Fabric Loader / Sodium.", &m[1]),
                )
            }
        ),
        pattern!(
            r"java\.lang\.StackOverflowError",
            "fatal",
            "StackOverflow (Mixin Loop)",
            |_| {
                (
                    "Stack overflow — almost always caused by a Mixin that calls itself recursively.".to_string(),
                    "Check for @Inject methods that call the original method and trigger the same injection again. Use @Unique helpers or @Redirect instead.".to_string(),
                )
            }
        ),
        pattern!(
            i: r"EXCEPTION_ACCESS_VIOLATION|opengl error|GL_INVALID_OPERATION",
            "fatal",
            "GPU / OpenGL Crash",
            |_| {
                (
                    "OpenGL or GPU driver error.".to_string(),
                    "Update GPU drivers. If using Sodium, check that no conflicting renderer mods (OptiFabric, VulkanMod) are in mods/.".to_string(),
                )
            }
        ),
        pattern!(
            i: r"Bootstrap JAR not found|beja-bootstrap.*?not found",
            "warn",
            "BejaClient Bootstrap Missing",
            |_| {
                (
                    "BejaClient bootstrap JAR could not be located.".to_string(),
                    "Trigger a re-download in the launcher or verify %APPDATA%\\BejaClient\\beja-libs\\ contains beja-bootstrap-*.jar.".to_string(),
                )
            }
        ),
    ]
}

fn severity_order(s: &str) -> u8 {
    match s {
        "fatal" => 0,
        "error" => 1,
        _ => 2,
    }
}

pub fn analyze_crash_log(log: &str) -> Vec<CrashFinding> {
    let mut findings = Vec::new();
    let mut seen = HashSet::new();

    for p in patterns() {
        let Some(m) = p.regex.captures(log) else { continue };
        let whole = m.get(0).unwrap().as_str();
        let key = format!("{}:{}", p.category, &whole[..whole.len().min(60)]);
        if !seen.insert(key) {
            continue;
        }
        let (human_readable, suggestion) = (p.explain)(&m);
        findings.push(CrashFinding { severity: p.severity.to_string(), category: p.category.to_string(), raw: whole.trim().to_string(), human_readable, suggestion });
    }

    findings.sort_by_key(|f| severity_order(&f.severity));
    findings
}
