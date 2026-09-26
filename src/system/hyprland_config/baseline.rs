//! What Hyprland runs without Omarchist's settings, and the JSON-path
//! helpers the sparse overrides are stored with.
//!
//! Omarchist only writes the keys the user changed on the Configuration
//! page (`manager.rs`). Every other key follows Omarchy: its value is what
//! the compositor reports. For an overridden key the compositor reports the
//! override itself, so its baseline comes from evaluating the user's
//! `hyprland.lua` with the keybind scanner's stubbed `hl` table, which
//! records every `hl.config` call made by a file other than
//! `omarchist.lua`; a key no file sets falls back to Hyprland's default.
use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Map, Value};

use crate::error::Result;
use crate::system::keybinds::scanner::{ScanEvent, run_scan};

/// Dotted option path (`general.gaps_in`) to the value the last
/// non-Omarchist Lua file set it to.
pub type Scanned = BTreeMap<String, Value>;

/// The file Omarchist writes; its `hl.config` calls are not part of the
/// baseline.
const OMARCHIST_LUA: &str = "omarchist.lua";

/// What `hyprctl` prints for an unset string option.
pub const EMPTY_PLACEHOLDER: &str = "[[EMPTY]]";

/// Evaluates `hyprland_lua` and collects every `hl.config` leaf set by a
/// file other than `omarchist.lua`, later calls overriding earlier ones.
pub fn scan_config(hyprland_lua: &Path) -> Result<Scanned> {
    Ok(collect_scanned(run_scan(hyprland_lua)?))
}

pub fn collect_scanned(events: Vec<ScanEvent>) -> Scanned {
    let mut scanned = Scanned::new();
    for event in events {
        if let ScanEvent::Config {
            source,
            path,
            value,
            ..
        } = event
            && source.file_name().and_then(|n| n.to_str()) != Some(OMARCHIST_LUA)
        {
            scanned.insert(path, value);
        }
    }
    scanned
}

/// The value at a dotted path.
pub fn get_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(root, |node, key| node.get(key))
}

/// Sets the value at a dotted path, creating the objects on the way.
pub fn set_path(root: &mut Value, path: &str, value: Value) {
    let mut node = root;
    let mut keys = path.split('.').peekable();
    while let Some(key) = keys.next() {
        if !node.is_object() {
            *node = Value::Object(Map::new());
        }
        let map = node.as_object_mut().expect("object");
        if keys.peek().is_none() {
            map.insert(key.to_string(), value);
            return;
        }
        node = map
            .entry(key.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
    }
}

/// Removes the value at a dotted path and every object left empty by it.
pub fn remove_path(root: &mut Value, path: &str) -> Option<Value> {
    let (head, tail) = match path.split_once('.') {
        Some((head, tail)) => (head, Some(tail)),
        None => (path, None),
    };
    let map = root.as_object_mut()?;
    match tail {
        None => map.remove(head),
        Some(tail) => {
            let child = map.get_mut(head)?;
            let removed = remove_path(child, tail);
            if child.as_object().is_some_and(Map::is_empty) {
                map.remove(head);
            }
            removed
        }
    }
}

/// Every leaf of `root` by dotted path. Arrays are leaves (a `Vec2`).
pub fn flatten(root: &Value) -> BTreeMap<String, Value> {
    fn walk(prefix: &str, node: &Value, out: &mut BTreeMap<String, Value>) {
        match node {
            Value::Object(map) => {
                for (key, child) in map {
                    let path = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{prefix}.{key}")
                    };
                    walk(&path, child, out);
                }
            }
            leaf => {
                out.insert(prefix.to_string(), leaf.clone());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk("", root, &mut out);
    out
}

/// Whether two option values are the same setting: numbers compare as
/// floats (Lua and `hyprctl` disagree about `1` and `1.0`), arrays element
/// by element.
pub fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(x), Some(y)) => (x - y).abs() <= 1e-9 * x.abs().max(y.abs()).max(1.0),
            _ => false,
        },
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| values_equal(x, y))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| values_equal(v, w)))
        }
        _ => a == b,
    }
}

/// Deep-merges `overrides` into `base`; objects merge, anything else replaces.
pub fn merge(base: &mut Value, overrides: &Value) {
    match (base, overrides) {
        (Value::Object(base), Value::Object(overrides)) => {
            for (key, value) in overrides {
                match base.get_mut(key) {
                    Some(existing) if existing.is_object() && value.is_object() => {
                        merge(existing, value)
                    }
                    _ => {
                        base.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (base, overrides) => *base = overrides.clone(),
    }
}

/// What Hyprland would run without `omarchist.lua`: `live` (the
/// compositor's values) for every key, except the overridden ones, which
/// take the value Omarchy's Lua sets or Hyprland's default when no file
/// sets them.
pub fn compute_baseline(
    live: &Value,
    scanned: &Scanned,
    overrides: &Value,
    defaults: &Value,
) -> Value {
    let mut baseline = live.clone();
    for path in flatten(overrides).keys() {
        let value = scanned
            .get(path)
            .or_else(|| get_path(defaults, path))
            .cloned()
            .unwrap_or(Value::Null);
        set_path(&mut baseline, path, value);
    }
    baseline
}

/// Defaults of the 1.x model that did not match Hyprland's. A legacy file
/// holds these for every key the reader never asked the compositor for, so
/// they say nothing about what the user chose.
const LEGACY_DEFAULTS: &[(&str, f64)] = &[
    ("decoration.blur.brightness", 0.8172),
    ("binds.workspace_center_on", 0.0),
    ("cursor.hotspot_padding", 1.0),
    ("general.border_size", 2.0),
    ("general.gaps_out", 10.0),
];

/// Turns a 1.x `state.json`, which held the whole configuration seeded
/// from the compositor, into the sparse overrides: a key is kept only when
/// its value differs from Hyprland's default (older versions never wrote
/// those), from the 1.x model's own default, and from what Omarchy's Lua
/// sets (those were pinned by the seed, not chosen by the user).
/// `[[EMPTY]]` placeholders and keys the model no longer has are dropped;
/// the retired `misc.vfr` carries over to `debug.vfr`.
pub fn migrate_legacy(legacy: &Value, scanned: &Scanned, defaults: &Value) -> Value {
    let mut overrides = Value::Object(Map::new());
    for (path, value) in flatten(legacy) {
        let path = match path.as_str() {
            "misc.vfr" => "debug.vfr".to_string(),
            _ => path,
        };
        let Some(default) = get_path(defaults, &path) else {
            continue;
        };
        if values_equal(&value, default) {
            continue;
        }
        if value.as_str() == Some(EMPTY_PLACEHOLDER) {
            continue;
        }
        if LEGACY_DEFAULTS
            .iter()
            .any(|(p, v)| *p == path && values_equal(&value, &Value::from(*v)))
        {
            continue;
        }
        if path == "cursor.hide_on_tablet" && value == Value::Bool(true) {
            continue;
        }
        if scanned
            .get(&path)
            .is_some_and(|set| values_equal(&value, set))
        {
            continue;
        }
        set_path(&mut overrides, &path, value);
    }
    sanitize(&overrides, defaults)
}

/// Keeps only the overrides the model knows, with a value of the key's
/// type. A stray key or a number where a string belongs would otherwise
/// stop the whole configuration from loading.
pub fn sanitize(overrides: &Value, defaults: &Value) -> Value {
    let mut out = Value::Object(Map::new());
    for (path, value) in flatten(overrides) {
        if get_path(defaults, &path).is_some_and(|default| same_kind(default, &value)) {
            set_path(&mut out, &path, value);
        }
    }
    out
}

fn same_kind(a: &Value, b: &Value) -> bool {
    matches!(
        (a, b),
        (Value::Number(_), Value::Number(_))
            | (Value::String(_), Value::String(_))
            | (Value::Bool(_), Value::Bool(_))
            | (Value::Array(_), Value::Array(_))
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;
    use crate::system::keybinds::scanner::parse_scan_output;
    use crate::types::hyprland_config::HyprlandConfig;

    fn defaults() -> Value {
        serde_json::to_value(HyprlandConfig::default()).unwrap()
    }

    #[test]
    fn paths_set_get_and_remove_with_pruning() {
        let mut root = json!({});
        set_path(&mut root, "input.touchpad.scroll_factor", json!(0.4));
        set_path(&mut root, "input.repeat_rate", json!(40));
        assert_eq!(
            get_path(&root, "input.touchpad.scroll_factor"),
            Some(&json!(0.4))
        );
        assert_eq!(
            remove_path(&mut root, "input.touchpad.scroll_factor"),
            Some(json!(0.4))
        );
        assert_eq!(root, json!({ "input": { "repeat_rate": 40 } }));
        assert_eq!(remove_path(&mut root, "input.repeat_rate"), Some(json!(40)));
        assert_eq!(root, json!({}));
        assert_eq!(remove_path(&mut root, "nope.nothing"), None);
    }

    #[test]
    fn flatten_keeps_arrays_as_leaves() {
        let flat =
            flatten(&json!({ "decoration": { "shadow": { "offset": [0, 0] }, "rounding": 8 } }));
        assert_eq!(flat["decoration.shadow.offset"], json!([0, 0]));
        assert_eq!(flat["decoration.rounding"], json!(8));
    }

    #[test]
    fn values_compare_numerically() {
        assert!(values_equal(&json!(1), &json!(1.0)));
        // Lua prints 0.4 with 17 significant digits.
        let lua_float: Value = serde_json::from_str("0.40000000000000002").unwrap();
        assert!(values_equal(&json!(0.4), &lua_float));
        assert!(!values_equal(&json!(1), &json!(2)));
        assert!(values_equal(&json!([1, 1]), &json!([1.0, 1.0])));
        assert!(!values_equal(&json!("a"), &json!("b")));
    }

    #[test]
    fn merge_is_deep() {
        let mut base = json!({ "general": { "gaps_in": 5, "gaps_out": 20 } });
        merge(
            &mut base,
            &json!({ "general": { "gaps_in": 2 }, "misc": { "vrr": 1 } }),
        );
        assert_eq!(
            base,
            json!({ "general": { "gaps_in": 2, "gaps_out": 20 }, "misc": { "vrr": 1 } })
        );
    }

    #[test]
    fn baseline_restores_omarchys_value_for_overridden_keys() {
        // The compositor reports the override; the baseline must not.
        let live = json!({ "general": { "gaps_in": 2, "gaps_out": 10, "border_size": 2 }, "misc": { "vrr": 2 } });
        let scanned = Scanned::from([("general.gaps_in".to_string(), json!(5))]);
        let overrides = json!({ "general": { "gaps_in": 2 }, "misc": { "vrr": 2 } });
        let baseline = compute_baseline(&live, &scanned, &overrides, &defaults());
        assert_eq!(baseline["general"]["gaps_in"], json!(5), "Omarchy's value");
        assert_eq!(
            baseline["general"]["gaps_out"],
            json!(10),
            "live value kept"
        );
        assert_eq!(
            baseline["misc"]["vrr"],
            json!(0),
            "Hyprland's default when no file sets it"
        );
    }

    #[test]
    fn legacy_state_keeps_only_what_the_user_changed() {
        let mut legacy = defaults();
        set_path(&mut legacy, "general.gaps_in", json!(2)); // user: Omarchy has 5
        set_path(&mut legacy, "general.gaps_out", json!(10)); // pinned: equals Omarchy
        set_path(&mut legacy, "input.repeat_rate", json!(40)); // pinned
        set_path(&mut legacy, "input.accel_profile", json!("[[EMPTY]]")); // placeholder
        set_path(&mut legacy, "decoration.dim_special", json!(0.6)); // user: nothing sets it
        set_path(&mut legacy, "misc.vfr", json!(false)); // retired key, user turned it off
        set_path(&mut legacy, "decoration.shadow.offset_x", json!(3.0)); // retired key
        set_path(&mut legacy, "cursor.hotspot_padding", json!(1)); // the 1.x model's wrong default
        set_path(&mut legacy, "cursor.hide_on_tablet", json!(true)); // same
        set_path(&mut legacy, "render.cm_sdr_eotf", json!(0)); // was an int in 1.x
        let scanned = Scanned::from([
            ("general.gaps_in".to_string(), json!(5)),
            ("general.gaps_out".to_string(), json!(10)),
            ("input.repeat_rate".to_string(), json!(40)),
        ]);
        let overrides = migrate_legacy(&legacy, &scanned, &defaults());
        assert_eq!(
            overrides,
            json!({
                "general": { "gaps_in": 2 },
                "decoration": { "dim_special": 0.6 },
                "debug": { "vfr": false }
            })
        );
    }

    #[test]
    fn sanitize_drops_unknown_keys_and_wrong_types() {
        let overrides = json!({
            "general": { "gaps_in": 3, "nope": 1, "layout": 5 },
            "render": { "cm_sdr_eotf": "srgb" },
            "misc": { "vfr": true }
        });
        assert_eq!(
            sanitize(&overrides, &defaults()),
            json!({ "general": { "gaps_in": 3 }, "render": { "cm_sdr_eotf": "srgb" } })
        );
    }

    #[test]
    fn scanned_config_skips_omarchist_lua() {
        let output = "config\t1\t/usr/share/omarchy/default/hypr/looknfeel.lua\tgeneral.gaps_in\t5\n\
                      config\t2\t/home/u/.config/hypr/omarchist.lua\tgeneral.gaps_in\t2\n\
                      config\t3\t/home/u/.config/hypr/looknfeel.lua\tgeneral.gaps_out\t7\n\
                      config\t4\t/home/u/.config/hypr/looknfeel.lua\tmisc.font_family\t\"a\\\\\"b\"\n\
                      done\t4\n";
        let scanned = collect_scanned(parse_scan_output(output));
        assert_eq!(scanned["general.gaps_in"], json!(5));
        assert_eq!(scanned["general.gaps_out"], json!(7));
        assert_eq!(scanned["misc.font_family"], json!("a\"b"));
    }

    #[test]
    fn scan_config_reads_nested_and_dotted_calls() {
        if std::process::Command::new(crate::system::keybinds::scanner::lua_binary())
            .arg("-v")
            .output()
            .is_err()
        {
            eprintln!("skipping: no lua interpreter");
            return;
        }
        let dir = std::env::temp_dir().join(format!("omarchist-baseline-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path: PathBuf = dir.join("hyprland.lua");
        std::fs::write(
            &path,
            "hl.config({ general = { gaps_in = 5, snap = { enabled = true } } })\n\
             hl.config({ [\"input.touchpad.scroll_factor\"] = 0.4, decoration = { shadow = { offset = { 1, 2 } } } })\n\
             hl.config({ general = { gaps_in = 7 } })\n",
        )
        .unwrap();
        let scanned = scan_config(&path).unwrap();
        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(scanned["general.gaps_in"], json!(7), "later calls win");
        assert_eq!(scanned["general.snap.enabled"], json!(true));
        assert!(values_equal(
            &scanned["input.touchpad.scroll_factor"],
            &json!(0.4)
        ));
        assert_eq!(scanned["decoration.shadow.offset"], json!([1, 2]));
    }
}
