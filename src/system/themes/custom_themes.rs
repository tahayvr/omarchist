use super::parse_colors::{parse_alacritty_toml, parse_colors_toml};
use super::preview_img::find_preview_image;
use super::utils::dir_to_title;

use crate::types::themes::{RawUserTheme, ThemeEntry, ThemeOrigin};
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_user_themes_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("omarchy").join("themes"))
}

fn load_theme_from_dir(theme_dir: &Path) -> Option<RawUserTheme> {
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

    // Themes with omarchist.json were created by this app; everything else is Community.
    let origin = if theme_dir.join("omarchist.json").exists() {
        ThemeOrigin::Omarchist
    } else {
        ThemeOrigin::Community
    };

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

    let theme_json_path = theme_dir.join("theme.json");
    let (author, apps) = if theme_json_path.exists() {
        match fs::read_to_string(&theme_json_path) {
            Ok(json_str) => {
                let v: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
                let author = v
                    .get("author")
                    .and_then(|a| a.as_str())
                    .map(|s| s.to_string());
                let apps = v.get("apps").cloned().unwrap_or(serde_json::Value::Null);
                (author, apps)
            }
            Err(_) => (None, serde_json::Value::Null),
        }
    } else {
        (None, serde_json::Value::Null)
    };

    Some(RawUserTheme {
        version: "1.0.0".to_string(),
        name: dir_name.to_string(),
        image,
        origin,
        created_at,
        modified_at,
        author,
        apps,
        colors,
    })
}

// Scan `~/.config/omarchy/themes/`
pub fn get_user_themes() -> Result<Vec<ThemeEntry>, String> {
    let themes_dir = get_user_themes_dir()
        .ok_or_else(|| "Could not determine user themes directory".to_string())?;

    if !themes_dir.exists() {
        return Ok(Vec::new());
    }

    let entries =
        fs::read_dir(&themes_dir).map_err(|e| format!("Failed to read themes directory: {e}"))?;

    let mut themes: Vec<ThemeEntry> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| load_theme_from_dir(&e.path()))
        .map(|raw| {
            let title = dir_to_title(&raw.name);
            raw.into_entry(title)
        })
        .collect();

    themes.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

    Ok(themes)
}
