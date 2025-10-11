use crate::services::themes::theme_cache::{CacheConfig, ThemeCache};
use std::sync::Arc;
use tokio::sync::OnceCell;

/// Global cache manager for the application
pub struct CacheManager {
    theme_cache: Arc<ThemeCache>,
}

impl CacheManager {
    /// Create a new cache manager with default configuration
    pub fn new() -> Self {
        Self {
            theme_cache: Arc::new(ThemeCache::new()),
        }
    }

    /// Create a new cache manager with custom theme cache configuration
    pub fn with_theme_config(config: CacheConfig) -> Self {
        Self {
            theme_cache: Arc::new(ThemeCache::with_config(config)),
        }
    }

    /// Get a reference to the theme cache (optimized to avoid unnecessary clones)
    pub fn theme_cache(&self) -> &Arc<ThemeCache> {
        &self.theme_cache
    }

    /// Get a cloned reference to the theme cache when ownership is needed
    pub fn theme_cache_cloned(&self) -> Arc<ThemeCache> {
        Arc::clone(&self.theme_cache)
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global cache manager instance
static CACHE_MANAGER: OnceCell<CacheManager> = OnceCell::const_new();

/// Initialize the global cache manager
pub async fn init_cache_manager() -> &'static CacheManager {
    CACHE_MANAGER
        .get_or_init(|| async { CacheManager::new() })
        .await
}

/// Initialize the global cache manager with custom configuration
pub async fn init_cache_manager_with_config(config: CacheConfig) -> &'static CacheManager {
    CACHE_MANAGER
        .get_or_init(|| async { CacheManager::with_theme_config(config) })
        .await
}

/// Get the global cache manager instance
pub async fn get_cache_manager() -> Result<&'static CacheManager, String> {
    CACHE_MANAGER
        .get()
        .ok_or_else(|| "Cache manager not initialized".to_string())
}

/// Get the global theme cache instance
pub async fn get_theme_cache() -> Result<Arc<ThemeCache>, String> {
    let manager = get_cache_manager().await?;
    Ok(manager.theme_cache_cloned())
}
