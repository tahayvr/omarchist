use crate::services::themes::{CustomThemeService, ThemeValidator};
use crate::types::ThemeError;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

/// Service for importing and exporting custom themes
pub struct ThemeImportExportService {
    custom_theme_service: CustomThemeService,
}

/// Result of a theme import operation
#[derive(Debug, serde::Serialize)]
pub struct ImportResult {
    pub success: bool,
    pub theme_name: String,
    pub message: String,
    pub conflict: Option<ConflictInfo>,
}

/// Information about a naming conflict during import
#[derive(Debug, serde::Serialize)]
pub struct ConflictInfo {
    pub existing_theme: String,
    pub suggested_name: String,
}

/// Result of theme validation
#[derive(Debug, serde::Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub theme_name: Option<String>,
}

impl ThemeImportExportService {
    /// Create a new import/export service
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let custom_theme_service = CustomThemeService::new(app_handle)?;

        Ok(Self {
            custom_theme_service,
        })
    }

    /// Export a custom theme to a JSON file
    pub fn export_theme(&self, theme_name: &str, destination: PathBuf) -> Result<PathBuf, ThemeError> {
        // Get the theme
        let theme = self
            .custom_theme_service
            .get_theme(theme_name)
            .map_err(|e| ThemeError::ExportFailed(format!("Failed to get theme: {}", e)))?;

        // Ensure destination has .json extension
        let mut export_path = destination;
        if export_path.extension().is_none() {
            export_path.set_extension("json");
        }

        // Serialize theme to JSON
        let theme_json = serde_json::to_string_pretty(&theme)
            .map_err(|e| ThemeError::ExportFailed(format!("Failed to serialize theme: {}", e)))?;

        // Write to file
        fs::write(&export_path, theme_json)
            .map_err(|e| ThemeError::ExportFailed(format!("Failed to write file: {}", e)))?;

        log::info!("Exported theme '{}' to {}", theme_name, export_path.display());

        Ok(export_path)
    }

    /// Import a theme from a JSON file
    pub fn import_theme_from_file(
        &self,
        file_path: &Path,
        rename_on_conflict: bool,
    ) -> Result<ImportResult, ThemeError> {
        // Read and parse the file
        let content = fs::read_to_string(file_path)
            .map_err(|e| ThemeError::ImportFailed(format!("Failed to read file: {}", e)))?;

        let mut theme_value: Value = serde_json::from_str(&content)
            .map_err(|e| ThemeError::ImportFailed(format!("Invalid JSON: {}", e)))?;

        // Validate it's an Omarchist theme
        if !ThemeValidator::is_omarchist_theme(&theme_value) {
            return Err(ThemeError::ImportFailed(
                "This file does not appear to be an Omarchist custom theme".to_string(),
            ));
        }

        // Validate the theme structure
        ThemeValidator::validate_theme(&theme_value)?;

        // Sanitize the theme data
        ThemeValidator::sanitize_theme(&mut theme_value)?;

        // Extract theme name
        let original_name = theme_value
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| ThemeError::ImportFailed("Missing theme name".to_string()))?
            .to_string();

        // Check for naming conflicts
        let theme_name = if self.theme_exists(&original_name) {
            if rename_on_conflict {
                // Generate a unique name
                let unique_name = self.generate_unique_name(&original_name);
                
                // Update the theme name in the JSON
                if let Some(obj) = theme_value.as_object_mut() {
                    obj.insert("name".to_string(), serde_json::Value::String(unique_name.clone()));
                }
                
                log::info!("Renamed theme from '{}' to '{}' due to conflict", original_name, unique_name);
                unique_name
            } else {
                return Ok(ImportResult {
                    success: false,
                    theme_name: original_name.clone(),
                    message: "Theme already exists".to_string(),
                    conflict: Some(ConflictInfo {
                        existing_theme: original_name.clone(),
                        suggested_name: self.generate_unique_name(&original_name),
                    }),
                });
            }
        } else {
            original_name.clone()
        };

        // Create the theme using CustomThemeService
        let apps_data = theme_value
            .get("apps")
            .cloned()
            .ok_or_else(|| ThemeError::ImportFailed("Missing apps configuration".to_string()))?;

        self.custom_theme_service
            .create_theme_advanced(theme_name.clone(), apps_data)
            .map_err(|e| ThemeError::ImportFailed(format!("Failed to create theme: {}", e)))?;

        log::info!("Successfully imported theme '{}'", theme_name);

        Ok(ImportResult {
            success: true,
            theme_name: theme_name.clone(),
            message: format!("Successfully imported theme '{}'", theme_name),
            conflict: None,
        })
    }

    /// Validate a theme file without importing it
    pub fn validate_theme_file(&self, file_path: &Path) -> ValidationResult {
        let mut errors = Vec::new();
        let mut theme_name = None;

        // Try to read the file
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Failed to read file: {}", e));
                return ValidationResult {
                    valid: false,
                    errors,
                    theme_name: None,
                };
            }
        };

        // Try to parse JSON
        let theme_value: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                errors.push(format!("Invalid JSON format: {}", e));
                return ValidationResult {
                    valid: false,
                    errors,
                    theme_name: None,
                };
            }
        };

        // Check if it's an Omarchist theme
        if !ThemeValidator::is_omarchist_theme(&theme_value) {
            errors.push("Not an Omarchist custom theme".to_string());
            return ValidationResult {
                valid: false,
                errors,
                theme_name: None,
            };
        }

        // Extract theme name
        if let Some(name) = theme_value.get("name").and_then(|n| n.as_str()) {
            theme_name = Some(name.to_string());
        }

        // Validate theme structure
        if let Err(e) = ThemeValidator::validate_theme(&theme_value) {
            errors.push(format!("Validation error: {}", e));
        }

        ValidationResult {
            valid: errors.is_empty(),
            errors,
            theme_name,
        }
    }

    /// Check if a theme with the given name exists
    fn theme_exists(&self, name: &str) -> bool {
        self.custom_theme_service.get_theme(name).is_ok()
    }

    /// Generate a unique theme name by appending a number
    fn generate_unique_name(&self, base_name: &str) -> String {
        let mut counter = 1;
        let mut unique_name = format!("{} ({})", base_name, counter);

        while self.theme_exists(&unique_name) {
            counter += 1;
            unique_name = format!("{} ({})", base_name, counter);
        }

        unique_name
    }
}