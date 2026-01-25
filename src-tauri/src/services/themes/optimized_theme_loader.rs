use super::color_extraction::ColorExtractor;
use super::get_sys_themes::SysTheme;
use crate::types::ThemeColors;
use dirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeOrigin {
    System,
    User,
}

/// Lightweight theme metadata for faster initial responses
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeMetadata {
    pub dir: String,
    pub title: String,
    pub is_system: bool,
    pub is_custom: bool,
    pub has_colors: bool,
    pub has_image: bool,
}

/// Color extraction cache to avoid recomputation
#[derive(Debug, Clone)]
pub struct ColorCache {
    cache: Arc<RwLock<HashMap<String, Option<ThemeColors>>>>,
}

impl Default for ColorCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get cached colors for a theme directory
    pub async fn get(&self, theme_dir: &str) -> Option<Option<ThemeColors>> {
        let cache = self.cache.read().await;
        cache.get(theme_dir).cloned()
    }

    /// Cache colors for a theme directory
    pub async fn set(&self, theme_dir: String, colors: Option<ThemeColors>) {
        let mut cache = self.cache.write().await;
        cache.insert(theme_dir, colors);
    }

    /// Clear the cache
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache size
    pub async fn size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }
}

/// Optimized theme loader with parallel processing and caching
pub struct OptimizedThemeLoader {
    color_cache: ColorCache,
}

impl OptimizedThemeLoader {
    pub fn new() -> Self {
        Self {
            color_cache: ColorCache::new(),
        }
    }

    /// User themes are stored in ~/.config/omarchy/themes
    fn user_themes_dir() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
            .ok_or_else(|| "Failed to determine config directory".to_string())?;
        Ok(config_dir.join("omarchy").join("themes"))
    }

    /// System themes are stored in ~/.local/share/omarchy/themes
    fn system_themes_dir() -> Result<PathBuf, String> {
        let data_dir = dirs::data_dir()
            .or_else(|| dirs::home_dir().map(|home| home.join(".local").join("share")))
            .ok_or_else(|| "Failed to determine data directory".to_string())?;
        Ok(data_dir.join("omarchy").join("themes"))
    }

    /// Locate a metadata file for the given theme directory supporting both new and legacy formats
    fn find_metadata_file(theme_dir: &Path) -> Option<PathBuf> {
        let dir_name = theme_dir.file_name()?.to_str()?;

        let candidates = [
            theme_dir.join(format!("{dir_name}.json")),
            theme_dir.join("custom_theme.json"),
        ];

        for candidate in candidates.iter() {
            if Self::is_valid_metadata_file(candidate) {
                return Some(candidate.clone());
            }
        }

        if let Ok(entries) = fs::read_dir(theme_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("json"))
                    .unwrap_or(false)
                    && Self::is_valid_metadata_file(&path)
                {
                    return Some(path);
                }
            }
        }

        None
    }

    /// Check whether the supplied path points to a metadata file that matches the custom theme schema
    fn is_valid_metadata_file(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        match fs::read_to_string(path) {
            Ok(content) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let has_modern_fields = json.get("name").and_then(|v| v.as_str()).is_some()
                        && json.get("created_at").and_then(|v| v.as_str()).is_some()
                        && json.get("modified_at").and_then(|v| v.as_str()).is_some()
                        && json.get("apps").is_some();

                    if has_modern_fields {
                        return true;
                    }

                    return Self::is_legacy_metadata_file(path, &json);
                }
                false
            },
            Err(_) => false,
        }
    }

    /// Legacy custom themes stored minimal metadata inside `custom_theme.json`
    fn is_legacy_metadata_file(path: &Path, json: &serde_json::Value) -> bool {
        if path.file_name().and_then(|f| f.to_str()) != Some("custom_theme.json") {
            return false;
        }

        json.as_object().is_some()
            && (json.get("apps").is_some() || json.get("alacritty").is_some())
    }

    /// Optimized helper function to convert directory name to title
    fn dir_name_to_title(dir_name: &str) -> String {
        let mut title = String::with_capacity(dir_name.len() + 10);
        let mut capitalize_next = true;

        for ch in dir_name.chars() {
            match ch {
                '-' | '_' => {
                    title.push(' ');
                    capitalize_next = true;
                },
                c if capitalize_next => {
                    title.extend(c.to_uppercase());
                    capitalize_next = false;
                },
                c => {
                    title.push(c);
                },
            }
        }
        title
    }

    /// Load themes with parallel processing for better performance
    pub async fn load_themes_parallel(&self) -> Result<Vec<SysTheme>, String> {
        // Collect themes from both origins.
        // User themes override system themes when directory names collide.
        let system_root = Self::system_themes_dir()?;
        let user_root = Self::user_themes_dir()?;

        let theme_entries = self.collect_theme_paths_with_origin(&system_root, &user_root)?;
        if theme_entries.is_empty() {
            return Ok(Vec::new());
        }

        let theme_count = theme_entries.len();

        log::info!(
            "Loading {} themes with parallel processing",
            theme_count
        );

        // Process themes in parallel using tokio::spawn
        let mut handles: Vec<JoinHandle<Result<SysTheme, String>>> = Vec::new();

        for (path, origin) in theme_entries {
            let color_cache = self.color_cache.clone();
            let handle = tokio::spawn(async move {
                Self::generate_theme_from_directory_async(&path, origin, color_cache).await
            });
            handles.push(handle);
        }

        // Collect results from all parallel tasks
        let mut themes = Vec::new();
        let mut errors = Vec::new();

        for handle in handles {
            match handle.await {
                Ok(Ok(theme)) => themes.push(theme),
                Ok(Err(e)) => errors.push(e),
                Err(e) => errors.push(format!("Task join error: {e}")),
            }
        }

        // Log any errors but continue with successful themes
        if !errors.is_empty() {
            log::warn!(
                "Encountered {} errors during parallel theme loading: {:?}",
                errors.len(),
                errors
            );
        }

        log::info!("Successfully loaded {} themes in parallel", themes.len());
        Ok(themes)
    }

    /// Load only theme metadata for faster initial responses
    pub async fn load_theme_metadata_only(&self) -> Result<Vec<ThemeMetadata>, String> {
        let system_root = Self::system_themes_dir()?;
        let user_root = Self::user_themes_dir()?;

        let theme_entries = self.collect_theme_paths_with_origin(&system_root, &user_root)?;
        if theme_entries.is_empty() {
            return Ok(Vec::new());
        }

        let theme_count = theme_entries.len();

        log::info!("Loading metadata for {} themes", theme_count);

        // Process metadata in parallel
        let mut handles: Vec<JoinHandle<Result<ThemeMetadata, String>>> = Vec::new();

        for (path, origin) in theme_entries {
            let handle = tokio::spawn(async move {
                Self::generate_theme_metadata(&path, origin).await
            });
            handles.push(handle);
        }

        // Collect metadata results
        let mut metadata = Vec::new();
        let mut errors = Vec::new();

        for handle in handles {
            match handle.await {
                Ok(Ok(meta)) => metadata.push(meta),
                Ok(Err(e)) => errors.push(e),
                Err(e) => errors.push(format!("Metadata task join error: {e}")),
            }
        }

        if !errors.is_empty() {
            log::warn!(
                "Encountered {} errors during metadata loading: {:?}",
                errors.len(),
                errors
            );
        }

        log::info!("Successfully loaded metadata for {} themes", metadata.len());
        Ok(metadata)
    }

    /// Collect all theme directory paths from both system and user roots.
    /// User themes override system themes on name collisions.
    fn collect_theme_paths_with_origin(
        &self,
        system_root: &Path,
        user_root: &Path,
    ) -> Result<Vec<(PathBuf, ThemeOrigin)>, String> {
        let mut by_dir: HashMap<String, (PathBuf, ThemeOrigin)> = HashMap::new();

        // System themes first
        if system_root.exists() {
            let entries = fs::read_dir(system_root)
                .map_err(|e| format!("Failed to read system themes directory: {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        by_dir.insert(dir_name.to_string(), (path, ThemeOrigin::System));
                    }
                }
            }
        }

        // User themes override system themes
        if user_root.exists() {
            let entries = fs::read_dir(user_root)
                .map_err(|e| format!("Failed to read user themes directory: {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        by_dir.insert(dir_name.to_string(), (path, ThemeOrigin::User));
                    }
                }
            }
        }

        let mut entries: Vec<(String, (PathBuf, ThemeOrigin))> = by_dir.into_iter().collect();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        Ok(entries.into_iter().map(|(_, v)| v).collect())
    }

    /// Generate theme metadata only (lightweight operation)
    async fn generate_theme_metadata(
        theme_dir: &Path,
        origin: ThemeOrigin,
    ) -> Result<ThemeMetadata, String> {
        let dir_name = theme_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "Invalid directory name".to_string())?;

        // Convert directory name to a nice title (optimized)
        let title = Self::dir_name_to_title(dir_name);

        let metadata_path = Self::find_metadata_file(theme_dir);
        let is_custom = origin == ThemeOrigin::User;
        let is_system = origin == ThemeOrigin::System;

        // Check if theme has color configuration files
        let has_colors = metadata_path.is_some() || theme_dir.join("alacritty.toml").exists();

        // Check if theme has image files
        let has_image = Self::has_image_files(theme_dir);

        Ok(ThemeMetadata {
            dir: dir_name.to_string(),
            title,
            is_system,
            is_custom,
            has_colors,
            has_image,
        })
    }

    /// Check if directory contains image files
    fn has_image_files(theme_dir: &Path) -> bool {
        if let Ok(entries) = fs::read_dir(theme_dir) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.is_file() {
                    if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
                        let ext_lower = extension.to_lowercase();
                        if matches!(
                            ext_lower.as_str(),
                            "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg"
                        ) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Generate full theme from directory with async color extraction and caching
    async fn generate_theme_from_directory_async(
        theme_dir: &Path,
        origin: ThemeOrigin,
        color_cache: ColorCache,
    ) -> Result<SysTheme, String> {
        let dir_name = theme_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "Invalid directory name".to_string())?;

        // Convert directory name to a nice title (optimized)
        let title = Self::dir_name_to_title(dir_name);

        let metadata_path = Self::find_metadata_file(theme_dir);
        let is_custom = origin == ThemeOrigin::User;
        let is_system = origin == ThemeOrigin::System;

        // Extract colors with caching
        let colors =
            Self::extract_theme_colors_cached(theme_dir, metadata_path.clone(), &color_cache).await;

        // Load image asynchronously
        let image_path = Self::load_theme_image_async(theme_dir).await;

        Ok(SysTheme {
            dir: dir_name.to_string(),
            title,
            description: format!("Auto-generated theme from {dir_name}"),
            image: image_path,
            is_system,
            is_custom,
            colors,
        })
    }

    /// Extract theme colors with caching to avoid recomputation
    async fn extract_theme_colors_cached(
        theme_dir: &Path,
        metadata_path: Option<PathBuf>,
        color_cache: &ColorCache,
    ) -> Option<ThemeColors> {
        let dir_name = theme_dir.file_name()?.to_str()?.to_string();

        // Check cache first
        if let Some(cached_colors) = color_cache.get(&dir_name).await {
            log::debug!("Using cached colors for theme: {dir_name}");
            return cached_colors;
        }

        // Extract colors if not cached
        let colors = Self::extract_theme_colors_direct(theme_dir, metadata_path.as_deref());

        // Cache the result (even if None)
        color_cache.set(dir_name.clone(), colors.clone()).await;
        log::debug!("Cached colors for theme: {dir_name}");

        colors
    }

    /// Direct color extraction (moved from original implementation)
    fn extract_theme_colors_direct(
        theme_dir: &Path,
        metadata_path: Option<&Path>,
    ) -> Option<ThemeColors> {
        if let Some(custom_theme_path) = metadata_path {
            match fs::read_to_string(custom_theme_path) {
                Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(theme_data) => {
                        if let Some(colors) = ColorExtractor::extract_from_custom_theme(&theme_data)
                        {
                            return Some(colors);
                        } else {
                            log::warn!(
                                "Failed to extract colors from custom theme at {}",
                                custom_theme_path.display()
                            );
                        }
                    },
                    Err(e) => {
                        log::warn!(
                            "Failed to parse custom theme JSON at {}: {e}",
                            custom_theme_path.display()
                        );
                    },
                },
                Err(e) => {
                    log::warn!(
                        "Failed to read custom theme file at {}: {e}",
                        custom_theme_path.display()
                    );
                },
            }
        }

        // For system themes or fallback, try to extract from alacritty.toml
        let alacritty_config_path = theme_dir.join("alacritty.toml");
        if alacritty_config_path.exists() {
            if let Some(colors) =
                ColorExtractor::extract_from_alacritty_config(&alacritty_config_path)
            {
                return Some(colors);
            }
        }

        None
    }

    /// Load theme image asynchronously
    async fn load_theme_image_async(theme_dir: &Path) -> String {
        // This is I/O bound, so we can spawn it as a blocking task
        let theme_dir_path = theme_dir.to_path_buf();
        let theme_dir_display = theme_dir.display().to_string();

        match tokio::task::spawn_blocking(move || Self::find_and_convert_image(&theme_dir_path))
            .await
        {
            Ok(Ok(image_path)) => image_path,
            Ok(Err(e)) => {
                log::warn!("Failed to load image for theme {theme_dir_display}: {e}");
                String::new()
            },
            Err(e) => {
                log::warn!("Image loading task failed for theme {theme_dir_display}: {e}");
                String::new()
            },
        }
    }

    /// Find and convert image to data URL (blocking operation)
    fn find_and_convert_image(theme_dir: &Path) -> Result<String, String> {
        if let Ok(entries) = fs::read_dir(theme_dir) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.is_file() {
                    if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
                        let ext_lower = extension.to_lowercase();
                        if matches!(
                            ext_lower.as_str(),
                            "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg"
                        ) {
                            return Self::convert_image_to_data_url(&file_path);
                        }
                    }
                }
            }
        }
        Ok(String::new())
    }

    /// Convert a local image file to a base64 data URL
    fn convert_image_to_data_url(image_path: &Path) -> Result<String, String> {
        if !image_path.exists() {
            return Err(format!("Image file does not exist: {image_path:?}"));
        }

        let image_data =
            fs::read(image_path).map_err(|e| format!("Failed to read image file: {e}"))?;

        // Determine MIME type based on file extension
        let mime_type = match image_path.extension().and_then(|ext| ext.to_str()) {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            Some("svg") => "image/svg+xml",
            _ => "image/png", // Default to PNG
        };

        let base64_data = Self::base64_encode(&image_data);
        Ok(format!("data:{mime_type};base64,{base64_data}"))
    }

    /// Optimized base64 encoding function with pre-allocated capacity
    fn base64_encode(data: &[u8]) -> String {
        if data.is_empty() {
            return String::new();
        }

        const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

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

    /// Clear the color cache
    pub async fn clear_cache(&self) {
        self.color_cache.clear().await;
        log::info!("Color extraction cache cleared");
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize,) {
        let size = self.color_cache.size().await;
        (size,)
    }
}

impl Default for OptimizedThemeLoader {
    fn default() -> Self {
        Self::new()
    }
}
