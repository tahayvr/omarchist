//! Omarchy's own settings, the ones that are not Hyprland options: idle
//! timings and the bar in `shell.json`, flag files under Omarchy's state
//! directory, and values that only an `omarchy-*` script reads or sets.
//! Each setting is a [`Backing`]: how to read it and how to write it, so
//! the Configuration page can treat them as data the way it treats
//! Hyprland options, and every write goes through Omarchy's own command
//! for it (never a hand-rolled copy of what the script does).
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::{Map, Value};

use crate::error::{Error, Result};
use crate::system::omarchy_paths::{omarchy_install_dir, omarchy_state_dir, shell_json_path};

/// Where a setting's current value comes from.
#[derive(Debug, Clone, Copy)]
pub enum Read {
    /// A dotted key in `~/.config/omarchy/shell.json` (`idle.lock`).
    ShellJson(&'static str),
    /// A setting of a bar widget's layout entry in `shell.json`.
    ShellWidget { id: &'static str, key: &'static str },
    /// A flag file under `~/.local/state/omarchy/` (`toggles/bar-off`):
    /// `true` when it exists, or when it does not if `inverted`.
    Flag { flag: &'static str, inverted: bool },
    /// The trimmed content of a file under `~/.local/state/omarchy/`.
    StateFile(&'static str),
    /// A `KEY=value` line of a shell-style file (`/etc/vconsole.conf`).
    EnvFile {
        path: &'static str,
        key: &'static str,
    },
    /// What a command prints, read as `parse` says.
    Command {
        argv: &'static [&'static str],
        parse: Parse,
    },
}

/// How a command's output becomes a value.
#[derive(Debug, Clone, Copy)]
pub enum Parse {
    /// The first non-empty line, as a string.
    Line,
    /// Whether the first non-empty line equals this.
    LineIs(&'static str),
    /// The second whitespace-separated token of the first line.
    SecondToken,
    /// The first integer in the output.
    Int,
    /// Whether the command succeeded.
    Succeeds,
    /// Whether the command failed.
    Fails,
    /// A boolean field of the JSON on the first line.
    JsonBool(&'static str),
    /// An integer field of the JSON on the first line, or `fallback`
    /// when it is null.
    JsonInt { field: &'static str, fallback: i64 },
}

/// How a new value is applied.
#[derive(Debug, Clone, Copy)]
pub enum Write {
    /// Set the dotted key in `shell.json` and tell the shell to reload.
    ShellJson(&'static str),
    /// Run every command of `on` or `off` for a boolean.
    Bool {
        on: &'static [&'static [&'static str]],
        off: &'static [&'static [&'static str]],
    },
    /// A command that flips a boolean, run only when the current value
    /// differs from the wanted one.
    ToggleIfDifferent(&'static [&'static str]),
    /// A command with the value appended as its last argument.
    Command(&'static [&'static str]),
    /// `Command`, followed by more commands once it succeeded.
    CommandAnd {
        argv: &'static [&'static str],
        then: &'static [&'static [&'static str]],
    },
    /// A command with the value appended as JSON plus `--json`
    /// (`omarchy bar set <id> <key> <value> --json`).
    CommandJson(&'static [&'static str]),
    /// A command with the value appended, run in Omarchy's floating
    /// terminal because it asks for sudo or confirmation.
    Terminal(&'static [&'static str]),
}

#[derive(Debug, Clone, Copy)]
pub struct Backing {
    pub read: Read,
    pub write: Write,
}

/// Where a dropdown's choices come from.
#[derive(Debug, Clone, Copy)]
pub enum Options {
    /// `(value, label)`.
    Static(&'static [(&'static str, &'static str)]),
    /// One choice per line of a command's output, value and label alike.
    Lines(&'static [&'static str]),
    /// `(value to set, value as read back, label, binary)`; a choice is
    /// offered only when `omarchy-cmd-present <binary>` succeeds.
    Installed(&'static [(&'static str, &'static str, &'static str, &'static str)]),
}

/// One choice of a dropdown: the value written, the value read back
/// (usually the same), and the label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    pub value: String,
    pub reads_as: String,
    pub label: String,
}

pub fn read(backing: &Backing) -> Option<Value> {
    read_from(&backing.read)
}

pub fn read_from(read: &Read) -> Option<Value> {
    match read {
        Read::ShellJson(path) => get_path(&load_shell_json()?, path).cloned(),
        Read::ShellWidget { id, key } => {
            let shell = load_shell_json()?;
            let layout = shell["bar"]["layout"].as_object()?;
            layout
                .values()
                .filter_map(Value::as_array)
                .flatten()
                .find(|entry| entry["id"].as_str() == Some(id))
                .and_then(|entry| entry.get(*key).cloned())
        }
        Read::Flag { flag, inverted } => {
            let exists = omarchy_state_dir()?.join(flag).exists();
            Some(Value::Bool(exists != *inverted))
        }
        Read::StateFile(rel) => fs::read_to_string(omarchy_state_dir()?.join(rel))
            .ok()
            .map(|s| Value::String(s.trim().to_string())),
        Read::EnvFile { path, key } => {
            let content = fs::read_to_string(path).ok()?;
            env_file_value(&content, key).map(Value::String)
        }
        Read::Command { argv, parse } => {
            let output = Command::new(argv[0])
                .args(&argv[1..])
                .stdin(Stdio::null())
                .output()
                .ok()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_output(*parse, output.status.success(), &stdout)
        }
    }
}

fn parse_output(parse: Parse, success: bool, stdout: &str) -> Option<Value> {
    let first = stdout.lines().map(str::trim).find(|l| !l.is_empty());
    match parse {
        Parse::Line => first.map(|l| Value::String(l.to_string())),
        Parse::LineIs(expected) => Some(Value::Bool(first == Some(expected))),
        Parse::SecondToken => first
            .and_then(|l| l.split_whitespace().nth(1))
            .map(|t| Value::String(t.to_string())),
        Parse::Int => {
            let digits: String = stdout
                .chars()
                .skip_while(|c| !c.is_ascii_digit() && *c != '-')
                .take_while(|c| c.is_ascii_digit() || *c == '-')
                .collect();
            digits.parse::<i64>().ok().map(Value::from)
        }
        Parse::Succeeds => Some(Value::Bool(success)),
        Parse::Fails => Some(Value::Bool(!success)),
        Parse::JsonBool(field) => {
            let json: Value = serde_json::from_str(first?).ok()?;
            json.get(field).and_then(Value::as_bool).map(Value::Bool)
        }
        Parse::JsonInt { field, fallback } => {
            let json: Value = serde_json::from_str(first?).ok()?;
            Some(Value::from(
                json.get(field).and_then(Value::as_i64).unwrap_or(fallback),
            ))
        }
    }
}

pub fn write(backing: &Backing, value: &Value) -> Result<()> {
    match backing.write {
        Write::ShellJson(path) => {
            let file = shell_json_path().ok_or(Error::UnknownDirectory("home"))?;
            shell_json_set(&file, path, value.clone())?;
            reload_shell_config();
            Ok(())
        }
        Write::Bool { on, off } => {
            let wanted = value.as_bool().unwrap_or(false);
            for argv in if wanted { on } else { off } {
                run(argv)?;
            }
            Ok(())
        }
        Write::ToggleIfDifferent(argv) => {
            let current = read(backing).and_then(|v| v.as_bool());
            if current != value.as_bool() {
                run(argv)?;
            }
            Ok(())
        }
        Write::Command(argv) => run_with(argv, &[&value_text(value)]),
        Write::CommandAnd { argv, then } => {
            run_with(argv, &[&value_text(value)])?;
            for argv in then {
                run(argv)?;
            }
            Ok(())
        }
        Write::CommandJson(argv) => run_with(argv, &[&value.to_string(), "--json"]),
        Write::Terminal(argv) => {
            let mut full: Vec<String> = argv.iter().map(|s| s.to_string()).collect();
            full.push(value_text(value));
            launch_in_terminal(&full)
        }
    }
}

/// `KEY=value` from a shell-style file, unquoted.
fn env_file_value(content: &str, key: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let line = line.trim();
        let rest = line.strip_prefix(key)?.strip_prefix('=')?;
        Some(rest.trim().trim_matches('"').trim_matches('\'').to_string())
    })
}

/// The value as a command argument: strings bare, numbers without a
/// trailing `.0`, booleans as `true`/`false`.
fn value_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => match n.as_i64() {
            Some(i) => i.to_string(),
            None => n.to_string(),
        },
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

/// Runs a command that only makes sense in a terminal (sudo, gum) the
/// way Omarchy's menu does: in its floating terminal.
pub fn launch_in_terminal(argv: &[String]) -> Result<()> {
    let line = argv
        .iter()
        .map(|a| shell_quote(a))
        .collect::<Vec<_>>()
        .join(" ");
    Command::new("omarchy-launch-floating-terminal-with-presentation")
        .arg(line)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| Error::io("Failed to open Omarchy's floating terminal", e))
}

fn shell_quote(arg: &str) -> String {
    if !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:=+@%".contains(c))
    {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

/// Whether a condition command (`omarchy-hw-laptop`) succeeds.
pub fn condition_holds(argv: &[&str]) -> bool {
    Command::new(argv[0])
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

pub fn choices(options: &Options) -> Vec<Choice> {
    match options {
        Options::Static(list) => list
            .iter()
            .map(|(value, label)| Choice {
                value: value.to_string(),
                reads_as: value.to_string(),
                label: label.to_string(),
            })
            .collect(),
        Options::Lines(argv) => Command::new(argv[0])
            .args(&argv[1..])
            .stdin(Stdio::null())
            .output()
            .map(|out| {
                String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(|l| Choice {
                        value: l.to_string(),
                        reads_as: l.to_string(),
                        label: l.to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Options::Installed(list) => list
            .iter()
            .filter(|(_, _, _, binary)| condition_holds(&["omarchy-cmd-present", binary]))
            .map(|(value, reads_as, label, _)| Choice {
                value: value.to_string(),
                reads_as: reads_as.to_string(),
                label: label.to_string(),
            })
            .collect(),
    }
}

fn run(argv: &[&str]) -> Result<()> {
    run_with(argv, &[])
}

fn run_with(argv: &[&str], extra: &[&str]) -> Result<()> {
    let output = Command::new(argv[0])
        .args(&argv[1..])
        .args(extra)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| Error::io(format!("Failed to run {}", argv[0]), e))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if stderr.is_empty() { stdout } else { stderr };
    Err(Error::Invalid(format!(
        "{} failed: {detail}",
        argv.iter()
            .chain(extra.iter())
            .copied()
            .collect::<Vec<_>>()
            .join(" ")
    )))
}

fn reload_shell_config() {
    let _ = Command::new("omarchy-shell")
        .args(["-q", "shell", "reloadConfig"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// The user's `shell.json`, or Omarchy's defaults when they have none yet
/// (the shell and `omarchy-shell-config` do the same).
fn load_shell_json() -> Option<Value> {
    let user = shell_json_path()?;
    let defaults = omarchy_install_dir().join("config/omarchy/shell.json");
    load_shell_json_from(&user, &defaults)
}

fn load_shell_json_from(user: &Path, defaults: &Path) -> Option<Value> {
    let path = if fs::metadata(user).is_ok_and(|m| m.len() > 0) {
        user
    } else {
        defaults
    };
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

/// Sets one key of `shell.json`, writing the whole file atomically.
pub fn shell_json_set(file: &Path, path: &str, value: Value) -> Result<()> {
    let defaults = omarchy_install_dir().join("config/omarchy/shell.json");
    let mut shell =
        load_shell_json_from(file, &defaults).unwrap_or_else(|| Value::Object(Map::new()));
    set_path(&mut shell, path, value);
    let content = serde_json::to_string_pretty(&shell)
        .map_err(|e| Error::json("Failed to serialize shell.json", e))?;
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).map_err(|e| Error::io("Failed to create the config folder", e))?;
    }
    let tmp: PathBuf = file.with_extension("json.tmp");
    fs::write(&tmp, content).map_err(|e| Error::io("Failed to write shell.json", e))?;
    fs::rename(&tmp, file).map_err(|e| Error::io("Failed to replace shell.json", e))
}

fn get_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(root, |node, key| node.get(key))
}

fn set_path(root: &mut Value, path: &str, value: Value) {
    let mut node = root;
    let mut keys = path.split('.').peekable();
    while let Some(key) = keys.next() {
        if !node.is_object() {
            *node = Value::Object(Map::new());
        }
        let map = node.as_object_mut().expect("object");
        if keys.peek().is_none() {
            map.insert(key.to_string(), value);
            return;
        }
        node = map
            .entry(key.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn output_parsers() {
        assert_eq!(
            parse_output(Parse::Line, true, "\nfoot\n"),
            Some(json!("foot"))
        );
        assert_eq!(
            parse_output(Parse::LineIs("on"), true, "on\n"),
            Some(json!(true))
        );
        assert_eq!(
            parse_output(Parse::LineIs("on"), true, "off\n"),
            Some(json!(false))
        );
        assert_eq!(
            parse_output(Parse::SecondToken, true, "band\t5\navailable\t5\n"),
            Some(json!("5"))
        );
        assert_eq!(
            parse_output(Parse::Int, true, "text size: 12 px\ngtk: 1.0\n"),
            Some(json!(12))
        );
        assert_eq!(parse_output(Parse::Succeeds, false, ""), Some(json!(false)));
        assert_eq!(parse_output(Parse::Fails, false, ""), Some(json!(true)));
        assert_eq!(
            parse_output(
                Parse::JsonBool("enabled"),
                true,
                r#"{"enabled":true,"temperature":4000}"#
            ),
            Some(json!(true))
        );
        assert_eq!(
            parse_output(
                Parse::JsonInt {
                    field: "temperature",
                    fallback: 6500
                },
                true,
                r#"{"enabled":false,"temperature":null}"#
            ),
            Some(json!(6500))
        );
    }

    #[test]
    fn env_files_are_read_by_key() {
        let content = "# comment\nKEYMAP=us\nXKBLAYOUT=\"de\"\nXKBMODEL=pc105\n";
        assert_eq!(env_file_value(content, "XKBLAYOUT").as_deref(), Some("de"));
        assert_eq!(env_file_value(content, "KEYMAP").as_deref(), Some("us"));
        assert_eq!(env_file_value(content, "XKBVARIANT"), None);
    }

    #[test]
    fn values_become_plain_arguments() {
        assert_eq!(value_text(&json!("foot")), "foot");
        assert_eq!(value_text(&json!(300)), "300");
        assert_eq!(value_text(&json!(1.6)), "1.6");
        assert_eq!(value_text(&json!(true)), "true");
        assert_eq!(shell_quote("omarchy-dns"), "omarchy-dns");
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn shell_json_set_creates_and_edits_the_file() {
        let dir = std::env::temp_dir().join(format!("omarchist-shell-json-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("shell.json");
        fs::write(
            &file,
            r#"{"version":1,"idle":{"screensaver":150,"lock":300},"bar":{"position":"top"}}"#,
        )
        .unwrap();
        shell_json_set(&file, "idle.lock", json!(600)).unwrap();
        shell_json_set(&file, "bar.transparent", json!(true)).unwrap();
        let shell: Value = serde_json::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
        fs::remove_dir_all(&dir).ok();
        assert_eq!(shell["idle"]["lock"], json!(600));
        assert_eq!(
            shell["idle"]["screensaver"],
            json!(150),
            "other keys survive"
        );
        assert_eq!(shell["bar"]["transparent"], json!(true));
        assert_eq!(shell["bar"]["position"], json!("top"));
    }

    #[test]
    fn static_choices_read_back_as_themselves() {
        let choices = choices(&Options::Static(&[("top", "Top"), ("bottom", "Bottom")]));
        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0].value, "top");
        assert_eq!(choices[0].reads_as, "top");
        assert_eq!(choices[1].label, "Bottom");
    }
}
