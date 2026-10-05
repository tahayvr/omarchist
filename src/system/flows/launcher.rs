//! The files that let other parts of the desktop start a flow: a `.desktop`
//! entry for the app launcher, a script in Omarchy's `post-boot` hook
//! directory for startup, and a script in the file manager's Scripts menu
//! that runs the flow on the selected files.
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::OmarchistAssets;
use crate::error::{Error, Result};
use crate::system::apps::data_home;
use crate::system::binary::{desktop_exec_quote, omarchist_binary, shell_quote};

use super::{Flow, icon_path};

/// Launchers render the icon on light and dark backgrounds alike, so the
/// Lucide `currentColor` stroke is replaced with a mid grey.
const ICON_STROKE: &str = "#9ca3af";

fn home() -> Result<PathBuf> {
    dirs::home_dir().ok_or(Error::UnknownDirectory("home"))
}

fn data_dir() -> Result<PathBuf> {
    data_home().ok_or(Error::UnknownDirectory("data"))
}

/// `$XDG_DATA_HOME/applications/omarchist-flow-<id>.desktop`
pub fn desktop_entry_path(id: &str) -> Result<PathBuf> {
    Ok(data_dir()?
        .join("applications")
        .join(format!("omarchist-flow-{id}.desktop")))
}

/// `$XDG_DATA_HOME/omarchist/flows/<id>.svg`
pub fn icon_file_path(id: &str) -> Result<PathBuf> {
    Ok(data_dir()?
        .join("omarchist/flows")
        .join(format!("{id}.svg")))
}

/// `~/.config/omarchy/hooks/post-boot.d/omarchist-flow-<id>`
pub fn startup_hook_path(id: &str) -> Result<PathBuf> {
    Ok(home()?
        .join(".config/omarchy/hooks/post-boot.d")
        .join(format!("omarchist-flow-{id}")))
}

/// `$XDG_DATA_HOME/nautilus/scripts`, whose executables Files lists under
/// Scripts in its right-click menu.
pub fn file_scripts_dir() -> Result<PathBuf> {
    Ok(data_dir()?.join("nautilus/scripts"))
}

/// The line that marks a script as a flow's, whatever the file is called.
fn file_script_marker(id: &str) -> String {
    format!("# omarchist-flow: {id}")
}

/// The menu entry's script. Files hands the selected paths over as
/// arguments, which become the flow's input, one per line.
pub fn file_script(flow: &Flow) -> String {
    format!(
        "#!/bin/sh\n# Managed by Omarchist: runs the '{}' flow on the selected files.\n{}\nexec {} flow run {} --trigger 'Files menu' -- \"$@\"\n",
        flow.id,
        file_script_marker(&flow.id),
        shell_quote(&omarchist_binary()),
        flow.id
    )
}

/// The file name is what the menu shows, so it is the flow's name, without
/// what a file name cannot hold.
fn file_script_name(flow: &Flow) -> String {
    let name: String = flow
        .name
        .trim()
        .chars()
        .map(|c| if c == '/' || c.is_control() { ' ' } else { c })
        .collect();
    let name = name.trim().trim_start_matches('.').to_string();
    if name.is_empty() {
        flow.id.clone()
    } else {
        name
    }
}

/// Every script in the Scripts folder that belongs to the flow `id`: the
/// file is named after the flow, which can be renamed, so they are found
/// by their marker line.
pub fn file_scripts_of(id: &str) -> Vec<PathBuf> {
    let Ok(dir) = file_scripts_dir() else {
        return Vec::new();
    };
    let marker = file_script_marker(id);
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && fs::read_to_string(path)
                    .is_ok_and(|text| text.lines().any(|line| line == marker))
        })
        .collect()
}

/// Writes the flow's Scripts entry under its current name and removes any
/// it had under an older one, or removes them all when the trigger is off.
fn sync_file_script(flow: &Flow) -> Result<()> {
    let wanted = flow
        .triggers
        .files
        .then(|| Ok::<_, Error>(file_scripts_dir()?.join(file_script_name(flow))))
        .transpose()?;
    for old in file_scripts_of(&flow.id) {
        if Some(&old) != wanted.as_ref() {
            remove(&old, "file manager entry")?;
        }
    }
    if let Some(path) = wanted {
        let text = file_script(flow);
        // A file of that name that is not this flow's is someone else's.
        if path.exists() && !file_scripts_of(&flow.id).contains(&path) {
            return Err(Error::Invalid(format!(
                "The file manager already has a script named '{}'",
                file_script_name(flow)
            )));
        }
        if fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
            write(&path, &text, "file manager entry")?;
        }
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
            .map_err(|e| Error::io("Failed to make the file manager entry executable", e))?;
    }
    Ok(())
}

/// The desktop entry text. Field codes only apply to `Exec`, which is fixed
/// here, so the name and comment need only be kept to one line (and their
/// backslashes doubled: the spec reads `\n` and `\\` as escapes).
pub fn desktop_entry(flow: &Flow, icon: &str) -> String {
    let escape = |s: &str| s.replace('\\', "\\\\").replace(['\n', '\r'], " ");
    let comment = if flow.description.trim().is_empty() {
        "Omarchist flow".to_string()
    } else {
        escape(flow.description.trim())
    };
    let exec = format!(
        "{} flow run {} --trigger Launcher",
        desktop_exec_quote(&omarchist_binary()),
        flow.id
    );
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={name}\n\
         Comment={comment}\n\
         Exec={exec}\n\
         Icon={icon}\n\
         Terminal=false\n\
         Categories=Utility;\n\
         Keywords=flow;omarchist;\n\
         X-Omarchist-Flow={id}\n",
        name = escape(flow.name.trim()),
        id = flow.id,
    )
}

/// Omarchy runs the `post-boot.d` scripts one after another, so the flow is
/// started in the background rather than holding up the hooks after it.
pub fn startup_hook(flow: &Flow) -> String {
    format!(
        "#!/bin/bash\n# Managed by Omarchist: runs the '{}' flow after boot.\nsetsid -f {} flow run {} --trigger Startup >/dev/null 2>&1\n",
        flow.id,
        shell_quote(&omarchist_binary()),
        flow.id
    )
}

/// Rewrites every flow's launcher entry and startup hook whose text is out
/// of date: the binary moved (a package upgrade, a build from `target/`) or
/// an older Omarchist wrote a bare `omarchist` that is not on the
/// compositor's PATH. Run at startup, off the UI thread.
pub fn refresh_all(flows: &[Flow]) -> Result<()> {
    for flow in flows {
        if flow.triggers.launcher {
            let entry = desktop_entry_path(&flow.id)?;
            let icon = icon_file_path(&flow.id)?;
            let icon = if icon.exists() {
                icon.to_string_lossy().to_string()
            } else {
                "omarchist".to_string()
            };
            let wanted = desktop_entry(flow, &icon);
            if fs::read_to_string(&entry).ok().as_deref() != Some(wanted.as_str()) {
                write(&entry, &wanted, "launcher entry")?;
            }
        }
        if flow.triggers.startup {
            let hook = startup_hook_path(&flow.id)?;
            let wanted = startup_hook(flow);
            if fs::read_to_string(&hook).ok().as_deref() != Some(wanted.as_str()) {
                write(&hook, &wanted, "startup hook")?;
                fs::set_permissions(&hook, fs::Permissions::from_mode(0o755))
                    .map_err(|e| Error::io("Failed to make the startup hook executable", e))?;
            }
        }
        if flow.triggers.files {
            sync_file_script(flow)?;
        }
    }
    Ok(())
}

fn icon_svg(icon: &str) -> Option<String> {
    let file = OmarchistAssets::get(&icon_path(icon))?;
    let svg = std::str::from_utf8(&file.data).ok()?;
    Some(svg.replace("currentColor", ICON_STROKE))
}

fn write(path: &PathBuf, content: &str, what: &str) -> Result<()> {
    if let Some(dir) = path.parent()
        && !dir.exists()
    {
        fs::create_dir_all(dir)
            .map_err(|e| Error::io(format!("Failed to create {what} directory"), e))?;
    }
    fs::write(path, content).map_err(|e| Error::io(format!("Failed to write {what}"), e))
}

fn remove(path: &PathBuf, what: &str) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::io(format!("Failed to remove {what}"), e)),
    }
}

/// Creates or removes the launcher entry and startup hook so they match
/// `flow.triggers`.
pub fn sync_triggers(flow: &Flow) -> Result<()> {
    let entry = desktop_entry_path(&flow.id)?;
    let icon_file = icon_file_path(&flow.id)?;
    if flow.triggers.launcher {
        let icon = match icon_svg(&flow.icon) {
            Some(svg) => {
                write(&icon_file, &svg, "flow icon")?;
                icon_file.to_string_lossy().to_string()
            }
            None => "omarchist".to_string(),
        };
        write(&entry, &desktop_entry(flow, &icon), "launcher entry")?;
    } else {
        remove(&entry, "launcher entry")?;
        remove(&icon_file, "flow icon")?;
    }

    let hook = startup_hook_path(&flow.id)?;
    if flow.triggers.startup {
        write(&hook, &startup_hook(flow), "startup hook")?;
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755))
            .map_err(|e| Error::io("Failed to make the startup hook executable", e))?;
    } else {
        remove(&hook, "startup hook")?;
    }
    sync_file_script(flow)
}

pub fn remove_triggers(id: &str) -> Result<()> {
    remove(&desktop_entry_path(id)?, "launcher entry")?;
    remove(&icon_file_path(id)?, "flow icon")?;
    remove(&startup_hook_path(id)?, "startup hook")?;
    for script in file_scripts_of(id) {
        remove(&script, "file manager entry")?;
    }
    Ok(())
}

/// Whether the flow `id` has any file outside Omarchist's own folders.
pub fn has_trigger_files(id: &str) -> bool {
    desktop_entry_path(id).is_ok_and(|p| p.exists())
        || startup_hook_path(id).is_ok_and(|p| p.exists())
        || !file_scripts_of(id).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_entry_escapes_field_codes_and_runs_the_flow() {
        let mut flow = Flow::new("morning".into(), "Morning 100%".into());
        flow.description = "Opens\neverything".into();
        let entry = desktop_entry(&flow, "/tmp/morning.svg");
        assert!(entry.contains("Name=Morning 100%\n"));
        assert!(entry.contains("Comment=Opens everything\n"));
        let binary = omarchist_binary();
        assert!(
            entry.contains(&format!(
                "Exec={} flow run morning --trigger Launcher\n",
                desktop_exec_quote(&binary)
            )),
            "{entry}"
        );
        assert!(entry.contains("Icon=/tmp/morning.svg\n"));
        assert!(entry.contains("X-Omarchist-Flow=morning\n"));

        let hook = startup_hook(&flow);
        assert!(hook.starts_with("#!/bin/bash\n"));
        assert!(hook.ends_with(&format!(
            "setsid -f {} flow run morning --trigger Startup >/dev/null 2>&1\n",
            shell_quote(&binary)
        )));
        assert!(
            !hook.contains("Morning 100%"),
            "the hook names the id, never the free-text name"
        );
    }

    #[test]
    fn the_file_manager_script_passes_the_files_on_as_input() {
        let mut flow = Flow::new("archive".into(), " Archive / these\nfiles ".into());
        flow.triggers.files = true;
        let script = file_script(&flow);
        assert!(script.starts_with("#!/bin/sh\n"));
        assert!(script.contains("\n# omarchist-flow: archive\n"));
        assert!(
            script.ends_with(&format!(
                "exec {} flow run archive --trigger 'Files menu' -- \"$@\"\n",
                shell_quote(&omarchist_binary())
            )),
            "{script}"
        );
        // What the menu shows is the name, as a file name.
        assert_eq!(file_script_name(&flow), "Archive   these files");
        flow.name = "/".into();
        assert_eq!(file_script_name(&flow), "archive");
    }

    #[test]
    fn icon_svg_uses_a_fixed_stroke() {
        let svg = icon_svg("workflow").expect("workflow icon is embedded");
        assert!(svg.contains(ICON_STROKE));
        assert!(!svg.contains("currentColor"));
        assert!(
            icon_svg("not-an-icon").is_some(),
            "falls back to the default icon"
        );
    }
}
