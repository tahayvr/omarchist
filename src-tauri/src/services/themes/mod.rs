// Theme-related services
pub mod color_extraction;
pub mod custom_themes;
pub mod get_current_theme;
pub mod get_sys_themes;
pub mod get_themes;
pub mod optimized_theme_loader;
pub mod theme_cache;
pub mod validator;

// Re-export commonly used types
pub use color_extraction::ColorExtractor;
pub use custom_themes::CustomThemeService;
pub use theme_cache::ThemeCache;
pub use validator::ThemeValidator;
// Theme types are now centralized in types module
pub use crate::types::{
    AlacrittyColors, AlacrittyConfig, AlacrittyPrimaryColors, CustomTheme, PrimaryColors,
    TerminalColors, Theme, ThemeColors, ThemeData,
};
