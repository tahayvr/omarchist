use std::fs;

use crate::types::themes::BrowserConfig;

use super::paths::get_custom_themes_dir;

pub fn update_chromium_config(theme_name: &str, config: &BrowserConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let hex = config.theme_color.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(15);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(15);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(25);

    let theme_path = theme_dir.join("chromium.theme");
    fs::write(&theme_path, format!("{},{},{}\n", r, g, b))
        .map_err(|e| format!("Failed to write chromium.theme: {}", e))?;

    Ok(())
}
