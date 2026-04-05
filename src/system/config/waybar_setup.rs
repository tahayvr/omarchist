use std::path::PathBuf;

use crate::assets::extract_default_dir;

pub fn ensure_waybar_config() -> Result<(), String> {
    let waybar_config_dir = get_waybar_config_dir()?;

    // If waybar config directory already exists, nothing to do
    if waybar_config_dir.exists() {
        return Ok(());
    }

    extract_default_dir("omarchist/waybar", &waybar_config_dir)?;

    println!("Created default waybar config at: {:?}", waybar_config_dir);

    Ok(())
}

fn get_waybar_config_dir() -> Result<PathBuf, String> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    Ok(home_dir.join(".config").join("omarchist").join("waybar"))
}
