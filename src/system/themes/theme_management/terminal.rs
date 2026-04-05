use std::fs;

use crate::types::themes::TerminalConfig;

use super::paths::get_custom_themes_dir;

pub fn update_terminal_configs(theme_name: &str, config: &TerminalConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let alacritty_path = theme_dir.join("alacritty.toml");
    fs::write(&alacritty_path, generate_alacritty_config(config))
        .map_err(|e| format!("Failed to write alacritty.toml: {}", e))?;

    let kitty_path = theme_dir.join("kitty.conf");
    fs::write(&kitty_path, generate_kitty_config(config))
        .map_err(|e| format!("Failed to write kitty.conf: {}", e))?;

    let ghostty_path = theme_dir.join("ghostty.conf");
    fs::write(&ghostty_path, generate_ghostty_config(config))
        .map_err(|e| format!("Failed to write ghostty.conf: {}", e))?;

    Ok(())
}

fn generate_alacritty_config(config: &TerminalConfig) -> String {
    format!(
        r#"
[colors]
[colors.primary]
background = "{}"
foreground = "{}"

[colors.cursor]
text = "{}"
cursor = "{}"

[colors.selection]
text = "CellForeground"
background = "{}"

[colors.normal]
black = "{}"
red = "{}"
green = "{}"
yellow = "{}"
blue = "{}"
magenta = "{}"
cyan = "{}"
white = "{}"

[colors.bright]
black = "{}"
red = "{}"
green = "{}"
yellow = "{}"
blue = "{}"
magenta = "{}"
cyan = "{}"
white = "{}"
"#,
        config.primary.background,
        config.primary.foreground,
        config.cursor.text,
        config.cursor.cursor,
        config.selection.background,
        config.normal.black,
        config.normal.red,
        config.normal.green,
        config.normal.yellow,
        config.normal.blue,
        config.normal.magenta,
        config.normal.cyan,
        config.normal.white,
        config.bright.black,
        config.bright.red,
        config.bright.green,
        config.bright.yellow,
        config.bright.blue,
        config.bright.magenta,
        config.bright.cyan,
        config.bright.white,
    )
}

fn generate_kitty_config(config: &TerminalConfig) -> String {
    format!(
        r#"
foreground              {}
background              {}
selection_foreground    {}
selection_background    {}

cursor                  {}
cursor_text_color       {}

# URL underline color when hovering with mouse
url_color               {}

# Tab bar colors
active_tab_foreground   {}
active_tab_background   {}
inactive_tab_foreground {}
inactive_tab_background {}
tab_bar_background      {}

color0  {}
color8  {}
color1  {}
color9  {}
color2  {}
color10 {}
color3  {}
color11 {}
color4  {}
color12 {}
color5  {}
color13 {}
color6  {}
color14 {}
color7  {}
color15 {}
"#,
        config.primary.foreground,
        config.primary.background,
        config.selection.foreground,
        config.selection.background,
        config.cursor.cursor,
        config.cursor.text,
        config.primary.foreground,
        config.primary.foreground,
        config.primary.background,
        config.primary.foreground,
        config.primary.background,
        config.primary.foreground,
        config.normal.black,
        config.bright.black,
        config.normal.red,
        config.bright.red,
        config.normal.green,
        config.bright.green,
        config.normal.yellow,
        config.bright.yellow,
        config.normal.blue,
        config.bright.blue,
        config.normal.magenta,
        config.bright.magenta,
        config.normal.cyan,
        config.bright.cyan,
        config.normal.white,
        config.bright.white,
    )
}

fn generate_ghostty_config(config: &TerminalConfig) -> String {
    format!(
        r#"
background = {}
foreground = {}

cursor-color = {}
cursor-text = {}

selection-background = {}
selection-foreground = {}

# normal colors
palette = 0={}
palette = 1={}
palette = 2={}
palette = 3={}
palette = 4={}
palette = 5={}
palette = 6={}
palette = 7={}

# bright colors
palette = 8={}
palette = 9={}
palette = 10={}
palette = 11={}
palette = 12={}
palette = 13={}
palette = 14={}
palette = 15={}
"#,
        config.primary.background,
        config.primary.foreground,
        config.cursor.cursor,
        config.cursor.text,
        config.selection.background,
        config.selection.foreground,
        config.normal.black,
        config.normal.red,
        config.normal.green,
        config.normal.yellow,
        config.normal.blue,
        config.normal.magenta,
        config.normal.cyan,
        config.normal.white,
        config.bright.black,
        config.bright.red,
        config.bright.green,
        config.bright.yellow,
        config.bright.blue,
        config.bright.magenta,
        config.bright.cyan,
        config.bright.white,
    )
}
