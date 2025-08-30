use regex::Regex;
use serde::Serialize;
use std::{fs, io, path::PathBuf};

#[derive(Debug, Serialize)]
pub struct SystemColors {
    pub foreground: String,
    pub background: String,
}

#[tauri::command]
pub fn get_system_theme_colors() -> Result<Option<SystemColors>, String> {
    let home = std::env::var("HOME").map_err(|e| format!("HOME not set: {e}"))?;
    let path = PathBuf::from(home).join(".config/omarchy/current/theme/waybar.css");

    let content = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                return Ok(None);
            } else {
                return Err(format!("Failed to read {path:?}: {e}"));
            }
        },
    };

    let re_bg = Regex::new(r"(?mi)@define-color\s+background\s+([^;]+);").unwrap();
    let re_fg = Regex::new(r"(?mi)@define-color\s+foreground\s+([^;]+);").unwrap();

    let background = re_bg.captures(&content).map(|c| c[1].trim().to_string());
    let foreground = re_fg.captures(&content).map(|c| c[1].trim().to_string());

    match (background, foreground) {
        (Some(background), Some(foreground)) => Ok(Some(SystemColors {
            foreground,
            background,
        })),
        _ => Ok(None),
    }
}
