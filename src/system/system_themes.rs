use super::parse_colors::{parse_alacritty_toml, parse_colors_toml};
use super::preview_img::find_preview_image;

use crate::types::themes::SysTheme;
use std::fs;
use std::path::{Path, PathBuf};

fn get_system_themes_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| {
        h.join(".local")
            .join("share")
            .join("omarchy")
            .join("themes")
    })
}

fn load_theme_from_dir_quick(
    theme_dir: &Path,
    is_system: bool,
    is_custom: bool,
) -> Option<SysTheme> {
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

    Some(SysTheme {
        dir: dir_name.to_string(),
        title: dir_to_title(dir_name),
        description: format!("Theme from {}", dir_name),
        image,
        is_system,
        is_custom,
        colors,
    })
}

/// Get system themes (image paths are resolved but not decoded;
/// the UI loads images via file:// URIs through GPUI's native img element)
pub fn get_system_themes() -> Result<Vec<SysTheme>, String> {
    let themes_dir = get_system_themes_dir()
        .ok_or_else(|| "Could not determine system themes directory".to_string())?;

    if !themes_dir.exists() {
        return Ok(Vec::new());
    }

    let mut themes = Vec::new();
    let entries =
        fs::read_dir(&themes_dir).map_err(|e| format!("Failed to read themes directory: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(theme) = load_theme_from_dir_quick(&path, true, false) {
                themes.push(theme);
            }
        }
    }

    themes.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

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
