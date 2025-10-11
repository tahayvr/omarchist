use crate::services::themes::theme_cache::CacheConfig;
use crate::types::AppCacheConfig;
use std::fs;
use std::path::Path;
use tauri::AppHandle;
use tauri::Manager;

/// Cache configuration manager
pub struct CacheConfigManager;

impl CacheConfigManager {
    /// Load cache configuration from file or create default
    pub fn load_config(app_handle: &AppHandle) -> Result<AppCacheConfig, String> {
        let config_path = Self::get_config_path(app_handle)?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read cache config: {e}"))?;

            let config: AppCacheConfig = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse cache config: {e}"))?;

            Ok(config)
        } else {
            // Create default config and save it
            let default_config = AppCacheConfig::default();
            Self::save_config(app_handle, &default_config)?;
            Ok(default_config)
        }
    }

    /// Save cache configuration to file
    pub fn save_config(app_handle: &AppHandle, config: &AppCacheConfig) -> Result<(), String> {
        let config_path = Self::get_config_path(app_handle)?;

        // Ensure the parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }

        let content = toml::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize cache config: {e}"))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write cache config: {e}"))?;

        Ok(())
    }

    /// Update specific cache configuration
    pub fn update_theme_cache_config(
        app_handle: &AppHandle,
        theme_config: CacheConfig,
    ) -> Result<AppCacheConfig, String> {
        let mut config = Self::load_config(app_handle)?;
        config.theme_cache = theme_config;
        Self::save_config(app_handle, &config)?;
        Ok(config)
    }

    /// Get the path to the cache configuration file
    fn get_config_path(app_handle: &AppHandle) -> Result<std::path::PathBuf, String> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {e}"))?;

        Ok(app_data_dir.join("cache_config.toml"))
    }

    /// Validate cache configuration
    pub fn validate_config(config: &AppCacheConfig) -> Result<(), String> {
        // Validate theme cache config
        if config.theme_cache.cache_duration_minutes == 0 {
            return Err("Cache duration must be greater than 0".to_string());
        }

        if config.theme_cache.max_cache_size == 0 {
            return Err("Max cache size must be greater than 0".to_string());
        }

        if config.theme_cache.background_refresh_interval == 0 {
            return Err("Background refresh interval must be greater than 0".to_string());
        }

        // Validate cache directory if specified
        if let Some(cache_dir) = &config.cache_directory {
            let path = Path::new(cache_dir);
            if !path.is_absolute() {
                return Err("Cache directory must be an absolute path".to_string());
            }
        }

        Ok(())
    }
}

/// Tauri command to get current cache configuration
#[tauri::command]
pub async fn get_cache_config(app_handle: AppHandle) -> Result<AppCacheConfig, String> {
    CacheConfigManager::load_config(&app_handle)
}

/// Tauri command to update cache configuration
#[tauri::command]
pub async fn update_cache_config(
    app_handle: AppHandle,
    config: AppCacheConfig,
) -> Result<AppCacheConfig, String> {
    // Validate the configuration first
    CacheConfigManager::validate_config(&config)?;

    // Save the configuration
    CacheConfigManager::save_config(&app_handle, &config)?;

    // Update the global cache manager if it exists
    if let Ok(cache_manager) = crate::services::cache::cache_manager::get_cache_manager().await {
        let theme_cache = cache_manager.theme_cache();
        theme_cache.update_config(config.theme_cache.clone()).await;
    }

    Ok(config)
}

/// Tauri command to reset cache configuration to defaults
#[tauri::command]
pub async fn reset_cache_config(app_handle: AppHandle) -> Result<AppCacheConfig, String> {
    let default_config = AppCacheConfig::default();
    CacheConfigManager::save_config(&app_handle, &default_config)?;

    // Update the global cache manager if it exists
    if let Ok(cache_manager) = crate::services::cache::cache_manager::get_cache_manager().await {
        let theme_cache = cache_manager.theme_cache();
        theme_cache
            .update_config(default_config.theme_cache.clone())
            .await;
    }

    Ok(default_config)
}
