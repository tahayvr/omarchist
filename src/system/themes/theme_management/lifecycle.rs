use std::fs;
use std::path::Path;

use chrono::Utc;

use crate::types::themes::EditingTheme;

use super::btop::parse_btop_theme;
use super::chromium::update_chromium_config;
use super::colors::update_colors_toml;
use super::icons::{parse_icons_theme, update_icons_theme};
use super::lock::{parse_lock_toml, update_lock_toml};
use super::paths::get_custom_themes_dir;
use crate::assets::extract_default_dir;

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

pub fn create_theme_from_defaults(theme_name: &str) -> Result<String, String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let new_theme_dir = themes_dir.join(theme_name);

    if new_theme_dir.exists() {
        return Err(format!("Theme '{}' already exists", theme_name));
    }

    extract_default_dir("theme", &new_theme_dir)?;
    update_theme_metadata(&new_theme_dir, theme_name)?;

    Ok(theme_name.to_string())
}

fn update_theme_metadata(theme_dir: &Path, theme_name: &str) -> Result<(), String> {
    let json_path = theme_dir.join("omarchist.json");

    if !json_path.exists() {
        return Ok(());
    }

    let now = Utc::now().to_rfc3339();

    let content = fs::read_to_string(&json_path)
        .map_err(|e| format!("Failed to read omarchist.json: {}", e))?;

    let updated_content = content
        .replace("{{THEME_NAME}}", theme_name)
        .replace("{{CREATED_AT}}", &now)
        .replace("{{MODIFIED_AT}}", &now)
        .replace("{{AUTHOR}}", "");

    fs::write(&json_path, updated_content)
        .map_err(|e| format!("Failed to write omarchist.json: {}", e))?;

    Ok(())
}

pub fn load_theme_for_editing(theme_name: &str) -> Result<EditingTheme, String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let json_path = theme_dir.join("omarchist.json");
    let mut editing_theme: EditingTheme = if json_path.exists() {
        let content = fs::read_to_string(&json_path)
            .map_err(|e| format!("Failed to read omarchist.json: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse omarchist.json: {}", e))?
    } else {
        EditingTheme::default()
    };

    editing_theme.is_light_theme =
        theme_dir.join("light.mode").exists() || editing_theme.colors.mode == "light";

    let icons_theme_path = theme_dir.join("icons.theme");
    if icons_theme_path.exists()
        && let Ok(content) = fs::read_to_string(&icons_theme_path)
        && let Some(icons_config) = parse_icons_theme(&content)
    {
        editing_theme.apps.icons = Some(icons_config);
    }

    let btop_theme_path = theme_dir.join("btop.theme");
    if btop_theme_path.exists()
        && let Ok(theme_content) = fs::read_to_string(&btop_theme_path)
        && let Some(config) = parse_btop_theme(&theme_content)
    {
        editing_theme.apps.btop = Some(config);
    }

    let lock_toml_path = theme_dir.join("shell.lock.toml");
    if lock_toml_path.exists()
        && let Ok(toml_content) = fs::read_to_string(&lock_toml_path)
        && let Some(config) = parse_lock_toml(&toml_content)
    {
        editing_theme.apps.lock = Some(config);
    }

    Ok(editing_theme)
}

pub fn save_theme_data(theme_name: &str, theme_data: &EditingTheme) -> Result<(), String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    let theme_dir = themes_dir.join(theme_name);

    if !theme_dir.exists() {
        return Err(format!("Theme '{}' not found", theme_name));
    }

    let mut updated_theme = theme_data.clone();
    updated_theme.modified_at = Utc::now().to_rfc3339();

    let json_path = theme_dir.join("omarchist.json");
    let json_content = serde_json::to_string_pretty(&updated_theme)
        .map_err(|e| format!("Failed to serialize theme data: {}", e))?;
    fs::write(&json_path, json_content)
        .map_err(|e| format!("Failed to write omarchist.json: {}", e))?;

    update_light_mode_file(&theme_dir, theme_data.is_light_theme)?;

    // colors.toml is Omarchist's source of truth for the theme's palette —
    // written unconditionally on every save. Everything else a theme could
    // need (terminal configs, the bar, notifications, window border colors,
    // etc.) is template-generated by Omarchy itself from this file.
    let mut colors_to_write = theme_data.colors.clone();
    colors_to_write.mode = if theme_data.is_light_theme {
        "light".to_string()
    } else {
        "dark".to_string()
    };
    update_colors_toml(theme_name, &colors_to_write)?;

    if let Some(ref chromium_config) = theme_data.apps.chromium {
        update_chromium_config(theme_name, chromium_config)?;
    }

    if let Some(ref btop_config) = theme_data.apps.btop {
        super::btop::update_btop_theme(theme_name, btop_config)?;
    }

    if let Some(ref lock_config) = theme_data.apps.lock {
        update_lock_toml(theme_name, lock_config)?;
    }

    if let Some(ref icons_config) = theme_data.apps.icons
        && let Some(theme_name_val) = icons_config.get("theme_name").and_then(|v| v.as_str())
    {
        update_icons_theme(theme_name, theme_name_val)?;
    }

    Ok(())
}

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

    let json_path = new_path.join("omarchist.json");
    if json_path.exists() {
        let content = fs::read_to_string(&json_path)
            .map_err(|e| format!("Failed to read omarchist.json: {}", e))?;
        let mut theme: EditingTheme = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse omarchist.json: {}", e))?;
        theme.name = new_name.to_string();
        theme.modified_at = Utc::now().to_rfc3339();

        let updated_content = serde_json::to_string_pretty(&theme)
            .map_err(|e| format!("Failed to serialize theme data: {}", e))?;
        fs::write(&json_path, updated_content)
            .map_err(|e| format!("Failed to write omarchist.json: {}", e))?;
    }

    Ok(())
}

fn update_light_mode_file(theme_dir: &Path, is_light: bool) -> Result<(), String> {
    let light_mode_path = theme_dir.join("light.mode");

    if is_light {
        if !light_mode_path.exists() {
            fs::write(&light_mode_path, "")
                .map_err(|e| format!("Failed to create light.mode file: {}", e))?;
        }
    } else if light_mode_path.exists() {
        fs::remove_file(&light_mode_path)
            .map_err(|e| format!("Failed to remove light.mode file: {}", e))?;
    }

    Ok(())
}
