use super::omarchy_version::GitHubRelease;
use crate::error::{Error, Result};
use isahc::AsyncReadResponseExt;
use isahc::config::{Configurable, RedirectPolicy};

pub async fn fetch_latest_release_notes() -> Result<(String, String)> {
    let request = isahc::Request::builder()
        .uri("https://api.github.com/repos/omacom/omarchy/releases/latest")
        .redirect_policy(RedirectPolicy::Follow)
        .header("User-Agent", "omarchist")
        .body(())
        .map_err(|e| Error::Network(format!("Failed to build request: {e}")))?;

    let mut response = isahc::send_async(request)
        .await
        .map_err(|e| Error::Network(format!("Failed to fetch releases: {e}")))?;

    if !response.status().is_success() {
        return Err(Error::Network(format!(
            "GitHub API returned status: {}",
            response.status()
        )));
    }

    let release: GitHubRelease = response
        .json::<GitHubRelease>()
        .await
        .map_err(|e| Error::Network(format!("Failed to parse release data: {e}")))?;

    // Skip prereleases
    if release.prerelease {
        return Err(Error::Invalid("Latest release is a prerelease".into()));
    }

    let tag = release.tag_name;
    let notes = release
        .body
        .map(|body| body.replace("\r\n", "\n"))
        .unwrap_or_else(|| "No release notes available.".to_string());

    Ok((tag, notes))
}
