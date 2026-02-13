use crate::types::themes::{
    BrowserConfig, EditingTheme, HyprlandConfig, HyprlockConfig, TerminalConfig, WalkerConfig,
    WaybarConfig,
};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

/// Get the custom themes directory path
fn get_custom_themes_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("omarchy").join("themes"))
}

/// Get the defaults theme directory path
fn get_defaults_theme_dir() -> PathBuf {
    PathBuf::from("defaults/theme")
}

/// Generate a unique theme name like "custom-theme-1", "custom-theme-2", etc.
pub fn generate_unique_theme_name() -> String {
    let themes_dir = match get_custom_themes_dir() {
        Some(dir) => dir,
        None => return format!("custom-theme-{}", Utc::now().timestamp()),
    };

    let base_name = "custom-theme";
    let mut counter = 1;

    loop {
        let name = format!("{}-{}", base_name, counter);
        let theme_path = themes_dir.join(&name);

        if !theme_path.exists() {
            return name;
        }

        counter += 1;

        // Safety check to prevent infinite loops
        if counter > 1000 {
            return format!("custom-theme-{}", Utc::now().timestamp());
        }
    }
}

/// Create a new theme by copying the defaults/theme folder
pub fn create_theme_from_defaults(theme_name: &str) -> Result<String, String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let defaults_dir = get_defaults_theme_dir();
    let new_theme_dir = themes_dir.join(theme_name);

    // Check if theme already exists
    if new_theme_dir.exists() {
        return Err(format!("Theme '{}' already exists", theme_name));
    }

    // Create the theme directory
    fs::create_dir_all(&new_theme_dir)
        .map_err(|e| format!("Failed to create theme directory: {}", e))?;

    // Copy all files from defaults/theme to the new theme directory
    copy_theme_files(&defaults_dir, &new_theme_dir)?;

    // Update custom_theme.json with the theme name and timestamps
    update_theme_metadata(&new_theme_dir, theme_name)?;

    Ok(theme_name.to_string())
}

/// Copy all files from source to destination directory
fn copy_theme_files(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.exists() {
        return Err(format!(
            "Source directory does not exist: {}",
            src.display()
        ));
    }

    let entries =
        fs::read_dir(src).map_err(|e| format!("Failed to read source directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .ok_or_else(|| "Invalid file name".to_string())?;
        let dest_path = dst.join(file_name);

        if path.is_dir() {
            // Recursively copy subdirectories (like backgrounds/)
            fs::create_dir_all(&dest_path)
                .map_err(|e| format!("Failed to create subdirectory: {}", e))?;
            copy_theme_files(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)
                .map_err(|e| format!("Failed to copy file {}: {}", path.display(), e))?;
        }
    }

    Ok(())
}

/// Update the custom_theme.json with actual metadata
fn update_theme_metadata(theme_dir: &Path, theme_name: &str) -> Result<(), String> {
    let json_path = theme_dir.join("custom_theme.json");

    if !json_path.exists() {
        return Ok(()); // File doesn't exist, skip
    }

    let now = Utc::now().to_rfc3339();

    let content = fs::read_to_string(&json_path)
        .map_err(|e| format!("Failed to read custom_theme.json: {}", e))?;

    let updated_content = content
        .replace("{{THEME_NAME}}", theme_name)
        .replace("{{CREATED_AT}}", &now)
        .replace("{{MODIFIED_AT}}", &now)
        .replace("{{AUTHOR}}", "");

    fs::write(&json_path, updated_content)
        .map_err(|e| format!("Failed to write custom_theme.json: {}", e))?;

    Ok(())
}

/// Load a theme for editing
pub fn load_theme_for_editing(theme_name: &str) -> Result<EditingTheme, String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    // Read custom_theme.json
    let json_path = theme_dir.join("custom_theme.json");
    let mut editing_theme: EditingTheme = if json_path.exists() {
        let content = fs::read_to_string(&json_path)
            .map_err(|e| format!("Failed to read custom_theme.json: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse custom_theme.json: {}", e))?
    } else {
        EditingTheme::default()
    };

    // Check for light.mode file
    let light_mode_path = theme_dir.join("light.mode");
    editing_theme.is_light_theme = light_mode_path.exists();

    // Load waybar.css colors if the file exists
    let waybar_css_path = theme_dir.join("waybar.css");
    if waybar_css_path.exists() {
        if let Ok(css_content) = fs::read_to_string(&waybar_css_path) {
            if let Some(config) = parse_waybar_css(&css_content) {
                editing_theme.apps.waybar = Some(config);
            }
        }
    }

    // Load hyprland.conf settings if the file exists
    let hyprland_conf_path = theme_dir.join("hyprland.conf");
    if hyprland_conf_path.exists() {
        if let Ok(conf_content) = fs::read_to_string(&hyprland_conf_path) {
            if let Some(config) = parse_hyprland_conf(&conf_content) {
                editing_theme.apps.hyprland = Some(config);
            }
        }
    }

    // Load icons.theme if the file exists
    let icons_theme_path = theme_dir.join("icons.theme");
    if icons_theme_path.exists() {
        if let Ok(content) = fs::read_to_string(&icons_theme_path) {
            if let Some(icons_config) = parse_icons_theme(&content) {
                editing_theme.apps.icons = Some(icons_config);
            }
        }
    }

    // Load hyprlock.conf if the file exists
    let hyprlock_conf_path = theme_dir.join("hyprlock.conf");
    if hyprlock_conf_path.exists() {
        if let Ok(conf_content) = fs::read_to_string(&hyprlock_conf_path) {
            if let Some(config) = parse_hyprlock_conf(&conf_content) {
                editing_theme.apps.hyprlock = Some(config);
            }
        }
    }

    Ok(editing_theme)
}

/// Parse waybar.css content to extract color values
fn parse_waybar_css(css_content: &str) -> Option<WaybarConfig> {
    let mut background = None;
    let mut foreground = None;

    for line in css_content.lines() {
        let line = line.trim();
        if line.starts_with("@define-color background") {
            // Extract color value after the space
            if let Some(color) = line.split_whitespace().nth(2) {
                background = Some(color.to_string());
            }
        } else if line.starts_with("@define-color foreground") {
            // Extract color value after the space
            if let Some(color) = line.split_whitespace().nth(2) {
                foreground = Some(color.to_string());
            }
        }
    }

    // Return config only if we found both colors
    if let (Some(bg), Some(fg)) = (background, foreground) {
        Some(WaybarConfig {
            background: bg,
            foreground: fg,
        })
    } else {
        None
    }
}

/// Save theme data
pub fn save_theme_data(theme_name: &str, theme_data: &EditingTheme) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    // Update modified_at timestamp
    let mut updated_theme = theme_data.clone();
    updated_theme.modified_at = Utc::now().to_rfc3339();

    // Write to custom_theme.json
    let json_path = theme_dir.join("custom_theme.json");
    let json_content = serde_json::to_string_pretty(&updated_theme)
        .map_err(|e| format!("Failed to serialize theme data: {}", e))?;

    fs::write(&json_path, json_content)
        .map_err(|e| format!("Failed to write custom_theme.json: {}", e))?;

    // Manage light.mode file
    update_light_mode_file(&theme_dir, theme_data.is_light_theme)?;

    // Update individual app config files based on theme_data.apps content
    if let Some(ref waybar_config) = theme_data.apps.waybar {
        update_waybar_css(theme_name, waybar_config)?;
    }

    if let Some(ref hyprland_config) = theme_data.apps.hyprland {
        update_hyprland_conf(theme_name, hyprland_config)?;
    }

    if let Some(ref walker_config) = theme_data.apps.walker {
        update_walker_css(theme_name, walker_config)?;
    }

    if let Some(ref terminal_config) = theme_data.apps.terminal {
        update_terminal_configs(theme_name, terminal_config)?;
    }

    if let Some(ref chromium_config) = theme_data.apps.chromium {
        update_chromium_config(theme_name, chromium_config)?;
    }

    if let Some(ref hyprlock_config) = theme_data.apps.hyprlock {
        update_hyprlock_conf(theme_name, hyprlock_config)?;
    }

    // Update icons.theme if icons config exists
    if let Some(ref icons_config) = theme_data.apps.icons {
        if let Some(theme_name_val) = icons_config.get("theme_name").and_then(|v| v.as_str()) {
            update_icons_theme(theme_name, theme_name_val)?;
        }
    }

    Ok(())
}

/// Update the icons.theme file with the given icon theme name
pub fn update_icons_theme(theme_name: &str, icon_theme_name: &str) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    // Write the icon theme name to icons.theme
    let icons_theme_path = theme_dir.join("icons.theme");
    fs::write(&icons_theme_path, format!("{}\n", icon_theme_name))
        .map_err(|e| format!("Failed to write icons.theme: {}", e))?;

    Ok(())
}

/// Parse icons.theme file content to extract icon theme name
fn parse_icons_theme(content: &str) -> Option<serde_json::Value> {
    let theme_name = content.trim();
    if theme_name.is_empty() {
        return None;
    }

    Some(serde_json::json!({
        "theme_name": theme_name
    }))
}

/// Update the chromium config file with the given theme color
pub fn update_chromium_config(theme_name: &str, config: &BrowserConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    // Convert hex color to RGB format
    let hex = config.theme_color.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(15);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(15);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(25);

    let theme_content = format!("{},{},{}\n", r, g, b);

    // Write to chromium.theme
    let theme_path = theme_dir.join("chromium.theme");
    fs::write(&theme_path, theme_content)
        .map_err(|e| format!("Failed to write chromium.theme: {}", e))?;

    Ok(())
}

/// Helper function to create or remove light.mode file
fn update_light_mode_file(theme_dir: &Path, is_light: bool) -> Result<(), String> {
    let light_mode_path = theme_dir.join("light.mode");

    if is_light {
        // Create light.mode file if it doesn't exist
        if !light_mode_path.exists() {
            fs::write(&light_mode_path, "") // Empty file
                .map_err(|e| format!("Failed to create light.mode file: {}", e))?;
        }
    } else {
        // Remove light.mode file if it exists
        if light_mode_path.exists() {
            fs::remove_file(&light_mode_path)
                .map_err(|e| format!("Failed to remove light.mode file: {}", e))?;
        }
    }

    Ok(())
}

/// Rename a theme
pub fn rename_theme(old_name: &str, new_name: &str) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let old_path = themes_dir.join(old_name);
    let new_path = themes_dir.join(new_name);

    if !old_path.exists() {
        return Err(format!("Theme '{}' not found", old_name));
    }

    if new_path.exists() {
        return Err(format!("Theme '{}' already exists", new_name));
    }

    fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename theme: {}", e))?;

    // Update the name in custom_theme.json
    let json_path = new_path.join("custom_theme.json");
    if json_path.exists() {
        let content = fs::read_to_string(&json_path)
            .map_err(|e| format!("Failed to read custom_theme.json: {}", e))?;
        let mut theme: EditingTheme = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse custom_theme.json: {}", e))?;
        theme.name = new_name.to_string();
        theme.modified_at = Utc::now().to_rfc3339();

        let updated_content = serde_json::to_string_pretty(&theme)
            .map_err(|e| format!("Failed to serialize theme data: {}", e))?;
        fs::write(&json_path, updated_content)
            .map_err(|e| format!("Failed to write custom_theme.json: {}", e))?;
    }

    Ok(())
}

/// Update the waybar.css file with the given colors
pub fn update_waybar_css(theme_name: &str, config: &WaybarConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    // Generate the CSS content
    let css_content = format!(
        "@define-color background {};\n@define-color foreground {};\n",
        config.background, config.foreground
    );

    // Write to waybar.css
    let css_path = theme_dir.join("waybar.css");
    fs::write(&css_path, css_content).map_err(|e| format!("Failed to write waybar.css: {}", e))?;

    Ok(())
}

/// Parse hyprland.conf content to extract window settings
fn parse_hyprland_conf(conf_content: &str) -> Option<HyprlandConfig> {
    let mut active_border = None;
    let mut inactive_border = None;
    let mut border_size = None;
    let mut gaps_in = None;
    let mut gaps_out = None;
    let mut rounding = None;

    let mut in_general_section = false;
    let mut in_decoration_section = false;

    for line in conf_content.lines() {
        let trimmed = line.trim();

        // Check for section headers
        if trimmed == "general {" {
            in_general_section = true;
            continue;
        }
        if trimmed == "decoration {" {
            in_decoration_section = true;
            continue;
        }
        if trimmed == "}" {
            in_general_section = false;
            in_decoration_section = false;
            continue;
        }

        // Parse general section
        if in_general_section {
            if let Some(value) = trimmed.strip_prefix("col.active_border = rgb(") {
                if let Some(end) = value.find(')') {
                    active_border = Some(value[..end].to_string());
                }
            } else if let Some(value) = trimmed.strip_prefix("col.inactive_border = rgb(") {
                if let Some(end) = value.find(')') {
                    inactive_border = Some(value[..end].to_string());
                }
            } else if let Some(value) = trimmed.strip_prefix("border_size = ") {
                border_size = value.parse::<i32>().ok();
            } else if let Some(value) = trimmed.strip_prefix("gaps_in = ") {
                gaps_in = value.parse::<i32>().ok();
            } else if let Some(value) = trimmed.strip_prefix("gaps_out = ") {
                gaps_out = value.parse::<i32>().ok();
            }
        }

        // Parse decoration section
        if in_decoration_section {
            if let Some(value) = trimmed.strip_prefix("rounding = ") {
                rounding = value.parse::<i32>().ok();
            }
        }
    }

    // Create config if we have at least some values
    if active_border.is_some()
        || inactive_border.is_some()
        || border_size.is_some()
        || gaps_in.is_some()
        || gaps_out.is_some()
        || rounding.is_some()
    {
        Some(HyprlandConfig {
            active_border: active_border.unwrap_or_else(|| "6e6e92".to_string()),
            inactive_border: inactive_border.unwrap_or_else(|| "5C5C5E".to_string()),
            border_size: border_size.unwrap_or(1),
            gaps_in: gaps_in.unwrap_or(5),
            gaps_out: gaps_out.unwrap_or(10),
            rounding: rounding.unwrap_or(0),
        })
    } else {
        None
    }
}

/// Update the hyprland.conf file with the given settings
pub fn update_hyprland_conf(theme_name: &str, config: &HyprlandConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    // Generate the conf content
    let conf_content = format!(
        "general {{\n    col.active_border = rgb({})\n    col.inactive_border = rgb({})\n    border_size = {}\n    gaps_in = {}\n    gaps_out = {}\n}}\n\ndecoration {{\n    rounding = {}\n}}\n",
        config.active_border,
        config.inactive_border,
        config.border_size,
        config.gaps_in,
        config.gaps_out,
        config.rounding
    );

    // Write to hyprland.conf
    let conf_path = theme_dir.join("hyprland.conf");
    fs::write(&conf_path, conf_content)
        .map_err(|e| format!("Failed to write hyprland.conf: {}", e))?;

    Ok(())
}

/// Update the walker.css file with the given colors
pub fn update_walker_css(theme_name: &str, config: &WalkerConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    // Generate the CSS content
    let css_content = format!(
        "@define-color selected-text {};\n@define-color text {};\n@define-color base {};\n@define-color border {};\n@define-color foreground {};\n@define-color background {};\n",
        config.selected_text,
        config.text,
        config.base,
        config.border,
        config.foreground,
        config.background
    );

    // Write to walker.css
    let css_path = theme_dir.join("walker.css");
    fs::write(&css_path, css_content).map_err(|e| format!("Failed to write walker.css: {}", e))?;

    Ok(())
}

/// Update all terminal emulator configs from unified terminal config
pub fn update_terminal_configs(theme_name: &str, config: &TerminalConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    // Generate and write alacritty.toml
    let alacritty_content = generate_alacritty_config(config);
    let alacritty_path = theme_dir.join("alacritty.toml");
    fs::write(&alacritty_path, alacritty_content)
        .map_err(|e| format!("Failed to write alacritty.toml: {}", e))?;

    // Generate and write kitty.conf
    let kitty_content = generate_kitty_config(config);
    let kitty_path = theme_dir.join("kitty.conf");
    fs::write(&kitty_path, kitty_content)
        .map_err(|e| format!("Failed to write kitty.conf: {}", e))?;

    // Generate and write ghostty.conf
    let ghostty_content = generate_ghostty_config(config);
    let ghostty_path = theme_dir.join("ghostty.conf");
    fs::write(&ghostty_path, ghostty_content)
        .map_err(|e| format!("Failed to write ghostty.conf: {}", e))?;

    Ok(())
}

/// Generate Alacritty TOML config
fn generate_alacritty_config(config: &TerminalConfig) -> String {
    format!(
        r#"# ────────────────────────────────────────────────────────────
# Omarchy Custom Theme for Alacritty
# Generated with Omarchist
# ────────────────────────────────────────────────────────────

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

/// Generate Kitty config
fn generate_kitty_config(config: &TerminalConfig) -> String {
    format!(
        r#"# ────────────────────────────────────────────────────────────
# Custom Theme for Kitty
# Made with Omarchist
# ────────────────────────────────────────────────────────────

## name: Custom Theme

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

/// Generate Ghostty config
fn generate_ghostty_config(config: &TerminalConfig) -> String {
    format!(
        r#"# ────────────────────────────────────────────────────────────
# Custom Theme for Ghostty
# Made by Omarchist
# ────────────────────────────────────────────────────────────

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

/// Parse hyprlock.conf content to extract color values
fn parse_hyprlock_conf(conf_content: &str) -> Option<HyprlockConfig> {
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
        } else if let Some(value) = trimmed.strip_prefix("$check_color = rgb(") {
            if let Some(end) = value.find(')') {
                check_color = Some(value[..end].to_string());
            }
        }
    }

    // Create config if we have at least some values
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

/// Update the hyprlock.conf file with the given settings
pub fn update_hyprlock_conf(theme_name: &str, config: &HyprlockConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    // Generate the conf content
    let conf_content = format!(
        "$color = rgb({})\n$inner_color = rgb({})\n$outer_color = rgb({})\n$font_color = rgb({})\n$check_color = rgb({})\n",
        config.color,
        config.inner_color,
        config.outer_color,
        config.font_color,
        config.check_color
    );

    // Write to hyprlock.conf
    let conf_path = theme_dir.join("hyprlock.conf");
    fs::write(&conf_path, conf_content)
        .map_err(|e| format!("Failed to write hyprlock.conf: {}", e))?;

    Ok(())
}
