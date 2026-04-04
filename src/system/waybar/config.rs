use std::fs;

use super::icons::new_module;
use super::jsonc::{find_top_level_key, find_value_end, strip_jsonc_comments};
use super::types::{BarSettings, WaybarConfig, WaybarModule, WaybarZone};

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

/// `value` is a JSON-serialized value (e.g. `"\"top\""`, `"26"`, `"true"`).
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
                    .map(|s| new_module(s, zone.clone(), &json))
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

    let zone_key = match zone {
        WaybarZone::Left => "modules-left",
        WaybarZone::Center => "modules-center",
        WaybarZone::Right => "modules-right",
    };
    let raw = append_to_zone_array(&raw, zone_key, module_key)
        .ok_or_else(|| format!("Could not find zone array \"{}\" in config", zone_key))?;

    let raw = if !default_config.is_empty() && default_config != "null" {
        let key_pat = format!("\"{}\"", module_key);
        if raw.contains(key_pat.as_str()) {
            raw
        } else {
            let block = format!("  \"{}\": {},\n", module_key, default_config);
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

// 1. If the key already exists, replace its value in-place.
// 2. If the key does not exist, insert it before the first `modules-` key
//    (or just before the closing `}` of the root object).
pub fn replace_top_level_value(src: &str, key: &str, value: &serde_json::Value) -> Option<String> {
    let value_str = value.to_string();
    let key_pat = format!("\"{}\"", key);

    // Case 1: key already present
    if let Some(key_pos) = src.find(key_pat.as_str()) {
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

        let value_abs_end = find_value_end(src, value_abs_start)?;

        let mut result = String::with_capacity(src.len());
        result.push_str(&src[..value_abs_start]);
        result.push_str(&value_str);
        result.push_str(&src[value_abs_end..]);
        return Some(result);
    }

    // Case 2: key absent — insert before first `modules-` key or before closing `}`
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

pub fn replace_module_array(src: &str, key: &str, values: &[&str]) -> Option<String> {
    let new_array = {
        let items: Vec<String> = values
            .iter()
            .map(|v| format!("\"{}\"", v.replace('"', "\\\"")))
            .collect();
        format!("[{}]", items.join(", "))
    };

    let key_pat = format!("\"{}\"", key);
    let key_pos = src.find(key_pat.as_str())?;

    let after_key = &src[key_pos + key_pat.len()..];
    let colon_offset = after_key.find(':')?;
    let after_colon = &after_key[colon_offset + 1..];

    let bracket_offset = after_colon.find('[')?;

    let open_pos = key_pos + key_pat.len() + colon_offset + 1 + bracket_offset;

    let mut depth = 0usize;
    let mut in_str = false;
    let mut close_pos = None;
    let mut chars = src[open_pos..].char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        if in_str {
            if ch == '\\' {
                chars.next();
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

pub fn append_to_zone_array(src: &str, zone_key: &str, module_key: &str) -> Option<String> {
    let key_pos = find_top_level_key(src, zone_key)?;
    let after_key = &src[key_pos..];
    let colon_offset = after_key.find(':')?;
    let after_colon_start = key_pos + colon_offset + 1;
    let after_colon = &src[after_colon_start..];

    let ws = after_colon
        .chars()
        .take_while(|c| c.is_whitespace())
        .map(|c| c.len_utf8())
        .sum::<usize>();
    let arr_start = after_colon_start + ws;

    if src.as_bytes().get(arr_start)? != &b'[' {
        return None;
    }

    let arr_end = find_value_end(src, arr_start)?;

    let inner = &src[arr_start + 1..arr_end - 1];

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_module_array_replaces_existing_array() {
        let src = r#"{
  "modules-left": ["clock", "cpu"],
  "modules-right": ["tray"]
}"#;
        let result = replace_module_array(src, "modules-left", &["memory", "disk"])
            .expect("replace_module_array should succeed when key exists");
        assert!(
            result.contains(r#""modules-left": ["memory", "disk"]"#),
            "the array should be replaced with the new values"
        );
        assert!(
            result.contains(r#""modules-right": ["tray"]"#),
            "other arrays should remain unchanged"
        );
    }

    #[test]
    fn replace_module_array_empty_replacement() {
        let src = r#"{"modules-left": ["clock", "cpu"]}"#;
        let result = replace_module_array(src, "modules-left", &[])
            .expect("replacing with an empty slice should succeed");
        assert!(
            result.contains(r#""modules-left": []"#),
            "replacing with empty slice should produce an empty JSON array"
        );
    }

    #[test]
    fn replace_module_array_missing_key_returns_none() {
        let src = r#"{"modules-left": ["clock"]}"#;
        let result = replace_module_array(src, "modules-right", &["tray"]);
        assert!(
            result.is_none(),
            "replace_module_array should return None when the key is absent"
        );
    }

    #[test]
    fn replace_module_array_single_item() {
        let src = r#"{"modules-center": ["clock", "cpu", "memory"]}"#;
        let result = replace_module_array(src, "modules-center", &["tray"])
            .expect("single-item replacement should succeed");
        assert!(
            result.contains(r#""modules-center": ["tray"]"#),
            "single-item replacement should produce a single-element array"
        );
    }

    #[test]
    fn append_to_zone_array_adds_to_non_empty_array() {
        let src = r#"{
  "modules-left": ["clock", "cpu"]
}"#;
        let result = append_to_zone_array(src, "modules-left", "memory")
            .expect("append to non-empty array should succeed");
        assert!(
            result.contains("\"memory\""),
            "the new module key should appear in the output"
        );
        assert!(
            result.contains("\"clock\"") && result.contains("\"cpu\""),
            "existing modules should be preserved"
        );
    }

    #[test]
    fn append_to_zone_array_adds_to_empty_array() {
        let src = r#"{"modules-left": []}"#;
        let result = append_to_zone_array(src, "modules-left", "clock")
            .expect("append to empty array should succeed");
        assert!(
            result.contains("\"clock\""),
            "the module key should be present in a previously-empty array"
        );
    }

    #[test]
    fn append_to_zone_array_missing_key_returns_none() {
        let src = r#"{"modules-left": ["clock"]}"#;
        let result = append_to_zone_array(src, "modules-right", "tray");
        assert!(
            result.is_none(),
            "append_to_zone_array should return None when the zone key is absent"
        );
    }

    #[test]
    fn replace_top_level_value_replaces_existing_string() {
        let src = r#"{"position": "top", "height": 30}"#;
        let new_val = serde_json::json!("bottom");
        let result = replace_top_level_value(src, "position", &new_val)
            .expect("replacing an existing string key should succeed");
        assert!(
            result.contains(r#""position": "bottom""#),
            "the value should be updated to 'bottom'"
        );
        assert!(
            result.contains("\"height\": 30"),
            "other keys should be unchanged"
        );
    }

    #[test]
    fn replace_top_level_value_replaces_existing_number() {
        let src = r#"{"height": 30, "spacing": 4}"#;
        let new_val = serde_json::json!(40);
        let result = replace_top_level_value(src, "height", &new_val)
            .expect("replacing an existing numeric key should succeed");
        assert!(
            result.contains("\"height\": 40"),
            "the numeric value should be updated"
        );
    }

    #[test]
    fn replace_top_level_value_inserts_missing_key_before_modules() {
        let src = "{\n  \"modules-left\": [\"clock\"]\n}";
        let new_val = serde_json::json!("top");
        let result = replace_top_level_value(src, "position", &new_val)
            .expect("inserting a new key should succeed");
        let pos_idx = result
            .find("position")
            .expect("position key should be present");
        let mod_idx = result
            .find("modules-left")
            .expect("modules-left key should be present");
        assert!(
            pos_idx < mod_idx,
            "inserted key should appear before modules-left"
        );
    }
}
