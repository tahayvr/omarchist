use smol::unblock;
use std::process::{Command, Stdio};

pub async fn apply_theme(dir: String) -> Result<(), String> {
    apply_theme_with_cmd("omarchy-theme-set", dir).await
}

async fn apply_theme_with_cmd(cmd: &'static str, dir: String) -> Result<(), String> {
    unblock(move || {
        let output = Command::new(cmd)
            .arg(&dir)
            .output()
            .map_err(|e| format!("Failed to execute {cmd}: {e}"))?;

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
    spawn_fire_and_forget("omarchy-theme-refresh")
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

fn spawn_fire_and_forget(cmd: &str) -> Result<(), String> {
    Command::new(cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn {cmd}: {e}"))?;

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

    /// A command that exits non-zero should resolve to Err containing the theme name.
    #[test]
    fn apply_theme_with_cmd_nonzero_exit_returns_err_with_theme_name() {
        smol::block_on(async {
            let result = apply_theme_with_cmd("false", "my-theme".to_string()).await;
            assert!(
                result.is_err(),
                "expected Err when command exits non-zero, got Ok"
            );
            let msg = result.unwrap_err();
            assert!(
                msg.contains("my-theme"),
                "error message should contain the theme name, got: {msg}"
            );
        });
    }

    /// A missing binary should resolve to Err containing a descriptive message.
    #[test]
    fn apply_theme_with_cmd_missing_binary_returns_err() {
        smol::block_on(async {
            let result =
                apply_theme_with_cmd("__omarchist_nonexistent_binary__", "any".to_string()).await;
            assert!(result.is_err(), "expected Err for missing binary, got Ok");
            let msg = result.unwrap_err();
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
        let msg = result.unwrap_err();
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
