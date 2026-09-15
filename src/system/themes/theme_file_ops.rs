use crate::error::{Error, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_system_theme_path(theme_name: &str) -> Option<PathBuf> {
    Some(crate::system::omarchy_paths::system_themes_dir().join(theme_name))
}

fn get_custom_theme_path(theme_name: &str) -> Option<PathBuf> {
    crate::system::omarchy_paths::user_themes_dir().map(|d| d.join(theme_name))
}

pub fn get_theme_path(theme_name: &str, is_system: bool) -> Option<PathBuf> {
    if is_system {
        get_system_theme_path(theme_name)
    } else {
        get_custom_theme_path(theme_name)
    }
}

pub fn open_theme_folder(theme_name: &str, is_system: bool) -> Result<()> {
    let path = get_theme_path(theme_name, is_system).ok_or(Error::UnknownDirectory("theme"))?;

    if !path.exists() {
        return Err(Error::Invalid(format!(
            "Theme folder does not exist: {}",
            path.display()
        )));
    }

    // Open in Nautilus
    Command::new("nautilus")
        .arg(&path)
        .spawn()
        .map_err(|e| Error::io("Failed to open Nautilus", e))?;

    Ok(())
}

pub fn delete_theme(theme_name: &str, is_system: bool) -> Result<()> {
    // Safety check: only allow deleting custom themes, not system themes
    if is_system {
        return Err(Error::Invalid("Cannot delete system themes".into()));
    }

    let path = get_theme_path(theme_name, is_system).ok_or(Error::UnknownDirectory("theme"))?;

    if !path.exists() {
        return Err(Error::Invalid(format!(
            "Theme folder does not exist: {}",
            path.display()
        )));
    }

    // Delete the directory and all its contents
    fs::remove_dir_all(&path).map_err(|e| Error::io("Failed to delete theme folder", e))?;

    Ok(())
}

pub fn get_backgrounds_dir(theme_name: &str, is_system: bool) -> Option<PathBuf> {
    get_theme_path(theme_name, is_system).map(|p| p.join("backgrounds"))
}

pub fn ensure_backgrounds_dir(theme_name: &str, is_system: bool) -> Result<PathBuf> {
    let backgrounds_dir =
        get_backgrounds_dir(theme_name, is_system).ok_or(Error::UnknownDirectory("backgrounds"))?;

    if !backgrounds_dir.exists() {
        fs::create_dir_all(&backgrounds_dir)
            .map_err(|e| Error::io("Failed to create backgrounds directory", e))?;
    }

    Ok(backgrounds_dir)
}

pub fn list_background_images(theme_name: &str, is_system: bool) -> Result<Vec<PathBuf>> {
    let backgrounds_dir =
        get_backgrounds_dir(theme_name, is_system).ok_or(Error::UnknownDirectory("backgrounds"))?;

    if !backgrounds_dir.exists() {
        return Ok(Vec::new());
    }

    let images: Vec<PathBuf> = fs::read_dir(&backgrounds_dir)
        .map_err(|e| Error::io("Failed to read backgrounds directory", e))?
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

pub fn add_background_image(
    theme_name: &str,
    is_system: bool,
    source_path: &std::path::Path,
) -> Result<PathBuf> {
    // Ensure backgrounds directory exists
    let backgrounds_dir = ensure_backgrounds_dir(theme_name, is_system)?;

    // Get the filename from the source path
    let filename = source_path
        .file_name()
        .ok_or_else(|| Error::Invalid("Invalid source file path".into()))?;

    let dest_path = backgrounds_dir.join(filename);

    // Copy the file (overwrites if exists)
    fs::copy(source_path, &dest_path)
        .map_err(|e| Error::io("Failed to copy background image", e))?;

    Ok(dest_path)
}

pub fn remove_background_image(theme_name: &str, is_system: bool, filename: &str) -> Result<()> {
    let backgrounds_dir =
        get_backgrounds_dir(theme_name, is_system).ok_or(Error::UnknownDirectory("backgrounds"))?;

    let file_path = backgrounds_dir.join(filename);

    if !file_path.exists() {
        return Err(Error::Invalid(format!(
            "Background image not found: {}",
            filename
        )));
    }

    fs::remove_file(&file_path).map_err(|e| Error::io("Failed to remove background image", e))?;

    Ok(())
}

pub fn clear_background_images(theme_name: &str, is_system: bool) -> Result<()> {
    let backgrounds_dir =
        get_backgrounds_dir(theme_name, is_system).ok_or(Error::UnknownDirectory("backgrounds"))?;

    if !backgrounds_dir.exists() {
        return Ok(());
    }

    // List all images and delete them
    let images = list_background_images(theme_name, is_system)?;
    for image_path in images {
        fs::remove_file(&image_path)
            .map_err(|e| Error::io("Failed to remove background image", e))?;
    }

    Ok(())
}

pub fn open_backgrounds_folder(theme_name: &str, is_system: bool) -> Result<()> {
    let backgrounds_dir = ensure_backgrounds_dir(theme_name, is_system)?;

    // Open in Nautilus
    Command::new("nautilus")
        .arg(&backgrounds_dir)
        .spawn()
        .map_err(|e| Error::io("Failed to open Nautilus", e))?;

    Ok(())
}

pub fn is_system_theme(theme_name: &str) -> bool {
    if let Some(path) = get_system_theme_path(theme_name) {
        path.exists()
    } else {
        false
    }
}

pub fn is_custom_theme(theme_name: &str) -> bool {
    if let Some(path) = get_custom_theme_path(theme_name) {
        path.exists()
    } else {
        false
    }
}
