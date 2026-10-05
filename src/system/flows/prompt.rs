//! How a flow asks the person at the keyboard: Omarchy's own menu for a
//! line of text or a pick from a list, and the desktop's file chooser for
//! a path. The runner goes through [`Prompter`] so tests can answer
//! without a desktop.
use std::io::Write;
use std::process::{Command, Stdio};

use crate::error::{Error, Result};

use super::MAX_OPTIONS;
use super::runner::Cancel;

/// Wide enough for a question; Omarchy's default suits a single word.
const MENU_WIDTH: &str = "420";
/// How long Omarchy's scripts wait for the shell to take the request. The
/// first menu of a session loads the menu itself, which can outlast the
/// scripts' own two seconds.
const SHELL_TIMEOUT: &str = "6s";

/// Asks and returns the answer, or `None` when the person dismissed the
/// prompt.
pub trait Prompter: Sync {
    fn ask(&self, prompt: &str, cancel: &Cancel) -> Result<Option<String>>;
    fn choose(&self, prompt: &str, options: &[String], cancel: &Cancel) -> Result<Option<String>>;
    fn pick(&self, prompt: &str, folder: bool) -> Result<Option<String>>;
}

/// The desktop's own prompts.
pub struct Desktop;

impl Prompter for Desktop {
    fn ask(&self, prompt: &str, cancel: &Cancel) -> Result<Option<String>> {
        let build = || {
            let mut cmd = Command::new("omarchy-menu-input");
            cmd.args([prompt.trim(), "--width", MENU_WIDTH]);
            cmd
        };
        menu(&build, None, cancel)
    }

    fn choose(&self, prompt: &str, options: &[String], cancel: &Cancel) -> Result<Option<String>> {
        let options = menu_options(options);
        if options.is_empty() {
            return Err(Error::Invalid(
                "There is nothing to choose from".to_string(),
            ));
        }
        // The options go in on stdin, one per line, so one that reads `--`
        // is never taken for the script's own separator.
        let build = || {
            let mut cmd = Command::new("omarchy-menu-select");
            cmd.args([prompt.trim(), "--", "--width", MENU_WIDTH]);
            cmd
        };
        menu(&build, Some(options.join("\n")), cancel)
    }

    fn pick(&self, prompt: &str, folder: bool) -> Result<Option<String>> {
        let mut dialog = rfd::FileDialog::new();
        if !prompt.trim().is_empty() {
            dialog = dialog.set_title(prompt.trim());
        }
        if let Some(home) = dirs::home_dir() {
            dialog = dialog.set_directory(home);
        }
        let path = if folder {
            dialog.pick_folder()
        } else {
            dialog.pick_file()
        };
        Ok(path.map(|p| p.display().to_string()))
    }
}

/// Options as the menu takes them: one line each, no tabs (the menu reads
/// a tab as "icon, then label"), blanks dropped, at most [`MAX_OPTIONS`].
pub fn menu_options(options: &[String]) -> Vec<String> {
    options
        .iter()
        .flat_map(|o| o.lines())
        .map(|line| line.replace('\t', " ").trim().to_string())
        .filter(|line| !line.is_empty())
        .take(MAX_OPTIONS)
        .collect()
}

/// Runs one of Omarchy's menu scripts and returns what was typed or
/// picked. A shell that was not ready to show the menu gets one more try.
fn menu(
    build: &dyn Fn() -> Command,
    stdin: Option<String>,
    cancel: &Cancel,
) -> Result<Option<String>> {
    match menu_once(build(), stdin.clone(), cancel) {
        Err(Error::Invalid(message)) if is_shell_not_ready(&message) => {
            menu_once(build(), stdin, cancel)
        }
        result => result,
    }
}

fn is_shell_not_ready(message: &str) -> bool {
    message.contains("not responding") || message.contains("not ready")
}

/// The script exits 1 with nothing printed when the menu is dismissed. A
/// Stop from the editor kills the script and closes the menu.
fn menu_once(mut cmd: Command, stdin: Option<String>, cancel: &Cancel) -> Result<Option<String>> {
    cmd.env("OMARCHY_SHELL_IPC_TIMEOUT", SHELL_TIMEOUT);
    cmd.stdin(if stdin.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::Invalid("Omarchy's menu is not installed, so the flow cannot ask".to_string())
        } else {
            Error::io("Could not open the menu", e)
        }
    })?;
    if let (Some(text), Some(mut pipe)) = (stdin, child.stdin.take()) {
        // Closed on drop, which is what ends the script's read.
        let _ = pipe.write_all(text.as_bytes());
    }
    cancel.watch(child.id());
    let output = child
        .wait_with_output()
        .map_err(|e| Error::io("Could not wait for the menu", e));
    cancel.unwatch();
    let output = output?;
    if cancel.is_cancelled() {
        let _ = Command::new("omarchy-shell")
            .args(["-q", "shell", "hide", "omarchy.menu"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        return Err(Error::Invalid("Stopped".to_string()));
    }
    let answer = String::from_utf8_lossy(&output.stdout);
    let answer = answer.trim_end_matches(['\n', '\r']);
    if output.status.success() && !answer.is_empty() {
        return Ok(Some(answer.to_string()));
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    if output.status.code() == Some(1) && stderr.is_empty() {
        // Dismissed.
        return Ok(None);
    }
    Err(Error::Invalid(if stderr.is_empty() {
        "The menu closed without an answer".to_string()
    } else {
        format!("The menu failed: {stderr}")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_are_one_clean_line_each() {
        let options = vec![
            "  Work  ".to_string(),
            "a\tb".to_string(),
            String::new(),
            "one\ntwo\n\nthree".to_string(),
            "--".to_string(),
        ];
        assert_eq!(
            menu_options(&options),
            vec!["Work", "a b", "one", "two", "three", "--"]
        );
    }

    #[test]
    fn a_long_list_is_cut() {
        let options: Vec<String> = (0..MAX_OPTIONS + 20).map(|n| n.to_string()).collect();
        assert_eq!(menu_options(&options).len(), MAX_OPTIONS);
    }
}
