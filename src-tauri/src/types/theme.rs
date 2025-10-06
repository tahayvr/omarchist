// Centralized theme type definitions
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Theme definition from themes.toml
#[derive(Debug, Serialize, Deserialize)]
pub struct Theme {
    pub title: String,
    pub description: String,
    #[serde(rename = "imageUrl")]
    pub image_url: String,
    #[serde(rename = "type")]
    pub theme_type: Vec<String>,
}

/// Container for theme data from themes.toml
#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeData {
    pub theme: Vec<Theme>,
}

/// Custom theme with multi-app support
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomTheme {
    pub name: String,
    pub created_at: String,
    pub modified_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub apps: Value,                 // Dynamic structure for all app configurations
    pub colors: Option<ThemeColors>, // Extracted color palette
}

/// Complete color palette extracted from a theme
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeColors {
    pub primary: PrimaryColors,
    pub terminal: TerminalColors,
}

/// Primary colors (background and foreground) from terminal theme
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrimaryColors {
    pub background: String,
    pub foreground: String,
}

/// Terminal color palette (excluding black and white)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TerminalColors {
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
}

/// Legacy Alacritty configuration structure for backwards compatibility
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlacrittyConfig {
    pub colors: AlacrittyColors,
}

/// Legacy Alacritty colors structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlacrittyColors {
    pub primary: AlacrittyPrimaryColors,
}

/// Legacy Alacritty primary colors structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlacrittyPrimaryColors {
    pub background: String,
    pub foreground: String,
    pub dim_foreground: String,
}

/// Cache statistics for theme operations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheStats {
    /// Number of cached themes
    pub cached_themes: usize,
    /// Cache hit rate as percentage
    pub hit_rate: f64,
    /// Total cache operations
    pub total_operations: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Last refresh timestamp
    pub last_refresh: Option<String>,
}

/// Theme metadata for caching and management
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeMetadata {
    /// Theme name/identifier
    pub name: String,
    /// Theme type (system, custom, community)
    pub theme_type: String,
    /// Last modified timestamp
    pub last_modified: Option<String>,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// Whether theme is currently cached
    pub is_cached: bool,
}
