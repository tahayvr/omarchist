use smol::unblock;
use std::process::{Command, Stdio};

pub async fn apply_theme(dir: String) -> Result<(), String> {
    unblock(move || {
        let output = Command::new("omarchy-theme-set")
            .arg(&dir)
            .output()
            .map_err(|e| format!("Failed to execute omarchy-theme-set: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to apply theme '{dir}': {stderr}"));
        }

        Ok(())
    })
    .await
}

// Refresh apps to apply theme changes
pub fn refresh_theme() -> Result<(), String> {
    Command::new("omarchy-theme-refresh")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn omarchy-theme-refresh: {e}"))?;

    Ok(())
}

// Execute a bash command without waiting for output (fire and forget)
pub fn execute_bash_command(command: String) -> Result<(), String> {
    Command::new("bash")
        .arg("-c")
        .arg(&command)
        .spawn()
        .map_err(|e| format!("Failed to spawn command: {e}"))?;

    Ok(())
}
// `uwsm app -- <app-name>` for launching apps
