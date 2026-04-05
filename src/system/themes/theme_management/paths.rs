use std::path::PathBuf;

pub(super) fn get_custom_themes_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("omarchy").join("themes"))
}

pub(super) fn get_defaults_theme_dir() -> PathBuf {
    PathBuf::from("defaults/theme")
}
