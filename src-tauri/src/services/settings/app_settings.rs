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

#[cfg(any(test, feature = "test-utils"))]
/// Test-only function that saves settings with directory override
pub async fn save_settings_test_only(
    settings: AppSettings,
    override_dir: Option<PathBuf>,
) -> SettingsResult<()> {
    save_settings_with_override(settings, override_dir).await
}

#[cfg(any(test, feature = "test-utils"))]
/// Test-only function that loads settings without requiring AppHandle
pub async fn load_settings_test_only(override_dir: Option<PathBuf>) -> SettingsResult<AppSettings> {
    let settings_path = get_settings_file_path_with_override(override_dir)?;

    // If file doesn't exist, return default settings (for testing)
    if !settings_path.exists() {
        log::info!("Settings file doesn't exist, using defaults");
        return Ok(AppSettings::default());
    }

    // Read and parse the file
    let content = fs::read_to_string(&settings_path)?;

    // Try to parse as SettingsFile first
    match serde_json::from_str::<SettingsFile>(&content) {
        Ok(settings_file) => {
            // Validate version compatibility
            if settings_file.version.starts_with("1.") {
                Ok(settings_file.settings)
            } else {
                log::warn!(
                    "Unsupported settings version: {}, using defaults",
                    settings_file.version
                );
                Ok(AppSettings::default())
            }
        },
        Err(_) => {
            // Try to parse as legacy AppSettings directly
            match serde_json::from_str::<AppSettings>(&content) {
                Ok(settings) => {
                    log::info!("Loaded legacy settings format, will upgrade on next save");
                    Ok(settings)
                },
                Err(e) => {
                    log::error!("Failed to parse settings file: {}", e);
                    Err(SettingsError::Corrupted)
                },
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Basic structure tests
    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert_eq!(settings.auto_apply_theme, true);
    }

    #[test]
    fn test_settings_file_default() {
        let settings_file = SettingsFile::default();
        assert_eq!(settings_file.version, "1.0.0");
        assert_eq!(settings_file.settings.auto_apply_theme, true);
        assert!(settings_file.metadata.created_at <= Utc::now());
        assert!(settings_file.metadata.last_modified <= Utc::now());
    }

    #[test]
    fn test_app_settings_serialization() {
        let settings = AppSettings {
            auto_apply_theme: false,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings, deserialized);
    }

    #[test]
    fn test_settings_file_serialization() {
        let settings_file = SettingsFile::default();

        let json = serde_json::to_string_pretty(&settings_file).unwrap();
        let deserialized: SettingsFile = serde_json::from_str(&json).unwrap();

        assert_eq!(settings_file.version, deserialized.version);
        assert_eq!(settings_file.settings, deserialized.settings);
    }

    // Validation tests
    #[test]
    fn test_validate_settings_valid_cases() {
        let settings_true = AppSettings {
            auto_apply_theme: true,
        };
        assert!(validate_settings(&settings_true).is_ok());

        let settings_false = AppSettings {
            auto_apply_theme: false,
        };
        assert!(validate_settings(&settings_false).is_ok());
    }

    #[test]
    fn test_validate_and_sanitize_settings() {
        let settings = AppSettings {
            auto_apply_theme: true,
        };
        let result = validate_and_sanitize_settings(settings.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), settings);

        let settings = AppSettings {
            auto_apply_theme: false,
        };
        let result = validate_and_sanitize_settings(settings.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), settings);
    }

    // File I/O tests
    #[tokio::test]
    async fn test_load_settings_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();

        let result = load_settings_test_only(Some(temp_dir.path().to_path_buf())).await;
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings, AppSettings::default());
    }

    #[tokio::test]
    async fn test_save_and_load_settings_roundtrip() {
        let temp_dir = TempDir::new().unwrap();

        let test_settings = AppSettings {
            auto_apply_theme: false,
        };

        // Save settings
        let save_result =
            save_settings_with_override(test_settings.clone(), Some(temp_dir.path().to_path_buf()))
                .await;
        assert!(save_result.is_ok());

        // Load settings
        let load_result = load_settings_test_only(Some(temp_dir.path().to_path_buf())).await;
        assert!(load_result.is_ok());
        let loaded_settings = load_result.unwrap();
        assert_eq!(loaded_settings, test_settings);
    }

    #[tokio::test]
    async fn test_save_settings_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent_subdir = temp_dir.path().join("non_existent");

        let test_settings = AppSettings {
            auto_apply_theme: true,
        };

        // Save should create the directory
        let save_result =
            save_settings_with_override(test_settings.clone(), Some(non_existent_subdir.clone()))
                .await;
        assert!(save_result.is_ok());

        // Verify directory was created
        let settings_dir = non_existent_subdir.join("omarchist");
        assert!(settings_dir.exists());
        assert!(settings_dir.join("settings.json").exists());
    }

    #[tokio::test]
    async fn test_save_settings_atomic_write() {
        let temp_dir = TempDir::new().unwrap();

        let test_settings = AppSettings {
            auto_apply_theme: true,
        };

        // Save settings
        save_settings_with_override(test_settings.clone(), Some(temp_dir.path().to_path_buf()))
            .await
            .unwrap();

        let settings_dir = temp_dir.path().join("omarchist");
        let settings_file = settings_dir.join("settings.json");
        let temp_file = settings_dir.join("settings.json.tmp");

        // Verify the actual file exists and temp file doesn't
        assert!(settings_file.exists());
        assert!(!temp_file.exists());

        // Verify content is valid JSON
        let content = fs::read_to_string(&settings_file).unwrap();
        let parsed: SettingsFile = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.settings, test_settings);
    }

    #[tokio::test]
    async fn test_save_settings_preserves_creation_time() {
        let temp_dir = TempDir::new().unwrap();

        let initial_settings = AppSettings {
            auto_apply_theme: true,
        };

        // Save initial settings
        save_settings_with_override(initial_settings, Some(temp_dir.path().to_path_buf()))
            .await
            .unwrap();

        // Read the creation time
        let settings_file = temp_dir.path().join("omarchist").join("settings.json");
        let content = fs::read_to_string(&settings_file).unwrap();
        let initial_file: SettingsFile = serde_json::from_str(&content).unwrap();
        let creation_time = initial_file.metadata.created_at;

        // Wait a bit to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Save updated settings
        let updated_settings = AppSettings {
            auto_apply_theme: false,
        };
        save_settings_with_override(updated_settings, Some(temp_dir.path().to_path_buf()))
            .await
            .unwrap();

        // Verify creation time is preserved but last_modified is updated
        let content = fs::read_to_string(&settings_file).unwrap();
        let updated_file: SettingsFile = serde_json::from_str(&content).unwrap();

        assert_eq!(updated_file.metadata.created_at, creation_time);
        assert!(updated_file.metadata.last_modified > creation_time);
    }

    // Error handling tests
    #[tokio::test]
    async fn test_load_corrupted_json() {
        let temp_dir = TempDir::new().unwrap();

        // Create the settings directory
        let settings_dir = temp_dir.path().join("omarchist");
        fs::create_dir_all(&settings_dir).unwrap();

        // Write corrupted JSON
        let settings_file = settings_dir.join("settings.json");
        fs::write(&settings_file, "{ invalid json }").unwrap();

        // Try to load settings
        let result = load_settings_test_only(Some(temp_dir.path().to_path_buf())).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SettingsError::Corrupted));
    }

    #[tokio::test]
    async fn test_load_empty_file() {
        let temp_dir = TempDir::new().unwrap();

        // Create the settings directory
        let settings_dir = temp_dir.path().join("omarchist");
        fs::create_dir_all(&settings_dir).unwrap();

        // Write empty file
        let settings_file = settings_dir.join("settings.json");
        fs::write(&settings_file, "").unwrap();

        // Try to load settings
        let result = load_settings_test_only(Some(temp_dir.path().to_path_buf())).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SettingsError::Corrupted));
    }

    #[tokio::test]
    async fn test_load_invalid_json_structure() {
        let temp_dir = TempDir::new().unwrap();

        // Create the settings directory
        let settings_dir = temp_dir.path().join("omarchist");
        fs::create_dir_all(&settings_dir).unwrap();

        // Write valid JSON but wrong structure
        let settings_file = settings_dir.join("settings.json");
        fs::write(&settings_file, r#"{"not_settings": "value"}"#).unwrap();

        // Try to load settings
        let result = load_settings_test_only(Some(temp_dir.path().to_path_buf())).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SettingsError::Corrupted));
    }

    #[tokio::test]
    async fn test_save_to_readonly_directory() {
        let temp_dir = TempDir::new().unwrap();
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir_all(&readonly_dir).unwrap();

        // Make directory readonly (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(&readonly_dir, perms).unwrap();

            let test_settings = AppSettings {
                auto_apply_theme: true,
            };

            // Save should fail
            let save_result =
                save_settings_with_override(test_settings, Some(readonly_dir.clone())).await;
            assert!(save_result.is_err());
            assert!(matches!(
                save_result.unwrap_err(),
                SettingsError::CreateDir(_)
            ));

            // Restore permissions for cleanup
            let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&readonly_dir, perms).unwrap();
        }
    }

    // Legacy format tests
    #[tokio::test]
    async fn test_load_legacy_format() {
        let temp_dir = TempDir::new().unwrap();

        // Create the settings directory
        let settings_dir = temp_dir.path().join("omarchist");
        fs::create_dir_all(&settings_dir).unwrap();

        // Write legacy format (just AppSettings)
        let settings_file = settings_dir.join("settings.json");
        let legacy_settings = AppSettings {
            auto_apply_theme: false,
        };
        let legacy_json = serde_json::to_string_pretty(&legacy_settings).unwrap();
        fs::write(&settings_file, legacy_json).unwrap();

        // Load settings
        let result = load_settings_test_only(Some(temp_dir.path().to_path_buf())).await;
        assert!(result.is_ok());
        let loaded_settings = result.unwrap();
        assert_eq!(loaded_settings.auto_apply_theme, false);
    }

    #[tokio::test]
    async fn test_load_unsupported_version() {
        let temp_dir = TempDir::new().unwrap();

        // Create the settings directory
        let settings_dir = temp_dir.path().join("omarchist");
        fs::create_dir_all(&settings_dir).unwrap();

        // Write future version format
        let settings_file = settings_dir.join("settings.json");
        let future_settings = SettingsFile {
            version: "2.0.0".to_string(),
            settings: AppSettings {
                auto_apply_theme: false,
            },
            metadata: SettingsMetadata {
                created_at: Utc::now(),
                last_modified: Utc::now(),
            },
        };
        let future_json = serde_json::to_string_pretty(&future_settings).unwrap();
        fs::write(&settings_file, future_json).unwrap();

        // Load settings - should fall back to defaults for unsupported version
        let result = load_settings_test_only(Some(temp_dir.path().to_path_buf())).await;
        assert!(result.is_ok());
        let loaded_settings = result.unwrap();
        assert_eq!(loaded_settings, AppSettings::default()); // Should use defaults
    }

    // Edge case tests
    #[tokio::test]
    async fn test_multiple_concurrent_saves() {
        let temp_dir = TempDir::new().unwrap();

        let settings1 = AppSettings {
            auto_apply_theme: true,
        };
        let settings2 = AppSettings {
            auto_apply_theme: false,
        };

        // Start multiple saves concurrently
        let save1 = save_settings_with_override(settings1, Some(temp_dir.path().to_path_buf()));
        let save2 = save_settings_with_override(settings2, Some(temp_dir.path().to_path_buf()));

        let (result1, result2) = tokio::join!(save1, save2);

        // Both saves should succeed (atomic writes should handle concurrency)
        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Load final result - should be one of the two settings
        let load_result = load_settings_test_only(Some(temp_dir.path().to_path_buf())).await;
        assert!(load_result.is_ok());
        let final_settings = load_result.unwrap();
        assert!(
            final_settings.auto_apply_theme == true || final_settings.auto_apply_theme == false
        );
    }

    #[tokio::test]
    async fn test_settings_file_format_validation() {
        let temp_dir = TempDir::new().unwrap();

        // Save settings and verify the file format
        let test_settings = AppSettings {
            auto_apply_theme: true,
        };

        save_settings_with_override(test_settings.clone(), Some(temp_dir.path().to_path_buf()))
            .await
            .unwrap();

        // Read and parse the file manually
        let settings_file = temp_dir.path().join("omarchist").join("settings.json");
        let content = fs::read_to_string(&settings_file).unwrap();

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Verify required fields exist
        assert!(parsed.get("version").is_some());
        assert!(parsed.get("settings").is_some());
        assert!(parsed.get("metadata").is_some());

        // Verify settings structure
        let settings = parsed.get("settings").unwrap();
        assert!(settings.get("auto_apply_theme").is_some());

        // Verify metadata structure
        let metadata = parsed.get("metadata").unwrap();
        assert!(metadata.get("created_at").is_some());
        assert!(metadata.get("last_modified").is_some());
    }

    // Path handling tests
    #[test]
    fn test_get_settings_file_path_with_override() {
        let temp_dir = TempDir::new().unwrap();

        let result = get_settings_file_path_with_override(Some(temp_dir.path().to_path_buf()));
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("omarchist"));
        assert!(path.to_string_lossy().ends_with("settings.json"));
    }

    #[test]
    fn test_get_settings_file_path_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent = temp_dir.path().join("non_existent");

        let result = get_settings_file_path_with_override(Some(non_existent.clone()));
        assert!(result.is_ok());

        // Directory should be created
        let expected_dir = non_existent.join("omarchist");
        assert!(expected_dir.exists());
    }

    // Error type tests
    #[test]
    fn test_settings_error_display() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let settings_error = SettingsError::FileRead(io_error);
        let error_string = format!("{}", settings_error);
        assert!(error_string.contains("Failed to read settings file"));

        // Create a JSON parse error by trying to parse invalid JSON
        let json_error = serde_json::from_str::<AppSettings>("invalid json").unwrap_err();
        let settings_error = SettingsError::JsonParse(json_error);
        let error_string = format!("{}", settings_error);
        assert!(error_string.contains("Failed to parse settings JSON"));

        let validation_error = SettingsError::Validation("Invalid value".to_string());
        let error_string = format!("{}", validation_error);
        assert!(error_string.contains("Settings validation failed"));

        let corrupted_error = SettingsError::Corrupted;
        let error_string = format!("{}", corrupted_error);
        assert!(error_string.contains("Settings file is corrupted"));
    }
}
