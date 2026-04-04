use super::types::WaybarModule;
use super::types::WaybarZone;

pub fn module_label(key: &str) -> String {
    let base = key
        .strip_prefix("custom/")
        .or_else(|| key.strip_prefix("hyprland/"))
        .or_else(|| key.strip_prefix("group/"))
        .unwrap_or(key);

    base.split(['-', '_', '/'])
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn extract_icon(key: &str, root: &serde_json::Value) -> String {
    if let Some(group_key) = key.strip_prefix("group/") {
        if let Some(block) = root.get(key)
            && let Some(modules) = block.get("modules").and_then(|v| v.as_array())
        {
            for child_key in modules.iter().filter_map(|v| v.as_str()) {
                let child_icon = extract_icon(child_key, root);
                if !child_icon.is_empty() {
                    return child_icon;
                }
            }
        }
        let _ = group_key;
        return String::new();
    }

    let block = root.get(key);

    if let Some(fmt) = block.and_then(|b| b.get("format")).and_then(|v| v.as_str()) {
        let resolved = resolve_format(fmt, block);
        if !resolved.is_empty() {
            return resolved;
        }
    }

    if let Some(first) = first_format_icon(block) {
        return first;
    }

    builtin_icon(key)
}

pub fn new_module(key: &str, zone: WaybarZone, json: &serde_json::Value) -> WaybarModule {
    let icon = extract_icon(key, json);
    WaybarModule {
        key: key.to_string(),
        label: module_label(key),
        icon,
        zone,
    }
}

fn resolve_format(fmt: &str, block: Option<&serde_json::Value>) -> String {
    let stripped = strip_span_tags(fmt);

    if stripped.contains("{icon}") {
        if let Some(icon) = first_format_icon(block) {
            return icon;
        }
        return String::new();
    }

    if stripped.contains('{') {
        return String::new();
    }

    stripped.trim().to_string()
}

/// Strip `<span ...>...</span>` wrapper tags from a format string,
/// keeping the inner characters. Also un-escapes `\uXXXX` sequences.
pub fn strip_span_tags(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for ch in s.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' && in_tag {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }

    unescape_unicode(&result)
}

/// Convert `\uXXXX` escape sequences in a string to the actual Unicode chars.
pub fn unescape_unicode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek() == Some(&'u') {
            chars.next(); // consume 'u'
            let hex: String = chars.by_ref().take(4).collect();
            if hex.len() == 4
                && let Ok(code) = u32::from_str_radix(&hex, 16)
                && let Some(unicode_char) = char::from_u32(code)
            {
                result.push(unicode_char);
                continue;
            }
            // Failed — push back literally
            result.push('\\');
            result.push('u');
            result.push_str(&hex);
        } else {
            result.push(ch);
        }
    }
    result
}

fn icon_from_value(v: &serde_json::Value) -> Option<String> {
    if let Some(arr) = v.as_array() {
        return arr
            .iter()
            .filter_map(|i| i.as_str())
            .find(|s| !s.trim().is_empty())
            .map(|s| s.to_string());
    }
    v.as_str()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
}

fn first_format_icon(block: Option<&serde_json::Value>) -> Option<String> {
    let icons = block?.get("format-icons")?;

    if let Some(arr) = icons.as_array() {
        return arr
            .iter()
            .filter_map(|v| v.as_str())
            .find(|s| !s.trim().is_empty())
            .map(|s| s.to_string());
    }

    if let Some(obj) = icons.as_object() {
        let preferred = ["default", "idle", "charging", "headphone", "headset"];
        for key in &preferred {
            if let Some(v) = obj.get(*key)
                && let Some(icon) = icon_from_value(v)
            {
                return Some(icon);
            }
        }
        for v in obj.values() {
            if let Some(icon) = icon_from_value(v) {
                return Some(icon);
            }
        }
    }

    None
}

pub fn builtin_icon(key: &str) -> String {
    match key {
        "clock" => "󰥔".to_string(),
        "tray" => "󱊔".to_string(),
        "cpu" => "󰍛".to_string(),
        "memory" => "󰘚".to_string(),
        "battery" => "󰁹".to_string(),
        "network" => "󰤨".to_string(),
        "bluetooth" => "".to_string(),
        "pulseaudio" | "wireplumber" => "".to_string(),
        "backlight" => "󰃠".to_string(),
        "temperature" => "".to_string(),
        "disk" => "󰋊".to_string(),
        "keyboard-state" => "󰌌".to_string(),
        "hyprland/workspaces" => "".to_string(),
        "hyprland/window" => "".to_string(),
        "custom/screenrecording-indicator" => "󰻂".to_string(),
        "custom/idle-indicator" => "󰒲".to_string(),
        "custom/notification-silencing-indicator" => "󰂚".to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_label_plain_key_capitalised() {
        assert_eq!(
            module_label("clock"),
            "Clock",
            "a plain key should be title-cased"
        );
    }

    #[test]
    fn module_label_strips_custom_prefix() {
        assert_eq!(
            module_label("custom/my-widget"),
            "My Widget",
            "custom/ prefix should be stripped and hyphens converted to spaces"
        );
    }

    #[test]
    fn module_label_strips_hyprland_prefix() {
        assert_eq!(
            module_label("hyprland/workspaces"),
            "Workspaces",
            "hyprland/ prefix should be stripped"
        );
    }

    #[test]
    fn module_label_strips_group_prefix() {
        assert_eq!(
            module_label("group/audio"),
            "Audio",
            "group/ prefix should be stripped"
        );
    }

    #[test]
    fn module_label_hyphen_becomes_space_with_capitalisation() {
        assert_eq!(
            module_label("keyboard-state"),
            "Keyboard State",
            "hyphens should become spaces and each word capitalised"
        );
    }

    #[test]
    fn module_label_underscore_becomes_space_with_capitalisation() {
        assert_eq!(
            module_label("my_module"),
            "My Module",
            "underscores should become spaces and each word capitalised"
        );
    }

    #[test]
    fn module_label_already_single_word_capitalised() {
        assert_eq!(module_label("cpu"), "Cpu");
        assert_eq!(module_label("memory"), "Memory");
        assert_eq!(module_label("tray"), "Tray");
    }

    #[test]
    fn unescape_unicode_converts_valid_sequence() {
        let result = unescape_unicode("\\u0041");
        assert_eq!(result, "A", "\\u0041 should unescape to 'A'");
    }

    #[test]
    fn unescape_unicode_converts_emoji_codepoint() {
        let result = unescape_unicode("\\u2764");
        assert_eq!(result, "❤", "\\u2764 should unescape to ❤");
    }

    #[test]
    fn unescape_unicode_mixed_content_preserved() {
        let result = unescape_unicode("hello \\u0041 world");
        assert_eq!(
            result, "hello A world",
            "text surrounding escape sequence should be preserved"
        );
    }

    #[test]
    fn unescape_unicode_invalid_hex_emitted_literally() {
        let result = unescape_unicode("\\uXYZW");
        assert_eq!(
            result, "\\uXYZW",
            "an invalid \\u escape should be emitted literally"
        );
    }

    #[test]
    fn unescape_unicode_incomplete_escape_emitted_literally() {
        let result = unescape_unicode("\\u00");
        assert_eq!(
            result, "\\u00",
            "an incomplete \\u escape should be emitted literally"
        );
    }

    #[test]
    fn unescape_unicode_no_escapes_unchanged() {
        let input = "plain string";
        assert_eq!(
            unescape_unicode(input),
            input,
            "strings without escapes should pass through unchanged"
        );
    }

    #[test]
    fn unescape_unicode_multiple_sequences() {
        let result = unescape_unicode("\\u0048\\u0069");
        assert_eq!(
            result, "Hi",
            "multiple escape sequences should all be converted"
        );
    }

    #[test]
    fn strip_span_tags_removes_open_and_close_tags() {
        let input = r#"<span color="red">X</span>"#;
        let result = strip_span_tags(input);
        assert_eq!(
            result, "X",
            "span tags should be stripped, inner content kept"
        );
    }

    #[test]
    fn strip_span_tags_no_tags_unchanged() {
        let input = "󰥔";
        assert_eq!(
            strip_span_tags(input),
            "󰥔",
            "input without span tags should be unchanged"
        );
    }

    #[test]
    fn strip_span_tags_empty_span_gives_empty() {
        let input = "<span></span>";
        assert_eq!(
            strip_span_tags(input),
            "",
            "an empty span should yield an empty string"
        );
    }

    #[test]
    fn strip_span_tags_unicode_escape_within_span_is_unescaped() {
        let input = r#"<span>\\u0041</span>"#;
        let result = strip_span_tags(input);
        assert!(
            !result.contains('<') && !result.contains('>'),
            "no angle brackets should remain after stripping tags"
        );
    }

    #[test]
    fn builtin_icon_known_modules_return_nonempty_strings() {
        let known = [
            "clock",
            "tray",
            "cpu",
            "memory",
            "battery",
            "network",
            "backlight",
            "disk",
            "keyboard-state",
            "custom/screenrecording-indicator",
            "custom/idle-indicator",
            "custom/notification-silencing-indicator",
        ];
        for key in &known {
            let icon = builtin_icon(key);
            assert!(
                !icon.is_empty(),
                "builtin_icon(\"{}\") should return a non-empty icon string",
                key
            );
        }
    }

    #[test]
    fn builtin_icon_unknown_key_returns_empty() {
        assert_eq!(
            builtin_icon("nonexistent-module"),
            "",
            "unknown module keys should return an empty string"
        );
    }

    #[test]
    fn builtin_icon_clock_is_correct_glyph() {
        assert_eq!(builtin_icon("clock"), "󰥔");
    }

    #[test]
    fn builtin_icon_pulseaudio_and_wireplumber_equal() {
        assert_eq!(
            builtin_icon("pulseaudio"),
            builtin_icon("wireplumber"),
            "pulseaudio and wireplumber should use the same icon"
        );
    }
}
