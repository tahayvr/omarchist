use super::parse_colors::{parse_alacritty_toml, parse_colors_toml};
use super::preview_img::find_preview_image;

use crate::types::themes::CustomTheme;
use std::fs;
use std::path::{Path, PathBuf};

fn get_custom_themes_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("omarchy").join("themes"))
}

fn load_theme_from_dir_quick(theme_dir: &Path) -> Option<CustomTheme> {
    let dir_name = theme_dir.file_name()?.to_str()?;
    let colors_path = theme_dir.join("colors.toml");
    let alacritty_path = theme_dir.join("alacritty.toml");
    let colors = if colors_path.exists() {
        parse_colors_toml(&colors_path)
    } else if alacritty_path.exists() {
        parse_alacritty_toml(&alacritty_path)
    } else {
        None
    };
    let image = find_preview_image(theme_dir).unwrap_or_default();

    // Get created_at and modified_at from directory metadata
    let metadata = fs::metadata(theme_dir).ok();
    let created_at = metadata
        .as_ref()
        .and_then(|m| m.created().ok())
        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
        .unwrap_or_default();
    let modified_at = metadata
        .as_ref()
        .and_then(|m| m.modified().ok())
        .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339())
        .unwrap_or_default();

    // Try to load author and apps from theme.json if present
    let theme_json_path = theme_dir.join("theme.json");
    let (author, apps) = if theme_json_path.exists() {
        match fs::read_to_string(&theme_json_path) {
            Ok(json_str) => {
                let v: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
                let author = v
                    .get("author")
                    .and_then(|a| a.as_str())
                    .map(|s| s.to_string());
                let apps = v
                    .get("apps")
                    .cloned()
                    .unwrap_or_else(|| serde_json::Value::Null);
                (author, apps)
            }
            Err(_) => (None, serde_json::Value::Null),
        }
    } else {
        (None, serde_json::Value::Null)
    };

    Some(CustomTheme {
        version: "1.0.0".to_string(),
        name: dir_name.to_string(),
        image,
        created_at,
        modified_at,
        author,
        apps,
        colors,
    })
}

pub fn get_custom_themes() -> Result<Vec<CustomTheme>, String> {
    let themes_dir = get_custom_themes_dir()
        .ok_or_else(|| "Could not determine custom themes directory".to_string())?;

    if !themes_dir.exists() {
        return Ok(Vec::new());
    }

    let mut themes = Vec::new();
    let entries =
        fs::read_dir(&themes_dir).map_err(|e| format!("Failed to read themes directory: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir()
            && let Some(theme) = load_theme_from_dir_quick(&path)
        {
            themes.push(theme);
        }
    }

    themes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(themes)
}

/// Convert directory name to a nice display title
pub fn dir_to_title(dir_name: &str) -> String {
    let mut title = String::with_capacity(dir_name.len() + 10);
    let mut capitalize_next = true;

    for ch in dir_name.chars() {
        match ch {
            '-' | '_' => {
                title.push(' ');
                capitalize_next = true;
            }
            c if capitalize_next => {
                title.extend(c.to_uppercase());
                capitalize_next = false;
            }
            c => title.push(c),
        }
    }
    title
}
