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

/// `unlock.png`: the logo Omarchy's Plymouth boot screen and SDDM login show
/// when this theme is picked with `omarchy-plymouth-switcher`.
pub const BOOT_LOGO_FILE: &str = "unlock.png";

/// `preview-unlock.png`: the switcher's thumbnail; a theme without one is not
/// listed by `omarchy-plymouth-list`.
pub const BOOT_PREVIEW_FILE: &str = "preview-unlock.png";

const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

/// The theme's boot logo, if it has one.
pub fn boot_logo(theme_name: &str, is_system: bool) -> Option<PathBuf> {
    get_theme_path(theme_name, is_system)
        .map(|dir| dir.join(BOOT_LOGO_FILE))
        .filter(|path| path.is_file())
}

/// Copies a PNG in as the theme's boot logo; Plymouth only loads PNG.
pub fn set_boot_logo(theme_name: &str, source_path: &std::path::Path) -> Result<PathBuf> {
    let dir = get_custom_theme_path(theme_name)
        .filter(|dir| dir.is_dir())
        .ok_or_else(|| Error::ThemeNotFound(theme_name.to_string()))?;
    let bytes = fs::read(source_path).map_err(|e| Error::io("Failed to read the image", e))?;
    if !bytes.starts_with(PNG_SIGNATURE) {
        return Err(Error::Invalid(
            "The boot logo must be a PNG image".to_string(),
        ));
    }
    let dest = dir.join(BOOT_LOGO_FILE);
    fs::write(&dest, bytes).map_err(|e| Error::io("Failed to write unlock.png", e))?;
    Ok(dest)
}

/// Renders `preview-unlock.png` from the logo and the theme's background and
/// foreground, so the boot screen switcher lists the theme.
pub fn render_boot_preview(theme_name: &str) -> Result<()> {
    let dir = get_custom_theme_path(theme_name)
        .filter(|dir| dir.is_dir())
        .ok_or_else(|| Error::ThemeNotFound(theme_name.to_string()))?;
    let palette = crate::system::themes::overrides::palette(theme_name)?;
    let color = |key: &str| {
        palette
            .get(key)
            .map(str::to_string)
            .ok_or_else(|| Error::Invalid(format!("The theme has no {key} color")))
    };
    crate::shell::theme_sh_commands::plymouth_preview(
        &color("background")?,
        &color("foreground")?,
        &dir.join(BOOT_LOGO_FILE),
        &dir.join(BOOT_PREVIEW_FILE),
    )
}

/// Removes the logo and its preview, so the switcher no longer lists the theme.
pub fn remove_boot_logo(theme_name: &str) -> Result<()> {
    let Some(dir) = get_custom_theme_path(theme_name) else {
        return Ok(());
    };
    for file in [BOOT_LOGO_FILE, BOOT_PREVIEW_FILE] {
        let path = dir.join(file);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| Error::io(format!("Failed to remove {file}"), e))?;
        }
    }
    Ok(())
}
