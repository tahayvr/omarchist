//! The files that let other parts of the desktop start a flow: a `.desktop`
//! entry for the app launcher and a script in Omarchy's `post-boot` hook
//! directory for startup.
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
        "{} flow run {}",
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
        "#!/bin/bash\n# Managed by Omarchist: runs the '{}' flow after boot.\nsetsid -f {} flow run {} >/dev/null 2>&1\n",
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
        assert!(entry.contains("Name=Morning 100%\n"));
        assert!(entry.contains("Comment=Opens everything\n"));
        let binary = omarchist_binary();
        assert!(
            entry.contains(&format!(
                "Exec={} flow run morning\n",
                desktop_exec_quote(&binary)
            )),
            "{entry}"
        );
        assert!(entry.contains("Icon=/tmp/morning.svg\n"));
        assert!(entry.contains("X-Omarchist-Flow=morning\n"));

        let hook = startup_hook(&flow);
        assert!(hook.starts_with("#!/bin/bash\n"));
        assert!(hook.ends_with(&format!(
            "setsid -f {} flow run morning >/dev/null 2>&1\n",
            shell_quote(&binary)
        )));
        assert!(
            !hook.contains("Morning 100%"),
            "the hook names the id, never the free-text name"
        );
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
