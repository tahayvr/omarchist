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

// Refresh apps and gnome to apply theme changes
pub fn refresh_theme() -> Result<(), String> {
    // Run a best-effort, silent bash script (no terminal)
    let script = r#"
# Restart components to apply new theme
if pgrep -x waybar >/dev/null; then
  omarchy-restart-waybar
fi
omarchy-restart-swayosd
omarchy-restart-terminal
omarchy-restart-hyprctl
omarchy-restart-btop
omarchy-restart-opencode
omarchy-restart-mako

# Change app-specific themes
omarchy-theme-set-gnome
omarchy-theme-set-browser
omarchy-theme-set-vscode
omarchy-theme-set-obsidian
omarchy-theme-set-keyboard

# Call hook on theme set
THEME_NAME="$(cat "$HOME/.config/omarchy/current/theme.name" 2>/dev/null | tr -d '[:space:]')"
omarchy-hook theme-set "$THEME_NAME"
"#;

    let status = Command::new("bash")
        .arg("-c")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("Failed to start bash: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        eprintln!("Desktop adjustments exited with status: {status}");
        Ok(())
    }
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
