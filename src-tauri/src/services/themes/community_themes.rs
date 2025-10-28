use crate::types::CommunityTheme;
use futures::stream::{self, StreamExt};
use log::{info, warn};
use reqwest::{Client, StatusCode};
use scraper::{Html, Selector};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const COMMUNITY_THEMES_URL: &str = "https://omarchythemes.com/";
const CACHE_TTL: Duration = Duration::from_secs(60 * 15);
const DETAIL_FETCH_CONCURRENCY: usize = 4;

/// In-memory cache for community themes to avoid re-downloading on every request.
#[derive(Default)]
struct CommunityThemesCache {
    themes: Vec<CommunityTheme>,
    fetched_at: Option<Instant>,
}

fn cache() -> &'static Mutex<CommunityThemesCache> {
    static CACHE: OnceLock<Mutex<CommunityThemesCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(CommunityThemesCache::default()))
}

#[tauri::command]
pub async fn get_community_themes(
    force_refresh: Option<bool>,
) -> Result<Vec<CommunityTheme>, String> {
    let should_refresh = force_refresh.unwrap_or(false);
    info!(
        "Community themes command invoked (force_refresh={})",
        should_refresh
    );
    fetch_community_themes(should_refresh).await
}

async fn fetch_community_themes(force_refresh: bool) -> Result<Vec<CommunityTheme>, String> {
    {
        let cache_guard = cache().lock().await;
        if !force_refresh {
            if let Some(fetched_at) = cache_guard.fetched_at {
                if fetched_at.elapsed() < CACHE_TTL && !cache_guard.themes.is_empty() {
                    info!(
                        "Returning {} community themes from cache",
                        cache_guard.themes.len()
                    );
                    return Ok(cache_guard.themes.clone());
                }
            }
        }
    }

    info!(
        "Fetching community themes from remote source (force_refresh={})",
        force_refresh
    );

    let client = build_http_client()?;
    let html = match download_community_page(&client).await {
        Ok(content) => content,
        Err(e) => {
            warn!("Failed to download community themes page: {e}");
            return Ok(Vec::new());
        },
    };

    let themes = match parse_community_themes(&html) {
        Ok(list) => list,
        Err(e) => {
            warn!("Failed to parse community themes page: {e}");
            return Ok(Vec::new());
        },
    };

    let themes = enrich_community_themes(&client, themes).await;

    {
        let mut cache_guard = cache().lock().await;
        cache_guard.themes = themes.clone();
        cache_guard.fetched_at = Some(Instant::now());
    }

    info!("Returning {} community themes to caller", themes.len());
    Ok(themes)
}

fn build_http_client() -> Result<Client, String> {
    Client::builder()
        .user_agent("Omarchist/0.6 (https://github.com/tahayvr/omarchist-app)")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))
}

async fn download_community_page(client: &Client) -> Result<String, String> {
    let response = client
        .get(COMMUNITY_THEMES_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch community themes: {e}"))?;

    if response.status() != StatusCode::OK {
        return Err(format!(
            "Unexpected status code {} from omarchythemes.com",
            response.status()
        ));
    }

    response
        .text()
        .await
        .map_err(|e| format!("Failed to read community themes response: {e}"))
}

fn parse_community_themes(html: &str) -> Result<Vec<CommunityTheme>, String> {
    let document = Html::parse_document(html);

    let anchor_selector = Selector::parse("a.flex.flex-col")
        .map_err(|e| format!("Failed to create selector for community theme anchors: {e}"))?;
    let title_selector =
        Selector::parse("h2").map_err(|e| format!("Failed to create selector for titles: {e}"))?;
    let author_selector =
        Selector::parse("h3").map_err(|e| format!("Failed to create selector for authors: {e}"))?;
    let image_selector =
        Selector::parse("img").map_err(|e| format!("Failed to create selector for images: {e}"))?;

    let mut themes = Vec::new();

    for anchor in document.select(&anchor_selector) {
        let href = match anchor.value().attr("href") {
            Some(link) if link.contains("/themes/") => normalize_url(link),
            _ => continue,
        };

        let slug = extract_slug(&href);
        if slug.is_empty() {
            continue;
        }

        let title = anchor
            .select(&title_selector)
            .next()
            .map(|el| normalize_text(&el.text().collect::<String>()))
            .filter(|text| !text.is_empty())
            .unwrap_or_else(|| slug.clone());

        let author = anchor
            .select(&author_selector)
            .next()
            .map(|el| normalize_text(&el.text().collect::<String>()))
            .and_then(|text| {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    None
                } else if let Some(rest) = trimmed.strip_prefix("by ") {
                    Some(rest.trim().to_string())
                } else {
                    Some(trimmed.to_string())
                }
            });

        let image_url = anchor
            .select(&image_selector)
            .next()
            .and_then(|img| img.value().attr("src"))
            .map(normalize_url);

        themes.push(CommunityTheme {
            title,
            author,
            detail_url: href,
            image_url,
            slug,
            github_url: None,
            install_command: None,
            install_url: None,
        });
    }

    if themes.is_empty() {
        warn!("No community themes were parsed from the remote HTML response");
        return Ok(Vec::new());
    }

    info!(
        "Parsed {} community themes from remote source",
        themes.len()
    );
    Ok(themes)
}

async fn enrich_community_themes(
    client: &Client,
    themes: Vec<CommunityTheme>,
) -> Vec<CommunityTheme> {
    let mut entries = stream::iter(themes.into_iter().enumerate())
        .map(|(index, theme)| {
            let client = client.clone();
            async move { (index, enrich_single_theme(&client, theme).await) }
        })
        .buffer_unordered(DETAIL_FETCH_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

    entries.sort_by_key(|(index, _)| *index);

    entries.into_iter().filter_map(|(_, theme)| theme).collect()
}

async fn enrich_single_theme(client: &Client, mut theme: CommunityTheme) -> Option<CommunityTheme> {
    let Some(github_url) = fetch_github_url(client, &theme.detail_url).await else {
        warn!(
            "Skipping community theme '{}' due to missing GitHub link",
            theme.slug
        );
        return None;
    };

    let trimmed_url = sanitize_source_url(&github_url);
    if trimmed_url.contains("learn.omacom.io") {
        info!(
            "Skipping official theme '{}' sourced from {}\u{a0}- already bundled with Omarchist",
            theme.slug, trimmed_url
        );
        return None;
    }

    let install_url = ensure_git_suffix(&trimmed_url);
    theme.github_url = Some(trimmed_url);
    theme.install_url = Some(install_url.clone());
    theme.install_command = Some(format!("omarchy-theme-install {}", install_url));

    Some(theme)
}

async fn fetch_github_url(client: &Client, detail_url: &str) -> Option<String> {
    let response = client.get(detail_url).send().await.ok()?;
    if response.status() != StatusCode::OK {
        warn!(
            "Community theme detail fetch returned status {} for {}",
            response.status(),
            detail_url
        );
        return None;
    }

    let body = response.text().await.ok()?;
    let document = Html::parse_document(&body);
    let anchor_selector = Selector::parse("a").ok()?;

    let mut candidates: Vec<String> = document
        .select(&anchor_selector)
        .filter_map(|anchor| anchor.value().attr("href"))
        .map(|href| href.trim().to_string())
        .filter(|href| href.contains("github.com"))
        .collect();

    if candidates.is_empty() {
        return None;
    }

    candidates.sort_by_key(|href| href.to_ascii_lowercase().contains("/tree/"));

    candidates
        .into_iter()
        .map(|href| sanitize_source_url(&href))
        .find(|href| !href.is_empty())
}

fn normalize_url(href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else {
        format!("{}{}", COMMUNITY_THEMES_URL.trim_end_matches('/'), href)
    }
}

fn extract_slug(url: &str) -> String {
    url.trim_end_matches('/')
        .split('/')
        .last()
        .unwrap_or("")
        .trim()
        .to_string()
}

fn normalize_text(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn ensure_git_suffix(url: &str) -> String {
    let trimmed = url.trim_end_matches('/');
    if trimmed.ends_with(".git") {
        trimmed.to_string()
    } else {
        format!("{trimmed}.git")
    }
}

fn sanitize_source_url(raw: &str) -> String {
    raw.trim().trim_matches('"').trim_matches('\'').to_string()
}
