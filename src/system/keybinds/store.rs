// Persists keybind overrides and pushes them into the live Hyprland config.
use std::fs;
use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::system::hyprland_config::manager;
use crate::system::omarchy_paths::omarchist_hyprland_dir;

use super::overrides::KeybindOverrides;

pub const OVERRIDES_FILE: &str = "keybinds.json";

/// `~/.config/omarchist/hyprland/keybinds.json`
pub fn overrides_path() -> Result<PathBuf> {
    let dir = omarchist_hyprland_dir().ok_or(Error::UnknownDirectory("home"))?;
    Ok(dir.join(OVERRIDES_FILE))
}

/// A missing file means no overrides; a corrupt one is an error so the UI
/// does not silently overwrite the user's edits.
pub fn load_overrides() -> Result<KeybindOverrides> {
    let path = overrides_path()?;
    if !path.exists() {
        return Ok(KeybindOverrides::default());
    }
    let content =
        fs::read_to_string(&path).map_err(|e| Error::io("Failed to read keybinds.json", e))?;
    serde_json::from_str(&content).map_err(|e| Error::json("Failed to parse keybinds.json", e))
}

/// Validates, writes the json, regenerates `omarchist.lua` (settings and
/// keybinds together) and asks Hyprland to reload.
pub fn save_overrides(overrides: &KeybindOverrides) -> Result<()> {
    overrides.validate()?;

    let path = overrides_path()?;
    if let Some(dir) = path.parent()
        && !dir.exists()
    {
        fs::create_dir_all(dir).map_err(|e| Error::io("Failed to create config directory", e))?;
    }
    let content = serde_json::to_string_pretty(overrides)
        .map_err(|e| Error::json("Failed to serialize keybinds", e))?;
    fs::write(&path, content).map_err(|e| Error::io("Failed to write keybinds.json", e))?;

    manager::write_omarchist_lua(&manager::saved_config())?;
    manager::reload_hyprland();
    Ok(())
}
