//! The background service that runs automations: a systemd user unit
//! that starts `omarchist automations run` with the graphical session.
//! It is installed when the person turns automations on and removed when
//! they turn them off or uninstall.
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::error::{Error, Result};
use crate::system::binary::omarchist_binary;
use crate::system::fs::write_atomic;

pub const UNIT: &str = "omarchist-automations.service";

/// `~/.config/systemd/user/omarchist-automations.service`
pub fn unit_path() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(".config/systemd/user").join(UNIT))
        .ok_or(Error::UnknownDirectory("home"))
}

/// The unit. It belongs to the graphical session, so it has Hyprland's
/// and Wayland's environment, stops at logout and starts at login.
pub fn unit_text(binary: &str) -> String {
    // systemd reads `%` as a specifier and splits unquoted words.
    let exec = binary.replace('%', "%%").replace('"', "\\\"");
    format!(
        "[Unit]\n\
         Description=Omarchist automations\n\
         Documentation=https://omarchist.com/flows/automations\n\
         PartOf=graphical-session.target\n\
         After=graphical-session.target\n\
         \n\
         [Service]\n\
         ExecStart=\"{exec}\" automations run\n\
         Restart=on-failure\n\
         RestartSec=5\n\
         \n\
         [Install]\n\
         WantedBy=graphical-session.target\n"
    )
}

fn systemctl(args: &[&str]) -> Result<bool> {
    Command::new("systemctl")
        .arg("--user")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .map_err(|e| Error::io("Could not run systemctl", e))
}

/// Whether the service is set to start with the session.
pub fn is_enabled() -> bool {
    unit_path().is_ok_and(|path| path.is_file())
        && systemctl(&["is-enabled", "--quiet", UNIT]).unwrap_or(false)
}

/// Whether the service is running now.
pub fn is_active() -> bool {
    systemctl(&["is-active", "--quiet", UNIT]).unwrap_or(false)
}

/// Installs the unit and starts it, now and at every login.
pub fn enable() -> Result<()> {
    let path = unit_path()?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| Error::io("Failed to create the systemd user folder", e))?;
    }
    write_atomic(
        &path,
        unit_text(&omarchist_binary()),
        "the automations service",
    )?;
    systemctl(&["daemon-reload"])?;
    if systemctl(&["enable", "--now", UNIT])? {
        Ok(())
    } else {
        // Leave nothing behind that looks half turned on.
        let _ = std::fs::remove_file(&path);
        let _ = systemctl(&["daemon-reload"]);
        Err(Error::Invalid(
            "systemd could not start the automations service".to_string(),
        ))
    }
}

/// Stops the service and removes the unit.
pub fn disable() -> Result<()> {
    let path = unit_path()?;
    if path.is_file() {
        systemctl(&["disable", "--now", UNIT])?;
        std::fs::remove_file(&path)
            .map_err(|e| Error::io("Failed to remove the automations service", e))?;
        systemctl(&["daemon-reload"])?;
    }
    Ok(())
}

/// Rewrites the unit when the binary it names has moved (a package
/// upgrade, a build from `target/`) and restarts the service so it runs
/// the new one. Run at startup; does nothing when automations are off.
pub fn refresh() -> Result<()> {
    let path = unit_path()?;
    if !path.is_file() {
        return Ok(());
    }
    let wanted = unit_text(&omarchist_binary());
    if std::fs::read_to_string(&path).ok().as_deref() == Some(wanted.as_str()) {
        return Ok(());
    }
    write_atomic(&path, wanted, "the automations service")?;
    systemctl(&["daemon-reload"])?;
    if is_active() {
        systemctl(&["restart", UNIT])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::unit_text;

    #[test]
    fn the_unit_runs_the_service_with_the_session() {
        let unit = unit_text("/usr/bin/omarchist");
        assert!(unit.contains("ExecStart=\"/usr/bin/omarchist\" automations run\n"));
        assert!(unit.contains("PartOf=graphical-session.target\n"));
        assert!(unit.contains("WantedBy=graphical-session.target\n"));
        assert!(unit.contains("Restart=on-failure\n"));
        // A path with a space or a percent sign stays one word.
        let odd = unit_text("/home/me/My Apps/100%/omarchist");
        assert!(
            odd.contains("ExecStart=\"/home/me/My Apps/100%%/omarchist\" automations run\n"),
            "{odd}"
        );
    }
}
