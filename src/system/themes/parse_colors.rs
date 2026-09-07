use crate::types::themes::{PrimaryColors, TerminalColors, ThemeColors};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Looks up `key`, falling back to `alias` — mirrors the bidirectional
// color0-15 <-> semantic-name aliasing Omarchy's own `omarchy-theme-color`
// resolver applies, confirmed against the real omacom/omarchy@quattro source.
// Quattro's own official themes (catppuccin, tokyo-night, etc.) use only the
// semantic names with no color0-15 keys at all, so without this fallback
// every real Quattro theme's preview swatch would silently show wrong colors.
fn lookup<'a>(colors: &'a HashMap<String, String>, key: &str, alias: &str) -> Option<&'a String> {
    colors.get(key).or_else(|| colors.get(alias))
}

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

    let background = lookup(&colors, "background", "color0");
    let foreground = lookup(&colors, "foreground", "color7");

    Some(ThemeColors {
        primary: PrimaryColors {
            background: background.cloned().unwrap_or_else(|| "#1e1e2e".to_string()),
            foreground: foreground.cloned().unwrap_or_else(|| "#cdd6f4".to_string()),
        },
        terminal: TerminalColors {
            black: lookup(&colors, "color0", "background")
                .or(background)
                .cloned()
                .unwrap_or_else(|| "#45475a".to_string()),
            red: lookup(&colors, "color1", "red")
                .cloned()
                .unwrap_or_else(|| "#f38ba8".to_string()),
            green: lookup(&colors, "color2", "green")
                .cloned()
                .unwrap_or_else(|| "#a6e3a1".to_string()),
            yellow: lookup(&colors, "color3", "yellow")
                .cloned()
                .unwrap_or_else(|| "#f9e2af".to_string()),
            blue: lookup(&colors, "color4", "blue")
                .cloned()
                .unwrap_or_else(|| "#89b4fa".to_string()),
            magenta: colors
                .get("color5")
                .or_else(|| colors.get("magenta"))
                .or_else(|| colors.get("purple"))
                .cloned()
                .unwrap_or_else(|| "#f5c2e7".to_string()),
            cyan: lookup(&colors, "color6", "cyan")
                .cloned()
                .unwrap_or_else(|| "#94e2d5".to_string()),
            white: lookup(&colors, "color7", "foreground")
                .or(foreground)
                .cloned()
                .unwrap_or_else(|| "#bac2de".to_string()),
        },
    })
}

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
