use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

/// Ensures the omarchist config directory exists with all default files
/// If the directory doesn't exist, copies the entire defaults/omarchist folder
/// Returns Ok(()) on success, or an error message on failure
pub fn ensure_config() -> Result<(), String> {
    let config_dir = get_config_dir()?;

    // If config directory already exists, just validate settings.json
    if config_dir.exists() {
        let settings_path = config_dir.join("settings.json");
        if settings_path.exists() {
            validate_settings(&settings_path)?;
        }
        return Ok(());
    }

    // Copy the entire defaults/omarchist folder to ~/.config/omarchist
    let defaults_dir = PathBuf::from("defaults/omarchist");
    copy_directory_recursive(&defaults_dir, &config_dir)?;

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

/// Recursively copy a directory and all its contents
fn copy_directory_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    // Create destination directory
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create directory '{}': {}", dst.display(), e))?;

    // Read source directory entries
    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory '{}': {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .ok_or_else(|| "Invalid file name".to_string())?;
        let dest_path = dst.join(file_name);

        if path.is_dir() {
            // Recursively copy subdirectory
            copy_directory_recursive(&path, &dest_path)?;
        } else {
            // Copy file
            fs::copy(&path, &dest_path)
                .map_err(|e| format!("Failed to copy file '{}': {}", path.display(), e))?;
        }
    }

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

/// Updates the font_size setting and saves to disk
/// font_size should be one of: "small", "medium", "large"
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

/// Gets the current font_size setting
pub fn get_font_size() -> Result<String, String> {
    let settings = read_settings()?;
    Ok(settings.settings.font_size)
}
