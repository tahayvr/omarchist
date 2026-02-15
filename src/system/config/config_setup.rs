use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const DEFAULT_SETTINGS_JSON: &str = include_str!("../../../defaults/settings.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsSchema {
    pub version: String,
    pub settings: SettingsConfig,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfig {
    pub font_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub created_at: String,
    pub last_modified: String,
}

/// Ensures the omarchist config directory exists and contains a valid settings.json
/// Returns Ok(()) on success, or an error message on failure
pub fn ensure_config() -> Result<(), String> {
    let config_dir = get_config_dir()?;
    let settings_path = config_dir.join("settings.json");

    // Check if settings.json already exists
    if settings_path.exists() {
        // Validate the existing settings.json
        validate_settings(&settings_path)?;
        return Ok(());
    }

    // Create the config directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    // Copy default settings and replace placeholders with current timestamp
    let timestamp = Utc::now().to_rfc3339();
    let settings_content = DEFAULT_SETTINGS_JSON
        .replace("{{CREATED_AT}}", &timestamp)
        .replace("{{MODIFIED_AT}}", &timestamp);

    fs::write(&settings_path, settings_content)
        .map_err(|e| format!("Failed to write settings.json: {}", e))?;

    println!("Created default settings at: {:?}", settings_path);

    Ok(())
}

/// Gets the omarchist config directory path (~/.config/omarchist)
fn get_config_dir() -> Result<PathBuf, String> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    Ok(home_dir.join(".config").join("omarchist"))
}

/// Validates that the settings.json file matches the expected schema
fn validate_settings(settings_path: &std::path::Path) -> Result<(), String> {
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

/// Gets the path to the settings.json file
pub fn get_settings_path() -> Result<PathBuf, String> {
    get_config_dir().map(|dir| dir.join("settings.json"))
}

/// Reads the current settings
pub fn read_settings() -> Result<SettingsSchema, String> {
    let path = get_settings_path()?;
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))
}

/// Saves settings to the config file
pub fn save_settings(settings: &SettingsSchema) -> Result<(), String> {
    let path = get_settings_path()?;
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&path, content).map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}
