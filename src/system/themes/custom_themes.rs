use super::parse_colors::{parse_alacritty_toml, parse_colors_toml};
use super::preview_img::find_preview_image;
use super::utils::dir_to_title;

use crate::types::themes::{ThemeEntry, ThemeOrigin};
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_user_themes_dir() -> Option<PathBuf> {
    crate::system::omarchy_paths::user_themes_dir()
}

fn load_theme_from_dir(theme_dir: &Path) -> Option<ThemeEntry> {
    let dir_name = theme_dir.file_name()?.to_str()?;

    // `alacritty.toml` is only consulted for themes that predate colors.toml —
    // Omarchy itself still converts those on apply, so their preview swatches
    // should keep working here too.
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

    Some(ThemeEntry {
        dir: dir_name.to_string(),
        title: dir_to_title(dir_name),
        origin,
        image,
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
        .collect();

    themes.sort_by_key(|a| a.title.to_lowercase());

    Ok(themes)
}
