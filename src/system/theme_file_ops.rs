use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get the full path for a system theme
fn get_system_theme_path(theme_name: &str) -> Option<PathBuf> {
    dirs::home_dir().map(|h| {
        h.join(".local")
            .join("share")
            .join("omarchy")
            .join("themes")
            .join(theme_name)
    })
}

/// Get the full path for a custom theme
fn get_custom_theme_path(theme_name: &str) -> Option<PathBuf> {
    dirs::home_dir().map(|h| {
        h.join(".config")
            .join("omarchy")
            .join("themes")
            .join(theme_name)
    })
}

/// Get the full path for a theme (system or custom)
pub fn get_theme_path(theme_name: &str, is_system: bool) -> Option<PathBuf> {
    if is_system {
        get_system_theme_path(theme_name)
    } else {
        get_custom_theme_path(theme_name)
    }
}

/// Open the theme folder in Nautilus file manager
pub fn open_theme_folder(theme_name: &str, is_system: bool) -> Result<(), String> {
    let path = get_theme_path(theme_name, is_system)
        .ok_or_else(|| "Could not determine theme path".to_string())?;

    if !path.exists() {
        return Err(format!("Theme folder does not exist: {}", path.display()));
    }

    // Open in Nautilus
    Command::new("nautilus")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open Nautilus: {}", e))?;

    Ok(())
}

/// Delete a theme folder
pub fn delete_theme(theme_name: &str, is_system: bool) -> Result<(), String> {
    // Safety check: only allow deleting custom themes, not system themes
    if is_system {
        return Err("Cannot delete system themes".to_string());
    }

    let path = get_theme_path(theme_name, is_system)
        .ok_or_else(|| "Could not determine theme path".to_string())?;

    if !path.exists() {
        return Err(format!("Theme folder does not exist: {}", path.display()));
    }

    // Delete the directory and all its contents
    fs::remove_dir_all(&path).map_err(|e| format!("Failed to delete theme folder: {}", e))?;

    Ok(())
}
