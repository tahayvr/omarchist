use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::types::hyprland_config::HyprlandConfig;

const CONFIG_DIR: &str = ".config/omarchist/hyprland";
const CONFIG_FILE: &str = "hyprland.conf";

pub struct HyprlandConfigManager {
    config_path: PathBuf,
    config: HyprlandConfig,
}

impl HyprlandConfigManager {
    pub fn load() -> Result<Self, String> {
        let config_path = get_config_path()?;

        // Ensure config directory exists
        ensure_config_dir()?;

        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read config file: {}", e))?;
            super::parser::parse_config(&content)
        } else {
            HyprlandConfig::default()
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    pub fn save(&self) -> Result<(), String> {
        let content = super::writer::write_config(&self.config);
        fs::write(&self.config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        // Reload Hyprland to apply changes
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
        &self.config_path
    }

    pub fn reset_to_defaults(&mut self) {
        self.config = HyprlandConfig::default();
    }
}

impl Clone for HyprlandConfigManager {
    fn clone(&self) -> Self {
        Self {
            config_path: self.config_path.clone(),
            config: self.config.clone(),
        }
    }
}

fn get_config_path() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Could not determine home directory")?;
    Ok(home_dir.join(CONFIG_DIR).join(CONFIG_FILE))
}

fn ensure_config_dir() -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not determine home directory")?;
    let config_dir = home_dir.join(CONFIG_DIR);

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    Ok(())
}

pub fn config_exists() -> bool {
    get_config_path().map(|p| p.exists()).unwrap_or(false)
}

pub fn delete_config() -> Result<(), String> {
    let config_path = get_config_path()?;
    if config_path.exists() {
        fs::remove_file(&config_path)
            .map_err(|e| format!("Failed to delete config file: {}", e))?;
    }
    Ok(())
}
