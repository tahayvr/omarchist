use std::path::PathBuf;

// Returns `~/.config/omarchist`, or `None` if the home directory cannot be determined.
pub fn omarchist_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("omarchist"))
}

// Returns `~/.config/omarchist/waybar/profiles/<profile_name>/config.jsonc`,
// or `None` if the home directory cannot be determined.
pub fn waybar_profile_config_path(profile_name: &str) -> Option<PathBuf> {
    omarchist_config_dir().map(|d| {
        d.join("waybar")
            .join("profiles")
            .join(profile_name)
            .join("config.jsonc")
    })
}

// Returns `~/.config/omarchist/waybar/profiles`, or `None` if home dir is unavailable.
pub fn waybar_profiles_dir() -> Option<PathBuf> {
    omarchist_config_dir().map(|d| d.join("waybar").join("profiles"))
}
