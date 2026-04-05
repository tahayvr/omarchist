use std::fs;

use crate::types::themes::WalkerConfig;

use super::paths::get_custom_themes_dir;

pub fn update_walker_css(theme_name: &str, config: &WalkerConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let css_content = format!(
        "@define-color selected-text {};\n@define-color text {};\n@define-color base {};\n@define-color border {};\n@define-color foreground {};\n@define-color background {};\n",
        config.selected_text,
        config.text,
        config.base,
        config.border,
        config.foreground,
        config.background
    );

    let css_path = theme_dir.join("walker.css");
    fs::write(&css_path, css_content).map_err(|e| format!("Failed to write walker.css: {}", e))?;

    Ok(())
}
