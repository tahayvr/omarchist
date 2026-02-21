use std::fs;
use std::path::PathBuf;

/// Ensures the waybar config directory exists in ~/.config/omarchist/waybar
/// If it doesn't exist, copies the defaults/omarchist/waybar folder
/// Returns Ok(()) on success, or an error message on failure
pub fn ensure_waybar_config() -> Result<(), String> {
    let waybar_config_dir = get_waybar_config_dir()?;

    // If waybar config directory already exists, nothing to do
    if waybar_config_dir.exists() {
        return Ok(());
    }

    // Copy the defaults/omarchist/waybar folder to ~/.config/omarchist/waybar
    let defaults_dir = PathBuf::from("defaults/omarchist/waybar");
    copy_directory_recursive(&defaults_dir, &waybar_config_dir)?;

    println!("Created default waybar config at: {:?}", waybar_config_dir);

    Ok(())
}

/// Recursively copy a directory and all its contents
fn copy_directory_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    // Create destination directory
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create directory '{}': {}", dst.display(), e))?;

    // Read source directory entries
    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory '{}': {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .ok_or_else(|| "Invalid file name".to_string())?;
        let dest_path = dst.join(file_name);

        if path.is_dir() {
            // Recursively copy subdirectory
            copy_directory_recursive(&path, &dest_path)?;
        } else {
            // Copy file
            fs::copy(&path, &dest_path)
                .map_err(|e| format!("Failed to copy file '{}': {}", path.display(), e))?;
        }
    }

    Ok(())
}

/// Gets the waybar config directory path (~/.config/omarchist/waybar)
fn get_waybar_config_dir() -> Result<PathBuf, String> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    Ok(home_dir.join(".config").join("omarchist").join("waybar"))
}
