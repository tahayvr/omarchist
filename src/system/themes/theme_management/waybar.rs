use std::fs;

use crate::types::themes::WaybarConfig;

use super::paths::get_custom_themes_dir;

pub(super) fn parse_waybar_css(css_content: &str) -> Option<WaybarConfig> {
    let mut background = None;
    let mut foreground = None;

    for line in css_content.lines() {
        let line = line.trim();
        if line.starts_with("@define-color background") {
            if let Some(color) = line.split_whitespace().nth(2) {
                background = Some(color.trim_end_matches(';').to_string());
            }
        } else if line.starts_with("@define-color foreground")
            && let Some(color) = line.split_whitespace().nth(2)
        {
            foreground = Some(color.trim_end_matches(';').to_string());
        }
    }

    if let (Some(bg), Some(fg)) = (background, foreground) {
        Some(WaybarConfig {
            background: bg,
            foreground: fg,
        })
    } else {
        None
    }
}

pub fn update_waybar_css(theme_name: &str, config: &WaybarConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let css_content = format!(
        "@define-color background {};\n@define-color foreground {};\n",
        config.background, config.foreground
    );

    let css_path = theme_dir.join("waybar.css");
    fs::write(&css_path, css_content).map_err(|e| format!("Failed to write waybar.css: {}", e))?;

    Ok(())
}
