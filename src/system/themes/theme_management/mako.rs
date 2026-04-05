use std::fs;

use crate::types::themes::MakoConfig;

use super::paths::get_custom_themes_dir;

pub(super) fn parse_mako_ini(ini_content: &str) -> Option<MakoConfig> {
    let mut text_color = None;
    let mut border_color = None;
    let mut background_color = None;

    for line in ini_content.lines() {
        let trimmed = line.trim();

        // Only parse the global section (before any [section] headers)
        if trimmed.starts_with('[') {
            break;
        }

        if let Some(value) = trimmed.strip_prefix("text-color=") {
            text_color = Some(value.to_string());
        } else if let Some(value) = trimmed.strip_prefix("border-color=") {
            border_color = Some(value.to_string());
        } else if let Some(value) = trimmed.strip_prefix("background-color=") {
            background_color = Some(value.to_string());
        }
    }

    if text_color.is_some() || border_color.is_some() || background_color.is_some() {
        Some(MakoConfig {
            text_color: text_color.unwrap_or_else(|| "#EDEDFE".to_string()),
            border_color: border_color.unwrap_or_else(|| "#00F59B".to_string()),
            background_color: background_color.unwrap_or_else(|| "#0F0F19".to_string()),
        })
    } else {
        None
    }
}

pub fn update_mako_ini(theme_name: &str, config: &MakoConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let ini_path = theme_dir.join("mako.ini");
    let existing_content = fs::read_to_string(&ini_path).unwrap_or_default();

    // Preserve non-color global settings and all [section] blocks
    let mut preserved_lines = Vec::new();
    let mut in_preserve_section = false;
    let mut colors_found = false;

    for line in existing_content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            in_preserve_section = true;
        }

        let is_color_line = trimmed.starts_with("text-color=")
            || trimmed.starts_with("border-color=")
            || trimmed.starts_with("background-color=");

        if in_preserve_section || (!is_color_line && !trimmed.is_empty()) {
            preserved_lines.push(line.to_string());
        } else if is_color_line {
            colors_found = true;
        }
    }

    let mut new_content = format!(
        "text-color={}\nborder-color={}\nbackground-color={}\n",
        config.text_color, config.border_color, config.background_color
    );

    if !preserved_lines.is_empty() {
        if colors_found {
            new_content.push('\n');
        }
        new_content.push_str(&preserved_lines.join("\n"));
    }

    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    fs::write(&ini_path, new_content).map_err(|e| format!("Failed to write mako.ini: {}", e))?;

    Ok(())
}
