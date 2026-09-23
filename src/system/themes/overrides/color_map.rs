//! Hex colors inside a JSON, YAML, TOML, or Lua file, found and replaced in
//! place so the rest of the file stays byte for byte as it was.

use std::ops::Range;

/// One quoted `"#rrggbb"` or `"#rrggbbaa"` in a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorEntry {
    /// Byte range of the color, `#` included, quotes excluded.
    pub range: Range<usize>,
    pub value: String,
    /// The key the color is assigned to, or empty inside an array.
    pub key: String,
    /// The object, table, or section the entry sits in, as a readable path.
    pub group: String,
    /// The `name` of the enclosing object, as in VS Code's `tokenColors`.
    pub name: Option<String>,
}

impl ColorEntry {
    pub fn label(&self) -> String {
        match (&self.name, self.key.is_empty()) {
            (Some(name), false) => format!("{name} · {}", self.key),
            (Some(name), true) => name.clone(),
            (None, false) => self.key.clone(),
            (None, true) => "Color".to_string(),
        }
    }
}

/// A color and every place the file uses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorUse {
    pub value: String,
    pub entries: Vec<usize>,
}

fn is_hex_color(text: &str) -> bool {
    (text.len() == 7 || text.len() == 9)
        && text.starts_with('#')
        && text[1..].chars().all(|c| c.is_ascii_hexdigit())
}

// The key a value on this line is assigned to: `"key": `, `key: `, `key = `.
fn key_before(prefix: &str) -> String {
    let prefix = prefix.trim_end();
    let Some(prefix) = prefix
        .strip_suffix(':')
        .or_else(|| prefix.strip_suffix('='))
    else {
        return String::new();
    };
    let prefix = prefix.trim_end();
    if let Some(quoted) = prefix.strip_suffix('"') {
        return quoted
            .rsplit_once('"')
            .map(|(_, key)| key.to_string())
            .unwrap_or_default();
    }
    let start = prefix
        .rfind(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '$')))
        .map_or(0, |i| i + 1);
    prefix[start..].to_string()
}

// The value of a `"name": "X"` or `name = "X"` assignment on the line.
fn name_on_line(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let rest = trimmed
        .strip_prefix("\"name\"")
        .or_else(|| trimmed.strip_prefix("name"))?;
    let rest = rest.trim_start().strip_prefix([':', '='])?.trim_start();
    let rest = rest.strip_prefix('"')?;
    rest.split_once('"').map(|(name, _)| name.to_string())
}

struct Frame {
    key: String,
    name: Option<String>,
    indent: usize,
}

fn group_path(heading: &[Frame], frames: &[Frame]) -> String {
    heading
        .iter()
        .chain(frames)
        .map(|frame| frame.key.as_str())
        .filter(|key| !key.is_empty())
        .collect::<Vec<_>>()
        .join(" › ")
}

// The `name` of the innermost object, except the document itself, whose name
// is the theme's rather than the entry's.
fn object_name(frames: &[Frame]) -> Option<String> {
    frames
        .iter()
        .skip(1)
        .rev()
        .find_map(|frame| frame.name.clone())
}

/// Every quoted hex color in `content`, in file order.
pub fn scan(content: &str) -> Vec<ColorEntry> {
    let mut entries = Vec::new();
    let mut frames: Vec<Frame> = Vec::new();
    // TOML `[section]` or a YAML `key:` with nothing after it.
    let mut heading: Vec<Frame> = Vec::new();
    let mut offset = 0;

    for line in content.split_inclusive('\n') {
        let text = line.trim_end_matches(['\n', '\r']);
        let trimmed = text.trim_start();
        let indent = text.len() - trimmed.len();

        if let Some(section) = trimmed
            .strip_prefix('[')
            .and_then(|rest| rest.split_once(']'))
            .map(|(section, _)| section)
            .filter(|section| !section.contains(['"', ',']) && !trimmed.starts_with("[["))
        {
            heading = vec![Frame {
                key: section.to_string(),
                name: None,
                indent: 0,
            }];
        } else if let Some(key) = trimmed.strip_suffix(':').filter(|key| {
            !key.is_empty() && !key.contains([' ', '"', '{', '[']) && !trimmed.starts_with('#')
        }) {
            heading.retain(|frame| frame.indent < indent);
            heading.push(Frame {
                key: key.to_string(),
                name: None,
                indent,
            });
        }

        if let Some(name) = name_on_line(text)
            && let Some(frame) = frames.last_mut()
        {
            frame.name = Some(name);
        }

        let bytes = text.as_bytes();
        let mut in_string = false;
        let mut string_start = 0;
        let mut escaped = false;
        for (i, &byte) in bytes.iter().enumerate() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    in_string = false;
                    let value = &text[string_start + 1..i];
                    if is_hex_color(value) {
                        entries.push(ColorEntry {
                            range: offset + string_start + 1..offset + i,
                            value: value.to_string(),
                            key: key_before(&text[..string_start]),
                            group: group_path(&heading, &frames),
                            name: object_name(&frames),
                        });
                    }
                }
                continue;
            }
            match byte {
                b'"' => {
                    in_string = true;
                    string_start = i;
                }
                b'#' if !trimmed.starts_with('"') && text[..i].trim().is_empty() => break,
                b'-' if bytes.get(i + 1) == Some(&b'-') => break,
                b'{' | b'[' => frames.push(Frame {
                    key: key_before(&text[..i]),
                    name: None,
                    indent,
                }),
                b'}' | b']' => {
                    frames.pop();
                }
                _ => {}
            }
        }

        offset += line.len();
    }

    entries
}

/// The distinct colors of `entries`, case-insensitively, in order of first use.
pub fn uses(entries: &[ColorEntry]) -> Vec<ColorUse> {
    let mut uses: Vec<ColorUse> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        let value = entry.value.to_ascii_lowercase();
        match uses.iter_mut().find(|u| u.value == value) {
            Some(existing) => existing.entries.push(index),
            None => uses.push(ColorUse {
                value,
                entries: vec![index],
            }),
        }
    }
    uses
}

/// Replaces the color at each of `ranges` with `hex` (`#rrggbb`), keeping an
/// entry's own alpha and letter case, so every range keeps its length.
pub fn replace(content: &str, ranges: &[Range<usize>], hex: &str) -> String {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() < 6 || !hex[..6].chars().all(|c| c.is_ascii_hexdigit()) {
        return content.to_string();
    }
    let rgb = &hex[..6];
    let mut out = content.to_string();
    for range in ranges {
        let Some(old) = content.get(range.clone()) else {
            continue;
        };
        let upper = old[1..].chars().any(|c| c.is_ascii_uppercase());
        let rgb = if upper {
            rgb.to_ascii_uppercase()
        } else {
            rgb.to_ascii_lowercase()
        };
        let alpha = old.get(7..).unwrap_or_default();
        out.replace_range(range.clone(), &format!("#{rgb}{alpha}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{replace, scan, uses};

    #[test]
    fn json_keys_groups_and_names() {
        let json = r##"{
  "name": "Omarchy",
  "overrides": {
    "claude": "#7aa2f7",
    "text": "#A9B1D6"
  },
  "tokenColors": [
    {
      "name": "Comment",
      "settings": {
        "foreground": "#414868"
      }
    }
  ],
  "list": ["#000000"]
}"##;
        let entries = scan(json);
        let summary: Vec<(String, String, String)> = entries
            .iter()
            .map(|e| (e.group.clone(), e.label(), e.value.clone()))
            .collect();
        assert_eq!(
            summary,
            [
                ("overrides".into(), "claude".into(), "#7aa2f7".into()),
                ("overrides".into(), "text".into(), "#A9B1D6".into()),
                (
                    "tokenColors › settings".into(),
                    "Comment · foreground".into(),
                    "#414868".into()
                ),
                ("list".into(), "Color".into(), "#000000".into()),
            ]
        );
        assert_eq!(&json[entries[0].range.clone()], "#7aa2f7");
    }

    #[test]
    fn names_label_object_entries() {
        let json = "[\n  {\n    \"name\": \"Comment\",\n    \"foreground\": \"#414868\"\n  }\n]\n";
        assert_eq!(scan(json)[0].label(), "Comment · foreground");
    }

    #[test]
    fn toml_yaml_and_lua() {
        let toml = "\"keyword\" = \"color5\"\n\n[palette]\nbackground = \"#1a1b26\"\n";
        let entries = scan(toml);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            (entries[0].group.as_str(), entries[0].key.as_str()),
            ("palette", "background")
        );

        let yaml = "name: omarchy\ncolors:\n  ui_text: \"#a9b1d6\"\n";
        let entries = scan(yaml);
        assert_eq!(
            (entries[0].group.as_str(), entries[0].key.as_str()),
            ("colors", "ui_text")
        );

        let lua = "return {\n  {\n    opts = {\n      colors = {\n        bg = \"#1a1b26\", -- \"#ffffff\"\n      },\n    },\n  },\n}\n";
        let entries = scan(lua);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            (entries[0].group.as_str(), entries[0].key.as_str()),
            ("opts › colors", "bg")
        );
    }

    #[test]
    fn replaces_in_place_keeping_alpha_and_case() {
        let json = "{\"a\": \"#AABBCC\", \"b\": \"#aabbcc80\", \"c\": \"#aabbcc\"}";
        let entries = scan(json);
        let all = uses(&entries);
        assert_eq!(all.len(), 2);
        let ranges: Vec<_> = all[0]
            .entries
            .iter()
            .map(|&i| entries[i].range.clone())
            .collect();
        assert_eq!(
            replace(json, &ranges, "#112233"),
            "{\"a\": \"#112233\", \"b\": \"#aabbcc80\", \"c\": \"#112233\"}"
        );
        let alpha = [entries[1].range.clone()];
        assert_eq!(
            replace(json, &alpha, "#112233ff"),
            "{\"a\": \"#AABBCC\", \"b\": \"#11223380\", \"c\": \"#aabbcc\"}"
        );
    }
}
