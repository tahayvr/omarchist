//! The Omarchist bar widget: a Quattro shell plugin (`defaults/plugin/
//! tahayvr.omarchist`, a manifest and one QML file) that lists the user's
//! flows and opens the app on a page. Omarchist installs it into
//! `~/.config/omarchy/plugins/` and enables it through Omarchy's own
//! `omarchy plugin` commands when the Settings page switch is on, and
//! refreshes the files at startup when the embedded version is newer.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::assets::{extract_default_dir, read_default_str};
use crate::error::{Error, Result};
use crate::system::omarchy_paths::{shell_json_path, user_plugins_dir};

pub const PLUGIN_ID: &str = "tahayvr.omarchist";
const EMBEDDED_DIR: &str = "plugin/tahayvr.omarchist";
/// The file in the plugin folder holding the path of the Omarchist binary;
/// the widget watches it, so it needs no shell restart to follow a move.
const COMMAND_FILE: &str = "command";

pub fn plugin_dir() -> Option<PathBuf> {
    user_plugins_dir().map(|dir| dir.join(PLUGIN_ID))
}

/// The version of the widget shipped in this build.
pub fn embedded_version() -> Result<String> {
    let manifest: Value =
        serde_json::from_str(&read_default_str(&format!("{EMBEDDED_DIR}/manifest.json"))?)
            .map_err(|e| Error::json("Failed to parse the embedded widget manifest", e))?;
    manifest["version"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| Error::Invalid("The widget manifest has no version".into()))
}

/// The version installed under the user's plugins, if any.
pub fn installed_version() -> Option<String> {
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(plugin_dir()?.join("manifest.json")).ok()?)
            .ok()?;
    manifest["version"].as_str().map(str::to_string)
}

pub fn is_installed() -> bool {
    plugin_dir().is_some_and(|dir| dir.join("manifest.json").is_file())
}

/// Whether the widget is on the bar: the shell keeps enabled plugins in
/// `shell.json`'s `bar.layout`.
pub fn is_enabled() -> bool {
    layout_entry().is_some()
}

fn layout_entry() -> Option<Value> {
    let shell: Value = serde_json::from_str(&fs::read_to_string(shell_json_path()?).ok()?).ok()?;
    let layout = shell["bar"]["layout"].as_object()?;
    layout
        .values()
        .filter_map(Value::as_array)
        .flatten()
        .find(|entry| entry["id"].as_str() == Some(PLUGIN_ID))
        .cloned()
}

/// Writes the widget's files: the manifest, the QML, and the path of this
/// binary.
pub fn install() -> Result<PathBuf> {
    let dir = plugin_dir().ok_or(Error::UnknownDirectory("home"))?;
    install_into(&dir)?;
    Ok(dir)
}

fn install_into(dir: &Path) -> Result<()> {
    // Version 1.0 drew flow icons from SVG files; every icon is a glyph now.
    let stale_icons = dir.join("icons");
    if stale_icons.is_dir() {
        fs::remove_dir_all(&stale_icons)
            .map_err(|e| Error::io("Failed to remove the old widget icons", e))?;
    }
    extract_default_dir(EMBEDDED_DIR, dir)?;
    fs::write(dir.join(COMMAND_FILE), format!("{}\n", current_command()?))
        .map_err(|e| Error::io("Failed to write the widget's command file", e))?;
    Ok(())
}

/// The widget runs Omarchist by the path the app was started from, so it
/// works whether the binary is on `PATH` or not.
fn current_command() -> Result<String> {
    Ok(crate::system::binary::omarchist_binary())
}

/// Installs the files, tells the shell to look again, and puts the widget
/// on the bar.
pub fn enable() -> Result<()> {
    install()?;
    rescan()?;
    if !is_enabled() {
        omarchy(&["plugin", "enable", PLUGIN_ID, "--section", "right"])?;
    }
    Ok(())
}

/// Takes the widget off the bar. The files stay so turning it back on is
/// instant; Omarchy's `omarchy plugin remove` deletes them.
pub fn disable() -> Result<()> {
    if is_enabled() {
        omarchy(&["plugin", "disable", PLUGIN_ID])?;
    }
    Ok(())
}

/// At startup: an enabled widget gets this build's files when they are
/// newer or the binary moved. A moved binary takes effect at once, since
/// the widget watches the command file; new files only take effect when
/// the shell restarts, because it keeps the widget it already created, so
/// a newer version restarts the shell. Returns the version that was put
/// in place, when one was.
pub fn ensure_current() -> Result<Option<String>> {
    if !is_enabled() {
        return Ok(None);
    }
    let dir = plugin_dir().ok_or(Error::UnknownDirectory("home"))?;
    let saved_command = fs::read_to_string(dir.join(COMMAND_FILE)).unwrap_or_default();
    let command = current_command()?;
    let installed = installed_version();
    let embedded = embedded_version()?;
    let newer = installed.as_deref() != Some(embedded.as_str());
    if newer {
        eprintln!(
            "Refreshing the bar widget: version {} -> {embedded}",
            installed.as_deref().unwrap_or("none")
        );
    } else if saved_command.trim() != command {
        eprintln!("Refreshing the bar widget: binary moved to {command}");
    } else {
        return Ok(None);
    }
    install()?;
    if !newer {
        rescan()?;
        return Ok(None);
    }
    // A restart rescans on its own, and it must not hang on a rescan the
    // shell's IPC fails to answer, which it does now and then.
    run("omarchy-restart-shell", &[])?;
    // The restart script can report success while its new instance has
    // refused to start ("already running", when the old one was still on
    // its way out); a shell that is not back is launched the way Omarchy
    // launches it, from Hyprland, so it gets the session's environment.
    if !shell_answers(Duration::from_secs(3)) {
        let _ = Command::new("hyprctl")
            .args(["dispatch", "hl.dsp.exec_cmd(\"omarchy-launch-shell\")"])
            .output();
    }
    Ok(Some(embedded))
}

/// Whether the shell answers a ping within `wait`.
fn shell_answers(wait: Duration) -> bool {
    let deadline = Instant::now() + wait;
    loop {
        let ok = Command::new("omarchy-shell")
            .args(["shell", "ping"])
            .env("OMARCHY_SHELL_IPC_TIMEOUT", "0.5s")
            .output()
            .is_ok_and(|out| out.status.success());
        if ok {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn rescan() -> Result<()> {
    run("omarchy-shell", &["shell", "rescanPlugins"])
}

fn omarchy(args: &[&str]) -> Result<()> {
    run("omarchy", args)
}

fn run(program: &str, args: &[&str]) -> Result<()> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| Error::io(format!("Failed to run {program}"), e))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if stderr.is_empty() { stdout } else { stderr };
    Err(Error::Invalid(format!(
        "{program} {} failed: {detail}",
        args.join(" ")
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::DefaultAssets;

    #[test]
    fn the_embedded_plugin_is_complete() {
        let manifest: Value = serde_json::from_str(
            &read_default_str(&format!("{EMBEDDED_DIR}/manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(manifest["schemaVersion"], 1);
        assert_eq!(manifest["id"], PLUGIN_ID);
        assert_eq!(manifest["kinds"], serde_json::json!(["bar-widget"]));
        let entry = manifest["entryPoints"]["barWidget"].as_str().unwrap();
        assert!(DefaultAssets::get(&format!("{EMBEDDED_DIR}/{entry}")).is_some());
        assert!(embedded_version().unwrap().split('.').count() == 3);
    }

    #[test]
    fn install_writes_the_command_file() {
        let dir =
            std::env::temp_dir().join(format!("omarchist-widget-install-{}", std::process::id()));
        install_into(&dir).unwrap();
        let command = fs::read_to_string(dir.join(COMMAND_FILE)).unwrap();
        assert_eq!(command.trim(), current_command().unwrap());
        assert!(dir.join("BarWidget.qml").is_file());
        fs::remove_dir_all(&dir).ok();
    }

    /// Omarchy's own validator accepts what `install` writes.
    #[test]
    fn omarchy_validates_the_installed_plugin() {
        if Command::new("omarchy").arg("--help").output().is_err() {
            eprintln!("skipping: no omarchy");
            return;
        }
        let dir = std::env::temp_dir().join(format!("omarchist-widget-{}", std::process::id()));
        extract_default_dir(EMBEDDED_DIR, &dir).unwrap();
        let output = Command::new("omarchy")
            .args(["plugin", "validate"])
            .arg(&dir)
            .output()
            .unwrap();
        fs::remove_dir_all(&dir).ok();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
