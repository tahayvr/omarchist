//! What a flow does that a person should know about before running one
//! somebody else wrote: commands that run as administrator, delete files,
//! run downloaded code, or run a variable as a command. These are
//! heuristics over the text of the steps that run commands. They point a
//! reader at the steps to read closely; they never replace reading them.
use serde::Serialize;

use super::condition::Condition;
use super::{Flow, StepKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    /// Can do lasting harm, or run code nobody reviewed.
    Danger,
    /// Worth knowing, harmless in most flows.
    Notice,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Risk {
    /// The step's number, as the editor shows it.
    pub step: usize,
    pub level: Level,
    pub what: &'static str,
}

pub const ADMIN: &str = "Runs a command as administrator";
pub const DELETES: &str = "Deletes files or folders";
pub const ERASES: &str = "Can erase a disk";
pub const DOWNLOADED_CODE: &str = "Downloads a script and runs it";
pub const VARIABLE_PROGRAM: &str = "Runs a program named by a variable";
pub const VARIABLE_CODE: &str = "Runs a variable as a shell command";
pub const SECRETS: &str = "Reads private keys or passwords";
pub const STARTUP: &str = "Changes what runs at startup";
pub const NETWORK: &str = "Uses the network";
pub const SENDS: &str = "Can send your clipboard or selected text over the network";

const ADMIN_PROGRAMS: &[&str] = &["sudo", "doas", "pkexec", "su", "run0"];
const SHELLS: &[&str] = &[
    "sh", "bash", "zsh", "fish", "dash", "python", "python3", "perl", "ruby", "node",
];
const DOWNLOADERS: &[&str] = &["curl", "wget"];
const NETWORK_PROGRAMS: &[&str] = &[
    "curl", "wget", "nc", "ncat", "netcat", "ssh", "scp", "sftp", "rsync", "ftp", "telnet",
];
const SECRET_PATHS: &[&str] = &[
    ".ssh/",
    ".gnupg",
    "id_rsa",
    "id_ed25519",
    "/etc/shadow",
    ".password-store",
    ".netrc",
    "keyrings/",
    ".aws/credentials",
];
const STARTUP_PATHS: &[&str] = &[
    ".bashrc",
    ".zshrc",
    ".profile",
    ".bash_profile",
    ".config/hypr/",
    "autostart",
    "crontab",
    "systemctl enable",
    "systemctl --user enable",
    ".config/systemd/",
];

/// How serious a risk named in a catalog entry is. One this build does
/// not know is treated as the serious kind.
pub fn level_of(what: &str) -> Level {
    if [STARTUP, NETWORK, SENDS].contains(&what) {
        Level::Notice
    } else {
        Level::Danger
    }
}

/// Every risk in the flow, in step order. A step can have several.
pub fn risks(flow: &Flow) -> Vec<Risk> {
    let mut found = Vec::new();
    for (index, (_, step)) in flow.walk().into_iter().enumerate() {
        let number = index + 1;
        let mut add = |level: Level, what: &'static str| {
            let risk = Risk {
                step: number,
                level,
                what,
            };
            if !found.contains(&risk) {
                found.push(risk);
            }
        };
        match &step.kind {
            StepKind::Exec { command, .. } => {
                for (level, what) in command_risks(command) {
                    add(level, what);
                }
            }
            // A Hyprland action can start a command too (`hl.dsp.exec_cmd`),
            // and its text is checked the same way.
            StepKind::Lua { expr } => {
                for (level, what) in command_risks(expr) {
                    add(level, what);
                }
            }
            StepKind::If {
                condition: Condition::Command { command },
                ..
            } => {
                for (level, what) in command_risks(command) {
                    add(level, what);
                }
            }
            StepKind::Action { action, .. } if action == "web.get" => {
                add(Level::Notice, NETWORK);
                if uses_private_text(&step.kind.references()) {
                    add(Level::Notice, SENDS);
                }
            }
            _ => {}
        }
    }
    found
}

/// The distinct risks of a flow, the dangerous ones first: what a card or
/// a review comment lists.
pub fn summary(flow: &Flow) -> Vec<(Level, &'static str)> {
    let mut distinct: Vec<(Level, &'static str)> = Vec::new();
    for risk in risks(flow) {
        if !distinct.iter().any(|(_, what)| *what == risk.what) {
            distinct.push((risk.level, risk.what));
        }
    }
    distinct.sort_by_key(|(level, _)| *level);
    distinct
}

fn uses_private_text(references: &[String]) -> bool {
    references
        .iter()
        .any(|name| matches!(name.as_str(), "clipboard" | "selection"))
}

/// The words of a command, lowercased, with shell punctuation dropped and
/// a path reduced to its last part (`/usr/bin/sudo` is `sudo`).
fn words(command: &str) -> Vec<String> {
    command
        .to_lowercase()
        .split(|c: char| c.is_whitespace() || ";|&()`<>\"'".contains(c))
        .filter(|word| !word.is_empty())
        .map(|word| {
            let word = word.trim_start_matches('$');
            match word.rsplit_once('/') {
                // A flag or a URL keeps its slashes; a program path does not.
                Some((_, last)) if !word.starts_with('-') && !word.contains("://") => {
                    last.to_string()
                }
                _ => word.to_string(),
            }
        })
        .collect()
}

fn command_risks(command: &str) -> Vec<(Level, &'static str)> {
    let mut found: Vec<(Level, &'static str)> = Vec::new();
    let lower = command.to_lowercase();
    let words = words(command);
    let has = |names: &[&str]| words.iter().any(|word| names.contains(&word.as_str()));

    if has(ADMIN_PROGRAMS) {
        found.push((Level::Danger, ADMIN));
    }
    // `rm` with a recursive flag anywhere after it.
    if let Some(at) = words.iter().position(|word| word == "rm")
        && words[at + 1..].iter().any(|word| {
            word == "--recursive"
                || (word.starts_with('-') && !word.starts_with("--") && word.contains('r'))
        })
    {
        found.push((Level::Danger, DELETES));
    }
    if has(&["shred", "wipefs", "mkswap"])
        || words.iter().any(|word| word.starts_with("mkfs"))
        || (has(&["dd"]) && lower.contains("of=/dev/"))
    {
        found.push((Level::Danger, ERASES));
    }
    if has(DOWNLOADERS) && runs_what_it_downloads(&lower) {
        found.push((Level::Danger, DOWNLOADED_CODE));
    }
    let has_reference = command.contains("{{");
    if command.trim_start().starts_with("{{") {
        found.push((Level::Danger, VARIABLE_PROGRAM));
    }
    if has_reference
        && (has(&["eval", "source"])
            || code_holds_a_reference(command)
            || lower.contains("$((")
            || lower.contains("hl.dsp.exec"))
    {
        found.push((Level::Danger, VARIABLE_CODE));
    }
    if SECRET_PATHS.iter().any(|path| lower.contains(path)) {
        found.push((Level::Danger, SECRETS));
    }
    if STARTUP_PATHS.iter().any(|path| lower.contains(path)) {
        found.push((Level::Notice, STARTUP));
    }
    if has(NETWORK_PROGRAMS) {
        found.push((Level::Notice, NETWORK));
        if lower.contains("{{clipboard}}") || lower.contains("{{selection}}") {
            found.push((Level::Notice, SENDS));
        }
    }
    found
}

/// `curl … | sh`, `bash <(curl …)`, `sh -c "$(curl …)"`, `eval "$(wget …)"`.
fn runs_what_it_downloads(lower: &str) -> bool {
    let piped = lower.split('|').skip(1).any(|after| {
        let first = after.split_whitespace().next().unwrap_or_default();
        let program = first.rsplit('/').next().unwrap_or(first);
        SHELLS.contains(&program) || program == "sudo"
    });
    let substituted = ["$(curl", "$(wget", "<(curl", "<(wget", "`curl", "`wget"]
        .iter()
        .any(|pattern| lower.contains(pattern));
    piped || substituted
}

/// The command split into arguments as a shell would, quotes kept
/// together and dropped. Enough to find the argument after `-c`; not a
/// shell parser.
fn arguments(command: &str) -> Vec<String> {
    let mut arguments = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut started = false;
    for c in command.chars() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => current.push(c),
            None if c == '\'' || c == '"' => {
                quote = Some(c);
                started = true;
            }
            None if c.is_whitespace() => {
                if started || !current.is_empty() {
                    arguments.push(std::mem::take(&mut current));
                }
                started = false;
            }
            None => current.push(c),
        }
    }
    if started || !current.is_empty() {
        arguments.push(current);
    }
    arguments
}

/// `sh -c "… {{x}} …"`: the string a shell or an interpreter is handed to
/// run has a variable in it. A variable passed after the code, as in
/// `sh -c 'echo "$1"' _ {{x}}`, is an argument, which is fine.
fn code_holds_a_reference(command: &str) -> bool {
    let arguments = arguments(command);
    arguments.iter().enumerate().any(|(ix, argument)| {
        let program = argument.rsplit('/').next().unwrap_or(argument);
        if !SHELLS.contains(&program.to_lowercase().as_str()) {
            return false;
        }
        // The code is what follows the first flag with a `c` or an `e`
        // in it (`-c`, `-lc`, `-e`), before any other argument.
        let mut rest = arguments[ix + 1..].iter();
        while let Some(flag) = rest.next() {
            if !flag.starts_with('-') {
                return false;
            }
            if !flag.starts_with("--") && (flag.contains('c') || flag.contains('e')) {
                return rest.next().is_some_and(|code| code.contains("{{"));
            }
        }
        false
    })
}

#[cfg(test)]
mod tests {
    use super::super::{Step, StepKind};
    use super::*;

    fn flow(commands: &[&str]) -> Flow {
        let mut flow = Flow::new("f".into(), "F".into());
        flow.steps = commands
            .iter()
            .map(|command| {
                Step::new(StepKind::Exec {
                    command: command.to_string(),
                    wait: true,
                })
            })
            .collect();
        flow
    }

    fn whats(command: &str) -> Vec<&'static str> {
        command_risks(command).into_iter().map(|(_, w)| w).collect()
    }

    #[test]
    fn everyday_commands_raise_nothing() {
        for command in [
            "omarchy-launch-browser",
            "notify-send 'Done' {{name}}",
            "echo {{clipboard}} >> ~/notes.txt",
            "rm ~/Downloads/old.zip",
            "tar -czf ~/archive.tar.gz {{input}}",
            "grep -r todo ~/Notes",
            "playerctl play-pause",
            "test {{answer}} = yes",
        ] {
            assert!(whats(command).is_empty(), "{command}: {:?}", whats(command));
        }
    }

    #[test]
    fn dangerous_commands_are_named() {
        assert_eq!(whats("sudo pacman -Syu"), vec![ADMIN]);
        assert_eq!(whats("/usr/bin/sudo -n true"), vec![ADMIN]);
        assert_eq!(whats("rm -rf ~/Projects"), vec![DELETES]);
        assert_eq!(whats("cd /tmp && rm -fr build"), vec![DELETES]);
        assert_eq!(whats("rm --recursive x"), vec![DELETES]);
        assert_eq!(whats("dd if=x.iso of=/dev/sda"), vec![ERASES]);
        assert_eq!(whats("mkfs.ext4 /dev/sdb1"), vec![ERASES]);
        assert_eq!(
            whats("curl -fsSL https://example.com/i.sh | sh"),
            vec![DOWNLOADED_CODE, NETWORK]
        );
        assert_eq!(
            whats("bash <(curl -s https://example.com/i.sh)"),
            vec![DOWNLOADED_CODE, NETWORK]
        );
        assert_eq!(
            whats("wget -qO- https://example.com/i.sh | sudo bash"),
            vec![ADMIN, DOWNLOADED_CODE, NETWORK]
        );
        assert_eq!(whats("cat ~/.ssh/id_ed25519"), vec![SECRETS]);
        assert_eq!(whats("echo x >> ~/.bashrc"), vec![STARTUP]);
    }

    #[test]
    fn a_variable_run_as_code_is_named() {
        assert_eq!(whats("{{tool}} --help"), vec![VARIABLE_PROGRAM]);
        assert_eq!(whats("sh -c {{script}}"), vec![VARIABLE_CODE]);
        assert_eq!(whats("bash -lc \"{{script}}\""), vec![VARIABLE_CODE]);
        assert_eq!(whats("eval {{script}}"), vec![VARIABLE_CODE]);
        assert_eq!(
            whats("python3 -c 'print({{expression}})'"),
            vec![VARIABLE_CODE]
        );
        // After the code, a variable is an argument to it.
        assert!(whats("sh -c 'echo \"$1\"' _ {{name}}").is_empty());
        assert!(whats("python3 -c 'import sys; print(sys.argv[1].upper())' {{name}}").is_empty());
        assert!(whats("bash ~/scripts/go.sh -c {{name}}").is_empty());
        // A variable as an argument is data, which is the normal case.
        assert!(whats("sh ~/scripts/go.sh {{name}}").is_empty());
        assert!(whats("xdg-open {{url}}").is_empty());
    }

    #[test]
    fn the_network_is_a_notice_and_private_text_over_it_is_named() {
        assert_eq!(whats("curl -s https://wttr.in"), vec![NETWORK]);
        assert_eq!(
            whats("curl -d {{clipboard}} https://example.com"),
            vec![NETWORK, SENDS]
        );
        assert_eq!(command_risks("ssh host uptime")[0].0, Level::Notice);
        // Downloading a file is not running it.
        assert_eq!(
            whats("curl -o ~/a.png https://example.com/a.png"),
            vec![NETWORK]
        );
    }

    #[test]
    fn risks_carry_step_numbers_and_the_summary_lists_each_once() {
        let flow = flow(&["echo hi", "sudo true", "curl -s https://a.b", "sudo ls"]);
        let found = risks(&flow);
        assert_eq!(
            found.iter().map(|r| (r.step, r.what)).collect::<Vec<_>>(),
            vec![(2, ADMIN), (3, NETWORK), (4, ADMIN)]
        );
        assert_eq!(
            summary(&flow),
            vec![(Level::Danger, ADMIN), (Level::Notice, NETWORK)]
        );
    }

    #[test]
    fn no_built_in_template_is_dangerous() {
        for template in crate::system::flows::templates::templates() {
            if !template.is_built_in() {
                continue;
            }
            let dangerous: Vec<_> = risks(&template.flow)
                .into_iter()
                .filter(|risk| risk.level == Level::Danger)
                .collect();
            assert!(dangerous.is_empty(), "{}: {dangerous:?}", template.key);
        }
    }
}
