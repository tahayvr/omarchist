//! The files that let other parts of the desktop start a flow: a `.desktop`
//! entry for the app launcher and a script in Omarchy's `post-boot` hook
//! directory for startup.
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::OmarchistAssets;
use crate::error::{Error, Result};

use super::{Flow, icon_path};

/// Launchers render the icon on light and dark backgrounds alike, so the
/// Lucide `currentColor` stroke is replaced with a mid grey.
const ICON_STROKE: &str = "#9ca3af";

fn home() -> Result<PathBuf> {
    dirs::home_dir().ok_or(Error::UnknownDirectory("home"))
}

/// `~/.local/share/applications/omarchist-flow-<id>.desktop`
pub fn desktop_entry_path(id: &str) -> Result<PathBuf> {
    Ok(home()?
        .join(".local/share/applications")
        .join(format!("omarchist-flow-{id}.desktop")))
}

/// `~/.local/share/omarchist/flows/<id>.svg`
pub fn icon_file_path(id: &str) -> Result<PathBuf> {
    Ok(home()?
        .join(".local/share/omarchist/flows")
        .join(format!("{id}.svg")))
}

/// `~/.config/omarchy/hooks/post-boot.d/omarchist-flow-<id>`
pub fn startup_hook_path(id: &str) -> Result<PathBuf> {
    Ok(home()?
        .join(".config/omarchy/hooks/post-boot.d")
        .join(format!("omarchist-flow-{id}")))
}

/// The desktop entry text. `%` is doubled in the name and comment because
/// the spec treats it as a field code introducer there too.
pub fn desktop_entry(flow: &Flow, icon: &str) -> String {
    let escape = |s: &str| s.replace('%', "%%").replace('\n', " ");
    let comment = if flow.description.trim().is_empty() {
        "Omarchist flow".to_string()
    } else {
        escape(flow.description.trim())
    };
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
        exec = flow.command(),
        id = flow.id,
    )
}

pub fn startup_hook(flow: &Flow) -> String {
    format!(
        "#!/bin/bash\n# Managed by Omarchist: runs the '{}' flow after boot.\n{}\n",
        flow.name.trim(),
        flow.command()
    )
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
    Ok(())
}

pub fn remove_triggers(id: &str) -> Result<()> {
    remove(&desktop_entry_path(id)?, "launcher entry")?;
    remove(&icon_file_path(id)?, "flow icon")?;
    remove(&startup_hook_path(id)?, "startup hook")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_entry_escapes_field_codes_and_runs_the_flow() {
        let mut flow = Flow::new("morning".into(), "Morning 100%".into());
        flow.description = "Opens\neverything".into();
        let entry = desktop_entry(&flow, "/tmp/morning.svg");
        assert!(entry.contains("Name=Morning 100%%\n"));
        assert!(entry.contains("Comment=Opens everything\n"));
        assert!(entry.contains("Exec=omarchist flow run 'morning'\n"));
        assert!(entry.contains("Icon=/tmp/morning.svg\n"));
        assert!(entry.contains("X-Omarchist-Flow=morning\n"));

        let hook = startup_hook(&flow);
        assert!(hook.starts_with("#!/bin/bash\n"));
        assert!(hook.ends_with("omarchist flow run 'morning'\n"));
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
