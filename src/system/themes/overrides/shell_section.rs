//! `shell.<section>.toml` files: one `[section]` of Omarchy's `shell.toml`.
//!
//! Values are edited line by line so the template's comments and layout
//! survive. Files written before Omarchy accepted a header (the keys at the
//! top level, as in the tokyo-night theme) are read and edited the same way.

/// The section name of a `[section]` header line, matching the header pattern
/// `omarchy-theme-set-templates` uses (a trailing comment is allowed).
pub fn header_name(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix('[')?;
    let (name, after) = rest.split_once(']')?;
    let after = after.trim_start();
    (!name.is_empty() && (after.is_empty() || after.starts_with('#'))).then_some(name)
}

/// Cuts one section, header included, out of a rendered `shell.toml`.
pub fn cut(shell: &str, section: &str) -> Option<String> {
    let mut lines = shell
        .lines()
        .skip_while(|line| header_name(line) != Some(section));
    let header = lines.next()?;
    let mut body: Vec<&str> = lines
        .take_while(|line| header_name(line).is_none())
        .collect();
    while body.last().is_some_and(|line| line.trim().is_empty()) {
        body.pop();
    }
    let mut out = format!("{}\n", header.trim());
    for line in body {
        out.push_str(line);
        out.push('\n');
    }
    Some(out)
}

/// Every `[section]` of a `shell.toml`, in order.
pub fn sections(shell: &str) -> Vec<&str> {
    shell.lines().filter_map(header_name).collect()
}

/// A key's value as text: strings unquoted, anything else as TOML spells it.
pub fn get(content: &str, section: &str, key: &str) -> Option<String> {
    let table = toml::from_str::<toml::Table>(content).ok()?;
    let table = match table.get(section) {
        Some(toml::Value::Table(inner)) => inner,
        _ => &table,
    };
    match table.get(key)? {
        toml::Value::String(value) => Some(value.clone()),
        value => Some(value.to_string()),
    }
}

/// Sets a key to a string value, keeping the line's alignment, or appends it.
pub fn set(content: &str, section: &str, key: &str, value: &str) -> String {
    let quoted = toml::Value::String(value.to_string()).to_string();
    let mut current: Option<&str> = None;
    let mut replaced = false;
    let mut out = String::with_capacity(content.len() + quoted.len());

    for line in content.lines() {
        if let Some(name) = header_name(line) {
            current = Some(name);
        }
        let in_section = current.is_none_or(|name| name == section);
        match line.split_once('=') {
            Some((name, _))
                if !replaced
                    && in_section
                    && !line.trim_start().starts_with('#')
                    && name.trim() == key =>
            {
                out.push_str(&format!("{name}= {quoted}"));
                replaced = true;
            }
            _ => out.push_str(line),
        }
        out.push('\n');
    }

    if !replaced {
        out.push_str(&format!("{key} = {quoted}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{cut, get, header_name, set};

    const SHELL: &str = "# comment\n\n[bar]\n# help\nbackground = \"#000000\"\n\n[lock] # trailing\ntext = \"#ffffff\"\n\n\n";

    #[test]
    fn headers() {
        assert_eq!(header_name("[bar]"), Some("bar"));
        assert_eq!(header_name("  [image-picker]  # c"), Some("image-picker"));
        assert_eq!(header_name("colors = [1, 2]"), None);
        assert_eq!(header_name("[bar] x"), None);
    }

    #[test]
    fn cuts_a_section_with_its_header_and_without_trailing_blanks() {
        assert_eq!(
            cut(SHELL, "bar").unwrap(),
            "[bar]\n# help\nbackground = \"#000000\"\n"
        );
        assert_eq!(
            cut(SHELL, "lock").unwrap(),
            "[lock] # trailing\ntext = \"#ffffff\"\n"
        );
        assert!(cut(SHELL, "menu").is_none());
    }

    #[test]
    fn reads_values_with_or_without_a_header() {
        let with = "[lock]\ntext = \"#ffffff\"\nborder-alpha = 1.0\n";
        assert_eq!(get(with, "lock", "text").as_deref(), Some("#ffffff"));
        assert_eq!(get(with, "lock", "border-alpha").as_deref(), Some("1.0"));
        let without = "text             = \"#a9b1d6\"\n";
        assert_eq!(get(without, "lock", "text").as_deref(), Some("#a9b1d6"));
        assert_eq!(get(with, "lock", "missing"), None);
    }

    #[test]
    fn sets_in_place_keeping_alignment_and_comments() {
        let content = "[lock]\n# the field\ntext             = \"#ffffff\"\n# text = \"x\"\n";
        assert_eq!(
            set(content, "lock", "text", "#000000"),
            "[lock]\n# the field\ntext             = \"#000000\"\n# text = \"x\"\n"
        );
    }

    #[test]
    fn appends_a_missing_key() {
        assert_eq!(
            set("text = \"#ffffff\"", "lock", "border", "#123456"),
            "text = \"#ffffff\"\nborder = \"#123456\"\n"
        );
    }
}
