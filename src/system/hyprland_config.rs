use crate::types::hyprland_config::HyprlandConfig;
use std::fs;
use std::path::{Path, PathBuf};

pub mod parser;
pub mod writer;

const CONFIG_DIR: &str = ".config/omarchist/hyprland";
const CONFIG_FILE: &str = "hyprland.conf";

/// Manager for Hyprland configuration
pub struct HyprlandConfigManager {
    config_path: PathBuf,
    config: HyprlandConfig,
}

impl HyprlandConfigManager {
    /// Load the configuration from disk, or create a default one if it doesn't exist
    pub fn load() -> Result<Self, String> {
        let config_path = get_config_path()?;

        // Ensure config directory exists
        ensure_config_dir()?;

        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read config file: {}", e))?;
            parser::parse_config(&content)
        } else {
            HyprlandConfig::default()
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    /// Save the current configuration to disk
    pub fn save(&self) -> Result<(), String> {
        let content = writer::write_config(&self.config);
        fs::write(&self.config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }

    /// Get a reference to the current configuration
    pub fn get(&self) -> &HyprlandConfig {
        &self.config
    }

    /// Get a mutable reference to the current configuration
    pub fn get_mut(&mut self) -> &mut HyprlandConfig {
        &mut self.config
    }

    /// Update the configuration using a closure
    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut HyprlandConfig),
    {
        f(&mut self.config);
    }

    /// Update the configuration and save it to disk
    pub fn update_and_save<F>(&mut self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut HyprlandConfig),
    {
        f(&mut self.config);
        self.save()
    }

    /// Get the path to the config file
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Reset configuration to defaults
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

/// Get the path to the configuration file
fn get_config_path() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Could not determine home directory")?;
    Ok(home_dir.join(CONFIG_DIR).join(CONFIG_FILE))
}

/// Ensure the configuration directory exists
fn ensure_config_dir() -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Could not determine home directory")?;
    let config_dir = home_dir.join(CONFIG_DIR);

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    Ok(())
}

/// Check if a configuration file exists
pub fn config_exists() -> bool {
    get_config_path().map(|p| p.exists()).unwrap_or(false)
}

/// Delete the configuration file (reset to system defaults)
pub fn delete_config() -> Result<(), String> {
    let config_path = get_config_path()?;
    if config_path.exists() {
        fs::remove_file(&config_path)
            .map_err(|e| format!("Failed to delete config file: {}", e))?;
    }
    Ok(())
}
