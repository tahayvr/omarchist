//! Omarchy's installed version and update availability, read through
//! Omarchy's own tooling so the answers match what `omarchy-update` would do.
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::error::{Error, Result};

/// How often the background check re-runs. Omarchy's own bar widget polls
/// `omarchy-update-available` on the same schedule.
pub const PERIODIC_CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

/// The installed version as `omarchy-version` prints it: the pacman package
/// version (`4.0.4-1`), or `dev (<hash>)` for a `$OMARCHY_PATH` checkout.
/// `None` when the command is missing or fails.
pub fn installed_version() -> Option<String> {
    let output = Command::new("omarchy-version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!version.is_empty()).then_some(version)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateCheck {
    UpToDate,
    /// One line per pending update, as `omarchy-update-available` prints
    /// them, for example `omarchy 4.0.4-1 -> 4.0.5-1`.
    Available(Vec<String>),
}

/// Runs `omarchy-update-available`, which compares the installed package with
/// the configured channel's repository through `checkupdates` and a dev
/// checkout with its upstream branch.
pub fn check_for_updates() -> Result<UpdateCheck> {
    let output = Command::new("omarchy-update-available")
        .output()
        .map_err(|e| Error::io("Failed to run omarchy-update-available", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_check(output.status.code(), &stdout)
}

fn parse_check(code: Option<i32>, stdout: &str) -> Result<UpdateCheck> {
    let lines: Vec<String> = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();
    match code {
        Some(0) if !lines.is_empty() => Ok(UpdateCheck::Available(lines)),
        // The script exits 1 both when nothing is pending and when one of its
        // own steps fails, so only its "up to date" message counts as a result.
        Some(1) if lines.iter().any(|line| line == "Omarchy is up to date") => {
            Ok(UpdateCheck::UpToDate)
        }
        Some(code) => Err(Error::Invalid(format!(
            "omarchy-update-available exited with status {code}"
        ))),
        None => Err(Error::Invalid(
            "omarchy-update-available was stopped by a signal".into(),
        )),
    }
}

/// Whether `omarchy-update` is running. It holds a `flock` on
/// `$XDG_RUNTIME_DIR/omarchy-update.lock` from its confirmation prompt to
/// its last step, so a failed non-blocking lock means an update is in
/// progress.
pub fn update_in_progress() -> bool {
    let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") else {
        return false;
    };
    let lock = Path::new(&runtime_dir).join("omarchy-update.lock");
    if !lock.is_file() {
        return false;
    }
    Command::new("flock")
        .arg("-n")
        .arg(&lock)
        .arg("true")
        .status()
        .map(|status| !status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_updates_are_listed() {
        let check = parse_check(Some(0), "omarchy 4.0.4-1 -> 4.0.5-1\n").unwrap();
        assert_eq!(
            check,
            UpdateCheck::Available(vec!["omarchy 4.0.4-1 -> 4.0.5-1".to_string()])
        );
    }

    #[test]
    fn dev_checkout_and_package_updates_are_both_listed() {
        let check = parse_check(
            Some(0),
            "omarchy-dev-checkout 3 new commits on origin/master\nomarchy 4.0.4-1 -> 4.0.5-1\n",
        )
        .unwrap();
        assert!(matches!(check, UpdateCheck::Available(lines) if lines.len() == 2));
    }

    #[test]
    fn up_to_date_message_is_recognised() {
        assert_eq!(
            parse_check(Some(1), "Omarchy is up to date\n").unwrap(),
            UpdateCheck::UpToDate
        );
    }

    #[test]
    fn a_failed_script_is_an_error_not_up_to_date() {
        assert!(parse_check(Some(1), "").is_err());
        assert!(parse_check(Some(127), "").is_err());
        assert!(parse_check(None, "").is_err());
    }

    #[test]
    fn success_without_output_is_an_error() {
        assert!(parse_check(Some(0), "\n").is_err());
    }
}
