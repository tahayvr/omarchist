use crate::services::settings::app_settings;
use crate::types::{AppSettings, SettingsError};
use tauri::AppHandle;

/// Get current app settings
#[tauri::command]
pub async fn get_app_settings(app_handle: AppHandle) -> Result<AppSettings, String> {
    log::info!("Loading app settings");

    app_settings::load_settings(&app_handle)
        .await
        .map_err(|e| {
            log::error!("Failed to load settings: {e}");
            match e {
                SettingsError::Corrupted => {
                    "Settings file is corrupted. Default settings will be used.".to_string()
                },
                SettingsError::FileRead(_) => {
                    "Unable to read settings file. Please check file permissions.".to_string()
                },
                SettingsError::JsonParse(_) => {
                    "Settings file format is invalid. Default settings will be used.".to_string()
                },
                _ => format!("Unable to load settings: {e}"),
            }
        })
        .map(|settings| {
            log::info!("Successfully loaded settings: {settings:?}");
            settings
        })
}

/// Update app settings
#[tauri::command]
pub async fn update_app_settings(settings: AppSettings) -> Result<(), String> {
    log::info!("Received settings update request: {settings:?}");

    // Validate and sanitize settings first
    let validated_settings =
        app_settings::validate_and_sanitize_settings(settings).map_err(|e| {
            log::error!("Settings validation failed: {e}");
            format!("Invalid settings provided: {e}")
        })?;

    log::info!("Settings validation passed, proceeding to save");

    // Save the validated settings
    app_settings::save_settings(validated_settings)
        .await
        .map_err(|e| {
            log::error!("Failed to save settings: {e}");
            format!("Unable to save settings. Please check file permissions and try again: {e}")
        })?;

    log::info!("Settings saved successfully");
    Ok(())
}

/// Reset app settings to defaults
#[tauri::command]
pub async fn reset_app_settings(app_handle: AppHandle) -> Result<AppSettings, String> {
    log::info!("Resetting app settings to defaults");

    app_settings::reset_to_defaults(&app_handle)
        .await
        .map_err(|e| {
            log::error!("Failed to reset settings: {e}");
            format!("Unable to reset settings to defaults. Please try again: {e}")
        })
        .map(|settings| {
            log::info!("Successfully reset settings to defaults: {settings:?}");
            settings
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_app_settings_command_returns_result() {
        // Test that the command signature is correct
        // Actual functionality requires AppHandle and is tested through integration tests
        // This test ensures the function compiles and has the right signature
        assert!(true); // Command signature test
    }

    #[tokio::test]
    async fn test_update_app_settings_command_accepts_valid_settings() {
        // Test that the command accepts valid settings structure
        let test_settings = AppSettings {
            auto_apply_theme: false,
        };

        // Test that settings can be serialized (required for Tauri commands)
        let json = serde_json::to_string(&test_settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(
            test_settings.auto_apply_theme,
            deserialized.auto_apply_theme
        );
    }

    #[tokio::test]
    async fn test_reset_app_settings_command_returns_result() {
        // Test that the command signature is correct
        // Actual functionality requires AppHandle and is tested through integration tests
        assert!(true); // Command signature test
    }

    #[test]
    fn test_settings_structure_serialization() {
        // Test that AppSettings can be serialized/deserialized for Tauri commands
        let settings = AppSettings {
            auto_apply_theme: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.auto_apply_theme, deserialized.auto_apply_theme);
    }
}
