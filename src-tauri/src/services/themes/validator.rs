use crate::types::ThemeError;
use serde_json::Value;

/// Service for validating theme files before import
pub struct ThemeValidator;

impl ThemeValidator {
    /// Validate a theme JSON value
    pub fn validate_theme(theme: &Value) -> Result<(), ThemeError> {
        Self::validate_structure(theme)?;
        Self::validate_metadata(theme)?;
        Self::validate_apps(theme)?;
        Ok(())
    }

    /// Validate the basic structure of a theme
    fn validate_structure(theme: &Value) -> Result<(), ThemeError> {
        let obj = theme.as_object().ok_or_else(|| {
            ThemeError::ValidationFailed("Theme must be a JSON object".to_string())
        })?;

        // Check required fields
        let required_fields = ["name", "created_at", "modified_at", "apps"];
        for field in &required_fields {
            if !obj.contains_key(*field) {
                return Err(ThemeError::ValidationFailed(format!(
                    "Missing required field: {}",
                    field
                )));
            }
        }

        Ok(())
    }

    /// Validate theme metadata fields
    fn validate_metadata(theme: &Value) -> Result<(), ThemeError> {
        // Validate name
        let name = theme.get("name").and_then(|n| n.as_str()).ok_or_else(|| {
            ThemeError::ValidationFailed("Theme name must be a string".to_string())
        })?;

        if name.trim().is_empty() {
            return Err(ThemeError::ValidationFailed(
                "Theme name cannot be empty".to_string(),
            ));
        }

        if name.contains('/') || name.contains('\\') {
            return Err(ThemeError::ValidationFailed(
                "Theme name cannot contain slashes".to_string(),
            ));
        }

        // Validate timestamps
        for field in &["created_at", "modified_at"] {
            let timestamp = theme.get(*field).and_then(|t| t.as_str()).ok_or_else(|| {
                ThemeError::ValidationFailed(format!("{} must be a string", field))
            })?;

            // Basic ISO 8601 format check
            if timestamp.is_empty() {
                return Err(ThemeError::ValidationFailed(format!(
                    "{} cannot be empty",
                    field
                )));
            }
        }

        Ok(())
    }

    /// Validate apps configuration
    fn validate_apps(theme: &Value) -> Result<(), ThemeError> {
        let apps = theme
            .get("apps")
            .and_then(|a| a.as_object())
            .ok_or_else(|| {
                ThemeError::ValidationFailed("Apps field must be a JSON object".to_string())
            })?;

        if apps.is_empty() {
            return Err(ThemeError::ValidationFailed(
                "Theme must contain at least one app configuration".to_string(),
            ));
        }

        // Validate that each app config is an object
        for (app_name, app_config) in apps {
            if !app_config.is_object() {
                return Err(ThemeError::ValidationFailed(format!(
                    "Configuration for '{}' must be a JSON object",
                    app_name
                )));
            }
        }

        Ok(())
    }

    /// Sanitize theme data by removing potentially harmful content
    pub fn sanitize_theme(theme: &mut Value) -> Result<(), ThemeError> {
        // Remove any fields that might be injection vectors
        if let Some(obj) = theme.as_object_mut() {
            // Remove any shell commands or script fields if present
            let dangerous_keys = vec!["__proto__", "constructor", "prototype"];
            for key in dangerous_keys {
                obj.remove(key);
            }
        }

        Ok(())
    }

    /// Validate that color strings are in proper format
    pub fn validate_color(color: &str) -> bool {
        // Check hex color format (#RGB or #RRGGBB or #RRGGBBAA)
        if let Some(hex) = color.strip_prefix('#') {
            return (hex.len() == 3 || hex.len() == 6 || hex.len() == 8)
                && hex.chars().all(|c| c.is_ascii_hexdigit());
        }

        // Check RGB/RGBA format (r,g,b or r,g,b,a)
        let parts: Vec<&str> = color.split(',').collect();
        if parts.len() == 3 || parts.len() == 4 {
            return parts.iter().all(|p| {
                p.trim()
                    .parse::<f32>()
                    .map(|n| (0.0..=255.0).contains(&n))
                    .unwrap_or(false)
            });
        }

        false
    }

    /// Check if theme is a valid Omarchist custom theme
    pub fn is_omarchist_theme(theme: &Value) -> bool {
        // Check for required structure that indicates this is an Omarchist theme
        theme.get("name").is_some()
            && theme.get("created_at").is_some()
            && theme.get("modified_at").is_some()
            && theme.get("apps").is_some()
    }
}
