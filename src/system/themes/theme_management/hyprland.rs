use std::fs;

use crate::types::themes::HyprlandConfig;

use super::paths::get_custom_themes_dir;

pub(super) fn parse_hyprland_conf(conf_content: &str) -> Option<HyprlandConfig> {
    let mut active_border = None;
    let mut inactive_border = None;
    let mut in_general_section = false;

    for line in conf_content.lines() {
        let trimmed = line.trim();

        if trimmed == "general {" {
            in_general_section = true;
            continue;
        }
        if trimmed == "}" {
            in_general_section = false;
            continue;
        }

        if in_general_section {
            if let Some(value) = trimmed.strip_prefix("col.active_border = rgb(")
                && let Some(end) = value.find(')')
            {
                active_border = Some(value[..end].to_string());
            } else if let Some(value) = trimmed.strip_prefix("col.inactive_border = rgb(")
                && let Some(end) = value.find(')')
            {
                inactive_border = Some(value[..end].to_string());
            }
        }
    }

    if active_border.is_some() || inactive_border.is_some() {
        Some(HyprlandConfig {
            active_border: active_border.unwrap_or_else(|| "6e6e92".to_string()),
            inactive_border: inactive_border.unwrap_or_else(|| "5C5C5E".to_string()),
        })
    } else {
        None
    }
}

pub fn update_hyprland_conf(theme_name: &str, config: &HyprlandConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let conf_content = format!(
        "general {{\n    col.active_border = rgb({})\n    col.inactive_border = rgb({})\n}}\n",
        config.active_border, config.inactive_border
    );

    let conf_path = theme_dir.join("hyprland.conf");
    fs::write(&conf_path, conf_content)
        .map_err(|e| format!("Failed to write hyprland.conf: {}", e))?;

    Ok(())
}
