use dirs;
use std::process::Command;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    prerelease: bool,
}

// Check if there's a new version available on GitHub
#[tauri::command]
pub async fn check_omarchy_update() -> Result<bool, String> {
    log::info!("Checking for Omarchy updates");

    // Get current version
    let current_version = get_omarchy_version()?;

    if current_version == "unknown" {
        log::warn!("Cannot check for updates - current version is unknown");
        return Ok(false);
    }

    // Fetch latest release from GitHub
    let client = reqwest::Client::builder()
        .user_agent("omarchist-app")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let response = client
        .get("https://api.github.com/repos/basecamp/omarchy/releases/latest")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release data: {e}"))?;

    // Skip prereleases
    if release.prerelease {
        log::info!("Latest release is a prerelease, skipping");
        return Ok(false);
    }

    let latest_version = release.tag_name.trim_start_matches('v');
    let current_version_clean = current_version.trim_start_matches('v');

    log::info!("Current version: {current_version_clean}, Latest version: {latest_version}");

    // Compare versions
    let update_available = compare_versions(current_version_clean, latest_version);

    if update_available {
        log::info!("Update available: {latest_version}");
    } else {
        log::info!("Already up to date");
    }

    Ok(update_available)
}

// Compare two semantic versions (returns true if remote is newer)
fn compare_versions(current: &str, latest: &str) -> bool {
    let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();

    let latest_parts: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();

    // Pad with zeros if needed
    let max_len = current_parts.len().max(latest_parts.len());

    for i in 0..max_len {
        let current_part = current_parts.get(i).copied().unwrap_or(0);
        let latest_part = latest_parts.get(i).copied().unwrap_or(0);

        if latest_part > current_part {
            return true;
        } else if latest_part < current_part {
            return false;
        }
    }

    false
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
