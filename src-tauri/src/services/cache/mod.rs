// Cache-related services
pub mod cache_config;
pub mod cache_manager;

// Re-export commonly used types
pub use crate::services::themes::theme_cache::CacheConfig;
pub use cache_manager::CacheManager;
// Cache types are now centralized in types module
pub use crate::types::AppCacheConfig;
