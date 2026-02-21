use std::process::{Command, Stdio};

/// Restart waybar on the user's system so config changes take effect.
pub fn restart_waybar() -> Result<(), String> {
    Command::new("omarchy-restart-waybar")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to restart waybar: {e}"))?;

    Ok(())
}
