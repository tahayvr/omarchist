use std::fs;

use super::paths::get_custom_themes_dir;

pub(super) fn parse_icons_theme(content: &str) -> Option<serde_json::Value> {
    let theme_name = content.trim();
    if theme_name.is_empty() {
        return None;
    }
    Some(serde_json::json!({ "theme_name": theme_name }))
}

pub fn update_icons_theme(theme_name: &str, icon_theme_name: &str) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let icons_theme_path = theme_dir.join("icons.theme");
    fs::write(&icons_theme_path, format!("{}\n", icon_theme_name))
        .map_err(|e| format!("Failed to write icons.theme: {}", e))?;

    Ok(())
}
