//! The Configuration page's model: the keys the user changed (overrides,
//! the only thing persisted, in `~/.config/omarchist/hyprland/state.json`)
//! layered over the baseline Hyprland runs without them (`baseline.rs`).
//! Every save regenerates the write-only `~/.config/hypr/omarchist.lua`
//! from the overrides alone, so Omarchy's own values are never pinned.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::{Error, Result};
use crate::system::fs::write_atomic;
use crate::system::omarchy_paths::{omarchist_hyprland_dir, user_hyprland_config_dir};
use crate::types::hyprland_config::HyprlandConfig;

use super::baseline::{self, Scanned};

const STATE_FILE: &str = "state.json";
/// Where a 1.x state file (the whole configuration) is kept after migration.
const LEGACY_STATE_FILE: &str = "state.json.legacy";
const LUA_FILE: &str = "omarchist.lua";
const STATE_VERSION: u32 = 2;

#[derive(Debug, Serialize, Deserialize)]
struct StateFile {
    version: u32,
    /// Sparse object in the shape of `HyprlandConfig`: only changed keys.
    overrides: Value,
}

pub struct HyprlandConfigManager {
    state_path: PathBuf,
    overrides: Value,
    baseline: Value,
    /// Set by [`Self::unavailable`]: nothing is saved.
    unavailable: bool,
}

impl HyprlandConfigManager {
    pub fn load() -> Result<Self> {
        let state_path = get_state_path()?;
        ensure_config_dir()?;

        let live = to_object(&super::hyprctl_reader::read_from_hyprctl());
        let scanned = scan_user_config();
        let overrides = load_overrides(&state_path, &scanned)?;
        let baseline = baseline::compute_baseline(&live, &scanned, &overrides, &defaults());

        Ok(Self {
            state_path,
            overrides,
            baseline,
            unavailable: false,
        })
    }

    /// A manager over the given baseline and overrides, for tests.
    pub fn from_parts(state_path: PathBuf, baseline: Value, overrides: Value) -> Self {
        Self {
            state_path,
            overrides,
            baseline,
            unavailable: false,
        }
    }

    /// Stands in when [`Self::load`] fails: Hyprland's defaults, no
    /// overrides, and a `save` that refuses, so a state file that could not
    /// be read is never overwritten.
    pub fn unavailable() -> Self {
        Self {
            state_path: PathBuf::new(),
            overrides: Value::Object(Map::new()),
            baseline: defaults(),
            unavailable: true,
        }
    }

    /// Nothing can be saved: `load` failed and this is the stand-in.
    pub fn is_unavailable(&self) -> bool {
        self.unavailable
    }

    /// Whether the compositor answered when the baseline was read. Without
    /// it (no Hyprland, or `hyprctl` missing) the baseline is Hyprland's
    /// stock defaults, which is not what the user's desktop runs.
    pub fn compositor_reachable() -> bool {
        Command::new("hyprctl")
            .args(["-j", "version"])
            .stdin(std::process::Stdio::null())
            .output()
            .is_ok_and(|o| o.status.success())
    }

    /// The settings in effect: the baseline with the overrides applied.
    pub fn effective(&self) -> HyprlandConfig {
        let mut value = self.baseline.clone();
        baseline::merge(&mut value, &self.overrides);
        serde_json::from_value(value).unwrap_or_default()
    }

    /// The value in effect at a dotted path.
    pub fn value(&self, path: &str) -> Option<Value> {
        baseline::get_path(&self.overrides, path)
            .or_else(|| baseline::get_path(&self.baseline, path))
            .cloned()
    }

    /// The value Omarchy (or Hyprland) gives the key without Omarchist.
    pub fn baseline_value(&self, path: &str) -> Option<Value> {
        baseline::get_path(&self.baseline, path).cloned()
    }

    pub fn is_overridden(&self, path: &str) -> bool {
        baseline::get_path(&self.overrides, path).is_some()
    }

    /// Records the user's choice. Choosing the baseline value again removes
    /// the override, so the key follows Omarchy from then on.
    pub fn set_value(&mut self, path: &str, value: Value) {
        if self.unavailable {
            return;
        }
        match self.baseline_value(path) {
            Some(base) if baseline::values_equal(&base, &value) => {
                baseline::remove_path(&mut self.overrides, path);
            }
            _ => baseline::set_path(&mut self.overrides, path, value),
        }
    }

    /// Drops the override so the key follows Omarchy again.
    pub fn reset(&mut self, path: &str) {
        baseline::remove_path(&mut self.overrides, path);
    }

    pub fn reset_all(&mut self) {
        self.overrides = Value::Object(Map::new());
    }

    pub fn overrides(&self) -> &Value {
        &self.overrides
    }

    /// Saves and applies the overrides. `Ok(true)` when the `require` line
    /// in `hyprland.lua` had gone missing and was put back (see
    /// `write_omarchist_lua`).
    pub fn save(&self) -> Result<bool> {
        if self.unavailable {
            return Err(Error::Invalid(
                "Hyprland settings could not be loaded, so they are not saved".into(),
            ));
        }
        write_state(&self.state_path, &self.overrides)?;
        // The caller reloads Hyprland (checked, so a rejected value is
        // reported rather than shown as applied).
        write_omarchist_lua(&self.overrides)
    }

    pub fn config_path(&self) -> Result<PathBuf> {
        get_lua_path()
    }
}

impl Clone for HyprlandConfigManager {
    fn clone(&self) -> Self {
        Self {
            state_path: self.state_path.clone(),
            overrides: self.overrides.clone(),
            baseline: self.baseline.clone(),
            unavailable: self.unavailable,
        }
    }
}

fn defaults() -> Value {
    to_object(&HyprlandConfig::default())
}

fn to_object(config: &HyprlandConfig) -> Value {
    serde_json::to_value(config).unwrap_or(Value::Object(Map::new()))
}

/// Every `hl.config` value set by the user's `hyprland.lua` and the files
/// it loads, other than `omarchist.lua`. Empty when the scan cannot run
/// (no `lua` interpreter): overridden keys then reset to Hyprland's
/// defaults rather than Omarchy's values.
fn scan_user_config() -> Scanned {
    let Some(dir) = user_hyprland_config_dir() else {
        return Scanned::new();
    };
    match baseline::scan_config(&dir.join("hyprland.lua")) {
        Ok(scanned) => scanned,
        Err(e) => {
            eprintln!("Could not read Omarchy's Hyprland settings: {e}");
            Scanned::new()
        }
    }
}

/// Reads the overrides, migrating a 1.x state file (the whole
/// configuration) in place and keeping it as `state.json.legacy`.
fn load_overrides(state_path: &Path, scanned: &Scanned) -> Result<Value> {
    if !state_path.exists() {
        return Ok(Value::Object(Map::new()));
    }
    let content =
        fs::read_to_string(state_path).map_err(|e| Error::io("Failed to read state file", e))?;
    // A file that does not parse is an error, not "no overrides": the
    // page then refuses to save (so nothing replaces it) and says why,
    // instead of every setting silently reading as Omarchy's.
    let parsed: Value = serde_json::from_str(&content).map_err(|e| {
        Error::Invalid(format!(
            "{} could not be read (fix or remove it): {e}",
            state_path.display()
        ))
    })?;
    if let Ok(state) = serde_json::from_value::<StateFile>(parsed.clone())
        && state.version >= STATE_VERSION
    {
        return Ok(baseline::sanitize(&state.overrides, &defaults()));
    }

    let overrides = baseline::migrate_legacy(&parsed, scanned, &defaults());
    let legacy_path = state_path.with_file_name(LEGACY_STATE_FILE);
    fs::rename(state_path, &legacy_path)
        .map_err(|e| Error::io("Failed to keep the old Hyprland state file", e))?;
    write_state(state_path, &overrides)?;
    eprintln!(
        "Migrated Hyprland settings: {} setting(s) kept, previous file at {}",
        baseline::flatten(&overrides).len(),
        legacy_path.display()
    );
    Ok(overrides)
}

fn write_state(state_path: &Path, overrides: &Value) -> Result<()> {
    let state = StateFile {
        version: STATE_VERSION,
        overrides: overrides.clone(),
    };
    let content = serde_json::to_string_pretty(&state)
        .map_err(|e| Error::json("Failed to serialize Hyprland state", e))?;
    write_atomic(state_path, content, "the Hyprland state file")
}

/// Asks Hyprland to re-read its config, in the background so saves never
/// block on the compositor.
pub fn reload_hyprland() {
    std::thread::spawn(|| {
        let _ = Command::new("hyprctl").arg("reload").output();
    });
}

/// Reloads Hyprland and waits for it, for the CLI.
pub fn reload_hyprland_blocking() {
    let _ = Command::new("hyprctl").arg("reload").output();
}

/// Reloads Hyprland and waits for it, then asks what it rejected. Blocking:
/// run it off the UI thread. `Ok(Some(text))` is Hyprland's own error list;
/// `Ok(None)` means the config loaded clean.
pub fn reload_hyprland_checked() -> Result<Option<String>> {
    Command::new("hyprctl")
        .arg("reload")
        .output()
        .map_err(|e| Error::io("Failed to run hyprctl reload", e))?;
    let output = Command::new("hyprctl")
        .args(["-j", "configerrors"])
        .output()
        .map_err(|e| Error::io("Failed to run hyprctl configerrors", e))?;
    let errors: Vec<String> = serde_json::from_slice(&output.stdout).unwrap_or_default();
    let errors: Vec<String> = errors
        .into_iter()
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty())
        .collect();
    Ok((!errors.is_empty()).then(|| errors.join("\n")))
}

/// The overrides last saved by the Configuration page, without touching
/// hyprctl. Used when only the keybinds block changed.
pub fn saved_overrides() -> Value {
    let Ok(state_path) = get_state_path() else {
        return Value::Object(Map::new());
    };
    // A 1.x file is migrated here too, so it needs the scan.
    let scanned = if state_path.exists()
        && fs::read_to_string(&state_path)
            .ok()
            .and_then(|c| serde_json::from_str::<StateFile>(&c).ok())
            .is_none()
    {
        scan_user_config()
    } else {
        Scanned::new()
    };
    load_overrides(&state_path, &scanned).unwrap_or_else(|e| {
        eprintln!("Failed to read Hyprland settings: {e}");
        Value::Object(Map::new())
    })
}

/// Regenerates `~/.config/hypr/omarchist.lua` from the overrides plus the
/// saved keybind overrides. Both the Configuration page and the Keybinds
/// page go through here so neither can drop the other's section.
///
/// Also makes sure `hyprland.lua` still requires the file: Omarchy's
/// `omarchy-refresh-hyprland` (the Configuration page's "Reset" runs it)
/// replaces the user's `hyprland.lua` with a copy that has no such line,
/// after which every save here would be inert. `Ok(true)` when the line had
/// to be put back, so the caller can say so.
pub fn write_omarchist_lua(overrides: &Value) -> Result<bool> {
    // A keybinds.json that cannot be read must not become "no keybinds":
    // the file (and the user's binds) is still there to be repaired.
    let keybinds = crate::system::keybinds::store::load_overrides()
        .map_err(|e| Error::Invalid(format!("keybinds.json could not be read: {e}")))?
        .valid_only();
    let keybinds_lua = crate::system::keybinds::overrides::emit_keybinds_lua(&keybinds);
    let lua_content = super::lua_writer::render_omarchist_lua(overrides, &keybinds_lua);
    let lua_path = get_lua_path()?;
    // Hyprland reloads on every write to ~/.config/hypr, so leave an
    // up-to-date file alone.
    if !fs::read_to_string(&lua_path).is_ok_and(|current| current == lua_content) {
        write_atomic(&lua_path, lua_content, "omarchist.lua")?;
    }
    crate::system::config::hypr_setup::ensure_require_line()
}

fn get_state_path() -> Result<PathBuf> {
    let dir = omarchist_hyprland_dir().ok_or(Error::UnknownDirectory("home"))?;
    Ok(dir.join(STATE_FILE))
}

fn get_lua_path() -> Result<PathBuf> {
    let dir = user_hyprland_config_dir().ok_or(Error::UnknownDirectory("home"))?;
    Ok(dir.join(LUA_FILE))
}

fn ensure_config_dir() -> Result<()> {
    let dir = omarchist_hyprland_dir().ok_or(Error::UnknownDirectory("home"))?;
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| Error::io("Failed to create config directory", e))?;
    }
    Ok(())
}

pub fn config_exists() -> bool {
    get_state_path().map(|p| p.exists()).unwrap_or(false)
}

pub fn delete_config() -> Result<()> {
    let state_path = get_state_path()?;
    if state_path.exists() {
        fs::remove_file(&state_path).map_err(|e| Error::io("Failed to delete state file", e))?;
    }
    // Keep the file (hyprland.lua requires it) with only the keybinds left.
    write_omarchist_lua(&Value::Object(Map::new())).map(|_| ())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// Every saved override is the value the running compositor reports,
    /// so what the page shows is what is in effect. Skipped without a
    /// compositor or a state file.
    #[test]
    fn saved_overrides_are_what_the_compositor_runs() {
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_none() {
            eprintln!("skipping: no compositor");
            return;
        }
        let Some(content) = get_state_path()
            .ok()
            .and_then(|path| fs::read_to_string(path).ok())
        else {
            eprintln!("skipping: no state file");
            return;
        };
        let state: Value = serde_json::from_str(&content).unwrap();
        if state["version"] != 2 {
            eprintln!("skipping: legacy state file");
            return;
        }
        let live = to_object(&super::super::hyprctl_reader::read_from_hyprctl());
        let mut problems = Vec::new();
        for (path, value) in baseline::flatten(&state["overrides"]) {
            match baseline::get_path(&live, &path) {
                Some(running) if baseline::values_equal(running, &value) => {}
                Some(running) => problems.push(format!("{path}: saved {value}, running {running}")),
                None => problems.push(format!("{path}: not read from the compositor")),
            }
        }
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }

    #[test]
    fn an_unavailable_manager_never_saves() {
        let manager = HyprlandConfigManager::unavailable();
        assert!(manager.save().is_err());
        assert_eq!(manager.value("general.gaps_in"), Some(json!(5)));
    }

    fn manager() -> HyprlandConfigManager {
        HyprlandConfigManager::from_parts(
            PathBuf::from("/nonexistent/state.json"),
            json!({ "general": { "gaps_in": 5, "gaps_out": 10 }, "debug": { "vfr": true } }),
            json!({ "general": { "gaps_in": 2 } }),
        )
    }

    #[test]
    fn values_come_from_overrides_then_baseline() {
        let manager = manager();
        assert_eq!(manager.value("general.gaps_in"), Some(json!(2)));
        assert_eq!(manager.value("general.gaps_out"), Some(json!(10)));
        assert_eq!(manager.baseline_value("general.gaps_in"), Some(json!(5)));
        assert!(manager.is_overridden("general.gaps_in"));
        assert!(!manager.is_overridden("general.gaps_out"));
        assert_eq!(manager.effective().general.gaps_in, 2);
        assert_eq!(manager.effective().general.gaps_out, 10);
    }

    #[test]
    fn setting_the_baseline_value_removes_the_override() {
        let mut manager = manager();
        manager.set_value("general.gaps_in", json!(5));
        assert!(!manager.is_overridden("general.gaps_in"));
        assert_eq!(manager.overrides(), &json!({}));

        manager.set_value("general.gaps_out", json!(0));
        assert_eq!(
            manager.overrides(),
            &json!({ "general": { "gaps_out": 0 } })
        );
        manager.reset("general.gaps_out");
        assert_eq!(manager.overrides(), &json!({}));
    }

    #[test]
    fn legacy_state_file_is_migrated_and_kept() {
        let dir = std::env::temp_dir().join(format!("omarchist-manager-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join(STATE_FILE);
        let mut legacy = defaults();
        baseline::set_path(&mut legacy, "general.gaps_in", json!(2));
        baseline::set_path(&mut legacy, "input.repeat_rate", json!(40));
        fs::write(&state_path, serde_json::to_string(&legacy).unwrap()).unwrap();

        let scanned = Scanned::from([("input.repeat_rate".to_string(), json!(40))]);
        let overrides = load_overrides(&state_path, &scanned).unwrap();
        assert_eq!(overrides, json!({ "general": { "gaps_in": 2 } }));
        assert!(dir.join(LEGACY_STATE_FILE).exists());
        let written: StateFile =
            serde_json::from_str(&fs::read_to_string(&state_path).unwrap()).unwrap();
        assert_eq!(written.version, STATE_VERSION);
        assert_eq!(written.overrides, overrides);

        // A second load reads the new format without scanning.
        assert_eq!(
            load_overrides(&state_path, &Scanned::new()).unwrap(),
            overrides
        );
        fs::remove_dir_all(&dir).ok();
    }
}
