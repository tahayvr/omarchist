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
