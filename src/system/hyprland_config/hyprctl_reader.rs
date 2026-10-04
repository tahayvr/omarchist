//! Reads the running compositor's settings with one `hyprctl --batch` of
//! `j/getoption` calls. The keys and their types come from the model
//! itself (`HyprlandConfig::default()` flattened to dotted paths), so
//! adding a setting is adding a field to the model.
use std::collections::HashMap;
use std::process::Command;

use serde::Deserialize;
use serde_json::Value;

use super::baseline::{EMPTY_PLACEHOLDER, flatten, set_path};
use crate::types::hyprland_config::*;

/// One `j/getoption` record. Hyprland reports each option under a
/// type-specific key: `int`, `float`, `bool`, `str`, `vec2`, and `css` /
/// `custom` / `gradient` for multi-value options such as gaps.
#[derive(Debug, Default, Deserialize)]
struct HyprctlOption {
    #[serde(default)]
    int: Option<i64>,
    #[serde(default)]
    float: Option<f64>,
    #[serde(rename = "bool", default)]
    boolean: Option<bool>,
    #[serde(rename = "str", default)]
    string: Option<String>,
    #[serde(default)]
    custom: Option<String>,
    #[serde(default)]
    css: Option<String>,
    #[serde(default)]
    vec2: Option<[f64; 2]>,
}

#[derive(Debug, Deserialize)]
struct HyprctlRecord {
    option: String,
    #[serde(flatten)]
    value: HyprctlOption,
}

/// The `hyprctl` spelling of a model path: colons between the parts.
pub fn hyprctl_key(path: &str) -> String {
    path.replace('.', ":")
}

struct Options(HashMap<String, HyprctlOption>);

impl Options {
    fn fetch(keys: &[String]) -> Self {
        let batch = keys
            .iter()
            .map(|k| format!("j/getoption {k}"))
            .collect::<Vec<_>>()
            .join("; ");

        // Unknown options print "no such option" instead of JSON; parse
        // whatever came back rather than failing the whole read.
        match Command::new("hyprctl").args(["--batch", &batch]).output() {
            Ok(output) => Self::parse(&String::from_utf8_lossy(&output.stdout)),
            Err(_) => Self(HashMap::new()),
        }
    }

    fn parse(text: &str) -> Self {
        Self(
            text.lines()
                .filter_map(|line| serde_json::from_str::<HyprctlRecord>(line.trim()).ok())
                .map(|record| (record.option, record.value))
                .collect(),
        )
    }

    fn get(&self, key: &str) -> Option<&HyprctlOption> {
        self.0.get(key)
    }

    /// The option's value in the shape of `default`, which says what type
    /// the model expects: bools also accept ints, integers also take the
    /// first number of a css list (`gaps_in` is `"5 5 5 5"`), floats also
    /// accept ints, and unset strings (`[[EMPTY]]`) are empty.
    fn value(&self, key: &str, default: &Value) -> Option<Value> {
        let opt = self.get(key)?;
        match default {
            Value::Bool(_) => opt.boolean.or(opt.int.map(|v| v != 0)).map(Value::Bool),
            Value::Number(n) if n.is_i64() => opt
                .int
                .or_else(|| {
                    let raw = opt.css.as_deref().or(opt.custom.as_deref())?;
                    raw.split_whitespace().next()?.parse::<i64>().ok()
                })
                .map(Value::from),
            Value::Number(_) => opt.float.or(opt.int.map(|v| v as f64)).map(Value::from),
            Value::String(_) => opt.string.as_ref().map(|s| {
                Value::String(if s == EMPTY_PLACEHOLDER {
                    String::new()
                } else {
                    s.clone()
                })
            }),
            Value::Array(_) => opt
                .vec2
                .map(|[x, y]| Value::Array(vec![Value::from(x), Value::from(y)])),
            _ => None,
        }
    }
}

/// Every option the model has, as `hyprctl` names it.
pub fn option_keys() -> Vec<String> {
    let defaults = serde_json::to_value(HyprlandConfig::default()).unwrap_or(Value::Null);
    flatten(&defaults)
        .keys()
        .map(|path| hyprctl_key(path))
        .collect()
}

/// Read as many Hyprland settings as possible from the running compositor and
/// return them as a [`HyprlandConfig`].
///
/// Only the fields that can be retrieved are overridden; everything else
/// keeps its `Default` value so we always return a fully-populated struct.
pub fn read_from_hyprctl() -> HyprlandConfig {
    read_from_options(&Options::fetch(&option_keys()))
}

fn read_from_options(opts: &Options) -> HyprlandConfig {
    let mut value = serde_json::to_value(HyprlandConfig::default()).unwrap_or(Value::Null);
    for (path, default) in flatten(&value) {
        if let Some(read) = opts.value(&hyprctl_key(&path), &default) {
            set_path(&mut value, &path, read);
        }
    }
    serde_json::from_value(value).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"option": "general:border_size", "int": 3, "set": true }


{"option": "general:gaps_in", "css": "7 7 7 7", "set": true }


{"option": "general:resize_on_border", "bool": true, "set": true }


{"option": "decoration:active_opacity", "float": 0.950000, "set": true }


{"option": "general:layout", "str": "master", "set": true }
{"option": "input:accel_profile", "str": "[[EMPTY]]", "set": true }
{"option": "decoration:shadow:offset", "vec2": [1,2], "set": false }
{"option": "input:follow_mouse", "int": 2, "set": true }
{"option": "misc:vrr", "int": 1, "set": true }
no such option
"#;

    #[test]
    fn read_from_options_reads_every_value_type_and_skips_noise() {
        let cfg = read_from_options(&Options::parse(SAMPLE));
        let defaults = HyprlandConfig::default();
        assert_eq!(cfg.general.border_size, 3);
        assert_eq!(cfg.general.gaps_in, 7, "first number of a css list");
        assert!(cfg.general.resize_on_border);
        assert_eq!(cfg.decoration.active_opacity, 0.95);
        assert_eq!(cfg.general.layout, "master");
        assert_eq!(cfg.decoration.shadow.offset, [1.0, 2.0]);
        assert_eq!(cfg.input.follow_mouse, 2);
        assert_eq!(cfg.misc.vrr, 1);
        assert_eq!(
            cfg.general.gaps_out, defaults.general.gaps_out,
            "unread keys keep defaults"
        );
    }

    #[test]
    fn unset_strings_read_as_empty_not_the_placeholder() {
        let cfg = read_from_options(&Options::parse(SAMPLE));
        assert_eq!(cfg.input.accel_profile, "");
    }

    #[test]
    fn option_keys_are_unique_and_colon_separated() {
        let keys = option_keys();
        let unique: std::collections::HashSet<_> = keys.iter().collect();
        assert_eq!(unique.len(), keys.len());
        assert!(keys.contains(&"input:touchpad:tap_to_click".to_string()));
        assert!(keys.iter().all(|k| !k.contains('.')));
    }
}

#[cfg(test)]
pub(crate) mod defaults_audit {
    //! `HyprlandConfig::default()` is what an overridden key resets to when
    //! no Lua file sets it, so it must match Hyprland's own defaults, and
    //! every key must be an option Hyprland knows. This compares the whole
    //! model against `hyprctl descriptions`, and is skipped where no
    //! compositor is running.
    use std::collections::HashMap;
    use std::process::Command;

    use serde_json::Value;

    use super::EMPTY_PLACEHOLDER;
    use crate::system::hyprland_config::baseline::{flatten, values_equal};
    use crate::types::hyprland_config::HyprlandConfig;

    /// `hyprctl descriptions` for every option, keyed by the model's
    /// spelling (dots, underscores), or `None` without a compositor.
    pub(crate) fn described_options() -> Option<HashMap<String, Value>> {
        let output = Command::new("hyprctl")
            .args(["descriptions", "-j"])
            .output()
            .ok()?;
        let described = serde_json::from_slice::<Vec<Value>>(&output.stdout).ok()?;
        Some(
            described
                .into_iter()
                .filter_map(|option| {
                    let name = option["name"].as_str()?;
                    Some((name.replace(':', ".").replace('-', "_"), option))
                })
                .collect(),
        )
    }

    #[test]
    fn rust_defaults_match_hyprland_descriptions() {
        let Some(described) = described_options() else {
            eprintln!("skipping: no hyprctl");
            return;
        };
        let defaults = serde_json::to_value(HyprlandConfig::default()).unwrap();
        let mut mismatches = Vec::new();
        for (path, ours) in flatten(&defaults) {
            let Some(option) = described.get(&path) else {
                mismatches.push(format!("{path}: not a Hyprland option"));
                continue;
            };
            let theirs = match &option["default"] {
                // `gaps_in` and friends describe as "5 5 5 5"; the model keeps one number.
                Value::String(s) if ours.is_number() => s
                    .split_whitespace()
                    .next()
                    .and_then(|t| t.parse::<f64>().ok())
                    .map(Value::from),
                Value::String(s) if s == EMPTY_PLACEHOLDER => Some(Value::String(String::new())),
                other => Some(other.clone()),
            };
            if !theirs.as_ref().is_some_and(|t| values_equal(&ours, t)) {
                mismatches.push(format!(
                    "{path}: model {ours}, Hyprland {}",
                    option["default"]
                ));
            }
        }
        assert!(
            mismatches.is_empty(),
            "defaults differ:\n{}",
            mismatches.join("\n")
        );
    }
}
