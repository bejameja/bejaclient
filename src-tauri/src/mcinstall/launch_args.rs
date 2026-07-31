//! Port of `src/main/services/mcinstall/launchArgs.ts`.

use super::platform::get_current_platform;
use super::rules::check_rules;
use super::types::{ArgumentEntry, ResolvedVersion};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

pub struct AuthCtx {
    pub uuid: String,
    pub username: String,
    pub access_token: String,
    pub user_type: String, // "msa"
}

pub struct ResolutionCtx {
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
}

pub struct ArgContext<'a> {
    pub version: &'a ResolvedVersion,
    pub game_dir: String,
    pub resource_dir: String,
    pub natives_dir: String,
    pub classpath: String,
    #[allow(dead_code)]
    pub java_path: String,
    pub auth: AuthCtx,
    pub resolution: ResolutionCtx,
    pub min_memory: Option<u32>,
    pub max_memory: Option<u32>,
    pub launcher_version: String,
}

fn substitute(token: &str, ctx: &ArgContext) -> String {
    let re = Regex::new(r"\$\{([a-zA-Z_]+)\}").unwrap();
    re.replace_all(token, |caps: &regex::Captures| {
        let key = &caps[1];
        resolve_substitution(key, ctx).unwrap_or_else(|| caps[0].to_string())
    })
    .to_string()
}

fn resolve_substitution(key: &str, ctx: &ArgContext) -> Option<String> {
    Some(match key {
        "auth_player_name" => ctx.auth.username.clone(),
        "version_name" => ctx.version.id.clone(),
        "game_directory" => ctx.game_dir.clone(),
        "assets_root" => Path::new(&ctx.resource_dir).join("assets").to_string_lossy().to_string(),
        "game_assets" => Path::new(&ctx.resource_dir).join("assets").join("virtual").join("legacy").to_string_lossy().to_string(),
        "assets_index_name" => ctx.version.assets.clone(),
        "auth_uuid" => ctx.auth.uuid.clone(),
        "auth_access_token" => ctx.auth.access_token.clone(),
        "auth_session" => ctx.auth.access_token.clone(),
        "user_type" => ctx.auth.user_type.clone(),
        "version_type" => ctx.version.r#type.clone(),
        "natives_directory" => ctx.natives_dir.clone(),
        "classpath" => ctx.classpath.clone(),
        "launcher_name" => "BejaClient".to_string(),
        "launcher_version" => ctx.launcher_version.clone(),
        "resolution_width" => ctx.resolution.width.to_string(),
        "resolution_height" => ctx.resolution.height.to_string(),
        "user_properties" => "{}".to_string(),
        _ => return None,
    })
}

fn current_features(ctx: &ArgContext) -> HashMap<String, bool> {
    HashMap::from([
        ("has_custom_resolution".to_string(), true),
        ("has_quick_plays_support".to_string(), false),
        ("is_demo_user".to_string(), false),
        ("is_quick_play_singleplayer".to_string(), false),
        ("is_quick_play_multiplayer".to_string(), false),
        ("is_quick_play_realms".to_string(), false),
        ("fullscreen".to_string(), ctx.resolution.fullscreen),
    ])
}

fn flatten_structured_args(entries: &[ArgumentEntry], ctx: &ArgContext) -> Vec<String> {
    let platform = get_current_platform();
    let features = current_features(ctx);
    let mut out = Vec::new();
    for entry in entries {
        if let Some(s) = entry.as_str() {
            out.push(substitute(s, ctx));
            continue;
        }
        // { rules, value } object form.
        let rules: Option<Vec<super::types::Rule>> = entry.get("rules").and_then(|r| serde_json::from_value(r.clone()).ok());
        if !check_rules(&rules, &platform, &features) {
            continue;
        }
        let Some(value) = entry.get("value") else { continue };
        if let Some(arr) = value.as_array() {
            for v in arr {
                if let Some(s) = v.as_str() {
                    out.push(substitute(s, ctx));
                }
            }
        } else if let Some(s) = value.as_str() {
            out.push(substitute(s, ctx));
        }
    }
    out
}

/// Standard JVM args for pre-1.13 versions, which have no `arguments.jvm` template at all.
fn legacy_jvm_args(ctx: &ArgContext) -> Vec<String> {
    let mut args = vec![format!("-Djava.library.path={}", ctx.natives_dir), "-cp".to_string(), ctx.classpath.clone()];
    if get_current_platform().name == "osx" {
        args.push("-XstartOnFirstThread".to_string());
    }
    args
}

pub fn build_jvm_args(ctx: &ArgContext, extra_jvm_args: &[String]) -> Vec<String> {
    let mut mem_args = Vec::new();
    if let Some(min) = ctx.min_memory {
        mem_args.push(format!("-Xms{min}M"));
    }
    if let Some(max) = ctx.max_memory {
        mem_args.push(format!("-Xmx{max}M"));
    }

    let version_jvm_args = match &ctx.version.arguments {
        Some(args) if !args.jvm.is_empty() => flatten_structured_args(&args.jvm, ctx),
        _ => legacy_jvm_args(ctx),
    };

    let mut out = mem_args;
    out.extend(version_jvm_args);
    out.extend(extra_jvm_args.iter().cloned());
    out
}

pub fn build_game_args(ctx: &ArgContext, extra_mc_args: &[String]) -> Vec<String> {
    let version_game_args = if let Some(args) = &ctx.version.arguments {
        flatten_structured_args(&args.game, ctx)
    } else if let Some(flat) = &ctx.version.minecraft_arguments {
        // Flat pre-1.13 string: split first, substitute second.
        flat.split_whitespace().map(|tok| substitute(tok, ctx)).collect()
    } else {
        Vec::new()
    };

    let mut out = version_game_args;
    out.extend(extra_mc_args.iter().cloned());
    out
}

pub fn assemble_command(ctx: &ArgContext, extra_jvm_args: &[String], extra_mc_args: &[String]) -> Vec<String> {
    let mut out = build_jvm_args(ctx, extra_jvm_args);
    out.push(ctx.version.main_class.clone());
    out.extend(build_game_args(ctx, extra_mc_args));
    out
}
