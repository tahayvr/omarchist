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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_manager_creation() {
        let manager = CacheManager::new();
        let theme_cache = manager.theme_cache();

        assert!(theme_cache.is_empty().await);
    }

    #[tokio::test]
    async fn test_cache_manager_with_config() {
        let config = CacheConfig {
            cache_duration_minutes: 15,
            preload_on_startup: true,
            background_refresh_interval: 20,
            max_cache_size: 300,
        };

        let manager = CacheManager::with_theme_config(config.clone());
        let theme_cache = manager.theme_cache();
        let retrieved_config = theme_cache.get_config().await;

        assert_eq!(retrieved_config.cache_duration_minutes, 15);
        assert!(retrieved_config.preload_on_startup);
        assert_eq!(retrieved_config.background_refresh_interval, 20);
        assert_eq!(retrieved_config.max_cache_size, 300);
    }

    #[tokio::test]
    async fn test_global_cache_manager_initialization() {
        // Note: This test might interfere with other tests due to global state
        // In a real application, you'd want to reset the global state between tests

        let manager = init_cache_manager().await;
        let theme_cache = manager.theme_cache();

        assert!(theme_cache.is_empty().await);

        // Test that subsequent calls return the same instance
        let manager2 = get_cache_manager().await.unwrap();
        let theme_cache2 = manager2.theme_cache();

        // Both should point to the same cache instance
        assert_eq!(Arc::as_ptr(&theme_cache), Arc::as_ptr(&theme_cache2));
    }
}
