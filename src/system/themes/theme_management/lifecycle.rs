use std::fs;
use std::path::Path;

use chrono::Utc;

use crate::types::themes::EditingTheme;

use super::btop::parse_btop_theme;
use super::chromium::update_chromium_config;
use super::colors::colors_config_from_terminal;
use super::colors::update_colors_toml;
use super::hyprland::{parse_hyprland_conf, update_hyprland_conf};
use super::hyprlock::{parse_hyprlock_conf, update_hyprlock_conf};
use super::icons::{parse_icons_theme, update_icons_theme};
use super::mako::{parse_mako_ini, update_mako_ini};
use super::paths::{get_custom_themes_dir, get_defaults_theme_dir};
use super::swayosd::{parse_swayosd_css, update_swayosd_css};
use super::terminal::update_terminal_configs;
use super::walker::update_walker_css;
use super::waybar::{parse_waybar_css, update_waybar_css};

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

    let defaults_dir = get_defaults_theme_dir();
    let new_theme_dir = themes_dir.join(theme_name);

    if new_theme_dir.exists() {
        return Err(format!("Theme '{}' already exists", theme_name));
    }

    fs::create_dir_all(&new_theme_dir)
        .map_err(|e| format!("Failed to create theme directory: {}", e))?;

    copy_theme_files(&defaults_dir, &new_theme_dir)?;
    update_theme_metadata(&new_theme_dir, theme_name)?;

    Ok(theme_name.to_string())
}

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

    editing_theme.is_light_theme = theme_dir.join("light.mode").exists();

    let waybar_css_path = theme_dir.join("waybar.css");
    if waybar_css_path.exists()
        && let Ok(css_content) = fs::read_to_string(&waybar_css_path)
        && let Some(config) = parse_waybar_css(&css_content)
    {
        editing_theme.apps.waybar = Some(config);
    }

    let hyprland_conf_path = theme_dir.join("hyprland.conf");
    if hyprland_conf_path.exists()
        && let Ok(conf_content) = fs::read_to_string(&hyprland_conf_path)
        && let Some(config) = parse_hyprland_conf(&conf_content)
    {
        editing_theme.apps.hyprland = Some(config);
    }

    let icons_theme_path = theme_dir.join("icons.theme");
    if icons_theme_path.exists()
        && let Ok(content) = fs::read_to_string(&icons_theme_path)
        && let Some(icons_config) = parse_icons_theme(&content)
    {
        editing_theme.apps.icons = Some(icons_config);
    }

    let hyprlock_conf_path = theme_dir.join("hyprlock.conf");
    if hyprlock_conf_path.exists()
        && let Ok(conf_content) = fs::read_to_string(&hyprlock_conf_path)
        && let Some(config) = parse_hyprlock_conf(&conf_content)
    {
        editing_theme.apps.hyprlock = Some(config);
    }

    let mako_ini_path = theme_dir.join("mako.ini");
    if mako_ini_path.exists()
        && let Ok(ini_content) = fs::read_to_string(&mako_ini_path)
        && let Some(config) = parse_mako_ini(&ini_content)
    {
        editing_theme.apps.mako = Some(config);
    }

    let btop_theme_path = theme_dir.join("btop.theme");
    if btop_theme_path.exists()
        && let Ok(theme_content) = fs::read_to_string(&btop_theme_path)
        && let Some(config) = parse_btop_theme(&theme_content)
    {
        editing_theme.apps.btop = Some(config);
    }

    let swayosd_css_path = theme_dir.join("swayosd.css");
    if swayosd_css_path.exists()
        && let Ok(css_content) = fs::read_to_string(&swayosd_css_path)
        && let Some(config) = parse_swayosd_css(&css_content)
    {
        editing_theme.apps.swayosd = Some(config);
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
        let colors = colors_config_from_terminal(terminal_config, &theme_data.colors.accent);
        update_colors_toml(theme_name, &colors)?;
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
        super::btop::update_btop_theme(theme_name, btop_config)?;
    }

    if let Some(ref swayosd_config) = theme_data.apps.swayosd {
        update_swayosd_css(theme_name, swayosd_config)?;
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
