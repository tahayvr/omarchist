
use dirs;
use std::process::{Command};


// Run Update script for Omarchy
#[tauri::command]
pub fn run_update_script(script_path: String) -> Result<(), String> {
    log::info!("Running script in Alacritty: {script_path}");

    // Get absolute path to the script
    let absolute_script_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?
        .join(&script_path);

    // Convert to string
    let script_path_str = absolute_script_path
        .to_str()
        .ok_or("Failed to convert script path to string")?;

    // Run alacritty with the script
    let output = Command::new("alacritty")
        .args([
            "-e",
            "bash",
            "-c",
            &format!(
                "cd '{}' && bash '{}'; echo 'Press any key to close...'; read -n 1",
                std::env::current_dir().unwrap().display(),
                script_path_str
            ),
        ])
        .spawn();

    match output {
        Ok(_) => {
            log::info!("Successfully launched Alacritty with script");
            Ok(())
        },
        Err(e) => {
            log::error!("Failed to launch Alacritty: {e}");
            Err(format!("Failed to launch Alacritty: {e}"))
        },
    }
}

// Get Omarchy version from git tags
#[tauri::command]
pub fn get_omarchy_version() -> Result<String, String> {
    log::info!("Getting Omarchy version from git tag");

    // Get the home directory
    let home_dir = dirs::home_dir().ok_or_else(|| "Failed to get home directory".to_string())?;

    let omarchy_path = home_dir.join(".local/share/omarchy");

    // Run git command to get the latest tag
    let output = Command::new("git")
        .args([
            "-C",
            omarchy_path
                .to_str()
                .ok_or("Failed to convert path to string")?,
            "describe",
            "--tags",
            "--abbrev=0",
        ])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                let version = String::from_utf8(result.stdout)
                    .map_err(|e| format!("Failed to parse git output: {e}"))?
                    .trim()
                    .to_string();

                if version.is_empty() {
                    Ok("unknown".to_string())
                } else {
                    log::info!("Found Omarchy version: {version}");
                    Ok(version)
                }
            } else {
                let error = String::from_utf8(result.stderr)
                    .unwrap_or_else(|_| "Unknown error".to_string());
                log::warn!("Git command failed: {error}");
                Ok("unknown".to_string())
            }
        },
        Err(e) => {
            log::warn!("Failed to run git command: {e}");
            Ok("unknown".to_string())
        },
    }
}
