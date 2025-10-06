// Domain-specific service modules
pub mod cache;
pub mod config;
pub mod hyprland;
pub mod import_export;
pub mod nvidia_detection;
pub mod settings;
pub mod themes;

// Utility services that don't fit into specific domains
pub mod cli_handler;
pub mod startup_cli;

// Re-export commonly used startup CLI types for easier access
pub use startup_cli::check_cli_args;
// CLI types are now centralized in types module
pub use crate::types::{StartupCliResult, StartupCommand};

// Re-export domain modules for easier access
pub use cache::*;
pub use config::*;
pub use hyprland::*;
pub use settings::*;
pub use themes::*;
