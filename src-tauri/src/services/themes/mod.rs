// Theme-related services
pub mod color_extraction;
pub mod community_themes;
pub mod custom_themes;
pub mod get_current_theme;
pub mod get_sys_themes;
pub mod get_themes;
pub mod optimized_theme_loader;
pub mod theme_cache;

// Re-export commonly used types
pub use color_extraction::ColorExtractor;
pub use theme_cache::ThemeCache;
// Theme types are now centralized in types module
pub use crate::types::{
    AlacrittyColors, AlacrittyConfig, AlacrittyPrimaryColors, CustomTheme, PrimaryColors,
    TerminalColors, Theme, ThemeColors, ThemeData,
};
