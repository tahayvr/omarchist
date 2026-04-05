use std::fs;

use crate::types::themes::SwayosdConfig;

use super::paths::get_custom_themes_dir;

pub(super) fn parse_swayosd_css(css_content: &str) -> Option<SwayosdConfig> {
    let mut background_color = None;
    let mut border_color = None;
    let mut label = None;
    let mut image = None;
    let mut progress = None;

    for line in css_content.lines() {
        let trimmed = line.trim();

        if let Some(value) = trimmed.strip_prefix("@define-color background-color ") {
            background_color = Some(value.trim_end_matches(';').to_string());
        } else if let Some(value) = trimmed.strip_prefix("@define-color border-color ") {
            border_color = Some(value.trim_end_matches(';').to_string());
        } else if let Some(value) = trimmed.strip_prefix("@define-color label ") {
            label = Some(value.trim_end_matches(';').to_string());
        } else if let Some(value) = trimmed.strip_prefix("@define-color image ") {
            image = Some(value.trim_end_matches(';').to_string());
        } else if let Some(value) = trimmed.strip_prefix("@define-color progress ") {
            progress = Some(value.trim_end_matches(';').to_string());
        }
    }

    if background_color.is_some()
        || border_color.is_some()
        || label.is_some()
        || image.is_some()
        || progress.is_some()
    {
        Some(SwayosdConfig {
            background_color: background_color.unwrap_or_else(|| "#0F0F19".to_string()),
            border_color: border_color.unwrap_or_else(|| "#33A1FF".to_string()),
            label: label.unwrap_or_else(|| "#8A8A8D".to_string()),
            image: image.unwrap_or_else(|| "#8A8A8D".to_string()),
            progress: progress.unwrap_or_else(|| "#8A8A8D".to_string()),
        })
    } else {
        None
    }
}

pub fn update_swayosd_css(theme_name: &str, config: &SwayosdConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let css_content = format!(
        "@define-color background-color {};\n@define-color border-color {};\n@define-color label {};\n@define-color image {};\n@define-color progress {};\n",
        config.background_color, config.border_color, config.label, config.image, config.progress
    );

    let css_path = theme_dir.join("swayosd.css");
    fs::write(&css_path, css_content).map_err(|e| format!("Failed to write swayosd.css: {}", e))?;

    Ok(())
}
