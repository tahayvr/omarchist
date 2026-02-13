use crate::types::themes::{EditingTheme, HyprlandConfig, WaybarConfig};
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
