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

/// Get the backgrounds directory for a theme
pub fn get_backgrounds_dir(theme_name: &str, is_system: bool) -> Option<PathBuf> {
    get_theme_path(theme_name, is_system).map(|p| p.join("backgrounds"))
}

/// Ensure the backgrounds directory exists, creating it if necessary
pub fn ensure_backgrounds_dir(theme_name: &str, is_system: bool) -> Result<PathBuf, String> {
    let backgrounds_dir = get_backgrounds_dir(theme_name, is_system)
        .ok_or_else(|| "Could not determine backgrounds directory path".to_string())?;

    if !backgrounds_dir.exists() {
        fs::create_dir_all(&backgrounds_dir)
            .map_err(|e| format!("Failed to create backgrounds directory: {}", e))?;
    }

    Ok(backgrounds_dir)
}

/// List all background images in a theme's backgrounds folder
pub fn list_background_images(theme_name: &str, is_system: bool) -> Result<Vec<PathBuf>, String> {
    let backgrounds_dir = get_backgrounds_dir(theme_name, is_system)
        .ok_or_else(|| "Could not determine backgrounds directory path".to_string())?;

    if !backgrounds_dir.exists() {
        return Ok(Vec::new());
    }

    let images: Vec<PathBuf> = fs::read_dir(&backgrounds_dir)
        .map_err(|e| format!("Failed to read backgrounds directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            if let Some(ext) = entry.path().extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                matches!(
                    ext.as_str(),
                    "jpg" | "jpeg" | "png" | "webp" | "bmp" | "gif"
                )
            } else {
                false
            }
        })
        .map(|entry| entry.path())
        .collect();

    Ok(images)
}

/// Add a background image to a theme (copies to backgrounds folder, overwriting if exists)
pub fn add_background_image(
    theme_name: &str,
    is_system: bool,
    source_path: &std::path::Path,
) -> Result<PathBuf, String> {
    // Ensure backgrounds directory exists
    let backgrounds_dir = ensure_backgrounds_dir(theme_name, is_system)?;

    // Get the filename from the source path
    let filename = source_path
        .file_name()
        .ok_or_else(|| "Invalid source file path".to_string())?;

    let dest_path = backgrounds_dir.join(filename);

    // Copy the file (overwrites if exists)
    fs::copy(source_path, &dest_path)
        .map_err(|e| format!("Failed to copy background image: {}", e))?;

    Ok(dest_path)
}

/// Remove a background image from a theme
pub fn remove_background_image(
    theme_name: &str,
    is_system: bool,
    filename: &str,
) -> Result<(), String> {
    let backgrounds_dir = get_backgrounds_dir(theme_name, is_system)
        .ok_or_else(|| "Could not determine backgrounds directory path".to_string())?;

    let file_path = backgrounds_dir.join(filename);

    if !file_path.exists() {
        return Err(format!("Background image not found: {}", filename));
    }

    fs::remove_file(&file_path).map_err(|e| format!("Failed to remove background image: {}", e))?;

    Ok(())
}

/// Open the backgrounds folder in Nautilus file manager
pub fn open_backgrounds_folder(theme_name: &str, is_system: bool) -> Result<(), String> {
    let backgrounds_dir = ensure_backgrounds_dir(theme_name, is_system)?;

    // Open in Nautilus
    Command::new("nautilus")
        .arg(&backgrounds_dir)
        .spawn()
        .map_err(|e| format!("Failed to open Nautilus: {}", e))?;

    Ok(())
}

/// Check if a theme is a system theme (exists in system themes directory)
pub fn is_system_theme(theme_name: &str) -> bool {
    if let Some(path) = get_system_theme_path(theme_name) {
        path.exists()
    } else {
        false
    }
}

/// Check if a theme is a custom theme (exists in custom themes directory)
pub fn is_custom_theme(theme_name: &str) -> bool {
    if let Some(path) = get_custom_theme_path(theme_name) {
        path.exists()
    } else {
        false
    }
}
