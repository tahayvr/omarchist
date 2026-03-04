use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// All fields are `Option` so that absent keys remain absent on save.
#[derive(Debug, Clone, Default)]
pub struct BarSettings {
    pub position: Option<String>,
    pub height: Option<u32>,
    pub layer: Option<String>,
    pub spacing: Option<u32>,
    pub exclusive: Option<bool>,
    pub passthrough: Option<bool>,
    pub output: Option<String>,
    pub margin_top: Option<i32>,
    pub margin_right: Option<i32>,
    pub margin_bottom: Option<i32>,
    pub margin_left: Option<i32>,
}

// from a profile's config.jsonc.
pub fn get_bar_settings(profile_name: &str) -> Option<BarSettings> {
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

    let str_field =
        |key: &str| -> Option<String> { json.get(key)?.as_str().map(|s| s.to_string()) };
    let u32_field = |key: &str| -> Option<u32> { json.get(key)?.as_u64().map(|v| v as u32) };
    let i32_field = |key: &str| -> Option<i32> { json.get(key)?.as_i64().map(|v| v as i32) };
    let bool_field = |key: &str| -> Option<bool> { json.get(key)?.as_bool() };

    // Waybar supports both `margin` (single value) and `margin-top/right/bottom/left`.
    // We prefer the per-side keys; fall back to the unified `margin` key for all sides.
    let unified_margin = json
        .get("margin")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);

    Some(BarSettings {
        position: str_field("position"),
        height: u32_field("height"),
        layer: str_field("layer"),
        spacing: u32_field("spacing"),
        exclusive: bool_field("exclusive"),
        passthrough: bool_field("passthrough"),
        output: str_field("output"),
        margin_top: i32_field("margin-top").or(unified_margin),
        margin_right: i32_field("margin-right").or(unified_margin),
        margin_bottom: i32_field("margin-bottom").or(unified_margin),
        margin_left: i32_field("margin-left").or(unified_margin),
    })
}

// `value` is a JSON-serialized string (e.g. `"\"top\""`, `"26"`, `"true"`).
pub fn set_bar_setting(
    profile_name: &str,
    key: &str,
    value: &serde_json::Value,
) -> Result<(), String> {
    let config_path = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles")
        .join(profile_name)
        .join("config.jsonc");

    let raw =
        fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;

    let new_raw = replace_top_level_value(&raw, key, value)
        .ok_or_else(|| format!("Could not locate or insert key \"{}\" in config", key))?;

    fs::write(&config_path, new_raw).map_err(|e| format!("Failed to write config: {}", e))
}

// 1. If the key already exists, replace its value in-place.
// 2. If the key does not exist, insert it before the first `modules-` key
//    (or just before the closing `}` of the root object).
fn replace_top_level_value(src: &str, key: &str, value: &serde_json::Value) -> Option<String> {
    let value_str = value.to_string();
    let key_pat = format!("\"{}\"", key);

    // Case 1: key already present
    if let Some(key_pos) = src.find(key_pat.as_str()) {
        // Find `:` after the key
        let after_key = &src[key_pos + key_pat.len()..];
        let colon_offset = after_key.find(':')?;
        let after_colon_start = key_pos + key_pat.len() + colon_offset + 1;
        let after_colon = &src[after_colon_start..];

        let value_start_offset = after_colon
            .chars()
            .take_while(|c| c.is_whitespace())
            .map(|c| c.len_utf8())
            .sum::<usize>();
        let value_abs_start = after_colon_start + value_start_offset;

        // Find the end of the value
        let value_abs_end = find_value_end(src, value_abs_start)?;

        let mut result = String::with_capacity(src.len());
        result.push_str(&src[..value_abs_start]);
        result.push_str(&value_str);
        result.push_str(&src[value_abs_end..]);
        return Some(result);
    }

    // Case 2: key absent
    let insert_line = format!("  \"{}\": {},\n", key, value_str);

    let anchor = if let Some(pos) = find_top_level_key(src, "modules-left") {
        pos
    } else {
        src.rfind('}')?
    };

    let mut result = String::with_capacity(src.len() + insert_line.len());
    result.push_str(&src[..anchor]);
    result.push_str(&insert_line);
    result.push_str(&src[anchor..]);
    Some(result)
}

fn find_top_level_key(src: &str, key: &str) -> Option<usize> {
    let key_pat = format!("\"{}\"", key);
    let mut depth = 0usize;
    let mut in_str = false;
    let mut i = 0usize;
    let bytes = src.as_bytes();

    while i < bytes.len() {
        let ch = bytes[i] as char;
        if in_str {
            if ch == '\\' {
                i += 2;
                continue;
            } else if ch == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match ch {
            '"' => {
                if depth == 1 && src[i..].starts_with(key_pat.as_str()) {
                    return Some(i);
                }
                in_str = true;
            }
            '{' | '[' => depth += 1,
            '}' | ']' => {
                if depth > 0 {
                    depth = depth.saturating_sub(1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn find_value_end(src: &str, start: usize) -> Option<usize> {
    let tail = &src[start..];
    let first_char = tail.chars().next()?;

    match first_char {
        '"' => {
            let mut i = 1usize;
            let bytes = tail.as_bytes();
            while i < bytes.len() {
                let c = bytes[i] as char;
                if c == '\\' {
                    i += 2;
                } else if c == '"' {
                    return Some(start + i + 1);
                } else {
                    i += 1;
                }
            }
            None
        }
        '{' | '[' => {
            let open = first_char;
            let close = if open == '{' { '}' } else { ']' };
            let mut depth = 0usize;
            let mut in_s = false;
            for (i, ch) in tail.char_indices() {
                if in_s {
                    if ch == '\\' {
                        continue;
                    } else if ch == '"' {
                        in_s = false;
                    }
                } else {
                    match ch {
                        '"' => in_s = true,
                        c if c == open => depth += 1,
                        c if c == close => {
                            depth -= 1;
                            if depth == 0 {
                                return Some(start + i + 1);
                            }
                        }
                        _ => {}
                    }
                }
            }
            None
        }
        _ => {
            let end = tail
                .find(|c: char| [',', '}', ']', '\n'].contains(&c))
                .unwrap_or(tail.len());
            Some(start + end)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WaybarZone {
    Left,
    Center,
    Right,
}

// A single module entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaybarModule {
    pub key: String,
    pub label: String,
    // Icon char extracted from the module's format/format-icons fields.
    pub icon: String,
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

fn extract_icon(key: &str, root: &serde_json::Value) -> String {
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

// Resolve a `format` string to a display character.
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
    stripped.trim().to_string()
}

// Strip `<span ...>...</span>` wrapper tags from a format string,
// keeping the inner characters. Also un-escapes `\uXXXX` sequences.
fn strip_span_tags(s: &str) -> String {
    // Remove <span ...> and </span>
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

    // Un-escape \uXXXX sequences (they arrive as literal backslash + u + 4 hex)
    unescape_unicode(&result)
}

// Convert `\uXXXX` escape sequences in a string to the actual Unicode chars.
fn unescape_unicode(s: &str) -> String {
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

// Extract a non-empty icon string from a single JSON value (string or array of strings).
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

// Get the first non-empty icon from `format-icons`, whether it's a string, array, or keyed object.
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
            if let Some(v) = obj.get(*key)
                && let Some(icon) = icon_from_value(v)
            {
                return Some(icon);
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

/// Create a new waybar profile by copying the omarchy-default files from the
/// bundled defaults directory. Returns the profile name on success.
pub fn create_waybar_profile(profile_name: &str) -> Result<String, String> {
    let name = profile_name.trim();
    if name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    // Destination: ~/.config/omarchist/waybar/profiles/<name>
    let dest = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles")
        .join(name);

    if dest.exists() {
        return Err(format!("A profile named \"{}\" already exists", name));
    }

    // Source: defaults/omarchist/waybar/profiles/omarchy-default (relative to cwd)
    let src = PathBuf::from("defaults/omarchist/waybar/profiles/omarchy-default");
    if !src.exists() {
        return Err(format!("Default profile source not found at {:?}", src));
    }

    copy_dir_recursive(&src, &dest)?;

    Ok(name.to_string())
}

/// Recursively copy all files and directories from src to dst.
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Failed to create directory {:?}: {}", dst, e))?;

    for entry in
        fs::read_dir(src).map_err(|e| format!("Failed to read directory {:?}: {}", src, e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy {:?} to {:?}: {}", src_path, dst_path, e))?;
        }
    }

    Ok(())
}

/// Rename an existing waybar profile directory.
///
/// Returns the new profile name on success. Fails if:
/// - `new_name` is empty or already exists
/// - the source directory does not exist
pub fn rename_waybar_profile(old_name: &str, new_name: &str) -> Result<String, String> {
    let new = new_name.trim();
    if new.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let profiles_dir = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles");

    let src = profiles_dir.join(old_name);
    let dst = profiles_dir.join(new);

    if !src.exists() {
        return Err(format!("Profile \"{}\" not found", old_name));
    }
    if dst.exists() {
        return Err(format!("A profile named \"{}\" already exists", new));
    }

    fs::rename(&src, &dst).map_err(|e| format!("Failed to rename profile: {}", e))?;

    Ok(new.to_string())
}

/// Duplicate an existing waybar profile into a new directory.
///
/// Returns the new profile name on success. Fails if `new_name` already exists.
pub fn duplicate_waybar_profile(source_name: &str, new_name: &str) -> Result<String, String> {
    let new = new_name.trim();
    if new.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    let profiles_dir = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles");

    let src = profiles_dir.join(source_name);
    let dst = profiles_dir.join(new);

    if !src.exists() {
        return Err(format!("Profile \"{}\" not found", source_name));
    }
    if dst.exists() {
        return Err(format!("A profile named \"{}\" already exists", new));
    }

    copy_dir_recursive(&src, &dst)?;

    Ok(new.to_string())
}

/// Delete a waybar profile directory.
///
/// Returns the name of a remaining profile to switch to, or `None` if no profiles
/// remain. Refuses to delete the profile if it is the only one left.
pub fn delete_waybar_profile(profile_name: &str) -> Result<Option<String>, String> {
    let profiles_dir = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles");

    let target = profiles_dir.join(profile_name);
    if !target.exists() {
        return Err(format!("Profile \"{}\" not found", profile_name));
    }

    // Guard: don't delete the last profile
    let remaining: Vec<String> = list_waybar_profiles()
        .into_iter()
        .filter(|n| n != profile_name)
        .collect();

    if remaining.is_empty() {
        return Err("Cannot delete the last profile".to_string());
    }

    fs::remove_dir_all(&target).map_err(|e| format!("Failed to delete profile: {}", e))?;

    // Return the first remaining profile (sorted) so the caller can switch to it
    let mut sorted = remaining;
    sorted.sort();
    Ok(sorted.into_iter().next())
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

// Apply / backup / restore

fn original_waybar_backup_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| {
        h.join(".config")
            .join("omarchist")
            .join("waybar")
            .join("backup-original")
    })
}

pub fn has_original_waybar_backup() -> bool {
    original_waybar_backup_dir()
        .map(|p| p.exists())
        .unwrap_or(false)
}

/// Takes a one-time backup of ~/.config/waybar
fn backup_original_waybar_config() -> Result<(), String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    let live_waybar = home.join(".config").join("waybar");
    if !live_waybar.exists() {
        // Nothing to back up.
        return Ok(());
    }

    let backup_dir = original_waybar_backup_dir()
        .ok_or_else(|| "Could not determine backup directory".to_string())?;

    if backup_dir.exists() {
        // Backup already taken — never overwrite it.
        return Ok(());
    }

    copy_dir_recursive(&live_waybar, &backup_dir)
        .map_err(|e| format!("Failed to back up original waybar config: {}", e))
}

/// Copies a profile's files from ~/.config/omarchist/waybar/profiles/<name>/
/// into ~/.config/waybar/, replacing whatever is there.
///
/// On the very first call this also backs up the user's original config to
/// ~/.config/omarchist/waybar/backup-original (only if not already backed up).
pub fn apply_waybar_profile(profile_name: &str) -> Result<(), String> {
    backup_original_waybar_config()?;

    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    let profile_dir = home
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles")
        .join(profile_name);

    if !profile_dir.exists() {
        return Err(format!("Profile \"{}\" not found", profile_name));
    }

    let live_waybar = home.join(".config").join("waybar");

    if live_waybar.exists() {
        fs::remove_dir_all(&live_waybar)
            .map_err(|e| format!("Failed to clear ~/.config/waybar: {}", e))?;
    }

    copy_dir_recursive(&profile_dir, &live_waybar)
        .map_err(|e| format!("Failed to apply profile \"{}\": {}", profile_name, e))
}

pub fn restore_original_waybar_config() -> Result<(), String> {
    let backup_dir = original_waybar_backup_dir()
        .ok_or_else(|| "Could not determine backup directory".to_string())?;

    if !backup_dir.exists() {
        return Err("No original waybar backup found".to_string());
    }

    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    let live_waybar = home.join(".config").join("waybar");

    if live_waybar.exists() {
        fs::remove_dir_all(&live_waybar)
            .map_err(|e| format!("Failed to clear ~/.config/waybar: {}", e))?;
    }

    copy_dir_recursive(&backup_dir, &live_waybar)
        .map_err(|e| format!("Failed to restore original waybar config: {}", e))
}

// Module config helpers

pub fn get_module_config(profile_name: &str, module_key: &str) -> serde_json::Value {
    let Some(config_path) = dirs::home_dir().map(|h| {
        h.join(".config")
            .join("omarchist")
            .join("waybar")
            .join("profiles")
            .join(profile_name)
            .join("config.jsonc")
    }) else {
        return serde_json::Value::Null;
    };

    let Ok(raw) = fs::read_to_string(&config_path) else {
        return serde_json::Value::Null;
    };
    let stripped = strip_jsonc_comments(&raw);
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&stripped) else {
        return serde_json::Value::Null;
    };

    json.get(module_key)
        .cloned()
        .unwrap_or(serde_json::Value::Null)
}

pub fn set_module_config_field(
    profile_name: &str,
    module_key: &str,
    field: &str,
    value: &serde_json::Value,
) -> Result<(), String> {
    let config_path = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles")
        .join(profile_name)
        .join("config.jsonc");

    let raw =
        fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;

    let stripped = strip_jsonc_comments(&raw);
    let mut json: serde_json::Value =
        serde_json::from_str(&stripped).map_err(|e| format!("Failed to parse config: {}", e))?;

    // Ensure the module block exists as an object, then set the field
    let block = json
        .as_object_mut()
        .ok_or_else(|| "Config is not a JSON object".to_string())?
        .entry(module_key)
        .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

    if let serde_json::Value::Object(obj) = block {
        obj.insert(field.to_string(), value.clone());
    } else {
        return Err(format!("Module \"{}\" config is not an object", module_key));
    }

    let new_raw = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialise config: {}", e))?;

    fs::write(&config_path, new_raw).map_err(|e| format!("Failed to write config: {}", e))
}

// Module Library — curated list of addable modules

/// A single entry in the module library.
#[derive(Debug, Clone)]
pub struct LibraryModule {
    /// The waybar key (e.g. "clock", "hyprland/workspaces")
    pub key: &'static str,
    /// Short display name
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub icon: &'static str,
    /// Default JSON config block inserted when the module is added.
    /// Use `null` (JSON null) for modules with no useful defaults.
    pub default_config: &'static str,
}

/// The curated library of known Waybar modules.
pub fn module_library() -> Vec<LibraryModule> {
    vec![
        // System
        LibraryModule {
            key: "cpu",
            name: "CPU",
            description: "CPU usage percentage",
            category: "System",
            icon: "󰍛",
            default_config: r#"{"interval": 5, "format": "󰍛 {usage}%"}"#,
        },
        LibraryModule {
            key: "memory",
            name: "Memory",
            description: "RAM usage with swap info",
            category: "System",
            icon: "󰘚",
            default_config: r#"{"interval": 5, "format": " {used:0.1f}G/{total:0.1f}G", "format-icons": ["", "", ""]}"#,
        },
        LibraryModule {
            key: "battery",
            name: "Battery",
            description: "Battery level and charging state",
            category: "System",
            icon: "󰁹",
            default_config: r#"{"interval": 30, "states": {"critical": 15, "warning": 30}, "format": "{icon} {capacity}%", "format-charging": " {capacity}%", "format-plugged": " {capacity}%", "format-icons": ["", "", "", "", ""], "tooltip-format": "{capacity}% - {time}"}"#,
        },
        LibraryModule {
            key: "temperature",
            name: "Temperature",
            description: "CPU / GPU temperature (requires hwmon-path)",
            category: "System",
            icon: "",
            default_config: r#"{"interval": 5, "hwmon-path": "/sys/class/hwmon/hwmon2/temp1_input", "critical-threshold": 80, "format": " {temperatureC}°C", "format-critical": " {temperatureC}°C"}"#,
        },
        LibraryModule {
            key: "disk",
            name: "Disk",
            description: "Disk usage for a path with tooltip",
            category: "System",
            icon: "󰋊",
            default_config: r#"{"interval": 30, "format": "󰋊 {percentage_used}%", "path": "/", "tooltip-format": "{used}/{total} used on {path}"}"#,
        },
        LibraryModule {
            key: "backlight",
            name: "Backlight",
            description: "Screen brightness with device auto-detection",
            category: "System",
            icon: "󰃠",
            default_config: r#"{"format": "{icon} {percent}%", "format-icons": ["󰃞", "󰃟", "󰃠"], "on-scroll-up": "light -A 5", "on-scroll-down": "light -U 5"}"#,
        },
        // Time
        LibraryModule {
            key: "clock",
            name: "Clock",
            description: "Date and time",
            category: "Time",
            icon: "󰥔",
            default_config: r#"{"format": "{:%H:%M}", "tooltip-format": "{:%A %d %B %Y}"}"#,
        },
        // Audio
        LibraryModule {
            key: "pulseaudio",
            name: "PulseAudio",
            description: "Volume control with bluetooth support",
            category: "Audio",
            icon: "󰕾",
            default_config: r#"{"format": "{icon} {volume}%", "format-muted": "󰝟", "format-bluetooth": " {volume}%", "format-bluetooth-muted": " 󰝟", "format-icons": {"default": ["󰕿", "󰖀", "󰕾"], "bluetooth": ["󰥰"]}, "on-click": "pactl set-sink-mute @DEFAULT_SINK@ toggle", "on-click-right": "pavucontrol", "tooltip-format": "{volume}% {desc}"}"#,
        },
        LibraryModule {
            key: "wireplumber",
            name: "WirePlumber",
            description: "Volume control with bluetooth",
            category: "Audio",
            icon: "󰕾",
            default_config: r#"{"format": "{icon} {volume}%", "format-muted": "󰝟", "format-bluetooth": " {volume}%", "format-icons": {"default": ["󰕿", "󰖀", "󰕾"], "bluetooth": ["󰥰"]}, "on-click": "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle", "on-click-right": "pavucontrol", "tooltip-format": "{volume}% {node_name}"}"#,
        },
        // Network
        LibraryModule {
            key: "network",
            name: "Network",
            description: "Wi-Fi / Ethernet status with tooltips",
            category: "Network",
            icon: "󰤨",
            default_config: r#"{"format-wifi": "󰤨 {signalStrength}%", "format-ethernet": "󰈀 {ipaddr}", "format-disconnected": "󰤭", "format-linked": "󰈀 {ifname}", "format-alt": "{ifname}: {ipaddr}/{cidr}", "tooltip-format-wifi": "{essid} ({signalStrength}%)", "tooltip-format-ethernet": "{ifname}: {ipaddr}/{cidr}", "on-click": "nm-connection-editor", "on-click-right": "foot nmtui"}"#,
        },
        LibraryModule {
            key: "bluetooth",
            name: "Bluetooth",
            description: "Bluetooth status",
            category: "Network",
            icon: "󰂯",
            default_config: r#"{"format": "󰂯", "format-connected": "󰂱 {device_alias}", "format-disabled": "󰂲"}"#,
        },
        // Hyprland
        LibraryModule {
            key: "hyprland/workspaces",
            name: "Workspaces",
            description: "Hyprland workspace switcher with icons",
            category: "Hyprland",
            icon: "",
            default_config: r#"{"format": "{name}:{icon}", "format-icons": {"1": "", "2": "", "3": "", "4": "", "5": "", "active": "󱓻", "default": ""}, "all-outputs": false, "sort-by": "number"}"#,
        },
        LibraryModule {
            key: "hyprland/window",
            name: "Active Window",
            description: "Title of the focused window with class support",
            category: "Hyprland",
            icon: "",
            default_config: r#"{"format": "{title}", "max-length": 50, "separate-outputs": true, "rewrite": {"(.*) — Mozilla Firefox": "🌎 $1", "(.*) - fish": "> [$1]"}}"#,
        },
        LibraryModule {
            key: "hyprland/submap",
            name: "Submap",
            description: "Active Hyprland key submap mode",
            category: "Hyprland",
            icon: "✌️",
            default_config: r#"{"format": "✌️ {}", "max-length": 20, "always-on": false, "default-submap": "Default"}"#,
        },
        LibraryModule {
            key: "hyprland/language",
            name: "Language",
            description: "Active keyboard language / layout",
            category: "Hyprland",
            icon: "󰌌",
            default_config: r#"{"format": "󰌌 {short}"}"#,
        },
        // Utilities
        LibraryModule {
            key: "tray",
            name: "System Tray",
            description: "System tray icon area",
            category: "Utilities",
            icon: "󱊔",
            default_config: r#"{"spacing": 4}"#,
        },
        LibraryModule {
            key: "keyboard-state",
            name: "Keyboard State",
            description: "Caps/Num/Scroll lock indicators",
            category: "Utilities",
            icon: "󰌌",
            default_config: r#"{"numlock": true, "capslock": true, "scrolllock": false, "format": "{name} {icon}", "format-icons": {"locked": "", "unlocked": ""}}"#,
        },
        LibraryModule {
            key: "idle-inhibitor",
            name: "Idle Inhibitor",
            description: "Prevent screen from sleeping",
            category: "Utilities",
            icon: "󰅶",
            default_config: r#"{"format": "{icon}", "format-icons": {"activated": "󰅶", "deactivated": "󰾪"}, "tooltip-format-activated": "Screen will stay on", "tooltip-format-deactivated": "Screen will sleep normally", "timeout": 0}"#,
        },
    ]
}

/// Add a module to the given zone of a profile's config.jsonc.
pub fn add_module_to_zone(
    profile_name: &str,
    module_key: &str,
    zone: &WaybarZone,
    default_config: &str,
) -> Result<(), String> {
    let config_path = dirs::home_dir()
        .ok_or_else(|| "Could not determine home directory".to_string())?
        .join(".config")
        .join("omarchist")
        .join("waybar")
        .join("profiles")
        .join(profile_name)
        .join("config.jsonc");

    let raw =
        fs::read_to_string(&config_path).map_err(|e| format!("Failed to read config: {}", e))?;

    // Step 1: append key to zone array
    let zone_key = match zone {
        WaybarZone::Left => "modules-left",
        WaybarZone::Center => "modules-center",
        WaybarZone::Right => "modules-right",
    };
    let raw = append_to_zone_array(&raw, zone_key, module_key)
        .ok_or_else(|| format!("Could not find zone array \"{}\" in config", zone_key))?;

    // Step 2: insert default config block if needed
    let raw = if !default_config.is_empty() && default_config != "null" {
        // Only insert if there is no existing block for this key
        let key_pat = format!("\"{}\"", module_key);
        if raw.contains(key_pat.as_str()) {
            // Already has a block — skip
            raw
        } else {
            let block = format!("  \"{}\": {},\n", module_key, default_config);
            // Insert just before the closing `}` of the root object
            let close = raw
                .rfind('}')
                .ok_or_else(|| "Malformed JSON: no closing `}`".to_string())?;
            let mut result = raw[..close].to_string();
            result.push_str(&block);
            result.push_str(&raw[close..]);
            result
        }
    } else {
        raw
    };

    fs::write(&config_path, raw).map_err(|e| format!("Failed to write config: {}", e))
}

/// Append `module_key` as a new string entry at the end of a zone array
/// (e.g. `"modules-left"`) in a JSONC string, preserving everything else.
fn append_to_zone_array(src: &str, zone_key: &str, module_key: &str) -> Option<String> {
    // Find the zone array key at depth-1
    let key_pos = find_top_level_key(src, zone_key)?;
    // Find `:` after the key
    let after_key = &src[key_pos..];
    let colon_offset = after_key.find(':')?;
    let after_colon_start = key_pos + colon_offset + 1;
    let after_colon = &src[after_colon_start..];

    // Skip whitespace
    let ws = after_colon
        .chars()
        .take_while(|c| c.is_whitespace())
        .map(|c| c.len_utf8())
        .sum::<usize>();
    let arr_start = after_colon_start + ws;

    if src.as_bytes().get(arr_start)? != &b'[' {
        return None;
    }

    // Find the matching `]`
    let arr_end = find_value_end(src, arr_start)?;

    // The content between `[` and `]`
    let inner = &src[arr_start + 1..arr_end - 1];

    // Build new array: trim trailing whitespace from inner, add entry
    let trimmed = inner.trim_end();
    let new_inner = if trimmed.is_empty() {
        format!("\"{}\"", module_key)
    } else if trimmed.ends_with(',') {
        format!("{} \"{}\"", trimmed, module_key)
    } else {
        format!("{}, \"{}\"", trimmed, module_key)
    };

    let mut result = String::with_capacity(src.len() + module_key.len() + 4);
    result.push_str(&src[..arr_start + 1]);
    result.push_str(&new_inner);
    result.push(']');
    result.push_str(&src[arr_end..]);
    Some(result)
}
