use super::omarchy_version::GitHubRelease;
use isahc::AsyncReadResponseExt;

pub async fn fetch_latest_release_notes() -> Result<(String, String), String> {
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
        return Err("Latest release is a prerelease".to_string());
    }

    let tag = release.tag_name;
    let notes = release
        .body
        .unwrap_or_else(|| "No release notes available.".to_string());

    Ok((tag, notes))
}
