use crate::error::{Error, Result};
use smol::unblock;
use std::path::Path;
use std::process::{Command, Stdio};

pub async fn apply_theme(dir: String) -> Result<()> {
    apply_theme_with_cmd("omarchy-theme-set", dir).await
}

/// `omarchy-theme-set <dir>`, waiting for it. Blocking: run it off the UI
/// thread (`cx.background_spawn`).
pub fn apply_theme_blocking(dir: &str) -> Result<()> {
    run_theme_cmd("omarchy-theme-set", dir)
}

async fn apply_theme_with_cmd(cmd: &'static str, dir: String) -> Result<()> {
    unblock(move || run_theme_cmd(cmd, &dir)).await
}

fn run_theme_cmd(cmd: &str, dir: &str) -> Result<()> {
    let output = Command::new(cmd)
        .arg(dir)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| Error::io(format!("Failed to execute {cmd}"), e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        return Err(Error::Invalid(if stderr.is_empty() {
            format!("{cmd} exited with {}", output.status)
        } else {
            stderr.to_string()
        }));
    }

    Ok(())
}

/// `omarchy-theme-refresh`, waiting for it. Blocking: run it off the UI
/// thread.
pub fn refresh_theme_blocking() -> Result<()> {
    let output = Command::new("omarchy-theme-refresh")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| Error::io("Failed to execute omarchy-theme-refresh", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        return Err(Error::Invalid(if stderr.is_empty() {
            format!("omarchy-theme-refresh exited with {}", output.status)
        } else {
            stderr.to_string()
        }));
    }
    Ok(())
}

/// Resolves a `colors.toml` into Omarchy's full palette (`key<TAB>value`
/// lines), aliases and derived shades included.
pub fn theme_color_all(colors_file: &Path) -> Result<String> {
    let output = Command::new("omarchy-theme-color")
        .arg("--file")
        .arg(colors_file)
        .arg("--all")
        .output()
        .map_err(|e| Error::io("Failed to execute omarchy-theme-color", e))?;

    if !output.status.success() {
        return Err(Error::Invalid(format!(
            "omarchy-theme-color failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Renders the boot screen preview `omarchy-plymouth-switcher` shows for a
/// theme, from its colors and logo.
/// Renders a Plymouth preview with Omarchy's own `omarchy-plymouth-preview`.
/// The script ends by opening the result in a fullscreen `imv` (it is a
/// user-facing preview command), which would cover the app and block until
/// the viewer is closed, so it runs with a PATH shim whose `imv` does
/// nothing; the written file is the result.
pub fn plymouth_preview(background: &str, text: &str, logo: &Path, output: &Path) -> Result<()> {
    let shim_dir = imv_shim_dir()?;
    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![shim_dir.clone()];
    paths.extend(std::env::split_paths(&path));
    let path = std::env::join_paths(paths)
        .map_err(|e| Error::Invalid(format!("Cannot build PATH for the preview: {e}")))?;
    let result = Command::new("omarchy-plymouth-preview")
        .arg(background)
        .arg(text)
        .arg(logo)
        .arg(output)
        .env("PATH", path)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| Error::io("Failed to execute omarchy-plymouth-preview", e));
    let _ = std::fs::remove_dir_all(&shim_dir);
    let result = result?;

    if !result.status.success() && !output.is_file() {
        return Err(Error::Invalid(format!(
            "omarchy-plymouth-preview failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        )));
    }

    Ok(())
}

/// A directory holding an `imv` that exits at once, to put first on PATH.
fn imv_shim_dir() -> Result<std::path::PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!(
        "omarchist-plymouth-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default()
    ));
    std::fs::create_dir_all(&dir).map_err(|e| Error::io("Failed to create the preview shim", e))?;
    let imv = dir.join("imv");
    std::fs::write(&imv, "#!/bin/sh\nexit 0\n")
        .map_err(|e| Error::io("Failed to create the preview shim", e))?;
    std::fs::set_permissions(&imv, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| Error::io("Failed to create the preview shim", e))?;
    Ok(dir)
}

// Refresh apps to apply theme changes
pub fn refresh_theme() -> Result<()> {
    spawn_fire_and_forget("omarchy-theme-refresh")
}

// Execute a bash command without waiting for output (fire and forget)
pub fn execute_bash_command(command: String) -> Result<()> {
    Command::new("bash")
        .arg("-c")
        .arg(&command)
        .spawn()
        .map_err(|e| Error::io("Failed to spawn command", e))?;

    Ok(())
}

fn spawn_fire_and_forget(cmd: &str) -> Result<()> {
    Command::new(cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| Error::io(format!("Failed to spawn {cmd}"), e))?;

    Ok(())
}
// `uwsm app -- <app-name>` for launching apps

#[cfg(test)]
mod tests {
    use super::*;

    // ── apply_theme ──────────────────────────────────────────────────────────

    /// A command that always exits 0 should resolve to Ok(()).
    #[test]
    fn apply_theme_with_cmd_success_returns_ok() {
        smol::block_on(async {
            let result = apply_theme_with_cmd("true", "my-theme".to_string()).await;
            assert!(
                result.is_ok(),
                "expected Ok when command exits 0, got {:?}",
                result
            );
        });
    }

    /// A command that exits non-zero without a message names the command
    /// and its status, so the notification is never "failed: ".
    #[test]
    fn apply_theme_with_cmd_nonzero_exit_returns_err() {
        smol::block_on(async {
            let result = apply_theme_with_cmd("false", "my-theme".to_string()).await;
            let msg = result.expect_err("non-zero exit is an error").to_string();
            assert!(msg.contains("false exited with"), "{msg}");
        });
    }

    /// A missing binary should resolve to Err containing a descriptive message.
    #[test]
    fn apply_theme_with_cmd_missing_binary_returns_err() {
        smol::block_on(async {
            let result =
                apply_theme_with_cmd("__omarchist_nonexistent_binary__", "any".to_string()).await;
            assert!(result.is_err(), "expected Err for missing binary, got Ok");
            let msg = result.unwrap_err().to_string();
            assert!(
                msg.contains("Failed to execute"),
                "error message should mention 'Failed to execute', got: {msg}"
            );
        });
    }

    // ── refresh_theme ────────────────────────────────────────────────────────

    /// spawn_fire_and_forget with a real binary should return Ok immediately
    /// (fire-and-forget; we do not wait for the process to finish).
    #[test]
    fn spawn_fire_and_forget_with_real_binary_returns_ok() {
        // `true` is always present and exits 0 — we only care that spawn succeeds.
        let result = spawn_fire_and_forget("true");
        assert!(
            result.is_ok(),
            "expected Ok when spawning a real binary, got {:?}",
            result
        );
    }

    /// spawn_fire_and_forget with a missing binary should return Err containing
    /// both the command name and a description.
    #[test]
    fn spawn_fire_and_forget_missing_binary_returns_err_with_cmd_name() {
        let result = spawn_fire_and_forget("__omarchist_nonexistent_binary__");
        assert!(result.is_err(), "expected Err for missing binary, got Ok");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Failed to spawn"),
            "error message should contain 'Failed to spawn', got: {msg}"
        );
        assert!(
            msg.contains("__omarchist_nonexistent_binary__"),
            "error message should contain the command name, got: {msg}"
        );
    }

    // ── execute_bash_command ─────────────────────────────────────────────────

    /// A valid bash command should return Ok (fire-and-forget spawn succeeded).
    #[test]
    fn execute_bash_command_valid_command_returns_ok() {
        let result = execute_bash_command("true".to_string());
        assert!(
            result.is_ok(),
            "expected Ok for a valid bash command, got {:?}",
            result
        );
    }

    /// Even a bash command that would exit non-zero is still a successful *spawn*,
    /// so execute_bash_command should return Ok (it never waits for exit status).
    #[test]
    fn execute_bash_command_failing_command_still_returns_ok() {
        let result = execute_bash_command("false".to_string());
        assert!(
            result.is_ok(),
            "expected Ok even for a command that exits non-zero (fire-and-forget), got {:?}",
            result
        );
    }

    /// An empty string is a valid argument for `bash -c` (it's a no-op), so Ok.
    #[test]
    fn execute_bash_command_empty_string_returns_ok() {
        let result = execute_bash_command(String::new());
        assert!(
            result.is_ok(),
            "expected Ok for an empty bash command, got {:?}",
            result
        );
    }
}
