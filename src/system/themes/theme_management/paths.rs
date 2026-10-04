use std::path::PathBuf;

pub(super) fn get_custom_themes_dir() -> Option<PathBuf> {
    crate::system::omarchy_paths::user_themes_dir()
}
