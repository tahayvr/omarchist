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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_versions_latest_has_higher_major_returns_true() {
        assert!(
            compare_versions("1.0.0", "2.0.0"),
            "a higher major version in latest should mean an update is available"
        );
    }

    #[test]
    fn compare_versions_latest_has_higher_minor_returns_true() {
        assert!(
            compare_versions("1.0.0", "1.1.0"),
            "a higher minor version in latest should mean an update is available"
        );
    }

    #[test]
    fn compare_versions_latest_has_higher_patch_returns_true() {
        assert!(
            compare_versions("1.0.0", "1.0.1"),
            "a higher patch version in latest should mean an update is available"
        );
    }

    #[test]
    fn compare_versions_equal_versions_returns_false() {
        assert!(
            !compare_versions("1.2.3", "1.2.3"),
            "identical versions should not be considered an update"
        );
    }

    #[test]
    fn compare_versions_current_newer_than_latest_returns_false() {
        assert!(
            !compare_versions("2.0.0", "1.9.9"),
            "a current version ahead of latest should not be an update"
        );
    }

    #[test]
    fn compare_versions_major_takes_precedence_over_minor_and_patch() {
        // 2.99.99 is newer than 3.0.0 only if major wins — 3 > 2 so update available.
        assert!(
            compare_versions("2.99.99", "3.0.0"),
            "major version should take precedence"
        );
        assert!(
            !compare_versions("3.0.0", "2.99.99"),
            "current major newer than latest: no update"
        );
    }

    #[test]
    fn compare_versions_different_component_lengths_padded_with_zeros() {
        // "1.0" vs "1.0.1" — shorter is padded to "1.0.0", so update is available.
        assert!(
            compare_versions("1.0", "1.0.1"),
            "shorter version string should be zero-padded for comparison"
        );
    }

    #[test]
    fn compare_versions_non_numeric_parts_treated_as_zero() {
        // Non-numeric parts are filtered out with filter_map, falling back to nothing.
        // "1.x.0" → [1, 0] after filtering; "1.1.0" → [1, 1, 0].
        // So "1.0" < "1.1.0" → update available.
        assert!(
            compare_versions("1.x.0", "1.1.0"),
            "non-numeric parts should be skipped (treated as absent/zero)"
        );
    }
}
