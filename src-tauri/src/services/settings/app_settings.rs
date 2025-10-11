use crate::types::{AppSettings, SettingsError, SettingsFile, SettingsMetadata, SettingsResult};
use chrono::Utc;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use std::fs;

/// Load default settings from the bundled resources directory
async fn load_default_settings_from_resources(
    app_handle: &AppHandle,
) -> SettingsResult<AppSettings> {
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| SettingsError::Validation(format!("Failed to get resource directory: {e}")))?;

    let default_settings_path = resource_dir.join("settings.json");

    if default_settings_path.exists() {
        let content = fs::read_to_string(&default_settings_path)?;
        match serde_json::from_str::<SettingsFile>(&content) {
            Ok(settings_file) => {
                log::info!("Loaded default settings from resources");
                Ok(settings_file.settings)
            },
            Err(e) => {
                log::warn!(
                    "Failed to parse default settings from resources: {e}, using hardcoded defaults"
                );
                Ok(AppSettings::default())
            },
        }
    } else {
        log::info!("No default settings file found in resources, using hardcoded defaults");
        Ok(AppSettings::default())
    }
}

/// Get the path to the settings file with optional directory override (for testing)
fn get_settings_file_path_with_override(override_dir: Option<PathBuf>) -> SettingsResult<PathBuf> {
    let app_data_dir = if let Some(dir) = override_dir {
        dir.join("omarchist")
    } else {
        // Check for XDG_CONFIG_HOME first, then fall back to dirs::config_dir() for Arch Linux
        if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg_config_home).join("omarchist")
        } else {
            dirs::config_dir()
                .ok_or(SettingsError::AppDataDir)?
                .join("omarchist")
        }
    };

    // Create the directory if it doesn't exist
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir).map_err(SettingsError::CreateDir)?;
    }

    Ok(app_data_dir.join("settings.json"))
}

/// Load settings from file
pub async fn load_settings(app_handle: &AppHandle) -> SettingsResult<AppSettings> {
    load_settings_with_override(app_handle, None).await
}

/// Load settings from file with optional directory override (for testing)
async fn load_settings_with_override(
    app_handle: &AppHandle,
    override_dir: Option<PathBuf>,
) -> SettingsResult<AppSettings> {
    let settings_path = get_settings_file_path_with_override(override_dir.clone())?;

    // If file doesn't exist, try to load defaults from resources and create user settings file
    if !settings_path.exists() {
        log::info!(
            "Settings file doesn't exist at {settings_path:?}, loading defaults from resources"
        );
        let default_settings = load_default_settings_from_resources(app_handle).await?;

        // Try to create the user settings file with defaults for future use
        if let Err(e) = save_settings_with_override(default_settings.clone(), override_dir).await {
            log::warn!("Failed to create initial settings file: {e}");
            // Continue with defaults even if we can't save them
        } else {
            log::info!("Created initial settings file with defaults");
        }

        return Ok(default_settings);
    }

    // Read and parse the file
    log::info!("Loading existing settings from: {settings_path:?}");
    let content = fs::read_to_string(&settings_path)?;

    // Try to parse as SettingsFile first
    match serde_json::from_str::<SettingsFile>(&content) {
        Ok(settings_file) => {
            // Validate version compatibility
            if settings_file.version.starts_with("1.") {
                // Validate the loaded settings
                match validate_settings(&settings_file.settings) {
                    Ok(()) => {
                        log::info!("Successfully loaded and validated settings from file");
                        Ok(settings_file.settings)
                    },
                    Err(validation_error) => {
                        log::error!("Settings validation failed after loading: {validation_error}");
                        log::warn!("Falling back to defaults due to validation failure");

                        // Try to backup the corrupted file for debugging
                        let backup_path = settings_path.with_extension("json.corrupted");
                        if let Err(backup_err) = fs::copy(&settings_path, &backup_path) {
                            log::warn!(
                                "Failed to create backup of corrupted settings: {backup_err}"
                            );
                        } else {
                            log::info!("Backed up corrupted settings to: {backup_path:?}");
                        }

                        // Return defaults instead of error for better user experience
                        let defaults = load_default_settings_from_resources(app_handle).await?;

                        // Try to save the defaults to fix the corrupted file
                        if let Err(save_err) =
                            save_settings_with_override(defaults.clone(), override_dir.clone())
                                .await
                        {
                            log::warn!(
                                "Failed to save default settings after corruption recovery: {save_err}"
                            );
                        } else {
                            log::info!("Successfully restored default settings after corruption");
                        }

                        Ok(defaults)
                    },
                }
            } else {
                log::warn!(
                    "Unsupported settings version: {}, falling back to defaults",
                    settings_file.version
                );
                let defaults = load_default_settings_from_resources(app_handle).await?;

                // Try to save the defaults with current version
                if let Err(save_err) =
                    save_settings_with_override(defaults.clone(), override_dir.clone()).await
                {
                    log::warn!("Failed to upgrade settings file to current version: {save_err}");
                } else {
                    log::info!("Successfully upgraded settings file to current version");
                }

                Ok(defaults)
            }
        },
        Err(parse_error) => {
            log::warn!("Failed to parse as SettingsFile format: {parse_error}");

            // Try to parse as legacy AppSettings directly
            match serde_json::from_str::<AppSettings>(&content) {
                Ok(settings) => {
                    log::info!("Successfully parsed as legacy settings format");

                    // Validate the legacy settings
                    match validate_settings(&settings) {
                        Ok(()) => {
                            log::info!(
                                "Legacy settings validated successfully, will upgrade on next save"
                            );
                            Ok(settings)
                        },
                        Err(validation_error) => {
                            log::error!("Legacy settings validation failed: {validation_error}");
                            log::warn!("Falling back to defaults due to legacy validation failure");

                            // Backup the corrupted legacy file
                            let backup_path = settings_path.with_extension("json.legacy_corrupted");
                            if let Err(backup_err) = fs::copy(&settings_path, &backup_path) {
                                log::warn!(
                                    "Failed to create backup of corrupted legacy settings: {backup_err}"
                                );
                            } else {
                                log::info!(
                                    "Backed up corrupted legacy settings to: {backup_path:?}"
                                );
                            }

                            let defaults = load_default_settings_from_resources(app_handle).await?;

                            // Save defaults to replace corrupted file
                            if let Err(save_err) =
                                save_settings_with_override(defaults.clone(), override_dir.clone())
                                    .await
                            {
                                log::warn!(
                                    "Failed to save defaults after legacy corruption recovery: {save_err}"
                                );
                            } else {
                                log::info!(
                                    "Successfully restored defaults after legacy corruption"
                                );
                            }

                            Ok(defaults)
                        },
                    }
                },
                Err(legacy_parse_error) => {
                    log::error!("Failed to parse settings file in any format. Parse errors - SettingsFile: {parse_error}, AppSettings: {legacy_parse_error}");
                    log::error!(
                        "Settings file content (first 200 chars): {}",
                        content.chars().take(200).collect::<String>()
                    );

                    // Backup the completely corrupted file
                    let backup_path = settings_path.with_extension("json.unparseable");
                    if let Err(backup_err) = fs::copy(&settings_path, &backup_path) {
                        log::warn!("Failed to create backup of unparseable settings: {backup_err}");
                    } else {
                        log::info!("Backed up unparseable settings to: {backup_path:?}");
                    }

                    // Return error for completely corrupted files
                    Err(SettingsError::Corrupted)
                },
            }
        },
    }
}

/// Save settings to file with atomic write
pub async fn save_settings(settings: AppSettings) -> SettingsResult<()> {
    save_settings_with_override(settings, None).await
}

/// Save settings to file with optional directory override (for testing)
async fn save_settings_with_override(
    settings: AppSettings,
    override_dir: Option<PathBuf>,
) -> SettingsResult<()> {
    // Validate and sanitize settings before saving
    let validated_settings = validate_and_sanitize_settings(settings)?;
    log::info!("Settings validated successfully before saving: {validated_settings:?}");

    let settings_path = get_settings_file_path_with_override(override_dir)?;

    // Create settings file with metadata
    let mut settings_file = SettingsFile {
        version: "1.0.0".to_string(),
        settings: validated_settings,
        metadata: SettingsMetadata {
            created_at: Utc::now(),
            last_modified: Utc::now(),
        },
    };

    // If file exists, preserve creation time
    if settings_path.exists() {
        if let Ok(existing_content) = fs::read_to_string(&settings_path) {
            if let Ok(existing_file) = serde_json::from_str::<SettingsFile>(&existing_content) {
                settings_file.metadata.created_at = existing_file.metadata.created_at;
            }
        }
    }

    // Serialize to JSON with pretty formatting
    let json_content = serde_json::to_string_pretty(&settings_file)?;

    // Atomic write: write to temp file first, then rename
    let temp_path = settings_path.with_extension("json.tmp");
    fs::write(&temp_path, json_content)?;
    fs::rename(temp_path, &settings_path)?;

    log::info!("Settings saved successfully to: {settings_path:?}");
    Ok(())
}

/// Reset settings to defaults
pub async fn reset_to_defaults(app_handle: &AppHandle) -> SettingsResult<AppSettings> {
    reset_to_defaults_with_override(app_handle, None).await
}

/// Reset settings to defaults with optional directory override (for testing)
async fn reset_to_defaults_with_override(
    app_handle: &AppHandle,
    override_dir: Option<PathBuf>,
) -> SettingsResult<AppSettings> {
    let default_settings = load_default_settings_from_resources(app_handle).await?;
    save_settings_with_override(default_settings.clone(), override_dir).await?;
    log::info!("Settings reset to defaults from resources");
    Ok(default_settings)
}

/// Validate settings values
pub fn validate_settings(settings: &AppSettings) -> SettingsResult<()> {
    log::debug!("Validating settings: {settings:?}");

    // Validate auto_apply_theme - ensure it's a valid boolean value
    // While Rust's type system ensures it's a bool, we can add logical validation
    // For example, checking if the value makes sense in the current context

    // Log the validation for debugging
    log::debug!(
        "Settings validation - auto_apply_theme: {}",
        settings.auto_apply_theme
    );

    // All validations passed
    log::debug!("Settings validation completed successfully");
    Ok(())
}

/// Validate and sanitize settings from potentially untrusted sources
pub fn validate_and_sanitize_settings(settings: AppSettings) -> SettingsResult<AppSettings> {
    log::debug!("Validating and sanitizing settings: {settings:?}");

    // Create a sanitized copy of the settings
    let sanitized_settings = AppSettings {
        auto_apply_theme: settings.auto_apply_theme, // Boolean is already safe
    };

    // Validate the sanitized settings
    validate_settings(&sanitized_settings)?;

    log::info!("Settings sanitized and validated successfully");
    Ok(sanitized_settings)
}
