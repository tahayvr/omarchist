//! `omarchist uninstall`: removes everything Omarchist put on the machine
//! outside its package, so `pacman -R` leaves nothing behind that changes
//! Hyprland or the bar. Themes made with Omarchist stay: they are the
//! user's, live in Omarchy's own folder, and keep working without the app.
use std::fs;
use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::system::bar_widget;
use crate::system::config::hypr_setup;
use crate::system::flows::{launcher, store};
use crate::system::fs::write_atomic;
use crate::system::omarchy_paths::user_hyprland_config_dir;

/// One thing the uninstall will do, for the confirmation listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Take the `require("hypr.omarchist")` line out of `hyprland.lua`.
    RequireLine(PathBuf),
    /// Delete `~/.config/hypr/omarchist.lua`.
    OmarchistLua(PathBuf),
    /// Disable the bar widget and delete its plugin folder.
    BarWidget(PathBuf),
    /// Delete a flow's launcher entry, icon, and startup hook.
    FlowTriggers { id: String, name: String },
    /// Delete `~/.config/omarchist` (settings, Hyprland state, keybind
    /// overrides, flows, templates).
    ConfigDir(PathBuf),
    /// Delete `~/.local/share/omarchist` (flow icons).
    DataDir(PathBuf),
}

impl Step {
    pub fn describe(&self) -> String {
        match self {
            Step::RequireLine(path) => {
                format!("Remove require(\"hypr.omarchist\") from {}", path.display())
            }
            Step::OmarchistLua(path) => format!("Delete {}", path.display()),
            Step::BarWidget(path) => {
                format!("Take Omarchist off the bar and delete {}", path.display())
            }
            Step::FlowTriggers { name, .. } => {
                format!("Remove the launcher entry and startup hook of flow \"{name}\"")
            }
            Step::ConfigDir(path) | Step::DataDir(path) => format!("Delete {}", path.display()),
        }
    }
}

/// What is on this machine, in the order it will be removed. Hyprland's
/// config goes first so a failure later never leaves a `require` of a file
/// that is gone.
pub fn plan() -> Vec<Step> {
    let mut steps = Vec::new();
    if let Ok(path) = hypr_setup::hypr_config_path()
        && fs::read_to_string(&path).is_ok_and(|content| hypr_setup::has_require_line(&content))
    {
        steps.push(Step::RequireLine(path));
    }
    if let Some(path) = user_hyprland_config_dir().map(|dir| dir.join("omarchist.lua"))
        && path.is_file()
    {
        steps.push(Step::OmarchistLua(path));
    }
    if let Some(dir) = bar_widget::plugin_dir()
        && (dir.is_dir() || bar_widget::is_enabled())
    {
        steps.push(Step::BarWidget(dir));
    }
    for flow in store::load_flows().unwrap_or_default() {
        let has_files = launcher::desktop_entry_path(&flow.id).is_ok_and(|p| p.exists())
            || launcher::startup_hook_path(&flow.id).is_ok_and(|p| p.exists());
        if has_files {
            steps.push(Step::FlowTriggers {
                id: flow.id.clone(),
                name: flow.name.clone(),
            });
        }
    }
    if let Some(dir) = config_dir()
        && dir.is_dir()
    {
        steps.push(Step::ConfigDir(dir));
    }
    if let Some(dir) = data_dir()
        && dir.is_dir()
    {
        steps.push(Step::DataDir(dir));
    }
    steps
}

/// Carries out one step.
pub fn run(step: &Step) -> Result<()> {
    match step {
        Step::RequireLine(path) => {
            let content = fs::read_to_string(path)
                .map_err(|e| Error::io("Failed to read hyprland.lua", e))?;
            let content = hypr_setup::without_require_line(&content);
            write_atomic(path, content, "hyprland.lua")
        }
        Step::OmarchistLua(path) => remove_file(path),
        Step::BarWidget(dir) => {
            bar_widget::disable()?;
            remove_dir(dir)
        }
        Step::FlowTriggers { id, .. } => launcher::remove_triggers(id),
        Step::ConfigDir(dir) | Step::DataDir(dir) => remove_dir(dir),
    }
}

fn config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".config").join("omarchist"))
}

fn data_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".local").join("share").join("omarchist"))
}

fn remove_file(path: &PathBuf) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::io(format!("Failed to delete {}", path.display()), e)),
    }
}

fn remove_dir(path: &PathBuf) -> Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::io(format!("Failed to delete {}", path.display()), e)),
    }
}
