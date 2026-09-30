//! `~/.config/omarchist/settings.json`: the app's own settings. A newer
//! Omarchist adds keys by merging the defaults into the user's file, never
//! by replacing it, so nothing the user chose is lost on an update.
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::assets::{extract_default_dir, read_default_str};
use crate::error::{Error, Result};
use crate::system::fs::write_atomic;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsSchema {
    pub version: String,
    pub settings: SettingsConfig,
    pub metadata: Metadata,
}

/// The user's choices on the Settings page. Every field has a default so
/// a file from an older Omarchist still loads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SettingsConfig {
    /// `small`, `medium`, or `large`.
    pub font_size: String,
    pub auto_apply_theme: bool,
    /// `omarchy` (follow the desktop theme), `light`, or `dark`.
    pub theme_mode: String,
    /// A CLI view name (`themes`, `config`, ...) or `last`.
    pub startup_page: String,
    pub check_updates: bool,
    pub update_check_hours: u32,
    pub notify_updates: bool,
    pub notify_flows: bool,
    /// The Omarchist bar widget is installed and enabled.
    pub bar_widget: bool,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            font_size: "small".to_string(),
            auto_apply_theme: false,
            theme_mode: "omarchy".to_string(),
            startup_page: "themes".to_string(),
            check_updates: true,
            update_check_hours: 6,
            notify_updates: false,
            notify_flows: false,
            bar_widget: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Metadata {
    pub created_at: String,
    pub last_modified: String,
    /// The page shown last, for `startup_page = "last"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_page: Option<String>,
}

pub const FONT_SIZES: &[&str] = &["small", "medium", "large"];
pub const THEME_MODES: &[&str] = &["omarchy", "light", "dark"];

pub fn ensure_config() -> Result<()> {
    let config_dir = get_config_dir()?;

    // Get the default settings version from the embedded defaults
    let default_version = get_default_settings_version()?;

    if config_dir.exists() {
        let settings_path = config_dir.join("settings.json");
        if settings_path.exists() {
            // Check if user's version needs updating
            match should_update_settings(&settings_path, &default_version)? {
                UpdateAction::Update => {
                    println!(
                        "Updating settings.json from older version to {}",
                        default_version
                    );
                    migrate_settings_file(&settings_path)?;
                }
                UpdateAction::Keep => {
                    validate_settings(&settings_path)?;
                }
            }
        } else {
            // settings.json doesn't exist, copy it from defaults
            copy_settings_from_default(&settings_path)?;
        }

        remove_legacy_hyprland_conf(&config_dir)?;

        return Ok(());
    }

    extract_default_dir("omarchist", &config_dir)?;

    // Update timestamps in settings.json
    let settings_path = config_dir.join("settings.json");
    if settings_path.exists() {
        let timestamp = Utc::now().to_rfc3339();
        let content = fs::read_to_string(&settings_path)
            .map_err(|e| Error::io("Failed to read settings.json", e))?;
        let updated_content = content
            .replace("{{CREATED_AT}}", &timestamp)
            .replace("{{MODIFIED_AT}}", &timestamp);
        write_atomic(&settings_path, updated_content, "settings.json")?;
    }

    println!("Created default config at: {:?}", config_dir);

    Ok(())
}

fn get_config_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or(Error::UnknownDirectory("home"))?;

    Ok(home_dir.join(".config").join("omarchist"))
}

fn validate_settings(settings_path: &Path) -> Result<()> {
    let content = fs::read_to_string(settings_path)
        .map_err(|e| Error::io("Failed to read settings.json", e))?;

    let settings: SettingsSchema = serde_json::from_str(&content)
        .map_err(|e| Error::json("Invalid settings.json schema", e))?;

    // Additional validation: check required fields are not empty
    if settings.version.is_empty() {
        return Err(Error::Invalid(
            "settings.json: version field is empty".into(),
        ));
    }

    if settings.settings.font_size.is_empty() {
        return Err(Error::Invalid(
            "settings.json: font_size field is empty".into(),
        ));
    }

    if settings.metadata.created_at.is_empty() {
        return Err(Error::Invalid(
            "settings.json: created_at field is empty".into(),
        ));
    }

    if settings.metadata.last_modified.is_empty() {
        return Err(Error::Invalid(
            "settings.json: last_modified field is empty".into(),
        ));
    }

    Ok(())
}

pub fn get_settings_path() -> Result<PathBuf> {
    get_config_dir().map(|dir| dir.join("settings.json"))
}

pub fn read_settings() -> Result<SettingsSchema> {
    let path = get_settings_path()?;
    let content = fs::read_to_string(&path).map_err(|e| Error::io("Failed to read settings", e))?;

    serde_json::from_str(&content).map_err(|e| Error::json("Failed to parse settings", e))
}

/// The user's settings, or the defaults when the file cannot be read.
pub fn settings() -> SettingsConfig {
    read_settings().map(|s| s.settings).unwrap_or_default()
}

pub fn save_settings(settings: &SettingsSchema) -> Result<()> {
    let path = get_settings_path()?;
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| Error::json("Failed to serialize settings", e))?;

    write_atomic(&path, content, "settings.json")?;

    Ok(())
}

/// Changes the settings in place and stamps the modification time.
pub fn update_settings(change: impl FnOnce(&mut SettingsConfig)) -> Result<()> {
    let mut schema = read_settings()?;
    change(&mut schema.settings);
    schema.metadata.last_modified = Utc::now().to_rfc3339();
    save_settings(&schema)
}

/// Records the page shown last, when the app is set to reopen on it.
pub fn remember_last_page(page: &str) -> Result<()> {
    let mut schema = read_settings()?;
    if schema.settings.startup_page != "last" || schema.metadata.last_page.as_deref() == Some(page)
    {
        return Ok(());
    }
    schema.metadata.last_page = Some(page.to_string());
    save_settings(&schema)
}

// font_size should be one of: "small", "medium", "large"
pub fn update_font_size(font_size: &str) -> Result<()> {
    if !FONT_SIZES.contains(&font_size) {
        return Err(Error::Invalid(format!(
            "Invalid font_size '{}'. Must be one of: small, medium, large",
            font_size
        )));
    }
    update_settings(|settings| settings.font_size = font_size.to_string())
}

pub fn get_font_size() -> Result<String> {
    let settings = read_settings()?;
    Ok(settings.settings.font_size)
}

// whether to update or keep the settings file
enum UpdateAction {
    Update,
    Keep,
}

fn get_default_settings_version() -> Result<String> {
    let content = read_default_str("omarchist/settings.json")?;
    let settings: SettingsSchema = serde_json::from_str(&content)
        .map_err(|e| Error::json("Failed to parse default settings.json", e))?;
    Ok(settings.version)
}

fn parse_version(version: &str) -> Result<(u32, u32, u32)> {
    // Remove 'v' prefix if present
    let version = version.trim_start_matches('v');

    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(Error::Invalid(format!(
            "Invalid version format '{}', expected X.Y.Z",
            version
        )));
    }

    let major = parts[0]
        .parse::<u32>()
        .map_err(|e| Error::Invalid(format!("Invalid major version '{}': {}", parts[0], e)))?;
    let minor = parts[1]
        .parse::<u32>()
        .map_err(|e| Error::Invalid(format!("Invalid minor version '{}': {}", parts[1], e)))?;
    let patch = parts[2]
        .parse::<u32>()
        .map_err(|e| Error::Invalid(format!("Invalid patch version '{}': {}", parts[2], e)))?;

    Ok((major, minor, patch))
}

// Compares two version strings
fn is_version_older(user_version: &str, default_version: &str) -> Result<bool> {
    let user = parse_version(user_version)?;
    let default = parse_version(default_version)?;

    if user.0 != default.0 {
        return Ok(user.0 < default.0);
    }
    if user.1 != default.1 {
        return Ok(user.1 < default.1);
    }
    Ok(user.2 < default.2)
}

fn should_update_settings(settings_path: &Path, default_version: &str) -> Result<UpdateAction> {
    let content = fs::read_to_string(settings_path)
        .map_err(|e| Error::io("Failed to read settings.json", e))?;
    let settings: SettingsSchema = serde_json::from_str(&content)
        .map_err(|e| Error::json("Failed to parse settings.json", e))?;

    if is_version_older(&settings.version, default_version)? {
        Ok(UpdateAction::Update)
    } else {
        Ok(UpdateAction::Keep)
    }
}

/// The default settings with the timestamps filled in.
fn default_settings_content(created_at: &str) -> Result<String> {
    let content = read_default_str("omarchist/settings.json")?;
    let timestamp = Utc::now().to_rfc3339();
    Ok(content
        .replace("{{CREATED_AT}}", created_at)
        .replace("{{MODIFIED_AT}}", &timestamp))
}

/// Brings an older file up to the current version, keeping every value
/// the user set and adding the keys they do not have yet.
fn migrate_settings_file(settings_path: &Path) -> Result<()> {
    let content = fs::read_to_string(settings_path)
        .map_err(|e| Error::io("Failed to read settings.json", e))?;
    let user: Value = serde_json::from_str(&content)
        .map_err(|e| Error::json("Failed to parse settings.json", e))?;
    let created_at = user["metadata"]["created_at"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    let defaults: Value = serde_json::from_str(&default_settings_content(&created_at)?)
        .map_err(|e| Error::json("Failed to parse default settings.json", e))?;
    let merged = merge_settings(&user, &defaults);
    let content = serde_json::to_string_pretty(&merged)
        .map_err(|e| Error::json("Failed to serialize settings", e))?;
    write_atomic(settings_path, content, "settings.json")
}

/// The defaults with the user's values on top: every key under `settings`
/// the defaults know keeps the user's value when it has the right type,
/// `metadata` keeps what the user had, and `version` is the defaults'.
pub fn merge_settings(user: &Value, defaults: &Value) -> Value {
    let mut merged = defaults.clone();
    if let (Some(target), Some(source)) = (
        merged.get_mut("settings").and_then(Value::as_object_mut),
        user.get("settings").and_then(Value::as_object),
    ) {
        for (key, value) in target.iter_mut() {
            if let Some(theirs) = source.get(key)
                && same_kind(value, theirs)
            {
                *value = theirs.clone();
            }
        }
    }
    if let (Some(target), Some(source)) = (
        merged.get_mut("metadata").and_then(Value::as_object_mut),
        user.get("metadata").and_then(Value::as_object),
    ) {
        for (key, value) in source {
            if key != "last_modified" {
                target.insert(key.clone(), value.clone());
            }
        }
    }
    if merged.get("metadata").is_none() {
        merged["metadata"] = Value::Object(Map::new());
    }
    merged
}

fn same_kind(a: &Value, b: &Value) -> bool {
    matches!(
        (a, b),
        (Value::Bool(_), Value::Bool(_))
            | (Value::Number(_), Value::Number(_))
            | (Value::String(_), Value::String(_))
    )
}

// Copies settings.json from defaults when it doesn't exist
fn copy_settings_from_default(settings_path: &Path) -> Result<()> {
    let content = default_settings_content(&Utc::now().to_rfc3339())?;
    write_atomic(settings_path, content, "settings.json")
}

// Pre-Quattro versions of Omarchist generated a hyprlang
// `~/.config/omarchist/hyprland/hyprland.conf` that the user's `hyprland.conf`
// sourced via a glob. Quattro's Hyprland config is Lua, and Omarchist now
// writes `~/.config/hypr/omarchist.lua` instead (see `hyprland_config`), so
// the old generated file is dead weight — delete it if an upgrade left it behind.
fn remove_legacy_hyprland_conf(config_dir: &Path) -> Result<()> {
    let legacy_conf = config_dir.join("hyprland").join("hyprland.conf");

    if legacy_conf.exists() {
        fs::remove_file(&legacy_conf)
            .map_err(|e| Error::io("Failed to remove legacy hyprland.conf", e))?;
        println!("Removed legacy config: {}", legacy_conf.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parse_version_valid_semver() {
        let result = parse_version("1.2.3").expect("valid semver should parse");
        assert_eq!(result, (1, 2, 3));
    }

    #[test]
    fn parse_version_strips_v_prefix() {
        let result = parse_version("v2.0.1").expect("v-prefixed semver should parse");
        assert_eq!(result, (2, 0, 1));
    }

    #[test]
    fn parse_version_rejects_bad_shapes() {
        assert!(parse_version("1.2").is_err());
        assert!(parse_version("1.2.3.4").is_err());
        assert!(parse_version("1.x.3").is_err());
        assert!(parse_version("").is_err());
    }

    #[test]
    fn is_version_older_compares_major_minor_patch() {
        assert!(is_version_older("1.0.0", "2.0.0").unwrap());
        assert!(is_version_older("1.0.0", "1.1.0").unwrap());
        assert!(is_version_older("1.0.0", "1.0.1").unwrap());
        assert!(!is_version_older("1.2.3", "1.2.3").unwrap());
        assert!(!is_version_older("2.0.0", "1.9.9").unwrap());
        assert!(is_version_older("2.5.0", "3.0.0").unwrap());
        assert!(is_version_older("v1.0.0", "v1.0.1").unwrap());
        assert!(is_version_older("not-a-version", "1.0.0").is_err());
    }

    #[test]
    fn merge_keeps_user_values_and_adds_new_keys() {
        let user = json!({
            "version": "1.1.0",
            "settings": { "font_size": "large", "auto_apply_theme": true, "stale_key": 1 },
            "metadata": { "created_at": "2025-01-01T00:00:00Z", "last_modified": "old", "last_page": "flows" }
        });
        let defaults = json!({
            "version": "1.2.0",
            "settings": { "font_size": "small", "auto_apply_theme": false, "check_updates": true, "update_check_hours": 6 },
            "metadata": { "created_at": "2025-01-01T00:00:00Z", "last_modified": "now" }
        });
        let merged = merge_settings(&user, &defaults);
        assert_eq!(merged["version"], "1.2.0");
        assert_eq!(merged["settings"]["font_size"], "large");
        assert_eq!(merged["settings"]["auto_apply_theme"], true);
        assert_eq!(
            merged["settings"]["check_updates"], true,
            "new key gets its default"
        );
        assert_eq!(merged["settings"]["update_check_hours"], 6);
        assert!(
            merged["settings"].get("stale_key").is_none(),
            "unknown keys are dropped"
        );
        assert_eq!(merged["metadata"]["created_at"], "2025-01-01T00:00:00Z");
        assert_eq!(merged["metadata"]["last_modified"], "now");
        assert_eq!(merged["metadata"]["last_page"], "flows");
    }

    #[test]
    fn merge_ignores_a_user_value_of_the_wrong_type() {
        let user = json!({ "settings": { "font_size": 12, "check_updates": "yes" } });
        let defaults = json!({ "version": "1.2.0", "settings": { "font_size": "small", "check_updates": true }, "metadata": {} });
        let merged = merge_settings(&user, &defaults);
        assert_eq!(merged["settings"]["font_size"], "small");
        assert_eq!(merged["settings"]["check_updates"], true);
    }

    #[test]
    fn the_shipped_defaults_parse_and_match_the_struct_defaults() {
        let content = read_default_str("omarchist/settings.json").unwrap();
        let schema: SettingsSchema = serde_json::from_str(&content).unwrap();
        let expected = serde_json::to_value(SettingsConfig::default()).unwrap();
        assert_eq!(serde_json::to_value(&schema.settings).unwrap(), expected);
    }

    #[test]
    fn migrate_settings_file_rewrites_in_place() {
        let dir = std::env::temp_dir().join(format!("omarchist-settings-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        fs::write(
            &path,
            r#"{"version":"1.1.0","settings":{"font_size":"large","auto_apply_theme":true},"metadata":{"created_at":"2025-01-01T00:00:00Z","last_modified":"2025-01-02T00:00:00Z"}}"#,
        )
        .unwrap();
        migrate_settings_file(&path).unwrap();
        let schema: SettingsSchema =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        fs::remove_dir_all(&dir).ok();
        assert_eq!(schema.version, get_default_settings_version().unwrap());
        assert_eq!(schema.settings.font_size, "large");
        assert!(schema.settings.auto_apply_theme);
        assert!(schema.settings.check_updates);
        assert_eq!(schema.metadata.created_at, "2025-01-01T00:00:00Z");
        assert_ne!(schema.metadata.last_modified, "2025-01-02T00:00:00Z");
    }
}
