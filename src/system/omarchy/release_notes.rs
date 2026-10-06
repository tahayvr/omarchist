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
        // A black-holed network must not leave "Loading…" up for minutes.
        .timeout(std::time::Duration::from_secs(15))
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
        .map(|body| github_flavored(&body))
        .unwrap_or_else(|| "No release notes available.".to_string());

    Ok((tag, notes))
}

/// Rewrites a release body the way GitHub shows it, since CommonMark alone
/// reads it differently: a line break inside a paragraph is a line break
/// (GitHub's "Download: …" and "SHA256: …" sit on two lines, not one), and
/// `@name` is a link to that person. Fenced code is left as it is.
pub fn github_flavored(body: &str) -> String {
    let body = body.replace("\r\n", "\n");
    let lines: Vec<&str> = body.lines().collect();
    let mut out = String::with_capacity(body.len() + lines.len() * 2);
    let mut in_fence = false;
    for (ix, line) in lines.iter().enumerate() {
        if is_fence(line) {
            in_fence = !in_fence;
        }
        if in_fence || is_fence(line) {
            out.push_str(line);
        } else {
            out.push_str(&link_mentions(line));
            let next = lines.get(ix + 1).copied().unwrap_or("");
            if !line.trim().is_empty()
                && !next.trim().is_empty()
                && !starts_block(next)
                && !line.ends_with("  ")
                && !line.ends_with('\\')
            {
                out.push_str("  ");
            }
        }
        out.push('\n');
    }
    if !body.ends_with('\n') {
        out.pop();
    }
    out
}

fn is_fence(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("```") || line.starts_with("~~~")
}

/// Whether a line opens a block of its own, so the line before it ends a
/// paragraph rather than breaking inside one.
fn starts_block(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with('#')
        || line.starts_with('>')
        || line.starts_with('|')
        || line.starts_with("- ")
        || line.starts_with("* ")
        || line.starts_with("+ ")
        || line.starts_with("---")
        || line.starts_with("***")
        || is_fence(line)
        || line
            .split_once(". ")
            .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// `@name` after a space, a bracket or the start of the line becomes a
/// GitHub profile link; inline code is skipped, as is anything that reads
/// as an e-mail address.
fn link_mentions(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    for (ix, part) in line.split('`').enumerate() {
        if ix > 0 {
            out.push('`');
        }
        if ix % 2 == 1 {
            out.push_str(part);
            continue;
        }
        let bytes = part.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let at_start = i == 0 || matches!(bytes[i - 1], b' ' | b'\t' | b'(' | b'[');
            if bytes[i] == b'@' && at_start {
                let end = i
                    + 1
                    + part[i + 1..]
                        .bytes()
                        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'-')
                        .count();
                let name = &part[i + 1..end];
                if !name.is_empty() && !name.starts_with('-') && !name.ends_with('-') {
                    out.push_str(&format!("[@{name}](https://github.com/{name})"));
                    i = end;
                    continue;
                }
            }
            let ch = part[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::github_flavored;

    #[test]
    fn a_line_break_in_a_paragraph_is_kept() {
        let body = "Download: https://x/y.iso\nSHA256: abc\n\nNext paragraph.";
        assert_eq!(
            github_flavored(body),
            "Download: https://x/y.iso  \nSHA256: abc\n\nNext paragraph."
        );
    }

    #[test]
    fn a_paragraph_before_a_block_ends_as_it_is() {
        let body = "Intro\n## Fixes\n- one\n- two\n1. first\n2. second\n\nTail";
        assert_eq!(github_flavored(body), body);
    }

    #[test]
    fn a_line_after_a_list_item_continues_it_on_a_new_line() {
        assert_eq!(github_flavored("- item\ncontinued"), "- item  \ncontinued");
    }

    #[test]
    fn fenced_code_is_left_alone() {
        let body = "Run:\n```sh\necho a\necho @b\n```\nDone";
        assert_eq!(
            github_flavored(body),
            "Run:\n```sh\necho a\necho @b\n```\nDone"
        );
    }

    #[test]
    fn mentions_become_links_but_not_emails_or_code() {
        assert_eq!(
            github_flavored("Fix by @spencerbull (@x-y) `@code` mail@host.org"),
            "Fix by [@spencerbull](https://github.com/spencerbull) ([@x-y](https://github.com/x-y)) `@code` mail@host.org"
        );
    }

    #[test]
    fn crlf_and_existing_hard_breaks_are_normalised() {
        assert_eq!(github_flavored("a  \r\nb\\\r\nc\r\n"), "a  \nb\\\nc\n");
    }
}
