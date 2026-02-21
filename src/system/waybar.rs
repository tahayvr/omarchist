use std::fs;

use serde::{Deserialize, Serialize};

/// The three zones of the waybar
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WaybarZone {
    Left,
    Center,
    Right,
}

/// A single module entry in the waybar config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaybarModule {
    /// The raw key from the config (e.g. "hyprland/workspaces", "custom/omarchy", "clock")
    pub key: String,
    /// Human-readable display label derived from the key
    pub label: String,
    /// Icon character(s) extracted from the module's format/format-icons fields.
    /// Empty string if none could be determined.
    pub icon: String,
    /// Which zone this module belongs to
    pub zone: WaybarZone,
}

impl WaybarModule {
    fn new(key: &str, zone: WaybarZone, json: &serde_json::Value) -> Self {
        let icon = extract_icon(key, json);
        Self {
            key: key.to_string(),
            label: module_label(key),
            icon,
            zone,
        }
    }
}

/// Derive a short display label from a waybar module key
fn module_label(key: &str) -> String {
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

/// Extract a representative icon string from a module's JSON config block.
fn extract_icon(key: &str, root: &serde_json::Value) -> String {
    // group/* modules: look up the expand-icon child or first module child
    if let Some(group_key) = key.strip_prefix("group/") {
        // Try the group's own config block first
        if let Some(block) = root.get(key) {
            // modules list inside the group
            if let Some(modules) = block.get("modules").and_then(|v| v.as_array()) {
                for child_key in modules.iter().filter_map(|v| v.as_str()) {
                    let child_icon = extract_icon(child_key, root);
                    if !child_icon.is_empty() {
                        return child_icon;
                    }
                }
            }
        }
        // Fallback label from group name
        let _ = group_key;
        return String::new();
    }

    // Look up this module's own config block in the root JSON
    let block = root.get(key);

    // 1. Plain `format` with no dynamic tokens — use it directly
    //    Handles: "", "󰍛", "", "<span ...>\ue900</span>", etc.
    if let Some(fmt) = block.and_then(|b| b.get("format")).and_then(|v| v.as_str()) {
        let resolved = resolve_format(fmt, block);
        if !resolved.is_empty() {
            return resolved;
        }
    }

    // 2. No format, but has format-icons (e.g. indicators with return-type: json)
    if let Some(first) = first_format_icon(block) {
        return first;
    }

    // 3. Known built-ins with no config block in this file
    builtin_icon(key)
}

/// Resolve a `format` string to a display character.
fn resolve_format(fmt: &str, block: Option<&serde_json::Value>) -> String {
    // Strip HTML-style <span ...>...</span> tags, keep inner text
    let stripped = strip_span_tags(fmt);

    // If it contains {icon}, substitute with the first format-icon value
    if stripped.contains("{icon}") {
        if let Some(icon) = first_format_icon(block) {
            return icon;
        }
        return String::new();
    }

    // If it contains any other {…} token (like {capacity}, {:L%A %H:%M}), skip
    if stripped.contains('{') {
        return String::new();
    }

    // Plain text — trim whitespace and return
    let trimmed = stripped.trim().to_string();
    trimmed
}

/// Strip `<span ...>...</span>` wrapper tags from a format string,
/// keeping the inner characters. Also un-escapes `\uXXXX` sequences.
fn strip_span_tags(s: &str) -> String {
    // Remove <span ...> and </span>
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    let mut in_tag = false;

    while let Some(ch) = chars.next() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' && in_tag {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }

    // Un-escape \uXXXX sequences (they arrive as literal backslash + u + 4 hex)
    unescape_unicode(&result)
}

/// Convert `\uXXXX` escape sequences in a string to the actual Unicode chars.
fn unescape_unicode(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek() == Some(&'u') {
            chars.next(); // consume 'u'
            let hex: String = chars.by_ref().take(4).collect();
            if hex.len() == 4 {
                if let Ok(code) = u32::from_str_radix(&hex, 16) {
                    if let Some(unicode_char) = char::from_u32(code) {
                        result.push(unicode_char);
                        continue;
                    }
                }
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

/// Extract a non-empty icon string from a single JSON value (string or array of strings).
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

/// Get the first non-empty icon from `format-icons`, whether it's a string, array, or keyed object.
fn first_format_icon(block: Option<&serde_json::Value>) -> Option<String> {
    let icons = block?.get("format-icons")?;

    // format-icons can be an array of icon strings
    if let Some(arr) = icons.as_array() {
        return arr
            .iter()
            .filter_map(|v| v.as_str())
            .find(|s| !s.trim().is_empty())
            .map(|s| s.to_string());
    }

    if let Some(obj) = icons.as_object() {
        // Preferred keys in order; fall back to any value that is non-empty
        let preferred = ["default", "idle", "charging", "headphone", "headset"];
        for key in &preferred {
            if let Some(v) = obj.get(*key) {
                if let Some(icon) = icon_from_value(v) {
                    return Some(icon);
                }
            }
        }
        // Last resort: first non-empty value in insertion order
        for v in obj.values() {
            if let Some(icon) = icon_from_value(v) {
                return Some(icon);
            }
        }
    }

    None
}

/// Fallback icons for built-in waybar modules that have no config block in the file.
fn builtin_icon(key: &str) -> String {
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
        // Indicator modules: no format/format-icons in config; use representative icons
        "custom/screenrecording-indicator" => "󰻂".to_string(),
        "custom/idle-indicator" => "󰒲".to_string(),
        "custom/notification-silencing-indicator" => "󰂚".to_string(),
        _ => String::new(),
    }
}

/// The parsed waybar configuration for one profile
#[derive(Debug, Clone)]
pub struct WaybarConfig {
    pub profile_name: String,
    pub modules_left: Vec<WaybarModule>,
    pub modules_center: Vec<WaybarModule>,
    pub modules_right: Vec<WaybarModule>,
}

impl WaybarConfig {
    pub fn all_modules(&self) -> Vec<&WaybarModule> {
        self.modules_left
            .iter()
            .chain(self.modules_center.iter())
            .chain(self.modules_right.iter())
            .collect()
    }
}

/// Strip `//` line comments and `/* */` block comments from a JSONC string.
fn strip_jsonc_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();
    let mut in_string = false;
    let mut in_block_comment = false;

    while let Some(ch) = chars.next() {
        if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if in_string {
            out.push(ch);
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            out.push(ch);
        } else if ch == '/' {
            match chars.peek() {
                Some('/') => {
                    for c in chars.by_ref() {
                        if c == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                }
                Some('*') => {
                    chars.next();
                    in_block_comment = true;
                }
                _ => {
                    out.push(ch);
                }
            }
        } else {
            out.push(ch);
        }
    }

    out
}

/// Save the current module order back to the config.jsonc file for a profile.
///
/// Only the `modules-left`, `modules-center`, and `modules-right` arrays are
/// rewritten. All other keys, comments, and module config blocks are preserved
/// exactly as they appear in the file.
pub fn save_waybar_config(config: &WaybarConfig) -> Result<(), String> {
    let config_path = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles")
        .join(&config.profile_name)
        .join("config.jsonc");

    let raw =
        fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;

    let keys_and_modules = [
        (
            "modules-left",
            config
                .modules_left
                .iter()
                .map(|m| m.key.as_str())
                .collect::<Vec<_>>(),
        ),
        (
            "modules-center",
            config
                .modules_center
                .iter()
                .map(|m| m.key.as_str())
                .collect::<Vec<_>>(),
        ),
        (
            "modules-right",
            config
                .modules_right
                .iter()
                .map(|m| m.key.as_str())
                .collect::<Vec<_>>(),
        ),
    ];

    let mut result = raw;
    for (json_key, module_keys) in &keys_and_modules {
        result = replace_module_array(&result, json_key, module_keys)
            .ok_or_else(|| format!("Could not find \"{}\" in config", json_key))?;
    }

    fs::write(&config_path, result).map_err(|e| format!("Failed to write config: {}", e))
}

/// Replace a JSON array value for the given key in a JSONC string, preserving
/// all surrounding content, comments, and formatting.
///
/// Matches: `"<key>": [... anything up to the closing `]` ...]`
/// The replacement array is written as a compact single-line JSON array.
fn replace_module_array(src: &str, key: &str, values: &[&str]) -> Option<String> {
    // Build compact JSON array: ["a", "b", "c"]
    let new_array = {
        let items: Vec<String> = values
            .iter()
            .map(|v| format!("\"{}\"", v.replace('"', "\\\"")))
            .collect();
        format!("[{}]", items.join(", "))
    };

    // Find the key in the source, then locate its array value and replace it.
    // We search for `"<key>"` followed by `:` (possibly with whitespace/newlines)
    // then the opening `[`. We track bracket depth to find the matching `]`.
    let key_pat = format!("\"{}\"", key);
    let key_pos = src.find(key_pat.as_str())?;

    // Find `:` after the key
    let after_key = &src[key_pos + key_pat.len()..];
    let colon_offset = after_key.find(':')?;
    let after_colon = &after_key[colon_offset + 1..];

    // Find opening `[`
    let bracket_offset = after_colon.find('[')?;

    // Absolute position of `[`
    let open_pos = key_pos + key_pat.len() + colon_offset + 1 + bracket_offset;

    // Walk forward from `[` to find the matching `]`, respecting strings
    let mut depth = 0usize;
    let mut in_str = false;
    let mut close_pos = None;
    let mut chars = src[open_pos..].char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        if in_str {
            if ch == '\\' {
                chars.next(); // skip escaped char
            } else if ch == '"' {
                in_str = false;
            }
        } else {
            match ch {
                '"' => in_str = true,
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        close_pos = Some(open_pos + i);
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    let close_pos = close_pos?;
    let mut result = String::with_capacity(src.len());
    result.push_str(&src[..open_pos]);
    result.push_str(&new_array);
    result.push_str(&src[close_pos + 1..]);
    Some(result)
}

/// Load and parse the waybar config for a given profile name.
pub fn load_waybar_config(profile_name: &str) -> Option<WaybarConfig> {
    let config_path = dirs::home_dir()?
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles")
        .join(profile_name)
        .join("config.jsonc");

    let raw = fs::read_to_string(&config_path).ok()?;
    let stripped = strip_jsonc_comments(&raw);
    let json: serde_json::Value = serde_json::from_str(&stripped).ok()?;

    let parse_modules = |list_key: &str, zone: WaybarZone| -> Vec<WaybarModule> {
        json.get(list_key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| WaybarModule::new(s, zone.clone(), &json))
                    .collect()
            })
            .unwrap_or_default()
    };

    Some(WaybarConfig {
        profile_name: profile_name.to_string(),
        modules_left: parse_modules("modules-left", WaybarZone::Left),
        modules_center: parse_modules("modules-center", WaybarZone::Center),
        modules_right: parse_modules("modules-right", WaybarZone::Right),
    })
}

/// List all available profile names from `~/.config/omarchist/waybar/profiles/`
pub fn list_waybar_profiles() -> Vec<String> {
    let Some(home) = dirs::home_dir() else {
        return vec![];
    };
    let profiles_dir = home
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles");

    fs::read_dir(&profiles_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default()
}
