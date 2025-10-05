// This file contains the commands that are used to interact with the shell.

use std::process::{Command, Stdio};

// Execute a bash command
#[tauri::command]
pub async fn execute_bash_command(command: String) -> Result<String, String> {
    log::info!("Executing bash command: {}", command);

    let output = Command::new("bash")
        .arg("-c")
        .arg(&command)
        .output()
        .map_err(|e| format!("Failed to execute command: {e}"))?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| format!("Failed to parse command output: {e}"))?;
        log::info!("Command executed successfully");
        Ok(stdout)
    } else {
        let stderr =
            String::from_utf8(output.stderr).unwrap_or_else(|_| "Unknown error".to_string());
        log::error!("Command failed: {}", stderr);
        Err(format!("Command failed: {}", stderr))
    }
}

// Execute a bash command without waiting for output (fire and forget)
#[tauri::command]
pub fn execute_bash_command_async(command: String) -> Result<(), String> {
    log::info!("Executing bash command (async): {}", command);

    Command::new("bash")
        .arg("-c")
        .arg(&command)
        .spawn()
        .map_err(|e| format!("Failed to spawn command: {e}"))?;

    log::info!("Command spawned successfully");
    Ok(())
}

// Apply theme using omarchy-theme-set
#[tauri::command]
pub async fn apply_theme(dir: String) -> Result<(), String> {
    let output = Command::new("omarchy-theme-set").arg(&dir).spawn();

    let result = match output {
        Ok(_) => {
            log::info!("Successfully started omarchy-theme-set for theme {dir}");
            Ok(())
        },
        Err(e) => {
            // Log the error but don't fail the process
            log::warn!("Failed to run omarchy-theme-set: {e}");
            log::info!("Continuing without theme application...");
            // Return Ok to not stop the process
            Ok(())
        },
    };

    // Invalidate cache after theme application to ensure fresh state
    if result.is_ok() {
        if let Ok(cache) = crate::services::cache::cache_manager::get_theme_cache().await {
            // Invalidate the specific theme that was applied
            cache.invalidate_theme(&dir).await;
            // Trigger background refresh to ensure cache is up to date
            let _ = cache.trigger_background_refresh().await;
        }
    }

    result
}

// Refresh apps and gnome
#[tauri::command]
pub fn refresh_theme_adjustments() -> Result<(), String> {
    // Run a best-effort, silent bash script (no terminal)
    let script = r#"
THEME_DIR="$HOME/.config/omarchy/current/theme"

# Change GNOME modes
if [[ -f "$THEME_DIR/light.mode" ]]; then
  if command -v gsettings >/dev/null 2>&1; then
    gsettings set org.gnome.desktop.interface color-scheme "prefer-light" || true
    gsettings set org.gnome.desktop.interface gtk-theme "Adwaita" || true
  fi
else
  if command -v gsettings >/dev/null 2>&1; then
    gsettings set org.gnome.desktop.interface color-scheme "prefer-dark" || true
    gsettings set org.gnome.desktop.interface gtk-theme "Adwaita-dark" || true
  fi
fi

# Change GNOME icon theme color
if [[ -f "$THEME_DIR/icons.theme" ]]; then
  ICON_THEME="$(<"$THEME_DIR/icons.theme")"
  if command -v gsettings >/dev/null 2>&1; then
    gsettings set org.gnome.desktop.interface icon-theme "$ICON_THEME" || true
  fi
else
  if command -v gsettings >/dev/null 2>&1; then
    gsettings set org.gnome.desktop.interface icon-theme "Yaru-blue" || true
  fi
fi

# Change Chromium colors
if command -v chromium &>/dev/null; then
  if [[ -f ~/.config/omarchy/current/theme/light.mode ]]; then
    chromium --no-startup-window --set-color-scheme="light"
  else
    chromium --no-startup-window --set-color-scheme="dark"
  fi

  if [[ -f ~/.config/omarchy/current/theme/chromium.theme ]]; then
    chromium --no-startup-window --set-theme-color="$(<~/.config/omarchy/current/theme/chromium.theme)"
  else
    # Use a default, neutral grey if theme doesn't have a color
    chromium --no-startup-window --set-theme-color="28,32,39"
  fi
fi

# Trigger Alacritty config reload
touch "$HOME/.config/alacritty/alacritty.toml" || true

pkill -SIGUSR2 btop 2>/dev/null || true
omarchy-restart-waybar >/dev/null 2>&1 || true
omarchy-restart-swayosd >/dev/null 2>&1 || true
makoctl reload >/dev/null 2>&1 || true
hyprctl reload >/dev/null 2>&1 || true
pkill -SIGUSR2 ghostty
pkill -SIGUSR1 kitty
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
        log::warn!("Desktop adjustments exited with status: {status}");
        Ok(())
    }
}
