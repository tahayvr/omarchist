// This file contains the commands that are used to interact with the shell.

use dirs;
use std::process::{Command, Stdio};

// Execute a bash command
#[tauri::command]
pub async fn execute_bash_command(command: String) -> Result<String, String> {
    log::info!("Executing bash command: {}", command);

    let output = Command::new("bash")
        .arg("-c")
        .arg(&command)
        .output()
        .map_err(|e| format!("Failed to execute command: {e}"))?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| format!("Failed to parse command output: {e}"))?;
        log::info!("Command executed successfully");
        Ok(stdout)
    } else {
        let stderr =
            String::from_utf8(output.stderr).unwrap_or_else(|_| "Unknown error".to_string());
        log::error!("Command failed: {}", stderr);
        Err(format!("Command failed: {}", stderr))
    }
}

// Execute a bash command without waiting for output (fire and forget)
#[tauri::command]
pub fn execute_bash_command_async(command: String) -> Result<(), String> {
    log::info!("Executing bash command (async): {}", command);

    Command::new("bash")
        .arg("-c")
        .arg(&command)
        .spawn()
        .map_err(|e| format!("Failed to spawn command: {e}"))?;

    log::info!("Command spawned successfully");
    Ok(())
}

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

// Apply theme using omarchy-theme-set
#[tauri::command]
pub async fn apply_theme(dir: String) -> Result<(), String> {
    let output = Command::new("omarchy-theme-set").arg(&dir).spawn();

    let result = match output {
        Ok(_) => {
            log::info!("Successfully started omarchy-theme-set for theme {dir}");
            Ok(())
        },
        Err(e) => {
            // Log the error but don't fail the process
            log::warn!("Failed to run omarchy-theme-set: {e}");
            log::info!("Continuing without theme application...");
            // Return Ok to not stop the process
            Ok(())
        },
    };

    // Invalidate cache after theme application to ensure fresh state
    if result.is_ok() {
        if let Ok(cache) = crate::services::cache::cache_manager::get_theme_cache().await {
            // Invalidate the specific theme that was applied
            cache.invalidate_theme(&dir).await;
            // Trigger background refresh to ensure cache is up to date
            let _ = cache.trigger_background_refresh().await;
        }
    }

    result
}

// Refresh apps and gnome
#[tauri::command]
pub fn refresh_theme_adjustments() -> Result<(), String> {
    // Run a best-effort, silent bash script (no terminal)
    let script = r#"
omarchy-restart-waybar
omarchy-restart-swayosd
hyprctl reload
pkill -SIGUSR2 btop
makoctl reload

omarchy-theme-set-terminal
omarchy-theme-set-gnome
omarchy-theme-set-eza
omarchy-theme-set-browser
omarchy-theme-set-vscode
"#;

    let status = Command::new("bash")
        .arg("-c")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("Failed to start bash: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        log::warn!("Desktop adjustments exited with status: {status}");
        Ok(())
    }
}
