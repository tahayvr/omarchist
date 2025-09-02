use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

pub struct LightModeService {
    themes_dir: PathBuf,
}

impl LightModeService {
    pub fn new(_app_handle: &AppHandle) -> Result<Self, String> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| "Failed to get home directory".to_string())?;
        
        let themes_dir = home_dir.join(".config").join("omarchy").join("themes");
        
        Ok(Self { themes_dir })
    }

    /// Sanitize theme name for directory usage (same logic as CustomThemeService)
    fn sanitize_name(name: &str) -> String {
        let mut result = String::with_capacity(name.len());
        
        for ch in name.chars() {
            match ch {
                ' ' => result.push('-'),
                c if c.is_alphanumeric() || c == '-' || c == '_' => {
                    result.extend(c.to_lowercase());
                },
                _ => {}, // Skip invalid characters
            }
        }
        
        result
    }

    /// Check if a theme is in light mode
    pub fn is_light_mode(&self, theme_name: &str) -> Result<bool, String> {
        let sanitized_name = Self::sanitize_name(theme_name);
        let theme_dir = self.themes_dir.join(&sanitized_name);
        
        if !theme_dir.exists() {
            return Err(format!("Theme '{}' not found", theme_name));
        }
        
        let light_mode_file = theme_dir.join("light.mode");
        Ok(light_mode_file.exists())
    }

    /// Enable light mode for a theme (create light.mode file)
    pub fn enable_light_mode(&self, theme_name: &str) -> Result<(), String> {
        let sanitized_name = Self::sanitize_name(theme_name);
        let theme_dir = self.themes_dir.join(&sanitized_name);
        
        if !theme_dir.exists() {
            return Err(format!("Theme '{}' not found", theme_name));
        }
        
        let light_mode_file = theme_dir.join("light.mode");
        
        // Create the light.mode file with timestamp
        let content = format!("Light mode enabled at: {}", chrono::Utc::now().to_rfc3339());
        fs::write(&light_mode_file, content)
            .map_err(|e| format!("Failed to create light.mode file: {}", e))?;
        
        log::info!("Enabled light mode for theme '{}'", theme_name);
        Ok(())
    }

    /// Disable light mode for a theme (remove light.mode file)
    pub fn disable_light_mode(&self, theme_name: &str) -> Result<(), String> {
        let sanitized_name = Self::sanitize_name(theme_name);
        let theme_dir = self.themes_dir.join(&sanitized_name);
        
        if !theme_dir.exists() {
            return Err(format!("Theme '{}' not found", theme_name));
        }
        
        let light_mode_file = theme_dir.join("light.mode");
        
        if light_mode_file.exists() {
            fs::remove_file(&light_mode_file)
                .map_err(|e| format!("Failed to remove light.mode file: {}", e))?;
            
            log::info!("Disabled light mode for theme '{}'", theme_name);
        }
        
        Ok(())
    }

    /// Set light mode for a theme (enable if true, disable if false)
    pub fn set_light_mode(&self, theme_name: &str, is_light: bool) -> Result<(), String> {
        if is_light {
            self.enable_light_mode(theme_name)
        } else {
            self.disable_light_mode(theme_name)
        }
    }
}

// Tauri commands
#[tauri::command(rename_all = "snake_case")]
pub async fn is_theme_light_mode(app_handle: AppHandle, theme_name: String) -> Result<bool, String> {
    let service = LightModeService::new(&app_handle)?;
    service.is_light_mode(&theme_name)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn set_theme_light_mode(
    app_handle: AppHandle, 
    theme_name: String, 
    is_light: bool
) -> Result<(), String> {
    let service = LightModeService::new(&app_handle)?;
    service.set_light_mode(&theme_name, is_light)
}