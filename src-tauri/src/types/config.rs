// Centralized configuration type definitions
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Application settings structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppSettings {
    /// Whether to automatically apply themes when entering edit mode
    pub auto_apply_theme: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_apply_theme: true,
        }
    }
}

/// Settings file structure with version and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsFile {
    /// Version of the settings file format
    pub version: String,
    /// The actual settings data
    pub settings: AppSettings,
    /// Metadata about the settings file
    pub metadata: SettingsMetadata,
}

impl Default for SettingsFile {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            version: "1.0.0".to_string(),
            settings: AppSettings::default(),
            metadata: SettingsMetadata {
                created_at: now,
                last_modified: now,
            },
        }
    }
}

/// Metadata for settings file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsMetadata {
    /// When the settings file was created
    pub created_at: DateTime<Utc>,
    /// When the settings file was last modified
    pub last_modified: DateTime<Utc>,
}

/// Application cache configuration that includes all cache settings
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppCacheConfig {
    /// Theme cache configuration
    pub theme_cache: crate::services::themes::theme_cache::CacheConfig,
    /// Whether to enable cache persistence (future feature)
    pub enable_persistence: bool,
    /// Global cache directory path (future feature)
    pub cache_directory: Option<String>,
}

/// Result of startup CLI processing
#[derive(Debug, Clone)]
pub struct StartupCliResult {
    /// Whether the application should continue with normal startup
    pub should_continue: bool,
    /// Reason for early exit (if applicable)
    pub exit_reason: Option<String>,
    /// Exit code to use if exiting early
    pub exit_code: i32,
}

/// Represents the different startup commands that can be detected
#[derive(Debug, Clone, PartialEq)]
pub enum StartupCommand {
    /// No CLI arguments, start normally
    Normal,
    /// Refresh command detected
    Refresh,
    /// Unknown command with the original command string
    Unknown(String),
}
