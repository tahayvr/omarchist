use crate::services::themes::get_sys_themes::SysTheme;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Configuration for the theme cache
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheConfig {
    /// Cache duration in minutes
    pub cache_duration_minutes: u64,
    /// Whether to preload themes on startup
    pub preload_on_startup: bool,
    /// Background refresh interval in minutes
    pub background_refresh_interval: u64,
    /// Maximum number of themes to cache
    pub max_cache_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_duration_minutes: 5,
            preload_on_startup: true,
            background_refresh_interval: 10,
            max_cache_size: 1000,
        }
    }
}

/// A cached theme entry with metadata
#[derive(Debug, Clone)]
pub struct CachedTheme {
    pub theme: SysTheme,
    pub cached_at: SystemTime,
    pub metadata_only: bool,
}

/// Thread-safe theme cache service
#[derive(Debug)]
pub struct ThemeCache {
    /// Cached themes storage
    themes: Arc<RwLock<HashMap<String, CachedTheme>>>,
    /// Cache configuration
    config: Arc<RwLock<CacheConfig>>,
    /// Last full cache refresh timestamp
    last_full_refresh: Arc<RwLock<Option<SystemTime>>>,
}

impl ThemeCache {
    /// Create a new theme cache with default configuration
    pub fn new() -> Self {
        Self {
            themes: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(CacheConfig::default())),
            last_full_refresh: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new theme cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            themes: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
            last_full_refresh: Arc::new(RwLock::new(None)),
        }
    }

    /// Get all cached themes
    pub async fn get_themes(&self) -> Result<Vec<SysTheme>, String> {
        let themes = self.themes.read().await;
        let cached_themes: Vec<SysTheme> =
            themes.values().map(|cached| cached.theme.clone()).collect();

        Ok(cached_themes)
    }

    /// Get a specific theme by directory name
    pub async fn get_theme(&self, dir: &str) -> Option<SysTheme> {
        let themes = self.themes.read().await;
        themes.get(dir).map(|cached| cached.theme.clone())
    }

    /// Cache a single theme
    pub async fn cache_theme(&self, theme: SysTheme, metadata_only: bool) -> Result<(), String> {
        let mut themes = self.themes.write().await;
        let config = self.config.read().await;

        // Check cache size limit
        if themes.len() >= config.max_cache_size {
            // Remove oldest entry if at capacity
            if let Some(oldest_key) = self.find_oldest_entry(&themes).await {
                themes.remove(&oldest_key);
            }
        }

        let cached_theme = CachedTheme {
            theme: theme.clone(),
            cached_at: SystemTime::now(),
            metadata_only,
        };

        themes.insert(theme.dir.clone(), cached_theme);
        Ok(())
    }

    /// Cache multiple themes
    pub async fn cache_themes(
        &self,
        themes_list: Vec<SysTheme>,
        metadata_only: bool,
    ) -> Result<(), String> {
        let mut themes = self.themes.write().await;
        let config = self.config.read().await;
        let now = SystemTime::now();

        // Clear cache if we're at capacity and adding many themes
        if themes.len() + themes_list.len() > config.max_cache_size {
            themes.clear();
        }

        for theme in themes_list {
            let cached_theme = CachedTheme {
                theme: theme.clone(),
                cached_at: now,
                metadata_only,
            };
            themes.insert(theme.dir.clone(), cached_theme);
        }

        // Update last full refresh timestamp
        let mut last_refresh = self.last_full_refresh.write().await;
        *last_refresh = Some(now);

        Ok(())
    }

    /// Check if the cache is valid (not expired)
    pub async fn is_cache_valid(&self) -> bool {
        let config = self.config.read().await;
        let last_refresh = self.last_full_refresh.read().await;

        if let Some(last_refresh_time) = *last_refresh {
            let cache_duration = Duration::from_secs(config.cache_duration_minutes * 60);
            let elapsed = SystemTime::now()
                .duration_since(last_refresh_time)
                .unwrap_or(Duration::from_secs(u64::MAX));

            elapsed < cache_duration
        } else {
            false
        }
    }

    /// Check if a specific theme entry is valid
    pub async fn is_theme_valid(&self, dir: &str) -> bool {
        let themes = self.themes.read().await;
        let config = self.config.read().await;

        if let Some(cached_theme) = themes.get(dir) {
            let cache_duration = Duration::from_secs(config.cache_duration_minutes * 60);
            let elapsed = SystemTime::now()
                .duration_since(cached_theme.cached_at)
                .unwrap_or(Duration::from_secs(u64::MAX));

            elapsed < cache_duration
        } else {
            false
        }
    }

    /// Invalidate the entire cache
    pub async fn invalidate(&self) {
        let mut themes = self.themes.write().await;
        let mut last_refresh = self.last_full_refresh.write().await;

        themes.clear();
        *last_refresh = None;
    }

    /// Invalidate a specific theme
    pub async fn invalidate_theme(&self, dir: &str) {
        let mut themes = self.themes.write().await;
        themes.remove(dir);
        log::info!("Invalidated cache for theme: {dir}");
    }

    /// Invalidate multiple themes by directory names
    pub async fn invalidate_themes(&self, dirs: &[String]) {
        let mut themes = self.themes.write().await;
        for dir in dirs {
            themes.remove(dir);
        }
        log::info!("Invalidated cache for {} themes", dirs.len());
    }

    /// Invalidate all custom themes (themes with is_custom = true)
    pub async fn invalidate_custom_themes(&self) {
        let mut themes = self.themes.write().await;
        let custom_theme_keys: Vec<String> = themes
            .iter()
            .filter(|(_, cached)| cached.theme.is_custom)
            .map(|(key, _)| key.clone())
            .collect();

        for key in &custom_theme_keys {
            themes.remove(key);
        }

        log::info!(
            "Invalidated cache for {} custom themes",
            custom_theme_keys.len()
        );
    }

    /// Invalidate all system themes (themes with is_system = true)
    pub async fn invalidate_system_themes(&self) {
        let mut themes = self.themes.write().await;
        let system_theme_keys: Vec<String> = themes
            .iter()
            .filter(|(_, cached)| cached.theme.is_system)
            .map(|(key, _)| key.clone())
            .collect();

        for key in &system_theme_keys {
            themes.remove(key);
        }

        log::info!(
            "Invalidated cache for {} system themes",
            system_theme_keys.len()
        );
    }

    /// Trigger background refresh after cache invalidation
    pub async fn trigger_background_refresh(&self) -> Result<Vec<SysTheme>, String> {
        log::info!("Triggering background cache refresh");

        // Import the optimized theme loader
        use crate::services::themes::optimized_theme_loader::OptimizedThemeLoader;

        let loader = OptimizedThemeLoader::new();
        let themes = loader.load_themes_parallel().await?;

        // Cache the refreshed themes
        self.cache_themes(themes.clone(), false).await?;

        log::info!("Background refresh completed with {} themes", themes.len());
        Ok(themes)
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let themes = self.themes.read().await;
        let config = self.config.read().await;
        let last_refresh = self.last_full_refresh.read().await;

        let valid_entries = self.count_valid_entries(&themes, &config).await;

        CacheStats {
            total_entries: themes.len(),
            valid_entries,
            expired_entries: themes.len() - valid_entries,
            cache_size_limit: config.max_cache_size,
            last_refresh: *last_refresh,
            cache_duration_minutes: config.cache_duration_minutes,
        }
    }

    /// Update cache configuration
    pub async fn update_config(&self, new_config: CacheConfig) {
        let mut config = self.config.write().await;
        *config = new_config;
    }

    /// Get current cache configuration
    pub async fn get_config(&self) -> CacheConfig {
        let config = self.config.read().await;
        config.clone()
    }

    /// Check if cache has any themes
    pub async fn is_empty(&self) -> bool {
        let themes = self.themes.read().await;
        themes.is_empty()
    }

    /// Get the number of cached themes
    pub async fn len(&self) -> usize {
        let themes = self.themes.read().await;
        themes.len()
    }

    /// Find the oldest cache entry for eviction (optimized to avoid cloning)
    async fn find_oldest_entry(&self, themes: &HashMap<String, CachedTheme>) -> Option<String> {
        themes
            .iter()
            .min_by_key(|(_, cached)| cached.cached_at)
            .map(|(key, _)| key.to_owned()) // Use to_owned() which is more explicit about the allocation
    }

    /// Count valid (non-expired) cache entries
    async fn count_valid_entries(
        &self,
        themes: &HashMap<String, CachedTheme>,
        config: &CacheConfig,
    ) -> usize {
        let cache_duration = Duration::from_secs(config.cache_duration_minutes * 60);
        let now = SystemTime::now();

        themes
            .values()
            .filter(|cached| {
                let elapsed = now
                    .duration_since(cached.cached_at)
                    .unwrap_or(Duration::from_secs(u64::MAX));
                elapsed < cache_duration
            })
            .count()
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
    pub cache_size_limit: usize,
    pub last_refresh: Option<SystemTime>,
    pub cache_duration_minutes: u64,
}

impl Default for ThemeCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::thread;
    use std::time::Duration as StdDuration;

    fn create_test_theme(dir: &str, title: &str) -> SysTheme {
        SysTheme {
            dir: dir.to_string(),
            title: title.to_string(),
            description: format!("Test theme {}", title),
            image: String::new(),
            is_system: false,
            is_custom: false,
            colors: None,
        }
    }

    #[tokio::test]
    async fn test_cache_creation() {
        let cache = ThemeCache::new();
        assert!(cache.is_empty().await);
        assert_eq!(cache.len().await, 0);
        assert!(!cache.is_cache_valid().await);
    }

    #[tokio::test]
    async fn test_cache_with_custom_config() {
        let config = CacheConfig {
            cache_duration_minutes: 10,
            preload_on_startup: false,
            background_refresh_interval: 15,
            max_cache_size: 500,
        };

        let cache = ThemeCache::with_config(config.clone());
        let retrieved_config = cache.get_config().await;

        assert_eq!(retrieved_config.cache_duration_minutes, 10);
        assert!(!retrieved_config.preload_on_startup);
        assert_eq!(retrieved_config.background_refresh_interval, 15);
        assert_eq!(retrieved_config.max_cache_size, 500);
    }

    #[tokio::test]
    async fn test_cache_single_theme() {
        let cache = ThemeCache::new();
        let theme = create_test_theme("test-theme", "Test Theme");

        let result = cache.cache_theme(theme.clone(), false).await;
        assert!(result.is_ok());

        assert!(!cache.is_empty().await);
        assert_eq!(cache.len().await, 1);

        let cached_theme = cache.get_theme("test-theme").await;
        assert!(cached_theme.is_some());
        assert_eq!(cached_theme.unwrap().title, "Test Theme");
    }

    #[tokio::test]
    async fn test_cache_multiple_themes() {
        let cache = ThemeCache::new();
        let themes = vec![
            create_test_theme("theme1", "Theme 1"),
            create_test_theme("theme2", "Theme 2"),
            create_test_theme("theme3", "Theme 3"),
        ];

        let result = cache.cache_themes(themes, false).await;
        assert!(result.is_ok());

        assert_eq!(cache.len().await, 3);
        assert!(cache.is_cache_valid().await);

        let all_themes = cache.get_themes().await.unwrap();
        assert_eq!(all_themes.len(), 3);
    }

    #[tokio::test]
    async fn test_cache_size_limit() {
        let config = CacheConfig {
            max_cache_size: 2,
            ..Default::default()
        };
        let cache = ThemeCache::with_config(config);

        // Add themes up to the limit
        let theme1 = create_test_theme("theme1", "Theme 1");
        let theme2 = create_test_theme("theme2", "Theme 2");

        cache.cache_theme(theme1, false).await.unwrap();
        cache.cache_theme(theme2, false).await.unwrap();
        assert_eq!(cache.len().await, 2);

        // Adding another theme should remove the oldest
        let theme3 = create_test_theme("theme3", "Theme 3");
        cache.cache_theme(theme3, false).await.unwrap();
        assert_eq!(cache.len().await, 2);

        // theme3 should be present, but theme1 might be evicted
        let theme3_cached = cache.get_theme("theme3").await;
        assert!(theme3_cached.is_some());
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = ThemeCache::new();
        let theme = create_test_theme("test-theme", "Test Theme");

        cache.cache_theme(theme, false).await.unwrap();
        assert!(!cache.is_empty().await);

        cache.invalidate().await;
        assert!(cache.is_empty().await);
        assert!(!cache.is_cache_valid().await);
    }

    #[tokio::test]
    async fn test_theme_invalidation() {
        let cache = ThemeCache::new();
        let theme1 = create_test_theme("theme1", "Theme 1");
        let theme2 = create_test_theme("theme2", "Theme 2");

        cache.cache_theme(theme1, false).await.unwrap();
        cache.cache_theme(theme2, false).await.unwrap();
        assert_eq!(cache.len().await, 2);

        cache.invalidate_theme("theme1").await;
        assert_eq!(cache.len().await, 1);

        let theme1_cached = cache.get_theme("theme1").await;
        let theme2_cached = cache.get_theme("theme2").await;
        assert!(theme1_cached.is_none());
        assert!(theme2_cached.is_some());
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = CacheConfig {
            cache_duration_minutes: 0, // Expire immediately for testing
            ..Default::default()
        };
        let cache = ThemeCache::with_config(config);
        let theme = create_test_theme("test-theme", "Test Theme");

        cache.cache_theme(theme, false).await.unwrap();

        // Wait a bit to ensure expiration
        thread::sleep(StdDuration::from_millis(10));

        assert!(!cache.is_cache_valid().await);
        assert!(!cache.is_theme_valid("test-theme").await);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = ThemeCache::new();
        let themes = vec![
            create_test_theme("theme1", "Theme 1"),
            create_test_theme("theme2", "Theme 2"),
        ];

        cache.cache_themes(themes, false).await.unwrap();

        let stats = cache.get_cache_stats().await;
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.valid_entries, 2);
        assert_eq!(stats.expired_entries, 0);
        assert!(stats.last_refresh.is_some());
    }

    #[tokio::test]
    async fn test_config_update() {
        let cache = ThemeCache::new();
        let new_config = CacheConfig {
            cache_duration_minutes: 20,
            preload_on_startup: false,
            background_refresh_interval: 30,
            max_cache_size: 200,
        };

        cache.update_config(new_config.clone()).await;
        let retrieved_config = cache.get_config().await;

        assert_eq!(retrieved_config.cache_duration_minutes, 20);
        assert!(!retrieved_config.preload_on_startup);
        assert_eq!(retrieved_config.background_refresh_interval, 30);
        assert_eq!(retrieved_config.max_cache_size, 200);
    }
}
