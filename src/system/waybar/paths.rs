use std::path::PathBuf;

// Returns `~/.config/omarchist`, or `None` if the home directory cannot be determined.
pub fn omarchist_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("omarchist"))
}

// Returns `~/.config/waybar`, or `None` if the home directory cannot be determined.
pub fn live_waybar_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("waybar"))
}

// Returns the live Waybar config file (`config.jsonc` preferred, then `config`),
// or `None` when no live config file exists.
pub fn live_waybar_config_path() -> Option<PathBuf> {
    let live_dir = live_waybar_dir()?;
    let jsonc = live_dir.join("config.jsonc");
    if jsonc.exists() {
        return Some(jsonc);
    }

    let plain = live_dir.join("config");
    if plain.exists() {
        return Some(plain);
    }

    None
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

// Returns `~/.config/omarchist/waybar/current-profile`, or `None` if home dir is unavailable.
pub fn waybar_current_profile_path() -> Option<PathBuf> {
    omarchist_config_dir().map(|d| d.join("waybar").join("current-profile"))
}
