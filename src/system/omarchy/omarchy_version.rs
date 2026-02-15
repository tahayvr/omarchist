use dirs;
use isahc::AsyncReadResponseExt;
use std::process::Command;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    prerelease: bool,
}

// Check if there's a new version available on GitHub
pub async fn check_omarchy_update(current_version: &str) -> Result<bool, String> {
    if current_version == "unknown" {
        return Ok(false);
    }

    // Fetch latest release from GitHub using isahc (runtime-agnostic)
    let request = isahc::Request::builder()
        .uri("https://api.github.com/repos/basecamp/omarchy/releases/latest")
        .header("User-Agent", "omarchist")
        .body(())
        .map_err(|e| format!("Failed to build request: {e}"))?;

    let mut response = isahc::send_async(request)
        .await
        .map_err(|e| format!("Failed to fetch releases: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()));
    }

    let release: GitHubRelease = response
        .json::<GitHubRelease>()
        .await
        .map_err(|e| format!("Failed to parse release data: {e}"))?;

    // Skip prereleases
    if release.prerelease {
        return Ok(false);
    }

    let latest_version = release.tag_name.trim_start_matches('v');
    let current_version_clean = current_version.trim_start_matches('v');

    // Compare versions
    let update_available = compare_versions(current_version_clean, latest_version);

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

// Get local Omarchy version from git tags
pub fn get_local_omarchy_version() -> Result<String, String> {
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
                    Ok(version)
                }
            } else {
                Ok("unknown".to_string())
            }
        }
        Err(_) => Ok("unknown".to_string()),
    }
}
