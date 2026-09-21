use crate::error::{Error, Result};
use isahc::AsyncReadResponseExt;
use isahc::config::{Configurable, RedirectPolicy};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    body: Option<String>,
}

/// The tag and body of the newest stable release on GitHub. This is only
/// used for the release notes; whether an update is installable is decided
/// by `updates::check_for_updates` against the package repository.
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

    let tag = release.tag_name;
    let notes = release
        .body
        .map(|body| body.replace("\r\n", "\n"))
        .unwrap_or_else(|| "No release notes available.".to_string());

    Ok((tag, notes))
}
