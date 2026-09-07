use std::fs;

use crate::types::themes::LockScreenConfig;

use super::paths::get_custom_themes_dir;

pub(super) fn parse_lock_toml(content: &str) -> Option<LockScreenConfig> {
    let mut text = None;
    let mut placeholder = None;
    let mut text_error = None;
    let mut border = None;
    let mut border_active = None;
    let mut border_error = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').to_string();

            match key {
                "text" => text = Some(value),
                "placeholder" => placeholder = Some(value),
                "text-error" => text_error = Some(value),
                "border" => border = Some(value),
                "border-active" => border_active = Some(value),
                "border-error" => border_error = Some(value),
                _ => {}
            }
        }
    }

    if text.is_some() || placeholder.is_some() || border.is_some() {
        Some(LockScreenConfig {
            text: text.unwrap_or_else(|| "#EDEDFE".to_string()),
            placeholder: placeholder.unwrap_or_else(|| "#EDEDFE".to_string()),
            text_error: text_error.unwrap_or_else(|| "#FF3366".to_string()),
            border: border.unwrap_or_else(|| "#33A1FF".to_string()),
            border_active: border_active.unwrap_or_else(|| "#33A1FF".to_string()),
            border_error: border_error.unwrap_or_else(|| "#FF3366".to_string()),
        })
    } else {
        None
    }
}

// Writes `shell.lock.toml` — confirmed key set and format against a real
// theme in omacom/omarchy@quattro (themes/tokyo-night/shell.lock.toml).
pub fn update_lock_toml(theme_name: &str, config: &LockScreenConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let toml_content = format!(
        "text             = \"{}\"\nplaceholder      = \"{}\"\ntext-error       = \"{}\"\nborder           = \"{}\"\nborder-active    = \"{}\"\nborder-error     = \"{}\"\n",
        config.text,
        config.placeholder,
        config.text_error,
        config.border,
        config.border_active,
        config.border_error,
    );

    let toml_path = theme_dir.join("shell.lock.toml");
    fs::write(&toml_path, toml_content)
        .map_err(|e| format!("Failed to write shell.lock.toml: {}", e))?;

    Ok(())
}
