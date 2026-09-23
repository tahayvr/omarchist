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

/// A key's value in the file, if it sets one.
pub fn get_value(content: &str, section: &str, key: &str) -> Option<toml::Value> {
    let mut table = toml::from_str::<toml::Table>(content).ok()?;
    let table = match table.remove(section) {
        Some(toml::Value::Table(inner)) => inner,
        _ => table,
    };
    table.get(key).cloned()
}

/// Every key the file sets, in file order.
pub fn keys(content: &str, section: &str) -> Vec<String> {
    let mut current: Option<&str> = None;
    content
        .lines()
        .filter_map(|line| {
            if let Some(name) = header_name(line) {
                current = Some(name);
                return None;
            }
            if current.is_some_and(|name| name != section) {
                return None;
            }
            assignment(line).map(|(key, _)| key.to_string())
        })
        .collect()
}

/// A key's value as text: strings unquoted, anything else as TOML spells it.
pub fn get(content: &str, section: &str, key: &str) -> Option<String> {
    match get_value(content, section, key)? {
        toml::Value::String(value) => Some(value),
        value => Some(value.to_string()),
    }
}

/// Sets a key to a string value, keeping the line's alignment, or appends it.
pub fn set(content: &str, section: &str, key: &str, value: &str) -> String {
    set_value(
        content,
        section,
        key,
        &toml::Value::String(value.to_string()),
    )
}

/// Sets a key, keeping the line's alignment, or appends it.
pub fn set_value(content: &str, section: &str, key: &str, value: &toml::Value) -> String {
    let spelled = value.to_string();
    let mut replaced = false;
    let mut out = String::with_capacity(content.len() + spelled.len());

    for (line, is_key) in lines_with_key(content, section, key) {
        match line.split_once('=') {
            Some((name, _)) if is_key && !replaced => {
                out.push_str(&format!("{name}= {spelled}"));
                replaced = true;
            }
            _ => out.push_str(line),
        }
        out.push('\n');
    }

    if !replaced {
        out.push_str(&format!("{key} = {spelled}\n"));
    }
    out
}

/// Removes a key's line so the shell falls back to its own default.
pub fn unset(content: &str, section: &str, key: &str) -> String {
    let mut out = String::with_capacity(content.len());
    for (line, is_key) in lines_with_key(content, section, key) {
        if !is_key {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

// Each line, and whether it assigns `key` inside `section` (or before any
// header, as in files written without one).
fn lines_with_key<'a>(
    content: &'a str,
    section: &'a str,
    key: &'a str,
) -> impl Iterator<Item = (&'a str, bool)> {
    let mut current: Option<&str> = None;
    content.lines().map(move |line| {
        if let Some(name) = header_name(line) {
            current = Some(name);
        }
        let in_section = current.is_none_or(|name| name == section);
        let is_key = in_section && assignment(line).is_some_and(|(name, _)| name == key);
        (line, is_key)
    })
}

// `key = value` on an uncommented line, with a key the shell could use.
fn assignment(line: &str) -> Option<(&str, &str)> {
    let line = line.trim_start();
    if line.starts_with('#') {
        return None;
    }
    let (key, value) = line.split_once('=')?;
    let key = key.trim();
    is_key_name(key).then_some((key, value.trim()))
}

fn is_key_name(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

fn parse_value(text: &str) -> Option<toml::Value> {
    toml::from_str::<toml::Table>(&format!("v = {}", text.trim()))
        .ok()?
        .remove("v")
}

/// How the Theme Designer edits a key, from its value in the generated file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    /// A `#rrggbb` color.
    Color,
    /// A reference such as `hyprland.active-border`, or a color in its place.
    Reference,
    /// A 0–1 opacity.
    Alpha,
    Integer,
    Float,
    Bool,
    /// Anything else, such as a gradient or a list of border widths.
    Text,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectionKey {
    pub key: String,
    /// The value Omarchy generates, or the commented-out example value.
    pub default: toml::Value,
    pub kind: KeyKind,
    /// Commented out in the template: the shell's own default applies unless
    /// the file sets it.
    pub optional: bool,
}

/// Keys that share one comment block in the template.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct KeyGroup {
    pub help: String,
    pub keys: Vec<SectionKey>,
}

fn kind_of(key: &str, value: &toml::Value) -> KeyKind {
    match value {
        toml::Value::Boolean(_) => KeyKind::Bool,
        toml::Value::Integer(_) => KeyKind::Integer,
        toml::Value::Float(_) if key.ends_with("alpha") => KeyKind::Alpha,
        toml::Value::Float(_) => KeyKind::Float,
        toml::Value::String(text) if text.starts_with("hyprland.") => KeyKind::Reference,
        toml::Value::String(text) if is_hex_color(text) => KeyKind::Color,
        _ => KeyKind::Text,
    }
}

pub fn is_hex_color(text: &str) -> bool {
    text.len() == 7 && text.starts_with('#') && text[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// The keys of a generated section, grouped under the comment above them.
/// A comment block with no keys after it (a section introduction) becomes a
/// group of its own.
pub fn schema(section: &str) -> Vec<KeyGroup> {
    let mut groups: Vec<KeyGroup> = Vec::new();
    let mut current = KeyGroup::default();

    let finish = |groups: &mut Vec<KeyGroup>, current: &mut KeyGroup| {
        if !current.help.is_empty() || !current.keys.is_empty() {
            groups.push(std::mem::take(current));
        }
    };

    for line in section
        .lines()
        .skip_while(|line| header_name(line).is_none())
        .skip(1)
    {
        let text = line.trim();
        if text.is_empty() {
            if current.keys.is_empty() && !current.help.is_empty() {
                finish(&mut groups, &mut current);
            }
            continue;
        }

        let (body, optional) = match text.strip_prefix('#') {
            Some(comment) => (comment.trim(), true),
            None => (text, false),
        };
        let key = body
            .split_once('=')
            .map(|(key, value)| (key.trim(), value))
            .filter(|(key, _)| is_key_name(key))
            .and_then(|(key, value)| Some((key, parse_value(value)?)));

        match key {
            Some((key, default)) => current.keys.push(SectionKey {
                kind: kind_of(key, &default),
                key: key.to_string(),
                default,
                optional,
            }),
            None if optional => {
                if !current.keys.is_empty() {
                    finish(&mut groups, &mut current);
                }
                if !current.help.is_empty() {
                    current.help.push(' ');
                }
                current.help.push_str(body);
            }
            None => {}
        }
    }
    finish(&mut groups, &mut current);
    groups
}

#[cfg(test)]
mod tests {
    use super::{KeyKind, cut, get, get_value, header_name, keys, schema, set, set_value, unset};

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

    #[test]
    fn typed_values_and_unset() {
        let content = "[spacing]\nscale = 1.0\nxs = 3\n";
        assert_eq!(
            set_value(content, "spacing", "xs", &toml::Value::Integer(4)),
            "[spacing]\nscale = 1.0\nxs = 4\n"
        );
        assert_eq!(
            get_value(content, "spacing", "scale"),
            Some(toml::Value::Float(1.0))
        );
        assert_eq!(unset(content, "spacing", "xs"), "[spacing]\nscale = 1.0\n");
        assert_eq!(keys(content, "spacing"), vec!["scale", "xs"]);
    }

    #[test]
    fn schema_groups_keys_under_their_comments() {
        let section = "[controls]\n# Shared tokens.\n\n# Normal: idle chrome.\nnormal-color = \"#a9b1d6\"\nnormal-fill-alpha = 0.04\nnormal-border = \"hyprland.active-border\"\n\n# Momentary fills.\npressed-fill-alpha = 0.22\n# heading = 16\nscale-with-font = true\nwidth = \"2 1\"\n";
        let groups = schema(section);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].help, "Shared tokens.");
        assert!(groups[0].keys.is_empty());
        assert_eq!(groups[1].help, "Normal: idle chrome.");
        let kinds: Vec<KeyKind> = groups[1].keys.iter().map(|k| k.kind).collect();
        assert_eq!(kinds, [KeyKind::Color, KeyKind::Alpha, KeyKind::Reference]);
        let last = &groups[2].keys;
        assert_eq!(last[1].key, "heading");
        assert!(last[1].optional);
        assert_eq!(last[1].kind, KeyKind::Integer);
        assert_eq!(last[2].kind, KeyKind::Bool);
        assert_eq!(last[3].kind, KeyKind::Text);
    }

    #[test]
    fn every_installed_section_has_a_schema() {
        let Ok(shell) = std::fs::read_to_string(
            crate::system::omarchy_paths::themed_templates_dir().join("shell.toml.tpl"),
        ) else {
            return;
        };
        for section in super::sections(&shell) {
            let text = cut(&shell, section).unwrap();
            let groups = schema(&text);
            assert!(
                groups.iter().any(|g| !g.keys.is_empty()),
                "[{section}] has no keys"
            );
        }
    }
}
