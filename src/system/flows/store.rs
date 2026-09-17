//! Reads and writes flows under `~/.config/omarchist/flows/`, one TOML file
//! per flow named after its id, and keeps the trigger files in step.
use std::fs;
use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::system::keybinds::Dispatcher;
use crate::system::keybinds::store::{load_overrides, save_overrides};

use super::launcher;
use super::{Flow, is_slug, run_command};

/// `~/.config/omarchist/flows`
pub fn flows_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|h| h.join(".config").join("omarchist").join("flows"))
        .ok_or(Error::UnknownDirectory("home"))
}

pub fn flow_path(id: &str) -> Result<PathBuf> {
    if !is_slug(id) {
        return Err(Error::Invalid(format!("Invalid flow id '{id}'")));
    }
    Ok(flows_dir()?.join(format!("{id}.toml")))
}

/// Every flow on disk, sorted by name. A file that does not parse is
/// reported on stderr and skipped, so one broken flow does not hide the
/// rest; it is never overwritten because saves go by id.
pub fn load_flows() -> Result<Vec<Flow>> {
    let dir = flows_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(&dir).map_err(|e| Error::io("Failed to read flows directory", e))?;
    let mut flows = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        match read_flow(&path) {
            Ok(flow) => flows.push(flow),
            Err(e) => eprintln!("Skipping flow {}: {e}", path.display()),
        }
    }
    flows.sort_by_key(|f| f.name.to_lowercase());
    Ok(flows)
}

fn read_flow(path: &PathBuf) -> Result<Flow> {
    let content = fs::read_to_string(path).map_err(|e| Error::io("Failed to read flow", e))?;
    let flow: Flow = toml::from_str(&content)
        .map_err(|e| Error::Invalid(format!("Failed to parse flow {}: {e}", path.display())))?;
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if flow.id != stem {
        return Err(Error::Invalid(format!(
            "Flow id '{}' does not match its file name",
            flow.id
        )));
    }
    Ok(flow)
}

pub fn load_flow(id: &str) -> Result<Flow> {
    let path = flow_path(id)?;
    if !path.exists() {
        return Err(Error::Invalid(format!("No flow with id '{id}'")));
    }
    read_flow(&path)
}

/// The flow with this id, or else the flow with this name (case-insensitive),
/// which is what the command line accepts.
pub fn find_flow(name_or_id: &str) -> Result<Flow> {
    if is_slug(name_or_id)
        && let Ok(flow) = load_flow(name_or_id)
    {
        return Ok(flow);
    }
    let wanted = name_or_id.trim().to_lowercase();
    load_flows()?
        .into_iter()
        .find(|f| f.name.trim().to_lowercase() == wanted)
        .ok_or_else(|| Error::Invalid(format!("No flow named '{name_or_id}'")))
}

pub fn existing_ids() -> Vec<String> {
    load_flows()
        .map(|flows| flows.into_iter().map(|f| f.id).collect())
        .unwrap_or_default()
}

/// Validates, writes the JSON, and creates or removes the launcher entry and
/// startup hook to match the flow's triggers.
pub fn save_flow(flow: &Flow) -> Result<()> {
    flow.validate()?;
    let path = flow_path(&flow.id)?;
    if let Some(dir) = path.parent()
        && !dir.exists()
    {
        fs::create_dir_all(dir).map_err(|e| Error::io("Failed to create flows directory", e))?;
    }
    let content = toml::to_string_pretty(flow)
        .map_err(|e| Error::Invalid(format!("Failed to serialize flow: {e}")))?;
    fs::write(&path, content).map_err(|e| Error::io("Failed to write flow", e))?;
    launcher::sync_triggers(flow)
}

/// Removes the flow, its trigger files, and any keybind override that ran it.
pub fn delete_flow(id: &str) -> Result<()> {
    let path = flow_path(id)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| Error::io("Failed to delete flow", e))?;
    }
    launcher::remove_triggers(id)?;
    remove_keybinds_running(id)
}

fn remove_keybinds_running(id: &str) -> Result<()> {
    let mut overrides = load_overrides()?;
    let runs_flow = Dispatcher::Exec(run_command(id));
    let before = overrides.overrides.len();
    overrides
        .overrides
        .retain(|o| o.bind().is_none_or(|bind| bind.dispatcher != runs_flow));
    if overrides.overrides.len() != before {
        save_overrides(&overrides)?;
    }
    Ok(())
}
