use std::path::PathBuf;

pub(super) fn get_custom_themes_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("omarchy").join("themes"))
}
