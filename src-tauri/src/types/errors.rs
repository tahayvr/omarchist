// Centralized error type definitions
use thiserror::Error;

/// Main application error type that encompasses all possible errors
#[derive(Debug, Error)]
pub enum AppError {
    /// I/O related errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML serialization/deserialization errors
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Theme-related errors
    #[error("Theme error: {0}")]
    Theme(#[from] ThemeError),

    /// Settings-related errors
    #[error("Settings error: {0}")]
    Settings(#[from] SettingsError),

    /// Cache-related errors
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),

    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Hyprland configuration errors
    #[error("Hyprland error: {0}")]
    Hyprland(#[from] HyprlandConfigError),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Generic application errors
    #[error("Application error: {0}")]
    Generic(String),
}

/// Theme-specific error types
#[derive(Debug, Error)]
pub enum ThemeError {
    /// Theme not found
    #[error("Theme '{0}' not found")]
    NotFound(String),

    /// Invalid theme format
    #[error("Invalid theme format: {0}")]
    InvalidFormat(String),

    /// Theme application failed
    #[error("Failed to apply theme: {0}")]
    ApplyFailed(String),

    /// Color extraction failed
    #[error("Failed to extract colors: {0}")]
    ColorExtractionFailed(String),

    /// Theme creation failed
    #[error("Failed to create theme: {0}")]
    CreationFailed(String),

    /// Theme update failed
    #[error("Failed to update theme: {0}")]
    UpdateFailed(String),

    /// Theme deletion failed
    #[error("Failed to delete theme: {0}")]
    DeletionFailed(String),

    /// Theme import failed
    #[error("Failed to import theme: {0}")]
    ImportFailed(String),

    /// Theme export failed
    #[error("Failed to export theme: {0}")]
    ExportFailed(String),

    /// Theme validation failed
    #[error("Theme validation failed: {0}")]
    ValidationFailed(String),

    /// Theme already exists
    #[error("Theme '{0}' already exists")]
    AlreadyExists(String),

    /// Invalid theme schema version
    #[error("Invalid schema version: {0}")]
    InvalidSchemaVersion(String),

    /// Theme file corrupted
    #[error("Theme file is corrupted or incomplete")]
    Corrupted,
}

/// Settings-specific error types
#[derive(Debug, Error)]
pub enum SettingsError {
    /// Failed to read settings file
    #[error("Failed to read settings file: {0}")]
    FileRead(#[from] std::io::Error),

    /// Failed to parse settings JSON
    #[error("Failed to parse settings JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Settings validation failed
    #[error("Settings validation failed: {0}")]
    Validation(String),

    /// Settings file is corrupted
    #[error("Settings file is corrupted")]
    Corrupted,

    /// Failed to get app data directory
    #[error("Failed to get app data directory")]
    AppDataDir,

    /// Failed to create settings directory
    #[error("Failed to create settings directory: {0}")]
    CreateDir(std::io::Error),
}

/// Cache-specific error types
#[derive(Debug, Error)]
pub enum CacheError {
    /// Cache initialization failed
    #[error("Cache initialization failed: {0}")]
    InitializationFailed(String),

    /// Cache operation failed
    #[error("Cache operation failed: {0}")]
    OperationFailed(String),

    /// Cache invalidation failed
    #[error("Cache invalidation failed: {0}")]
    InvalidationFailed(String),

    /// Cache configuration error
    #[error("Cache configuration error: {0}")]
    ConfigurationError(String),
}

/// Configuration-specific error types
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    /// Configuration parsing failed
    #[error("Configuration parsing failed: {0}")]
    ParseFailed(String),

    /// Configuration validation failed
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),

    /// Configuration generation failed
    #[error("Configuration generation failed: {0}")]
    GenerationFailed(String),
}

/// Hyprland configuration-specific error types
#[derive(Debug, Error)]
pub enum HyprlandConfigError {
    /// Hyprland config value failed validation
    #[error("Hyprland config validation failed for '{field}': {message}")]
    Validation { field: String, message: String },

    /// Hyprland config parsing failed
    #[error("Hyprland config parse failed for '{field}': {message}")]
    Parse { field: String, message: String },

    /// Hyprland override file not found
    #[error("Hyprland override file not found at {path}")]
    FileNotFound { path: String },

    /// Required source directive missing from primary config
    #[error("Hyprland source directive missing in {path}")]
    SourceDirectiveMissing { path: String },

    /// Hyprland serialization error
    #[error("Hyprland config serialization error: {0}")]
    Serialization(String),

    /// Underlying I/O error
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Result type for application operations
pub type AppResult<T> = Result<T, AppError>;

/// Result type for theme operations
pub type ThemeResult<T> = Result<T, ThemeError>;

/// Result type for settings operations
pub type SettingsResult<T> = Result<T, SettingsError>;

/// Result type for cache operations
pub type CacheResult<T> = Result<T, CacheError>;

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Result type for Hyprland configuration operations
pub type HyprlandResult<T> = Result<T, HyprlandConfigError>;

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Generic(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Generic(s.to_string())
    }
}

impl From<String> for ThemeError {
    fn from(s: String) -> Self {
        ThemeError::ApplyFailed(s)
    }
}

impl From<&str> for ThemeError {
    fn from(s: &str) -> Self {
        ThemeError::ApplyFailed(s.to_string())
    }
}
