use std::fs;

use crate::types::themes::HyprlockConfig;

use super::paths::get_custom_themes_dir;

pub(super) fn parse_hyprlock_conf(conf_content: &str) -> Option<HyprlockConfig> {
    let mut color = None;
    let mut inner_color = None;
    let mut outer_color = None;
    let mut font_color = None;
    let mut check_color = None;

    for line in conf_content.lines() {
        let trimmed = line.trim();

        if let Some(value) = trimmed.strip_prefix("$color = rgb(") {
            if let Some(end) = value.find(')') {
                color = Some(value[..end].to_string());
            }
        } else if let Some(value) = trimmed.strip_prefix("$inner_color = rgb(") {
            if let Some(end) = value.find(')') {
                inner_color = Some(value[..end].to_string());
            }
        } else if let Some(value) = trimmed.strip_prefix("$outer_color = rgb(") {
            if let Some(end) = value.find(')') {
                outer_color = Some(value[..end].to_string());
            }
        } else if let Some(value) = trimmed.strip_prefix("$font_color = rgb(") {
            if let Some(end) = value.find(')') {
                font_color = Some(value[..end].to_string());
            }
        } else if let Some(value) = trimmed.strip_prefix("$check_color = rgb(")
            && let Some(end) = value.find(')')
        {
            check_color = Some(value[..end].to_string());
        }
    }

    if color.is_some()
        || inner_color.is_some()
        || outer_color.is_some()
        || font_color.is_some()
        || check_color.is_some()
    {
        Some(HyprlockConfig {
            color: color.unwrap_or_else(|| "0f0f19".to_string()),
            inner_color: inner_color.unwrap_or_else(|| "0f0f19".to_string()),
            outer_color: outer_color.unwrap_or_else(|| "33a0ff".to_string()),
            font_color: font_color.unwrap_or_else(|| "ff66f5".to_string()),
            check_color: check_color.unwrap_or_else(|| "ffea00".to_string()),
        })
    } else {
        None
    }
}

pub fn update_hyprlock_conf(theme_name: &str, config: &HyprlockConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let conf_content = format!(
        "$color = rgb({})\n$inner_color = rgb({})\n$outer_color = rgb({})\n$font_color = rgb({})\n$check_color = rgb({})\n",
        config.color, config.inner_color, config.outer_color, config.font_color, config.check_color
    );

    let conf_path = theme_dir.join("hyprlock.conf");
    fs::write(&conf_path, conf_content)
        .map_err(|e| format!("Failed to write hyprlock.conf: {}", e))?;

    Ok(())
}
