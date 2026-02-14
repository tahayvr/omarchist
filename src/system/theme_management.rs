use crate::types::themes::{
    BrowserConfig, BtopConfig, EditingTheme, HyprlandConfig, HyprlockConfig, MakoConfig,
    SwayosdConfig, TerminalConfig, WalkerConfig, WaybarConfig,
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

    // Load mako.ini if the file exists
    let mako_ini_path = theme_dir.join("mako.ini");
    if mako_ini_path.exists() {
        if let Ok(ini_content) = fs::read_to_string(&mako_ini_path) {
            if let Some(config) = parse_mako_ini(&ini_content) {
                editing_theme.apps.mako = Some(config);
            }
        }
    }

    // Load btop.theme if the file exists
    let btop_theme_path = theme_dir.join("btop.theme");
    if btop_theme_path.exists() {
        if let Ok(theme_content) = fs::read_to_string(&btop_theme_path) {
            if let Some(config) = parse_btop_theme(&theme_content) {
                editing_theme.apps.btop = Some(config);
            }
        }
    }

    // Load swayosd.css if the file exists
    let swayosd_css_path = theme_dir.join("swayosd.css");
    if swayosd_css_path.exists() {
        if let Ok(css_content) = fs::read_to_string(&swayosd_css_path) {
            if let Some(config) = parse_swayosd_css(&css_content) {
                editing_theme.apps.swayosd = Some(config);
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
            // Extract color value after the space and strip trailing semicolons
            if let Some(color) = line.split_whitespace().nth(2) {
                background = Some(color.trim_end_matches(';').to_string());
            }
        } else if line.starts_with("@define-color foreground") {
            // Extract color value after the space and strip trailing semicolons
            if let Some(color) = line.split_whitespace().nth(2) {
                foreground = Some(color.trim_end_matches(';').to_string());
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

    if let Some(ref mako_config) = theme_data.apps.mako {
        update_mako_ini(theme_name, mako_config)?;
    }

    if let Some(ref btop_config) = theme_data.apps.btop {
        update_btop_theme(theme_name, btop_config)?;
    }

    if let Some(ref swayosd_config) = theme_data.apps.swayosd {
        update_swayosd_css(theme_name, swayosd_config)?;
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

/// Generate Kitty config
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

/// Generate Ghostty config
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
        config.color, config.inner_color, config.outer_color, config.font_color, config.check_color
    );

    // Write to hyprlock.conf
    let conf_path = theme_dir.join("hyprlock.conf");
    fs::write(&conf_path, conf_content)
        .map_err(|e| format!("Failed to write hyprlock.conf: {}", e))?;

    Ok(())
}

/// Parse mako.ini content to extract color values
fn parse_mako_ini(ini_content: &str) -> Option<MakoConfig> {
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

    // Create config if we have at least some values
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

/// Update the mako.ini file with the given settings
pub fn update_mako_ini(theme_name: &str, config: &MakoConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    // Read existing content to preserve non-color settings and sections
    let ini_path = theme_dir.join("mako.ini");
    let existing_content = fs::read_to_string(&ini_path).unwrap_or_default();

    // Extract non-color settings and sections from existing content
    let mut preserved_lines = Vec::new();
    let mut in_preserve_section = false;
    let mut colors_found = false;

    for line in existing_content.lines() {
        let trimmed = line.trim();

        // Once we hit a section header, preserve everything from there
        if trimmed.starts_with('[') {
            in_preserve_section = true;
        }

        if in_preserve_section {
            preserved_lines.push(line.to_string());
        } else if !trimmed.starts_with("text-color=")
            && !trimmed.starts_with("border-color=")
            && !trimmed.starts_with("background-color=")
            && !trimmed.is_empty()
        {
            // Preserve non-color global settings
            preserved_lines.push(line.to_string());
        } else if trimmed.starts_with("text-color=")
            || trimmed.starts_with("border-color=")
            || trimmed.starts_with("background-color=")
        {
            colors_found = true;
        }
    }

    // Generate the new ini content with updated colors
    let mut new_content = format!(
        "text-color={}\nborder-color={}\nbackground-color={}\n",
        config.text_color, config.border_color, config.background_color
    );

    // Add preserved settings and sections
    if !preserved_lines.is_empty() {
        // Add a blank line before preserved content if we had colors before
        if colors_found {
            new_content.push('\n');
        }
        new_content.push_str(&preserved_lines.join("\n"));
    }

    // Ensure file ends with newline
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    // Write to mako.ini
    fs::write(&ini_path, new_content).map_err(|e| format!("Failed to write mako.ini: {}", e))?;

    Ok(())
}

/// Parse btop.theme content to extract color values
fn parse_btop_theme(theme_content: &str) -> Option<BtopConfig> {
    let mut main_bg = None;
    let mut main_fg = None;
    let mut title = None;
    let mut hi_fg = None;
    let mut selected_bg = None;
    let mut selected_fg = None;
    let mut inactive_fg = None;
    let mut proc_misc = None;
    let mut cpu_box = None;
    let mut mem_box = None;
    let mut net_box = None;
    let mut proc_box = None;
    let mut div_line = None;
    let mut temp_start = None;
    let mut temp_mid = None;
    let mut temp_end = None;
    let mut cpu_start = None;
    let mut cpu_mid = None;
    let mut cpu_end = None;
    let mut free_start = None;
    let mut free_mid = None;
    let mut free_end = None;
    let mut cached_start = None;
    let mut cached_mid = None;
    let mut cached_end = None;
    let mut available_start = None;
    let mut available_mid = None;
    let mut available_end = None;
    let mut used_start = None;
    let mut used_mid = None;
    let mut used_end = None;
    let mut download_start = None;
    let mut download_mid = None;
    let mut download_end = None;
    let mut upload_start = None;
    let mut upload_mid = None;
    let mut upload_end = None;

    for line in theme_content.lines() {
        let trimmed = line.trim();

        // Parse theme[key]="value" format
        if let Some(key_value) = trimmed.strip_prefix("theme[") {
            if let Some(end_idx) = key_value.find("]=\"") {
                let key = &key_value[..end_idx];
                let value_part = &key_value[end_idx + 3..];
                if let Some(end_quote) = value_part.find('"') {
                    let value = value_part[..end_quote].to_string();

                    match key {
                        "main_bg" => main_bg = Some(value),
                        "main_fg" => main_fg = Some(value),
                        "title" => title = Some(value),
                        "hi_fg" => hi_fg = Some(value),
                        "selected_bg" => selected_bg = Some(value),
                        "selected_fg" => selected_fg = Some(value),
                        "inactive_fg" => inactive_fg = Some(value),
                        "proc_misc" => proc_misc = Some(value),
                        "cpu_box" => cpu_box = Some(value),
                        "mem_box" => mem_box = Some(value),
                        "net_box" => net_box = Some(value),
                        "proc_box" => proc_box = Some(value),
                        "div_line" => div_line = Some(value),
                        "temp_start" => temp_start = Some(value),
                        "temp_mid" => temp_mid = Some(value),
                        "temp_end" => temp_end = Some(value),
                        "cpu_start" => cpu_start = Some(value),
                        "cpu_mid" => cpu_mid = Some(value),
                        "cpu_end" => cpu_end = Some(value),
                        "free_start" => free_start = Some(value),
                        "free_mid" => free_mid = Some(value),
                        "free_end" => free_end = Some(value),
                        "cached_start" => cached_start = Some(value),
                        "cached_mid" => cached_mid = Some(value),
                        "cached_end" => cached_end = Some(value),
                        "available_start" => available_start = Some(value),
                        "available_mid" => available_mid = Some(value),
                        "available_end" => available_end = Some(value),
                        "used_start" => used_start = Some(value),
                        "used_mid" => used_mid = Some(value),
                        "used_end" => used_end = Some(value),
                        "download_start" => download_start = Some(value),
                        "download_mid" => download_mid = Some(value),
                        "download_end" => download_end = Some(value),
                        "upload_start" => upload_start = Some(value),
                        "upload_mid" => upload_mid = Some(value),
                        "upload_end" => upload_end = Some(value),
                        _ => {}
                    }
                }
            }
        }
    }

    // Create config if we have at least some values
    if main_bg.is_some()
        || main_fg.is_some()
        || title.is_some()
        || hi_fg.is_some()
        || selected_bg.is_some()
    {
        Some(BtopConfig {
            main_bg: main_bg.unwrap_or_else(|| "#0F0F19".to_string()),
            main_fg: main_fg.unwrap_or_else(|| "#EDEDFE".to_string()),
            title: title.unwrap_or_else(|| "#6e6e92".to_string()),
            hi_fg: hi_fg.unwrap_or_else(|| "#33A1FF".to_string()),
            selected_bg: selected_bg.unwrap_or_else(|| "#f59e0b".to_string()),
            selected_fg: selected_fg.unwrap_or_else(|| "#EDEDFE".to_string()),
            inactive_fg: inactive_fg.unwrap_or_else(|| "#333333".to_string()),
            proc_misc: proc_misc.unwrap_or_else(|| "#8a8a8d".to_string()),
            cpu_box: cpu_box.unwrap_or_else(|| "#6e6e92".to_string()),
            mem_box: mem_box.unwrap_or_else(|| "#6e6e92".to_string()),
            net_box: net_box.unwrap_or_else(|| "#6e6e92".to_string()),
            proc_box: proc_box.unwrap_or_else(|| "#6e6e92".to_string()),
            div_line: div_line.unwrap_or_else(|| "#6e6e92".to_string()),
            temp_start: temp_start.unwrap_or_else(|| "#00F59B".to_string()),
            temp_mid: temp_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            temp_end: temp_end.unwrap_or_else(|| "#FF3366".to_string()),
            cpu_start: cpu_start.unwrap_or_else(|| "#00F59B".to_string()),
            cpu_mid: cpu_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            cpu_end: cpu_end.unwrap_or_else(|| "#FF3366".to_string()),
            free_start: free_start.unwrap_or_else(|| "#00F59B".to_string()),
            free_mid: free_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            free_end: free_end.unwrap_or_else(|| "#FF3366".to_string()),
            cached_start: cached_start.unwrap_or_else(|| "#00F59B".to_string()),
            cached_mid: cached_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            cached_end: cached_end.unwrap_or_else(|| "#FF3366".to_string()),
            available_start: available_start.unwrap_or_else(|| "#00F59B".to_string()),
            available_mid: available_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            available_end: available_end.unwrap_or_else(|| "#FF3366".to_string()),
            used_start: used_start.unwrap_or_else(|| "#00F59B".to_string()),
            used_mid: used_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            used_end: used_end.unwrap_or_else(|| "#FF3366".to_string()),
            download_start: download_start.unwrap_or_else(|| "#00F59B".to_string()),
            download_mid: download_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            download_end: download_end.unwrap_or_else(|| "#FF3366".to_string()),
            upload_start: upload_start.unwrap_or_else(|| "#00F59B".to_string()),
            upload_mid: upload_mid.unwrap_or_else(|| "#FF66F6".to_string()),
            upload_end: upload_end.unwrap_or_else(|| "#FF3366".to_string()),
        })
    } else {
        None
    }
}

/// Update the btop.theme file with the given settings
pub fn update_btop_theme(theme_name: &str, config: &BtopConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    // Generate the theme content with all color values
    let theme_content = format!(
        r#"# Main background, empty for terminal default, need to be empty if you want transparent background
theme[main_bg]="{}"

# Main text color
theme[main_fg]="{}"

# Title color for boxes
theme[title]="{}"

# Highlight color for keyboard shortcuts
theme[hi_fg]="{}"

# Background color of selected item in processes box
theme[selected_bg]="{}"

# Foreground color of selected item in processes box
theme[selected_fg]="{}"

# Color of inactive/disabled text
theme[inactive_fg]="{}"

# Misc colors for processes box including mini cpu graphs, details memory graph and details status text
theme[proc_misc]="{}"

# Cpu box outline color
theme[cpu_box]="{}"

# Memory/disks box outline color
theme[mem_box]="{}"

# Net up/down box outline color
theme[net_box]="{}"

# Processes box outline color
theme[proc_box]="{}"

# Box divider line and small boxes line color
theme[div_line]="{}"

# Temperature graph colors
theme[temp_start]="{}"
theme[temp_mid]="{}"
theme[temp_end]="{}"

# CPU graph colors
theme[cpu_start]="{}"
theme[cpu_mid]="{}"
theme[cpu_end]="{}"

# Mem/Disk free meter
theme[free_start]="{}"
theme[free_mid]="{}"
theme[free_end]="{}"

# Mem/Disk cached meter
theme[cached_start]="{}"
theme[cached_mid]="{}"
theme[cached_end]="{}"

# Mem/Disk available meter
theme[available_start]="{}"
theme[available_mid]="{}"
theme[available_end]="{}"

# Mem/Disk used meter
theme[used_start]="{}"
theme[used_mid]="{}"
theme[used_end]="{}"

# Download graph colors
theme[download_start]="{}"
theme[download_mid]="{}"
theme[download_end]="{}"

# Upload graph colors
theme[upload_start]="{}"
theme[upload_mid]="{}"
theme[upload_end]="{}"
"#,
        config.main_bg,
        config.main_fg,
        config.title,
        config.hi_fg,
        config.selected_bg,
        config.selected_fg,
        config.inactive_fg,
        config.proc_misc,
        config.cpu_box,
        config.mem_box,
        config.net_box,
        config.proc_box,
        config.div_line,
        config.temp_start,
        config.temp_mid,
        config.temp_end,
        config.cpu_start,
        config.cpu_mid,
        config.cpu_end,
        config.free_start,
        config.free_mid,
        config.free_end,
        config.cached_start,
        config.cached_mid,
        config.cached_end,
        config.available_start,
        config.available_mid,
        config.available_end,
        config.used_start,
        config.used_mid,
        config.used_end,
        config.download_start,
        config.download_mid,
        config.download_end,
        config.upload_start,
        config.upload_mid,
        config.upload_end,
    );

    // Write to btop.theme
    let theme_path = theme_dir.join("btop.theme");
    fs::write(&theme_path, theme_content)
        .map_err(|e| format!("Failed to write btop.theme: {}", e))?;

    Ok(())
}

/// Parse swayosd.css content to extract color values
fn parse_swayosd_css(css_content: &str) -> Option<SwayosdConfig> {
    let mut background_color = None;
    let mut border_color = None;
    let mut label = None;
    let mut image = None;
    let mut progress = None;

    for line in css_content.lines() {
        let trimmed = line.trim();

        if let Some(value) = trimmed.strip_prefix("@define-color background-color ") {
            background_color = Some(value.trim_end_matches(';').to_string());
        } else if let Some(value) = trimmed.strip_prefix("@define-color border-color ") {
            border_color = Some(value.trim_end_matches(';').to_string());
        } else if let Some(value) = trimmed.strip_prefix("@define-color label ") {
            label = Some(value.trim_end_matches(';').to_string());
        } else if let Some(value) = trimmed.strip_prefix("@define-color image ") {
            image = Some(value.trim_end_matches(';').to_string());
        } else if let Some(value) = trimmed.strip_prefix("@define-color progress ") {
            progress = Some(value.trim_end_matches(';').to_string());
        }
    }

    // Create config if we have at least some values
    if background_color.is_some()
        || border_color.is_some()
        || label.is_some()
        || image.is_some()
        || progress.is_some()
    {
        Some(SwayosdConfig {
            background_color: background_color.unwrap_or_else(|| "#0F0F19".to_string()),
            border_color: border_color.unwrap_or_else(|| "#33A1FF".to_string()),
            label: label.unwrap_or_else(|| "#8A8A8D".to_string()),
            image: image.unwrap_or_else(|| "#8A8A8D".to_string()),
            progress: progress.unwrap_or_else(|| "#8A8A8D".to_string()),
        })
    } else {
        None
    }
}

/// Update the swayosd.css file with the given settings
pub fn update_swayosd_css(theme_name: &str, config: &SwayosdConfig) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    // Generate the CSS content
    let css_content = format!(
        "@define-color background-color {};\n@define-color border-color {};\n@define-color label {};\n@define-color image {};\n@define-color progress {};\n",
        config.background_color, config.border_color, config.label, config.image, config.progress
    );

    // Write to swayosd.css
    let css_path = theme_dir.join("swayosd.css");
    fs::write(&css_path, css_content).map_err(|e| format!("Failed to write swayosd.css: {}", e))?;

    Ok(())
}
