use std::fs;

use crate::types::themes::{ColorsConfig, TerminalConfig};

use super::paths::get_custom_themes_dir;

pub fn update_colors_toml(theme_name: &str, colors: &ColorsConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let toml_content = format!(
        r#"accent = "{}"
cursor = "{}"
foreground = "{}"
background = "{}"
selection_foreground = "{}"
selection_background = "{}"

color0 = "{}"
color1 = "{}"
color2 = "{}"
color3 = "{}"
color4 = "{}"
color5 = "{}"
color6 = "{}"
color7 = "{}"
color8 = "{}"
color9 = "{}"
color10 = "{}"
color11 = "{}"
color12 = "{}"
color13 = "{}"
color14 = "{}"
color15 = "{}"
"#,
        colors.accent,
        colors.cursor,
        colors.foreground,
        colors.background,
        colors.selection_foreground,
        colors.selection_background,
        colors.color0,
        colors.color1,
        colors.color2,
        colors.color3,
        colors.color4,
        colors.color5,
        colors.color6,
        colors.color7,
        colors.color8,
        colors.color9,
        colors.color10,
        colors.color11,
        colors.color12,
        colors.color13,
        colors.color14,
        colors.color15,
    );

    let toml_path = theme_dir.join("colors.toml");
    fs::write(&toml_path, toml_content)
        .map_err(|e| format!("Failed to write colors.toml: {}", e))?;

    Ok(())
}

pub fn colors_config_from_terminal(terminal: &TerminalConfig, accent: &str) -> ColorsConfig {
    ColorsConfig {
        accent: accent.to_string(),
        cursor: terminal.cursor.cursor.clone(),
        foreground: terminal.primary.foreground.clone(),
        background: terminal.primary.background.clone(),
        selection_foreground: terminal.selection.foreground.clone(),
        selection_background: terminal.selection.background.clone(),
        color0: terminal.normal.black.clone(),
        color1: terminal.normal.red.clone(),
        color2: terminal.normal.green.clone(),
        color3: terminal.normal.yellow.clone(),
        color4: terminal.normal.blue.clone(),
        color5: terminal.normal.magenta.clone(),
        color6: terminal.normal.cyan.clone(),
        color7: terminal.normal.white.clone(),
        color8: terminal.bright.black.clone(),
        color9: terminal.bright.red.clone(),
        color10: terminal.bright.green.clone(),
        color11: terminal.bright.yellow.clone(),
        color12: terminal.bright.blue.clone(),
        color13: terminal.bright.magenta.clone(),
        color14: terminal.bright.cyan.clone(),
        color15: terminal.bright.white.clone(),
    }
}
