use crate::types::{PrimaryColors, TerminalColors, ThemeColors};
use serde_json::Value;
use std::fs;
use std::path::Path;
use toml;

/// Color extraction service for theme configurations
pub struct ColorExtractor;

impl ColorExtractor {
    /// Extract colors from a custom theme JSON file
    pub fn extract_from_custom_theme(theme_data: &Value) -> Option<ThemeColors> {
        let alacritty = theme_data.get("alacritty")?;
        let colors = alacritty.get("colors")?;

        // Extract primary colors
        let primary = colors.get("primary")?;
        let background = Self::normalize_color(primary.get("background")?.as_str()?)?;
        let foreground = Self::normalize_color(primary.get("foreground")?.as_str()?)?;

        // Extract terminal colors (prefer normal over bright)
        let normal = colors.get("normal");
        let bright = colors.get("bright");

        let terminal_colors = Self::extract_terminal_colors(normal, bright)?;

        Some(ThemeColors {
            primary: PrimaryColors {
                background,
                foreground,
            },
            terminal: terminal_colors,
        })
    }

    /// Extract colors from an Alacritty TOML configuration file
    pub fn extract_from_alacritty_config(config_path: &Path) -> Option<ThemeColors> {
        let content = fs::read_to_string(config_path).ok()?;
        let config: Value = toml::from_str(&content).ok()?;

        let colors = config.get("colors")?;

        // Extract primary colors
        let primary = colors.get("primary")?;
        let background = Self::normalize_color(primary.get("background")?.as_str()?)?;
        let foreground = Self::normalize_color(primary.get("foreground")?.as_str()?)?;

        // Extract terminal colors
        let normal = colors.get("normal");
        let bright = colors.get("bright");

        let terminal_colors = Self::extract_terminal_colors(normal, bright)?;

        Some(ThemeColors {
            primary: PrimaryColors {
                background,
                foreground,
            },
            terminal: terminal_colors,
        })
    }

    /// Extract terminal colors with fallback logic (normal -> bright -> defaults)
    fn extract_terminal_colors(
        normal: Option<&Value>,
        bright: Option<&Value>,
    ) -> Option<TerminalColors> {
        let color_source = normal.or(bright)?;

        let red = Self::normalize_color(color_source.get("red")?.as_str()?)
            .or_else(|| Self::get_fallback_terminal_color("red"))?;
        let green = Self::normalize_color(color_source.get("green")?.as_str()?)
            .or_else(|| Self::get_fallback_terminal_color("green"))?;
        let yellow = Self::normalize_color(color_source.get("yellow")?.as_str()?)
            .or_else(|| Self::get_fallback_terminal_color("yellow"))?;
        let blue = Self::normalize_color(color_source.get("blue")?.as_str()?)
            .or_else(|| Self::get_fallback_terminal_color("blue"))?;
        let magenta = Self::normalize_color(color_source.get("magenta")?.as_str()?)
            .or_else(|| Self::get_fallback_terminal_color("magenta"))?;
        let cyan = Self::normalize_color(color_source.get("cyan")?.as_str()?)
            .or_else(|| Self::get_fallback_terminal_color("cyan"))?;

        Some(TerminalColors {
            red,
            green,
            yellow,
            blue,
            magenta,
            cyan,
        })
    }

    /// Normalize and validate color format to hex
    pub fn normalize_color(color: &str) -> Option<String> {
        let trimmed = color.trim();

        // Already in hex format
        if trimmed.starts_with('#') && Self::is_valid_hex_color(trimmed) {
            return Some(trimmed.to_lowercase());
        }

        // Try to convert from other formats (rgb, rgba, etc.)
        if let Some(hex) = Self::convert_to_hex(trimmed) {
            return Some(hex);
        }

        None
    }

    /// Validate if a string is a valid hex color
    fn is_valid_hex_color(color: &str) -> bool {
        if !color.starts_with('#') {
            return false;
        }

        let hex_part = &color[1..];
        if hex_part.len() != 6 && hex_part.len() != 3 {
            return false;
        }

        hex_part.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Convert color from other formats to hex (basic implementation)
    fn convert_to_hex(color: &str) -> Option<String> {
        // Handle 3-digit hex without #
        if color.len() == 3 && color.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(format!("#{}", color.to_lowercase()));
        }

        // Handle 6-digit hex without #
        if color.len() == 6 && color.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(format!("#{}", color.to_lowercase()));
        }

        // TODO: Add support for rgb(), rgba(), hsl() formats if needed
        None
    }

    /// Get fallback colors when theme data is incomplete
    pub fn get_fallback_colors() -> ThemeColors {
        ThemeColors {
            primary: PrimaryColors {
                background: "#1a1a1a".to_string(),
                foreground: "#ffffff".to_string(),
            },
            terminal: TerminalColors {
                red: "#ff5555".to_string(),
                green: "#50fa7b".to_string(),
                yellow: "#f1fa8c".to_string(),
                blue: "#8be9fd".to_string(),
                magenta: "#ff79c6".to_string(),
                cyan: "#8be9fd".to_string(),
            },
        }
    }

    /// Get fallback color for a specific terminal color
    fn get_fallback_terminal_color(color_name: &str) -> Option<String> {
        let fallback_colors = Self::get_fallback_colors();
        match color_name {
            "red" => Some(fallback_colors.terminal.red),
            "green" => Some(fallback_colors.terminal.green),
            "yellow" => Some(fallback_colors.terminal.yellow),
            "blue" => Some(fallback_colors.terminal.blue),
            "magenta" => Some(fallback_colors.terminal.magenta),
            "cyan" => Some(fallback_colors.terminal.cyan),
            _ => None,
        }
    }

    /// Validate and sanitize a color string
    pub fn validate_and_sanitize_color(color: &str) -> Option<String> {
        Self::normalize_color(color)
    }
}
