// This file contains the functions to get the themes from the data/themes.toml file

use crate::types::{Theme, ThemeData};
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

fn get_data_dir(app_handle: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource directory: {e}"))?;
    Ok(resource_dir.join("data"))
}

#[tauri::command]
pub async fn get_themes(app_handle: AppHandle) -> Result<Vec<Theme>, String> {
    let data_dir = get_data_dir(&app_handle).map_err(|e| e.to_string())?;
    let themes_path = data_dir.join("themes.toml");

    let content = std::fs::read_to_string(&themes_path)
        .map_err(|e| format!("Failed to read themes.toml: {e}"))?;

    let theme_data: ThemeData =
        toml::from_str(&content).map_err(|e| format!("Failed to parse themes.toml: {e}"))?;

    Ok(theme_data.theme)
}
