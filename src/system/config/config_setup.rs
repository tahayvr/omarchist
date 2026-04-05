use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::assets::{extract_default_dir, read_default_str};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsSchema {
    pub version: String,
    pub settings: SettingsConfig,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfig {
    pub font_size: String,
    #[serde(default)]
    pub auto_apply_theme: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub created_at: String,
    pub last_modified: String,
}

pub fn ensure_config() -> Result<(), String> {
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
                    replace_settings_file(&settings_path)?;
                }
                UpdateAction::Keep => {
                    validate_settings(&settings_path)?;
                }
            }
        } else {
            // settings.json doesn't exist, copy it from defaults
            copy_settings_from_default(&settings_path)?;
        }

        // Hyprland errors if the sourced file is missing.
        ensure_hyprland_conf(&config_dir)?;

        return Ok(());
    }

    extract_default_dir("omarchist", &config_dir)?;

    // Update timestamps in settings.json
    let settings_path = config_dir.join("settings.json");
    if settings_path.exists() {
        let timestamp = Utc::now().to_rfc3339();
        let content = fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings.json: {}", e))?;
        let updated_content = content
            .replace("{{CREATED_AT}}", &timestamp)
            .replace("{{MODIFIED_AT}}", &timestamp);
        fs::write(&settings_path, updated_content)
            .map_err(|e| format!("Failed to write settings.json: {}", e))?;
    }

    println!("Created default config at: {:?}", config_dir);

    Ok(())
}

fn get_config_dir() -> Result<PathBuf, String> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    Ok(home_dir.join(".config").join("omarchist"))
}

fn validate_settings(settings_path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(settings_path)
        .map_err(|e| format!("Failed to read settings.json: {}", e))?;

    let settings: SettingsSchema = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid settings.json schema: {}", e))?;

    // Additional validation: check required fields are not empty
    if settings.version.is_empty() {
        return Err("settings.json: version field is empty".to_string());
    }

    if settings.settings.font_size.is_empty() {
        return Err("settings.json: font_size field is empty".to_string());
    }

    if settings.metadata.created_at.is_empty() {
        return Err("settings.json: created_at field is empty".to_string());
    }

    if settings.metadata.last_modified.is_empty() {
        return Err("settings.json: last_modified field is empty".to_string());
    }

    Ok(())
}

pub fn get_settings_path() -> Result<PathBuf, String> {
    get_config_dir().map(|dir| dir.join("settings.json"))
}

pub fn read_settings() -> Result<SettingsSchema, String> {
    let path = get_settings_path()?;
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))
}

pub fn save_settings(settings: &SettingsSchema) -> Result<(), String> {
    let path = get_settings_path()?;
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&path, content).map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

// font_size should be one of: "small", "medium", "large"
pub fn update_font_size(font_size: &str) -> Result<(), String> {
    let mut settings = read_settings()?;

    // Validate font_size value
    let valid_sizes = ["small", "medium", "large"];
    if !valid_sizes.contains(&font_size) {
        return Err(format!(
            "Invalid font_size '{}'. Must be one of: small, medium, large",
            font_size
        ));
    }

    settings.settings.font_size = font_size.to_string();
    settings.metadata.last_modified = Utc::now().to_rfc3339();

    save_settings(&settings)?;

    println!("Updated font_size to: {}", font_size);

    Ok(())
}

pub fn get_font_size() -> Result<String, String> {
    let settings = read_settings()?;
    Ok(settings.settings.font_size)
}

// whether to update or keep the settings file
enum UpdateAction {
    Update,
    Keep,
}

fn get_default_settings_version() -> Result<String, String> {
    let content = read_default_str("omarchist/settings.json")?;
    let settings: SettingsSchema = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse default settings.json: {}", e))?;
    Ok(settings.version)
}

fn parse_version(version: &str) -> Result<(u32, u32, u32), String> {
    // Remove 'v' prefix if present
    let version = version.trim_start_matches('v');

    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(format!(
            "Invalid version format '{}', expected X.Y.Z",
            version
        ));
    }

    let major = parts[0]
        .parse::<u32>()
        .map_err(|e| format!("Invalid major version '{}': {}", parts[0], e))?;
    let minor = parts[1]
        .parse::<u32>()
        .map_err(|e| format!("Invalid minor version '{}': {}", parts[1], e))?;
    let patch = parts[2]
        .parse::<u32>()
        .map_err(|e| format!("Invalid patch version '{}': {}", parts[2], e))?;

    Ok((major, minor, patch))
}

// Compares two version strings
fn is_version_older(user_version: &str, default_version: &str) -> Result<bool, String> {
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

fn should_update_settings(
    settings_path: &Path,
    default_version: &str,
) -> Result<UpdateAction, String> {
    let content = fs::read_to_string(settings_path)
        .map_err(|e| format!("Failed to read settings.json: {}", e))?;
    let settings: SettingsSchema = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))?;

    if is_version_older(&settings.version, default_version)? {
        Ok(UpdateAction::Update)
    } else {
        Ok(UpdateAction::Keep)
    }
}

fn replace_settings_file(settings_path: &Path) -> Result<(), String> {
    let content = read_default_str("omarchist/settings.json")?;

    // Update timestamps
    let timestamp = Utc::now().to_rfc3339();
    let updated_content = content
        .replace("{{CREATED_AT}}", &timestamp)
        .replace("{{MODIFIED_AT}}", &timestamp);
    fs::write(settings_path, updated_content)
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;

    Ok(())
}

// Copies settings.json from defaults when it doesn't exist
fn copy_settings_from_default(settings_path: &Path) -> Result<(), String> {
    replace_settings_file(settings_path)
}

fn ensure_hyprland_conf(config_dir: &Path) -> Result<(), String> {
    let hypr_dir = config_dir.join("hyprland");
    let hypr_conf = hypr_dir.join("hyprland.conf");

    if hypr_conf.exists() {
        return Ok(());
    }

    // Create the hyprland directory if it doesn't exist yet
    fs::create_dir_all(&hypr_dir)
        .map_err(|e| format!("Failed to create hyprland config directory: {}", e))?;

    let content = read_default_str("omarchist/hyprland/hyprland.conf")?;
    fs::write(&hypr_conf, content)
        .map_err(|e| format!("Failed to write default hyprland.conf: {}", e))?;

    println!("Copied default hyprland.conf to: {}", hypr_conf.display());

    Ok(())
}

#[cfg(test)]
mod tests {
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
    fn parse_version_zero_components() {
        let result = parse_version("0.0.0").expect("all-zero version should parse");
        assert_eq!(result, (0, 0, 0));
    }

    #[test]
    fn parse_version_large_numbers() {
        let result = parse_version("10.20.30").expect("large component numbers should parse");
        assert_eq!(result, (10, 20, 30));
    }

    #[test]
    fn parse_version_too_few_parts_returns_err() {
        assert!(
            parse_version("1.2").is_err(),
            "a version with fewer than 3 parts should return Err"
        );
    }

    #[test]
    fn parse_version_too_many_parts_returns_err() {
        assert!(
            parse_version("1.2.3.4").is_err(),
            "a version with more than 3 parts should return Err"
        );
    }

    #[test]
    fn parse_version_non_numeric_part_returns_err() {
        assert!(
            parse_version("1.x.3").is_err(),
            "a non-numeric version component should return Err"
        );
    }

    #[test]
    fn parse_version_empty_string_returns_err() {
        assert!(parse_version("").is_err(), "empty string should return Err");
    }

    #[test]
    fn is_version_older_major_bump_is_older() {
        let result = is_version_older("1.0.0", "2.0.0").expect("valid versions should compare");
        assert!(result, "1.0.0 should be considered older than 2.0.0");
    }

    #[test]
    fn is_version_older_minor_bump_is_older() {
        let result = is_version_older("1.0.0", "1.1.0").expect("valid versions should compare");
        assert!(result, "1.0.0 should be considered older than 1.1.0");
    }

    #[test]
    fn is_version_older_patch_bump_is_older() {
        let result = is_version_older("1.0.0", "1.0.1").expect("valid versions should compare");
        assert!(result, "1.0.0 should be considered older than 1.0.1");
    }

    #[test]
    fn is_version_older_equal_versions_not_older() {
        let result = is_version_older("1.2.3", "1.2.3").expect("equal versions should compare");
        assert!(!result, "equal versions should not be considered older");
    }

    #[test]
    fn is_version_older_user_newer_than_default_not_older() {
        let result = is_version_older("2.0.0", "1.9.9").expect("valid versions should compare");
        assert!(
            !result,
            "a newer user version should not be considered older than default"
        );
    }

    #[test]
    fn is_version_older_major_takes_precedence_over_minor() {
        let result = is_version_older("2.5.0", "3.0.0").expect("valid versions should compare");
        assert!(result, "major version difference should dominate");
    }

    #[test]
    fn is_version_older_v_prefix_handled() {
        let result =
            is_version_older("v1.0.0", "v1.0.1").expect("v-prefixed versions should compare");
        assert!(result, "v-prefixed versions should compare correctly");
    }

    #[test]
    fn is_version_older_invalid_version_returns_err() {
        assert!(
            is_version_older("not-a-version", "1.0.0").is_err(),
            "an invalid version string should return Err"
        );
    }
}
