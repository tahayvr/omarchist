//! Reads and writes flows under `~/.config/omarchist/flows/`, one TOML file
//! per flow named after its id, and keeps the trigger files in step.
use std::fs;
use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::system::keybinds::Dispatcher;
use crate::system::keybinds::store::{load_overrides, save_overrides};

use super::launcher;
use super::{Flow, is_slug, parse_flow, run_command_id};

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

/// A file in the flows directory is `<id>.toml`. A file without an id
/// takes its stem, so a shared file copied in only needs the right name.
fn read_flow(path: &PathBuf) -> Result<Flow> {
    let content = fs::read_to_string(path).map_err(|e| Error::io("Failed to read flow", e))?;
    let mut flow =
        parse_flow(&content).map_err(|e| Error::Invalid(format!("{e} (in {})", path.display())))?;
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if flow.id.is_empty() {
        if !is_slug(stem) {
            return Err(Error::Invalid(format!(
                "File name '{stem}' is not a valid flow id; use lowercase letters, digits, and hyphens"
            )));
        }
        flow.id = stem.to_string();
    } else if flow.id != stem {
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

/// Whether a keybind dispatcher runs the flow with this id, however the
/// command was quoted.
pub fn runs_flow(dispatcher: &Dispatcher, id: &str) -> bool {
    matches!(dispatcher, Dispatcher::Exec(command) if run_command_id(command).as_deref() == Some(id))
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

/// Validates, writes the TOML, and creates or removes the launcher entry and
/// startup hook to match the flow's triggers.
pub fn save_flow(flow: &Flow) -> Result<()> {
    flow.validate()?;
    let path = flow_path(&flow.id)?;
    if let Some(dir) = path.parent()
        && !dir.exists()
    {
        fs::create_dir_all(dir).map_err(|e| Error::io("Failed to create flows directory", e))?;
    }
    fs::write(&path, flow.to_toml()?).map_err(|e| Error::io("Failed to write flow", e))?;
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
    let before = overrides.overrides.len();
    overrides
        .overrides
        .retain(|o| o.bind().is_none_or(|bind| !runs_flow(&bind.dispatcher, id)));
    if overrides.overrides.len() != before {
        save_overrides(&overrides)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str, content: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("omarchist-flow-store-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn a_file_without_an_id_takes_its_stem() {
        let path = temp_file("night-shift.toml", "name = \"Night shift\"\n");
        let flow = read_flow(&path).unwrap();
        assert_eq!(flow.id, "night-shift");
        assert_eq!(flow.format, super::super::FORMAT);
    }

    #[test]
    fn a_stem_that_is_not_a_slug_is_rejected() {
        let path = temp_file("Night Shift.toml", "name = \"Night shift\"\n");
        let error = read_flow(&path).unwrap_err().to_string();
        assert!(error.contains("not a valid flow id"), "{error}");
    }

    #[test]
    fn an_id_that_differs_from_the_stem_is_rejected() {
        let path = temp_file("a.toml", "id = \"b\"\nname = \"B\"\n");
        assert!(read_flow(&path).is_err());
    }
}
