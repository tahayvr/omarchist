use crate::error::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::system::omarchy_paths::{omarchist_hyprland_dir, user_hyprland_config_dir};
use crate::types::hyprland_config::HyprlandConfig;

const STATE_FILE: &str = "state.json";
const LUA_FILE: &str = "omarchist.lua";

pub struct HyprlandConfigManager {
    state_path: PathBuf,
    lua_path: PathBuf,
    config: HyprlandConfig,
}

impl HyprlandConfigManager {
    pub fn load() -> Result<Self> {
        let state_path = get_state_path()?;
        let lua_path = get_lua_path()?;

        ensure_config_dir()?;

        // Always seed from the live compositor values so the UI reflects what
        // Hyprland is actually running, regardless of whether we have a saved
        // state file. If a saved state file exists, its values take
        // precedence over the live snapshot.
        let base = super::hyprctl_reader::read_from_hyprctl();

        let config = if state_path.exists() {
            let content = fs::read_to_string(&state_path)
                .map_err(|e| Error::io("Failed to read state file", e))?;
            serde_json::from_str(&content).unwrap_or(base)
        } else {
            base
        };

        Ok(Self {
            state_path,
            lua_path,
            config,
        })
    }

    pub fn save(&self) -> Result<()> {
        // state.json is the round-trip source of truth Omarchist reads back
        // on the next load — never parsed from Lua.
        let state_content = serde_json::to_string_pretty(&self.config)
            .map_err(|e| Error::json("Failed to serialize Hyprland state", e))?;
        fs::write(&self.state_path, state_content)
            .map_err(|e| Error::io("Failed to write state file", e))?;

        // omarchist.lua is write-only — generated fresh every save, never
        // read back.
        write_omarchist_lua(&self.config)?;
        reload_hyprland();

        Ok(())
    }

    pub fn get(&self) -> &HyprlandConfig {
        &self.config
    }

    pub fn get_mut(&mut self) -> &mut HyprlandConfig {
        &mut self.config
    }

    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut HyprlandConfig),
    {
        f(&mut self.config);
    }

    pub fn update_and_save<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut HyprlandConfig),
    {
        f(&mut self.config);
        self.save()
    }

    pub fn config_path(&self) -> &Path {
        &self.lua_path
    }

    pub fn reset_to_defaults(&mut self) {
        self.config = HyprlandConfig::default();
    }
}

impl Clone for HyprlandConfigManager {
    fn clone(&self) -> Self {
        Self {
            state_path: self.state_path.clone(),
            lua_path: self.lua_path.clone(),
            config: self.config.clone(),
        }
    }
}

/// Asks Hyprland to re-read its config, in the background so saves never
/// block on the compositor.
pub fn reload_hyprland() {
    std::thread::spawn(|| {
        let _ = Command::new("hyprctl").arg("reload").output();
    });
}

/// The settings last saved by the Configuration page, without touching
/// hyprctl. Used when only the keybinds block changed.
pub fn saved_config() -> HyprlandConfig {
    get_state_path()
        .ok()
        .filter(|p| p.exists())
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

/// Regenerates `~/.config/hypr/omarchist.lua` from the settings plus the
/// saved keybind overrides. Both the Configuration page and the Keybinds
/// page go through here so neither can drop the other's section.
pub fn write_omarchist_lua(config: &HyprlandConfig) -> Result<()> {
    let keybinds = crate::system::keybinds::store::load_overrides().unwrap_or_default();
    let keybinds_lua = crate::system::keybinds::overrides::emit_keybinds_lua(&keybinds);
    let lua_content = super::lua_writer::render_omarchist_lua(config, &keybinds_lua);
    fs::write(get_lua_path()?, lua_content)
        .map_err(|e| Error::io("Failed to write omarchist.lua", e))
}

fn get_state_path() -> Result<PathBuf> {
    let dir = omarchist_hyprland_dir().ok_or(Error::UnknownDirectory("home"))?;
    Ok(dir.join(STATE_FILE))
}

fn get_lua_path() -> Result<PathBuf> {
    let dir = user_hyprland_config_dir().ok_or(Error::UnknownDirectory("home"))?;
    Ok(dir.join(LUA_FILE))
}

fn ensure_config_dir() -> Result<()> {
    let dir = omarchist_hyprland_dir().ok_or(Error::UnknownDirectory("home"))?;

    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| Error::io("Failed to create config directory", e))?;
    }

    Ok(())
}

pub fn config_exists() -> bool {
    get_state_path().map(|p| p.exists()).unwrap_or(false)
}

pub fn delete_config() -> Result<()> {
    let state_path = get_state_path()?;
    if state_path.exists() {
        fs::remove_file(&state_path).map_err(|e| Error::io("Failed to delete state file", e))?;
    }

    // Keep the file (hyprland.lua requires it) with only the keybinds left.
    write_omarchist_lua(&HyprlandConfig::default())
}
