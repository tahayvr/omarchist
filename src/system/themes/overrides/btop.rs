//! `btop.theme`: `theme[key]="value"` lines, edited in place so every key and
//! comment of Omarchy's template survives.

fn key_of(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("theme[")?;
    let (key, after) = rest.split_once(']')?;
    after.trim_start().starts_with('=').then_some(key)
}

pub fn get(content: &str, key: &str) -> Option<String> {
    let line = content.lines().find(|line| key_of(line) == Some(key))?;
    let (_, value) = line.split_once('=')?;
    Some(value.trim().trim_matches('"').to_string())
}

pub fn set(content: &str, key: &str, value: &str) -> String {
    let entry = format!("theme[{key}]=\"{value}\"");
    let mut replaced = false;
    let mut out = String::with_capacity(content.len() + entry.len());

    for line in content.lines() {
        if !replaced && key_of(line) == Some(key) {
            out.push_str(&entry);
            replaced = true;
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }

    if !replaced {
        out.push_str(&entry);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{get, set};

    const THEME: &str =
        "# Main background\ntheme[main_bg]=\"#1a1b26\"\n\ntheme[hi_fg]=\"#7aa2f7\"\n";

    #[test]
    fn reads_keys() {
        assert_eq!(get(THEME, "main_bg").as_deref(), Some("#1a1b26"));
        assert_eq!(get(THEME, "hi_fg").as_deref(), Some("#7aa2f7"));
        assert_eq!(get(THEME, "main_fg"), None);
    }

    #[test]
    fn sets_in_place_and_appends_missing_keys() {
        assert_eq!(
            set(THEME, "hi_fg", "#ffffff"),
            "# Main background\ntheme[main_bg]=\"#1a1b26\"\n\ntheme[hi_fg]=\"#ffffff\"\n"
        );
        assert!(set(THEME, "title", "#000000").ends_with("theme[title]=\"#000000\"\n"));
    }
}
