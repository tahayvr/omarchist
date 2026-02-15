use crate::types::themes::{PrimaryColors, TerminalColors, ThemeColors};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Parse colors from a colors.toml file
pub fn parse_colors_toml(path: &Path) -> Option<ThemeColors> {
    let content = fs::read_to_string(path).ok()?;
    let mut colors: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').to_string();
            colors.insert(key, value);
        }
    }

    Some(ThemeColors {
        primary: PrimaryColors {
            background: colors
                .get("background")
                .cloned()
                .unwrap_or_else(|| "#1e1e2e".to_string()),
            foreground: colors
                .get("foreground")
                .cloned()
                .unwrap_or_else(|| "#cdd6f4".to_string()),
        },
        terminal: TerminalColors {
            black: colors
                .get("color0")
                .cloned()
                .unwrap_or_else(|| "#45475a".to_string()),
            red: colors
                .get("color1")
                .cloned()
                .unwrap_or_else(|| "#f38ba8".to_string()),
            green: colors
                .get("color2")
                .cloned()
                .unwrap_or_else(|| "#a6e3a1".to_string()),
            yellow: colors
                .get("color3")
                .cloned()
                .unwrap_or_else(|| "#f9e2af".to_string()),
            blue: colors
                .get("color4")
                .cloned()
                .unwrap_or_else(|| "#89b4fa".to_string()),
            magenta: colors
                .get("color5")
                .cloned()
                .unwrap_or_else(|| "#f5c2e7".to_string()),
            cyan: colors
                .get("color6")
                .cloned()
                .unwrap_or_else(|| "#94e2d5".to_string()),
            white: colors
                .get("color7")
                .cloned()
                .unwrap_or_else(|| "#bac2de".to_string()),
        },
    })
}

/// Parse colors from an alacritty.toml file
pub fn parse_alacritty_toml(path: &Path) -> Option<ThemeColors> {
    let content = fs::read_to_string(path).ok()?;
    let mut colors: HashMap<String, String> = HashMap::new();
    let mut background = None;
    let mut foreground = None;
    let mut current_section = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();

            match current_section.as_str() {
                "colors.primary" => {
                    if key == "background" {
                        background = Some(value);
                    } else if key == "foreground" {
                        foreground = Some(value);
                    }
                }
                "colors.normal" => {
                    colors.insert(key.to_string(), value);
                }
                _ => {}
            }
        }
    }

    Some(ThemeColors {
        primary: PrimaryColors {
            background: background.unwrap_or_else(|| "#1e1e2e".to_string()),
            foreground: foreground.unwrap_or_else(|| "#cdd6f4".to_string()),
        },
        terminal: TerminalColors {
            black: colors
                .get("black")
                .cloned()
                .unwrap_or_else(|| "#45475a".to_string()),
            red: colors
                .get("red")
                .cloned()
                .unwrap_or_else(|| "#f38ba8".to_string()),
            green: colors
                .get("green")
                .cloned()
                .unwrap_or_else(|| "#a6e3a1".to_string()),
            yellow: colors
                .get("yellow")
                .cloned()
                .unwrap_or_else(|| "#f9e2af".to_string()),
            blue: colors
                .get("blue")
                .cloned()
                .unwrap_or_else(|| "#89b4fa".to_string()),
            magenta: colors
                .get("magenta")
                .cloned()
                .unwrap_or_else(|| "#f5c2e7".to_string()),
            cyan: colors
                .get("cyan")
                .cloned()
                .unwrap_or_else(|| "#94e2d5".to_string()),
            white: colors
                .get("white")
                .cloned()
                .unwrap_or_else(|| "#bac2de".to_string()),
        },
    })
}
