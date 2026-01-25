use super::color_extraction::ColorExtractor;
use super::optimized_theme_loader::{OptimizedThemeLoader, ThemeMetadata};
use crate::services::cache::cache_manager::get_theme_cache;
use crate::types::ThemeColors;
use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SysTheme {
    pub dir: String,   // Directory name for the theme
    pub title: String, // Display name for the theme
    pub description: String,
    pub image: String,
    pub is_system: bool,             // Indicates if this is a system theme
    pub is_custom: bool,             // Indicates if this is a custom theme
    pub colors: Option<ThemeColors>, // Extracted color palette from theme configuration
}

/// Global instance of the optimized theme loader
static THEME_LOADER: OnceLock<OptimizedThemeLoader> = OnceLock::new();

/// Get or initialize the global theme loader instance
fn get_theme_loader() -> &'static OptimizedThemeLoader {
    THEME_LOADER.get_or_init(OptimizedThemeLoader::new)
}

/// Scans the system themes directory and returns a list of themes with their info
/// Includes color extraction for each discovered theme directory with performance optimizations
/// This function now uses cache-first strategy with fallback to direct filesystem scanning
#[tauri::command]
pub async fn get_sys_themes() -> Result<Vec<SysTheme>, String> {
    // Try cache first if available
    if let Ok(cache) = get_theme_cache().await {
        if cache.is_cache_valid().await && !cache.is_empty().await {
            if let Ok(cached_themes) = cache.get_themes().await {
                log::info!(
                    "Returning {} themes from cache (get_sys_themes)",
                    cached_themes.len()
                );
                return Ok(cached_themes);
            }
        }
    }

    // Cache miss or invalid, proceed with direct filesystem scan
    get_sys_themes_direct().await
}

/// Direct filesystem scan for themes (bypasses cache)
/// Now uses optimized parallel processing for better performance
async fn get_sys_themes_direct() -> Result<Vec<SysTheme>, String> {
    log::info!("Performing optimized parallel filesystem scan for themes");

    let theme_loader = get_theme_loader();

    // Use the optimized parallel theme loading
    let themes = theme_loader.load_themes_parallel().await?;

    log::info!("Optimized parallel scan found {} themes", themes.len());

    // Try to cache the results if cache is available
    if let Ok(cache) = get_theme_cache().await {
        if let Err(e) = cache.cache_themes(themes.clone(), false).await {
            log::warn!("Failed to cache themes after parallel scan: {e}");
        } else {
            log::info!(
                "Successfully cached {} themes after parallel scan",
                themes.len()
            );
        }
    }

    Ok(themes)
}

/// Check if a theme directory contains a custom theme metadata file
/// Supports both old (custom_theme.json) and new ({theme-name}.json) formats
fn is_custom_theme(theme_dir: &Path) -> bool {
    // Check for old format first
    if theme_dir.join("custom_theme.json").is_file() {
        return true;
    }

    // Check for new format: look for JSON file with custom theme structure
    // Custom theme metadata has "name", "created_at", "modified_at", "apps" fields
    // App config files (vscode.json, kitty.json) don't have this structure
    if let Ok(entries) = fs::read_dir(theme_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "json" {
                        // Skip known app config files
                        if let Some(filename) = path.file_name() {
                            let filename_str = filename.to_string_lossy();
                            if filename_str == "vscode.json"
                                || filename_str == "kitty.json"
                                || filename_str == "ghostty.json"
                                || filename_str == "alacritty.json"
                                || filename_str == "chromium.json"
                                || filename_str == "waybar.json"
                            {
                                continue;
                            }
                        }

                        // Try to parse and check for custom theme structure
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                // Check if it has the custom theme structure
                                if json.get("name").is_some()
                                    && json.get("created_at").is_some()
                                    && json.get("apps").is_some()
                                {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

/// Extract colors from theme configuration files with comprehensive error handling
/// Returns None if no extractable colors are found, allowing graceful degradation
fn extract_theme_colors(theme_dir: &Path, is_custom: bool) -> Option<ThemeColors> {
    // Performance optimization: Check file existence before attempting to read
    if is_custom {
        // For custom themes, try to extract from metadata JSON file
        // Check old format first
        let mut custom_theme_path = theme_dir.join("custom_theme.json");

        // If old format doesn't exist, look for new format ({theme-name}.json)
        if !custom_theme_path.exists() {
            if let Ok(entries) = fs::read_dir(theme_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "json" {
                                custom_theme_path = path;
                                break;
                            }
                        }
                    }
                }
            }
        }

        if custom_theme_path.exists() {
            match fs::read_to_string(&custom_theme_path) {
                Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(theme_data) => {
                        if let Some(colors) = ColorExtractor::extract_from_custom_theme(&theme_data)
                        {
                            return Some(colors);
                        } else {
                            log::warn!(
                                "Failed to extract colors from custom theme at {custom_theme_path:?}"
                            );
                        }
                    },
                    Err(e) => {
                        log::warn!(
                            "Failed to parse custom theme JSON at {custom_theme_path:?}: {e}"
                        );
                    },
                },
                Err(e) => {
                    log::warn!("Failed to read custom theme file at {custom_theme_path:?}: {e}");
                },
            }
        }
    }

    // For system themes or fallback, try to extract from alacritty.toml
    let alacritty_config_path = theme_dir.join("alacritty.toml");
    if alacritty_config_path.exists() {
        match ColorExtractor::extract_from_alacritty_config(&alacritty_config_path) {
            Some(colors) => return Some(colors),
            _none => {
                log::warn!(
                    "Failed to extract colors from Alacritty config at {alacritty_config_path:?}"
                );
            },
        }
    }

    // No extractable colors found - this is handled gracefully by returning None
    None
}

/// Generate theme info from directory name
fn generate_theme_from_directory(theme_dir: &Path) -> Result<SysTheme, String> {
    let dir_name = theme_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Invalid directory name".to_string())?;

    // Convert directory name to a nice title (replace hyphens/underscores with spaces and capitalize)
    // Optimized version that avoids multiple string allocations
    let title = {
        let mut title = String::with_capacity(dir_name.len() + 10); // Pre-allocate with estimated capacity
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
    };

    // Look for any image files with common extensions
    let mut image_path = String::new();

    // Read all files in the theme directory and look for image files
    if let Ok(entries) = fs::read_dir(theme_dir) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            // Check if it's a file (not a directory) and has an image extension
            if file_path.is_file() {
                if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
                    let ext_lower = extension.to_lowercase();
                    if matches!(
                        ext_lower.as_str(),
                        "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg"
                    ) {
                        match convert_image_to_data_url(&file_path) {
                            Ok(data_url) => {
                                image_path = data_url;
                                break;
                            },
                            Err(e) => {
                                log::warn!("Failed to load image {file_path:?}: {e}");
                            },
                        }
                    }
                }
            }
        }
    }

    let is_custom = is_custom_theme(theme_dir);

    // Check if the theme directory is a symlink (system theme)
    let is_system = if is_custom {
        false
    } else {
        fs::symlink_metadata(theme_dir)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
    };

    // Extract colors from theme configuration
    let colors = extract_theme_colors(theme_dir, is_custom);

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

/// Convert a local image file to a base64 data URL
fn convert_image_to_data_url(image_path: &Path) -> Result<String, String> {
    if !image_path.exists() {
        return Err(format!("Image file does not exist: {image_path:?}"));
    }

    let image_data = fs::read(image_path).map_err(|e| format!("Failed to read image file: {e}"))?;

    // Determine MIME type based on file extension
    let mime_type = match image_path.extension().and_then(|ext| ext.to_str()) {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        _ => "image/png", // Default to PNG
    };

    let base64_data = base64_encode(&image_data);
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

/// Get a specific system theme by folder name
#[tauri::command]
pub async fn get_sys_theme_by_name(theme_name: String) -> Result<Option<SysTheme>, String> {
    let system_root = dirs::data_dir()
        .or_else(|| dirs::home_dir().map(|home| home.join(".local").join("share")))
        .ok_or_else(|| "Failed to determine data directory".to_string())?
        .join("omarchy")
        .join("themes");

    let user_root = dirs::config_dir()
        .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
        .ok_or_else(|| "Failed to determine config directory".to_string())?
        .join("omarchy")
        .join("themes");

    // Prefer system themes, fallback to user themes for compatibility.
    let theme_path = system_root.join(&theme_name);
    let theme_path = if theme_path.exists() {
        theme_path
    } else {
        user_root.join(&theme_name)
    };

    if !theme_path.exists() || !theme_path.is_dir() {
        return Ok(None);
    }

    match generate_theme_from_directory(&theme_path) {
        Ok(theme) => Ok(Some(theme)),
        Err(e) => Err(format!("Failed to generate theme '{theme_name}': {e}")),
    }
}

/// Get system themes using cache-first strategy with fallback to direct loading
#[tauri::command]
pub async fn get_themes_cached() -> Result<Vec<SysTheme>, String> {
    // Try to get themes from cache first
    match get_theme_cache().await {
        Ok(cache) => {
            // Check if cache is valid and has themes
            if cache.is_cache_valid().await && !cache.is_empty().await {
                // Return cached themes
                match cache.get_themes().await {
                    Ok(themes) => {
                        log::info!("Returning {} themes from cache", themes.len());
                        return Ok(themes);
                    },
                    Err(e) => {
                        log::warn!("Failed to get themes from cache: {e}");
                        // Continue to fallback
                    },
                }
            }

            // Cache is invalid or empty, load themes and cache them
            log::info!("Cache invalid or empty, loading themes from filesystem");
            match get_sys_themes().await {
                Ok(themes) => {
                    // Cache the loaded themes
                    if let Err(e) = cache.cache_themes(themes.clone(), false).await {
                        log::warn!("Failed to cache themes: {e}");
                    } else {
                        log::info!("Successfully cached {} themes", themes.len());
                    }
                    Ok(themes)
                },
                Err(e) => {
                    // If direct loading fails, try to return any cached themes as fallback
                    log::error!("Failed to load themes from filesystem: {e}");
                    match cache.get_themes().await {
                        Ok(cached_themes) if !cached_themes.is_empty() => {
                            log::info!(
                                "Returning {} stale cached themes as fallback",
                                cached_themes.len()
                            );
                            Ok(cached_themes)
                        },
                        _ => Err(e),
                    }
                },
            }
        },
        Err(e) => {
            log::error!("Failed to get theme cache: {e}");
            // Fallback to direct loading without cache
            get_sys_themes().await
        },
    }
}

/// Preload themes into cache for faster subsequent access
#[tauri::command]
pub async fn preload_themes() -> Result<(), String> {
    log::info!("Starting theme preload");

    match get_theme_cache().await {
        Ok(cache) => {
            // Check if cache already has valid themes
            if cache.is_cache_valid().await && !cache.is_empty().await {
                log::info!("Cache already contains valid themes, skipping preload");
                return Ok(());
            }

            // Load themes in background and cache them
            match get_sys_themes().await {
                Ok(themes) => {
                    if let Err(e) = cache.cache_themes(themes.clone(), false).await {
                        log::error!("Failed to cache preloaded themes: {e}");
                        return Err(format!("Failed to cache preloaded themes: {e}"));
                    }
                    log::info!("Successfully preloaded {} themes into cache", themes.len());
                    Ok(())
                },
                Err(e) => {
                    log::error!("Failed to preload themes: {e}");
                    Err(format!("Failed to preload themes: {e}"))
                },
            }
        },
        Err(e) => {
            log::error!("Failed to get theme cache for preloading: {e}");
            Err(format!("Failed to get theme cache for preloading: {e}"))
        },
    }
}

/// Refresh theme cache by invalidating current cache and reloading themes
#[tauri::command]
pub async fn refresh_theme_cache() -> Result<Vec<SysTheme>, String> {
    log::info!("Refreshing theme cache");

    // Clear the color extraction cache as well
    let theme_loader = get_theme_loader();
    theme_loader.clear_cache().await;

    match get_theme_cache().await {
        Ok(cache) => {
            // Invalidate current cache
            cache.invalidate().await;
            log::info!("Cache invalidated");

            // Load fresh themes from filesystem
            match get_sys_themes().await {
                Ok(themes) => {
                    // Cache the fresh themes
                    if let Err(e) = cache.cache_themes(themes.clone(), false).await {
                        log::warn!("Failed to cache refreshed themes: {e}");
                    } else {
                        log::info!("Successfully cached {} refreshed themes", themes.len());
                    }
                    Ok(themes)
                },
                Err(e) => {
                    log::error!("Failed to load themes during cache refresh: {e}");
                    Err(format!("Failed to load themes during cache refresh: {e}"))
                },
            }
        },
        Err(e) => {
            log::error!("Failed to get theme cache for refresh: {e}");
            // Fallback to direct loading without cache
            get_sys_themes().await
        },
    }
}

/// Get lightweight theme metadata for faster initial responses
#[tauri::command]
pub async fn get_theme_metadata() -> Result<Vec<ThemeMetadata>, String> {
    log::info!("Loading theme metadata");

    let theme_loader = get_theme_loader();
    theme_loader.load_theme_metadata_only().await
}

/// Clear color extraction cache
#[tauri::command]
pub async fn clear_color_cache() -> Result<(), String> {
    log::info!("Clearing color extraction cache");

    let theme_loader = get_theme_loader();
    theme_loader.clear_cache().await;

    Ok(())
}

/// Get cache statistics for monitoring
#[tauri::command]
pub async fn get_cache_stats() -> Result<serde_json::Value, String> {
    let theme_loader = get_theme_loader();
    let (color_cache_size,) = theme_loader.get_cache_stats().await;

    let mut stats = serde_json::Map::new();
    stats.insert(
        "color_cache_size".to_string(),
        serde_json::Value::Number(color_cache_size.into()),
    );

    // Add theme cache stats if available
    if let Ok(cache) = get_theme_cache().await {
        let theme_cache_size = if cache.is_empty().await { 0 } else { 1 };
        let is_valid = cache.is_cache_valid().await;
        stats.insert(
            "theme_cache_size".to_string(),
            serde_json::Value::Number(theme_cache_size.into()),
        );
        stats.insert(
            "theme_cache_valid".to_string(),
            serde_json::Value::Bool(is_valid),
        );
    }

    Ok(serde_json::Value::Object(stats))
}

/// Invalidate cache for a specific theme
#[tauri::command]
pub async fn invalidate_theme_cache(theme_dir: String) -> Result<(), String> {
    log::info!("Invalidating cache for theme: {theme_dir}");
    if let Ok(theme_cache) = get_theme_cache().await {
        theme_cache.invalidate_theme(&theme_dir).await;
    }
    Ok(())
}

/// Invalidate cache for multiple themes
#[tauri::command]
pub async fn invalidate_themes_cache(theme_dirs: Vec<String>) -> Result<(), String> {
    log::info!("Invalidating cache for {} themes", theme_dirs.len());
    if let Ok(theme_cache) = get_theme_cache().await {
        theme_cache.invalidate_themes(&theme_dirs).await;
    }
    Ok(())
}

/// Invalidate cache for all custom themes
#[tauri::command]
pub async fn invalidate_custom_themes_cache() -> Result<(), String> {
    log::info!("Invalidating cache for all custom themes");
    if let Ok(theme_cache) = get_theme_cache().await {
        theme_cache.invalidate_custom_themes().await;
    }
    Ok(())
}

/// Invalidate cache for all system themes
#[tauri::command]
pub async fn invalidate_system_themes_cache() -> Result<(), String> {
    log::info!("Invalidating cache for all system themes");
    if let Ok(theme_cache) = get_theme_cache().await {
        theme_cache.invalidate_system_themes().await;
    }
    Ok(())
}

/// Invalidate cache and trigger background refresh
#[tauri::command]
pub async fn invalidate_and_refresh_cache() -> Result<Vec<SysTheme>, String> {
    log::info!("Invalidating entire cache and triggering background refresh");
    if let Ok(theme_cache) = get_theme_cache().await {
        // Invalidate entire cache
        theme_cache.invalidate().await;

        // Trigger background refresh
        return theme_cache.trigger_background_refresh().await;
    }

    // Fallback to direct theme loading if cache is not available
    get_sys_themes().await
}
