use std::process::{Command, Stdio};

/// Launch a floating terminal running the omarchy update script.
pub fn launch_omarchy_update() -> Result<(), String> {
    Command::new("omarchy-launch-floating-terminal-with-presentation")
        .arg("omarchy-update")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch omarchy update: {e}"))?;

    Ok(())
}
