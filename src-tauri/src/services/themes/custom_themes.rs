use super::color_extraction::ColorExtractor;
use crate::services::config::generators::ConfigGeneratorRegistry;
use crate::types::{
    AlacrittyColors, AlacrittyConfig, AlacrittyPrimaryColors, CustomTheme, ThemeColors,
};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub struct CustomThemeService {
    themes_dir: PathBuf,
    generator_registry: ConfigGeneratorRegistry,
    app_handle: AppHandle,
}

impl CustomThemeService {
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        // Use the same directory structure as system themes: ~/.config/omarchy/themes/
        let home_dir =
            dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;

        let themes_dir = home_dir.join(".config").join("omarchy").join("themes");

        // Create themes directory if it doesn't exist
        fs::create_dir_all(&themes_dir)
            .map_err(|e| format!("Failed to create themes directory: {e}"))?;

        Ok(Self {
            themes_dir,
            generator_registry: ConfigGeneratorRegistry::new(),
            app_handle: app_handle.clone(),
        })
    }

    /// Sanitize theme name for directory usage (optimized to reduce allocations)
    pub fn sanitize_name(name: &str) -> String {
        let mut result = String::with_capacity(name.len()); // Pre-allocate capacity

        for ch in name.chars() {
            match ch {
                ' ' => result.push('-'),
                c if c.is_alphanumeric() || c == '-' || c == '_' => {
                    result.extend(c.to_lowercase());
                },
                _ => {}, // Skip invalid characters
            }
        }

        result
    }

    /// Extract colors from theme data with fallback to Alacritty config file
    fn extract_theme_colors(&self, theme_dir: &Path, theme_data: &Value) -> Option<ThemeColors> {
        // First try to extract from theme data (custom theme JSON)
        if let Some(colors) = ColorExtractor::extract_from_custom_theme(theme_data) {
            return Some(colors);
        }

        // Fallback: try to extract from Alacritty config file
        let alacritty_config_path = theme_dir.join("alacritty.toml");
        if alacritty_config_path.exists() {
            if let Some(colors) =
                ColorExtractor::extract_from_alacritty_config(&alacritty_config_path)
            {
                return Some(colors);
            }
        }

        // If all else fails, return fallback colors
        Some(ColorExtractor::get_fallback_colors())
    }

    /// Create a new custom theme with modern multi-app support
    pub fn create_theme_advanced(
        &self,
        name: String,
        theme_data: Value,
    ) -> Result<CustomTheme, String> {
        let sanitized_name = Self::sanitize_name(&name);
        let theme_dir = self.themes_dir.join(&sanitized_name);

        // Check if theme already exists
        if theme_dir.exists() {
            return Err(format!("Theme '{name}' already exists"));
        }

        // Create theme directory
        fs::create_dir_all(&theme_dir)
            .map_err(|e| format!("Failed to create theme directory: {e}"))?;

        // Create backgrounds subdirectory
        let backgrounds_dir = theme_dir.join("backgrounds");
        fs::create_dir_all(&backgrounds_dir)
            .map_err(|e| format!("Failed to create backgrounds directory: {e}"))?;

        let now = chrono::Utc::now().to_rfc3339();

        // Split payload into app data and optional metadata
        let (apps_value, author_update) = Self::split_theme_payload(&theme_data);
        let author = author_update.unwrap_or(None);

        // Extract colors from theme data
        let colors = self.extract_theme_colors(&theme_dir, &apps_value);

        // Create theme metadata
        let theme = CustomTheme {
            name: name.clone(),
            created_at: now.clone(),
            modified_at: now,
            author,
            apps: apps_value.clone(),
            colors,
        };

        // Generate config files for each app using the generator registry
        for app_name in self.generator_registry.get_all_apps() {
            if let Some(generator) = self.generator_registry.get_generator(app_name) {
                let has_config = apps_value.get(app_name).is_some();
                match generator.generate_config(&apps_value) {
                    Ok(config_content) => {
                        let config_path = theme_dir.join(generator.get_file_name());
                        fs::write(&config_path, config_content)
                            .map_err(|e| format!("Failed to write {app_name} config: {e}"))?;
                        if !has_config {
                            log::warn!(
                                "No config data found for app '{app_name}' in new theme '{name}'"
                            );
                        }
                    },
                    Err(e) => {
                        log::warn!("Failed to generate {app_name} config for '{name}': {e}");
                    },
                }
            }
        }

        // Save theme metadata with theme name as filename
        let metadata_path = theme_dir.join(format!("{}.json", sanitized_name));
        let metadata_content = serde_json::to_string_pretty(&theme)
            .map_err(|e| format!("Failed to serialize theme metadata: {e}"))?;
        fs::write(&metadata_path, metadata_content)
            .map_err(|e| format!("Failed to write theme metadata: {e}"))?;

        log::info!(
            "Created custom theme '{}' in directory: {}",
            name,
            theme_dir.display()
        );

        Ok(theme)
    }

    /// Create a new custom theme (legacy method for backwards compatibility)
    pub fn create_theme(
        &self,
        name: String,
        background: String,
        foreground: String,
    ) -> Result<CustomTheme, String> {
        // Convert legacy parameters to new format
        let theme_data = serde_json::json!({
            "alacritty": {
                "colors": {
                    "primary": {
                        "background": background,
                        "foreground": foreground
                    }
                }
            }
        });

        self.create_theme_advanced(name, theme_data)
    }

    /// Update an existing theme with advanced multi-app support
    pub fn update_theme_advanced(
        &self,
        name: &str,
        theme_data: Value,
    ) -> Result<CustomTheme, String> {
        let sanitized_name = Self::sanitize_name(name);
        let theme_dir = self.themes_dir.join(&sanitized_name);

        if !theme_dir.exists() {
            return Err(format!("Theme '{name}' not found"));
        }

        // Load existing theme metadata
        let mut theme = self.load_theme_metadata(&sanitized_name)?;

        // Split payload into app updates and optional metadata updates
        let (apps_update, author_update) = Self::split_theme_payload(&theme_data);

        // Deep-merge incoming app data into existing apps so we don't wipe other apps
        let mut merged_apps = theme.apps.clone();
        let mut apps_changed = false;
        if !apps_update.is_null() {
            Self::deep_merge(&mut merged_apps, &apps_update);
            theme.apps = merged_apps;
            apps_changed = true;
        }

        // Update author metadata if provided
        let mut metadata_changed = apps_changed;
        if let Some(author_payload) = author_update {
            if theme.author != author_payload {
                theme.author = author_payload.clone();
                metadata_changed = true;
            }
        }

        // Refresh modified timestamp if anything changed
        if metadata_changed {
            theme.modified_at = chrono::Utc::now().to_rfc3339();
        }

        // Re-extract colors after update when apps data changed
        if apps_changed {
            theme.colors = self.extract_theme_colors(&theme_dir, &theme.apps);
        }

        // Regenerate config files for each app
        for app_name in self.generator_registry.get_all_apps() {
            if let Some(generator) = self.generator_registry.get_generator(app_name) {
                match generator.generate_config(&theme.apps) {
                    Ok(config_content) => {
                        let config_path = theme_dir.join(generator.get_file_name());
                        log::debug!("Writing {} config to {}", app_name, config_path.display());
                        fs::write(&config_path, config_content)
                            .map_err(|e| format!("Failed to write {app_name} config: {e}"))?;
                    },
                    Err(e) => {
                        log::warn!("Failed to generate {app_name} config: {e}");
                    },
                }
            }
        }

        // Update the metadata file
        let metadata_path = self.get_metadata_path(&theme_dir, &sanitized_name);
        let metadata_content = serde_json::to_string_pretty(&theme)
            .map_err(|e| format!("Failed to serialize theme metadata: {e}"))?;
        fs::write(&metadata_path, metadata_content)
            .map_err(|e| format!("Failed to write theme metadata: {e}"))?;

        log::info!("Updated custom theme '{name}'");

        Ok(theme)
    }

    /// Deep-merge JSON values: when both sides are objects, merge keys recursively.
    /// Otherwise, overwrite target with source.
    fn deep_merge(target: &mut Value, src: &Value) {
        use serde_json::Value::*;
        match (target, src) {
            (Object(t_map), Object(s_map)) => {
                for (k, v) in s_map {
                    match (t_map.get_mut(k), v) {
                        (Some(t_child), Object(_)) => {
                            Self::deep_merge(t_child, v);
                        },
                        (Some(t_child), _) => {
                            *t_child = v.clone();
                        },
                        (None, _) => {
                            t_map.insert(k.clone(), v.clone());
                        },
                    }
                }
            },
            (t, s) => {
                *t = s.clone();
            },
        }
    }

    /// Split incoming payload into app data and optional metadata updates (author)
    fn split_theme_payload(payload: &Value) -> (Value, Option<Option<String>>) {
        if let Some(obj) = payload.as_object() {
            let apps_value = if let Some(apps) = obj.get("apps") {
                apps.clone()
            } else {
                payload.clone()
            };

            let author_update = obj.get("metadata").and_then(|metadata| {
                if let Some(author_value) = metadata.get("author") {
                    if let Some(author_str) = author_value.as_str() {
                        let trimmed = author_str.trim();
                        if trimmed.is_empty() {
                            Some(None)
                        } else {
                            Some(Some(trimmed.to_string()))
                        }
                    } else {
                        Some(None)
                    }
                } else {
                    None
                }
            });

            (apps_value, author_update)
        } else {
            (payload.clone(), None)
        }
    }

    /// Update an existing theme (legacy method for backwards compatibility)
    pub fn update_theme(
        &self,
        name: &str,
        alacritty_config: AlacrittyConfig,
    ) -> Result<CustomTheme, String> {
        // Convert legacy config to new format
        let theme_data = serde_json::json!({
            "alacritty": {
                "colors": {
                    "primary": {
                        "background": alacritty_config.colors.primary.background,
                        "foreground": alacritty_config.colors.primary.foreground,
                        "dim_foreground": alacritty_config.colors.primary.dim_foreground,
                    }
                }
            }
        });

        self.update_theme_advanced(name, theme_data)
    }

    /// Get available app schemas for the UI
    pub fn get_app_schemas(&self) -> Value {
        let mut schemas = serde_json::Map::new();

        for app_name in self.generator_registry.get_all_apps() {
            if let Some(schema) = self.generator_registry.get_schema_for_app(app_name) {
                schemas.insert(app_name.to_string(), schema);
            }
        }

        Value::Object(schemas)
    }

    /// Get a theme by name
    pub fn get_theme(&self, name: &str) -> Result<CustomTheme, String> {
        let sanitized_name = Self::sanitize_name(name);
        self.load_theme_metadata(&sanitized_name)
    }

    /// List all custom themes (only returns themes with our custom metadata file)
    pub fn list_themes(&self) -> Result<Vec<CustomTheme>, String> {
        let mut themes = Vec::new();

        let entries = fs::read_dir(&self.themes_dir)
            .map_err(|e| format!("Failed to read themes directory: {e}"))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Check for metadata file (either new or old format)
                    let metadata_path = self.get_metadata_path(&path, dir_name);
                    if metadata_path.exists() {
                        match self.load_theme_metadata(dir_name) {
                            Ok(theme) => themes.push(theme),
                            Err(e) => {
                                log::warn!("Failed to load custom theme '{dir_name}': {e}")
                            },
                        }
                    }
                }
            }
        }

        Ok(themes)
    }

    /// Delete a theme
    pub fn delete_theme(&self, name: &str) -> Result<(), String> {
        let sanitized_name = Self::sanitize_name(name);
        let theme_dir = self.themes_dir.join(&sanitized_name);

        if !theme_dir.exists() {
            return Err(format!("Theme '{name}' not found"));
        }

        fs::remove_dir_all(&theme_dir)
            .map_err(|e| format!("Failed to delete theme directory: {e}"))?;

        Ok(())
    }

    /// Initialize a new custom theme by copying template files
    pub fn init_theme(&self, name: String, description: String) -> Result<CustomTheme, String> {
        let sanitized_name = Self::sanitize_name(&name);
        let theme_dir = self.themes_dir.join(&sanitized_name);

        // Check if theme already exists
        if theme_dir.exists() {
            return Err(format!("Theme '{name}' already exists"));
        }

        // Create theme directory
        fs::create_dir_all(&theme_dir)
            .map_err(|e| format!("Failed to create theme directory: {e}"))?;

        // Copy template files
        self.copy_template_files(&theme_dir, &name, &description)?;

        // Load the created theme metadata (this will automatically extract colors)
        let theme = self.load_theme_metadata(&sanitized_name)?;

        log::info!(
            "Initialized custom theme '{}' in directory: {}",
            name,
            theme_dir.display()
        );

        Ok(theme)
    }

    /// Copy all template files to the new theme directory
    fn copy_template_files(
        &self,
        theme_dir: &Path,
        name: &str,
        description: &str,
    ) -> Result<(), String> {
        // Get template directory path from Tauri resources
        let resource_dir = self
            .app_handle
            .path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource directory: {e}"))?;

        // The resources are copied to target/debug/resources/ in development
        let template_dir = resource_dir.join("resources").join("template");

        if !template_dir.exists() {
            return Err(format!(
                "Template directory not found in resources at: {}",
                template_dir.display()
            ));
        }

        self.copy_dir_recursive(&template_dir, theme_dir, name, description)?;

        Ok(())
    }

    /// Recursively copy directory contents and replace placeholders in custom_theme.json
    fn copy_dir_recursive(
        &self,
        src: &Path,
        dst: &Path,
        name: &str,
        description: &str,
    ) -> Result<(), String> {
        let entries =
            fs::read_dir(src).map_err(|e| format!("Failed to read template directory: {e}"))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                // Create directory and copy contents recursively
                fs::create_dir_all(&dst_path)
                    .map_err(|e| format!("Failed to create directory: {e}"))?;
                self.copy_dir_recursive(&src_path, &dst_path, name, description)?;
            } else {
                // Check if this is the custom_theme.json template
                if entry.file_name() == "custom_theme.json" {
                    self.copy_and_process_metadata_template(
                        &src_path,
                        &dst_path,
                        name,
                        description,
                    )?;
                } else {
                    // Copy file normally
                    fs::copy(&src_path, &dst_path).map_err(|e| {
                        format!("Failed to copy file {}: {}", src_path.display(), e)
                    })?;
                }
            }
        }

        Ok(())
    }

    /// Copy and process the custom_theme.json template with placeholder replacement
    fn copy_and_process_metadata_template(
        &self,
        src: &Path,
        dst: &Path,
        name: &str,
        _description: &str,
    ) -> Result<(), String> {
        let template_content = fs::read_to_string(src)
            .map_err(|e| format!("Failed to read metadata template: {e}"))?;

        let now = chrono::Utc::now().to_rfc3339();

        // Replace placeholders (no description)
        let processed_content = template_content
            .replace("{{THEME_NAME}}", name)
            .replace("{{CREATED_AT}}", &now)
            .replace("{{MODIFIED_AT}}", &now)
            .replace("{{AUTHOR}}", "");

        fs::write(dst, processed_content)
            .map_err(|e| format!("Failed to write processed metadata: {e}"))?;

        Ok(())
    }

    /// Get the metadata file path for a theme (with backward compatibility)
    fn get_metadata_path(
        &self,
        theme_dir: &std::path::Path,
        sanitized_name: &str,
    ) -> std::path::PathBuf {
        // New format: {theme-name}.json
        let new_path = theme_dir.join(format!("{}.json", sanitized_name));

        // Backward compatibility: check for old custom_theme.json
        let old_path = theme_dir.join("custom_theme.json");

        if new_path.exists() {
            new_path
        } else if old_path.exists() {
            old_path
        } else {
            // Default to new format for new themes
            new_path
        }
    }

    /// Load theme metadata from JSON file
    fn load_theme_metadata(&self, sanitized_name: &str) -> Result<CustomTheme, String> {
        let theme_dir = self.themes_dir.join(sanitized_name);
        let metadata_path = self.get_metadata_path(&theme_dir, sanitized_name);

        let content = fs::read_to_string(&metadata_path)
            .map_err(|e| format!("Failed to read theme metadata: {e}"))?;

        let mut theme: CustomTheme = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse theme metadata: {e}"))?;

        // If colors are missing (backwards compatibility), extract them now
        if theme.colors.is_none() {
            theme.colors = self.extract_theme_colors(&theme_dir, &theme.apps);

            // Save the updated metadata with colors (using the same path we read from)
            if let Ok(updated_content) = serde_json::to_string_pretty(&theme) {
                if let Err(e) = fs::write(&metadata_path, updated_content) {
                    log::warn!("Failed to update theme metadata with colors: {e}");
                }
            }
        }

        Ok(theme)
    }

    /// Update author metadata for a theme
    pub fn set_theme_author(
        &self,
        name: &str,
        author: Option<String>,
    ) -> Result<CustomTheme, String> {
        let sanitized_name = Self::sanitize_name(name);
        let theme_dir = self.themes_dir.join(&sanitized_name);

        if !theme_dir.exists() {
            return Err(format!("Theme '{name}' not found"));
        }

        let mut theme = self.load_theme_metadata(&sanitized_name)?;

        let normalized_author = author.and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        if theme.author == normalized_author {
            return Ok(theme);
        }

        theme.author = normalized_author;
        theme.modified_at = chrono::Utc::now().to_rfc3339();

        let metadata_path = self.get_metadata_path(&theme_dir, &sanitized_name);
        let metadata_content = serde_json::to_string_pretty(&theme)
            .map_err(|e| format!("Failed to serialize theme metadata: {e}"))?;
        fs::write(&metadata_path, metadata_content)
            .map_err(|e| format!("Failed to write theme metadata: {e}"))?;

        Ok(theme)
    }
    /// Get list of background images for a theme
    pub fn get_theme_backgrounds(&self, theme_name: &str) -> Result<Vec<String>, String> {
        let sanitized_name = Self::sanitize_name(theme_name);
        let theme_dir = self.themes_dir.join(&sanitized_name);
        let backgrounds_dir = theme_dir.join("backgrounds");

        if !backgrounds_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backgrounds = Vec::new();
        let entries = fs::read_dir(&backgrounds_dir)
            .map_err(|e| format!("Failed to read backgrounds directory: {e}"))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    if matches!(
                        ext.as_str(),
                        "jpg" | "jpeg" | "png" | "webp" | "bmp" | "gif"
                    ) {
                        if let Some(filename) = path.file_name() {
                            backgrounds.push(filename.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        backgrounds.sort();
        Ok(backgrounds)
    }

    /// Add background images to a theme by copying files
    pub fn add_theme_backgrounds(
        &self,
        theme_name: &str,
        source_paths: Vec<String>,
    ) -> Result<Vec<String>, String> {
        let sanitized_name = Self::sanitize_name(theme_name);
        let theme_dir = self.themes_dir.join(&sanitized_name);

        if !theme_dir.exists() {
            return Err(format!("Theme '{theme_name}' not found"));
        }

        let backgrounds_dir = theme_dir.join("backgrounds");

        // Create backgrounds directory if it doesn't exist
        fs::create_dir_all(&backgrounds_dir)
            .map_err(|e| format!("Failed to create backgrounds directory: {e}"))?;

        let mut copied_files = Vec::new();

        for source_path in source_paths {
            let source = Path::new(&source_path);

            if !source.exists() {
                log::warn!("Source file does not exist: {source_path}");
                continue;
            }

            if !source.is_file() {
                log::warn!("Source path is not a file: {source_path}");
                continue;
            }

            // Validate file extension
            if let Some(extension) = source.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if !matches!(
                    ext.as_str(),
                    "jpg" | "jpeg" | "png" | "webp" | "bmp" | "gif"
                ) {
                    log::warn!("Unsupported image format: {source_path}");
                    continue;
                }
            } else {
                log::warn!("File has no extension: {source_path}");
                continue;
            }

            // Get filename and create destination path
            if let Some(filename) = source.file_name() {
                let dest_path = backgrounds_dir.join(filename);

                // Copy the file
                match fs::copy(source, &dest_path) {
                    Ok(_) => {
                        copied_files.push(filename.to_string_lossy().to_string());
                        log::debug!(
                            "Copied background image: {} -> {}",
                            source_path,
                            dest_path.display()
                        );
                    },
                    Err(e) => {
                        log::warn!("Failed to copy {source_path}: {e}");
                    },
                }
            }
        }

        Ok(copied_files)
    }

    /// Remove a background image from a theme
    pub fn remove_theme_background(&self, theme_name: &str, filename: &str) -> Result<(), String> {
        let sanitized_name = Self::sanitize_name(theme_name);
        let theme_dir = self.themes_dir.join(&sanitized_name);
        let backgrounds_dir = theme_dir.join("backgrounds");
        let file_path = backgrounds_dir.join(filename);

        if !file_path.exists() {
            return Err(format!("Background image '{filename}' not found"));
        }

        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to remove background image: {e}"))?;

        log::debug!("Removed background image: {}", file_path.display());
        Ok(())
    }

    /// Get background image data as base64 for preview
    pub fn get_background_image_data(
        &self,
        theme_name: &str,
        filename: &str,
    ) -> Result<String, String> {
        let sanitized_name = Self::sanitize_name(theme_name);
        let theme_dir = self.themes_dir.join(&sanitized_name);
        let backgrounds_dir = theme_dir.join("backgrounds");
        let file_path = backgrounds_dir.join(filename);

        if !file_path.exists() {
            return Err(format!("Background image '{filename}' not found"));
        }

        // Read the file and convert to base64
        let image_data =
            fs::read(&file_path).map_err(|e| format!("Failed to read background image: {e}"))?;

        // Determine MIME type based on file extension
        let mime_type = match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("webp") => "image/webp",
            Some("bmp") => "image/bmp",
            Some("gif") => "image/gif",
            _ => "image/jpeg", // default fallback
        };

        // Encode as base64 data URL using our optimized implementation
        let base64_data = Self::base64_encode(&image_data);
        Ok(format!("data:{mime_type};base64,{base64_data}"))
    }

    /// Optimized base64 encoding function
    fn base64_encode(data: &[u8]) -> String {
        if data.is_empty() {
            return String::new();
        }

        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        // Pre-allocate with exact capacity to avoid reallocations
        let output_len = data.len().div_ceil(3) * 4;
        let mut result = String::with_capacity(output_len);

        for chunk in data.chunks(3) {
            let mut buf = [0u8; 3];
            for (i, &byte) in chunk.iter().enumerate() {
                buf[i] = byte;
            }

            let b = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);

            result.push(CHARS[((b >> 18) & 63) as usize] as char);
            result.push(CHARS[((b >> 12) & 63) as usize] as char);
            result.push(if chunk.len() > 1 {
                CHARS[((b >> 6) & 63) as usize] as char
            } else {
                '='
            });
            result.push(if chunk.len() > 2 {
                CHARS[(b & 63) as usize] as char
            } else {
                '='
            });
        }

        result
    }
}

// Tauri commands
#[tauri::command]
pub async fn create_custom_theme(
    app_handle: AppHandle,
    name: String,
    background: String,
    foreground: String,
) -> Result<CustomTheme, String> {
    log::info!("Creating custom theme '{name}' with colors: bg={background}, fg={foreground}");
    let service = CustomThemeService::new(&app_handle)?;
    let result = service.create_theme(name.clone(), background, foreground);
    let sanitized_name = CustomThemeService::sanitize_name(&name);

    // Invalidate cache for the created theme
    if result.is_ok() {
        if let Ok(cache) = crate::services::cache::cache_manager::get_theme_cache().await {
            cache.invalidate_theme(&sanitized_name).await;
            // Trigger background refresh to include the new theme
            let _ = cache.trigger_background_refresh().await;
        }
    }

    result
}

#[tauri::command]
pub async fn create_custom_theme_advanced(
    app_handle: AppHandle,
    name: String,
    theme_data: Value,
) -> Result<CustomTheme, String> {
    log::info!("Creating advanced custom theme '{name}'");
    let service = CustomThemeService::new(&app_handle)?;
    let result = service.create_theme_advanced(name.clone(), theme_data);
    let sanitized_name = CustomThemeService::sanitize_name(&name);

    // Invalidate cache for the created theme
    if result.is_ok() {
        if let Ok(cache) = crate::services::cache::cache_manager::get_theme_cache().await {
            cache.invalidate_theme(&sanitized_name).await;
            // Trigger background refresh to include the new theme
            let _ = cache.trigger_background_refresh().await;
        }
    }

    result
}

#[tauri::command]
pub async fn update_custom_theme(
    app_handle: AppHandle,
    name: String,
    background: String,
    foreground: String,
    dim_foreground: String,
) -> Result<CustomTheme, String> {
    let service = CustomThemeService::new(&app_handle)?;

    let alacritty_config = AlacrittyConfig {
        colors: AlacrittyColors {
            primary: AlacrittyPrimaryColors {
                background,
                foreground,
                dim_foreground,
            },
        },
    };

    let result = service.update_theme(&name, alacritty_config);
    let sanitized_name = CustomThemeService::sanitize_name(&name);

    // Invalidate cache for the updated theme
    if result.is_ok() {
        if let Ok(cache) = crate::services::cache::cache_manager::get_theme_cache().await {
            cache.invalidate_theme(&sanitized_name).await;
            // Trigger background refresh to update the theme
            let _ = cache.trigger_background_refresh().await;
        }
    }

    result
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_custom_theme_advanced(
    app_handle: AppHandle,
    name: String,
    theme_data: Value,
) -> Result<CustomTheme, String> {
    let service = CustomThemeService::new(&app_handle)?;
    let result = service.update_theme_advanced(&name, theme_data);
    let sanitized_name = CustomThemeService::sanitize_name(&name);

    // Invalidate cache for the updated theme
    if result.is_ok() {
        if let Ok(cache) = crate::services::cache::cache_manager::get_theme_cache().await {
            cache.invalidate_theme(&sanitized_name).await;
            // Trigger background refresh to update the theme
            let _ = cache.trigger_background_refresh().await;
        }
    }

    result
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_theme_author(
    app_handle: AppHandle,
    theme_name: String,
    author: Option<String>,
) -> Result<CustomTheme, String> {
    let service = CustomThemeService::new(&app_handle)?;
    let result = service.set_theme_author(&theme_name, author);
    let sanitized_name = CustomThemeService::sanitize_name(&theme_name);

    if result.is_ok() {
        if let Ok(cache) = crate::services::cache::cache_manager::get_theme_cache().await {
            cache.invalidate_theme(&sanitized_name).await;
            let _ = cache.trigger_background_refresh().await;
        }
    }

    result
}

#[tauri::command]
pub async fn get_custom_theme(app_handle: AppHandle, name: String) -> Result<CustomTheme, String> {
    let service = CustomThemeService::new(&app_handle)?;
    service.get_theme(&name)
}

#[tauri::command]
pub async fn list_custom_themes(app_handle: AppHandle) -> Result<Vec<CustomTheme>, String> {
    let service = CustomThemeService::new(&app_handle)?;
    service.list_themes()
}

#[tauri::command]
pub async fn delete_custom_theme(app_handle: AppHandle, name: String) -> Result<(), String> {
    let service = CustomThemeService::new(&app_handle)?;
    let result = service.delete_theme(&name);
    let sanitized_name = CustomThemeService::sanitize_name(&name);

    // Invalidate cache for the deleted theme
    if result.is_ok() {
        if let Ok(cache) = crate::services::cache::cache_manager::get_theme_cache().await {
            cache.invalidate_theme(&sanitized_name).await;
            // Trigger background refresh to remove the theme from cache
            let _ = cache.trigger_background_refresh().await;
        }
    }

    result
}

#[tauri::command]
pub async fn init_custom_theme(app_handle: AppHandle, name: String) -> Result<CustomTheme, String> {
    log::info!("Initializing custom theme '{name}'");
    let service = CustomThemeService::new(&app_handle)?;
    service.init_theme(name, String::new())
}

#[tauri::command]
pub async fn get_app_schemas(app_handle: AppHandle) -> Result<Value, String> {
    let service = CustomThemeService::new(&app_handle)?;
    Ok(service.get_app_schemas())
}

#[tauri::command]
pub async fn get_theme_backgrounds(
    app_handle: AppHandle,
    theme_name: String,
) -> Result<Vec<String>, String> {
    let service = CustomThemeService::new(&app_handle)?;
    service.get_theme_backgrounds(&theme_name)
}

#[tauri::command]
pub async fn add_theme_backgrounds(
    app_handle: AppHandle,
    theme_name: String,
    source_paths: Vec<String>,
) -> Result<Vec<String>, String> {
    let service = CustomThemeService::new(&app_handle)?;
    service.add_theme_backgrounds(&theme_name, source_paths)
}

#[tauri::command]
pub async fn remove_theme_background(
    app_handle: AppHandle,
    theme_name: String,
    filename: String,
) -> Result<(), String> {
    let service = CustomThemeService::new(&app_handle)?;
    service.remove_theme_background(&theme_name, &filename)
}

#[tauri::command]
pub async fn get_background_image_data(
    app_handle: AppHandle,
    theme_name: String,
    filename: String,
) -> Result<String, String> {
    let service = CustomThemeService::new(&app_handle)?;
    service.get_background_image_data(&theme_name, &filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(
            CustomThemeService::sanitize_name("My Cool Theme"),
            "my-cool-theme"
        );
        assert_eq!(
            CustomThemeService::sanitize_name("Test_Theme-123"),
            "test_theme-123"
        );
        assert_eq!(
            CustomThemeService::sanitize_name("Special@#$%Theme"),
            "specialtheme"
        );
    }

    #[test]
    fn test_theme_creation() {
        // Skip this test since it requires a real AppHandle
        // which is not available in unit tests
    }
}
