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
    pub fn load() -> Result<Self, String> {
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
                .map_err(|e| format!("Failed to read state file: {}", e))?;
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

    pub fn save(&self) -> Result<(), String> {
        // state.json is the round-trip source of truth Omarchist reads back
        // on the next load — never parsed from Lua.
        let state_content = serde_json::to_string_pretty(&self.config)
            .map_err(|e| format!("Failed to serialize Hyprland state: {}", e))?;
        fs::write(&self.state_path, state_content)
            .map_err(|e| format!("Failed to write state file: {}", e))?;

        // omarchist.lua is write-only — generated fresh every save, never
        // read back.
        let lua_content = super::lua_writer::write_lua_config(&self.config);
        fs::write(&self.lua_path, lua_content)
            .map_err(|e| format!("Failed to write omarchist.lua: {}", e))?;

        Self::reload_hyprland();

        Ok(())
    }

    fn reload_hyprland() {
        // Run hyprctl reload in background - don't block on it
        std::thread::spawn(|| {
            let _ = Command::new("hyprctl").arg("reload").output();
        });
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

    pub fn update_and_save<F>(&mut self, f: F) -> Result<(), String>
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

fn get_state_path() -> Result<PathBuf, String> {
    let dir = omarchist_hyprland_dir().ok_or("Could not determine home directory")?;
    Ok(dir.join(STATE_FILE))
}

fn get_lua_path() -> Result<PathBuf, String> {
    let dir = user_hyprland_config_dir().ok_or("Could not determine home directory")?;
    Ok(dir.join(LUA_FILE))
}

fn ensure_config_dir() -> Result<(), String> {
    let dir = omarchist_hyprland_dir().ok_or("Could not determine home directory")?;

    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    Ok(())
}

pub fn config_exists() -> bool {
    get_state_path().map(|p| p.exists()).unwrap_or(false)
}

pub fn delete_config() -> Result<(), String> {
    let state_path = get_state_path()?;
    if state_path.exists() {
        fs::remove_file(&state_path).map_err(|e| format!("Failed to delete state file: {}", e))?;
    }

    let lua_path = get_lua_path()?;
    if lua_path.exists() {
        fs::remove_file(&lua_path).map_err(|e| format!("Failed to delete omarchist.lua: {}", e))?;
    }

    Ok(())
}
