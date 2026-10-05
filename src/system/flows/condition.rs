//! What an `if` step checks. A condition reads the machine through
//! [`Probe`], so tests can decide what it sees.
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::vars::{self, Vars};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "check", rename_all = "snake_case")]
pub enum Condition {
    /// `value` is `to`, ignoring case and surrounding spaces.
    Equals { value: String, to: String },
    /// `value` contains `text`, ignoring case.
    Contains { value: String, text: String },
    /// `value` is empty or only spaces. A variable no step has set yet
    /// counts as empty.
    Empty { value: String },
    /// The shell command exits with status 0.
    Command { command: String },
    /// A window of this class is open.
    AppOpen { class: String },
    /// The machine runs on its battery.
    OnBattery,
    /// The time of day is from `from` up to `to` (`HH:MM`); a range that
    /// ends before it starts runs over midnight.
    TimeBetween { from: String, to: String },
}

impl Default for Condition {
    fn default() -> Self {
        Condition::Equals {
            value: String::new(),
            to: String::new(),
        }
    }
}

/// What a condition can ask about the machine.
pub trait Probe: Sync {
    /// Whether the shell command succeeds, with `env` set for it.
    fn command_succeeds(&self, command: &str, env: &[(String, String)]) -> Result<bool>;
    /// The classes of the open windows.
    fn open_classes(&self) -> Vec<String>;
    fn on_battery(&self) -> bool;
    /// Minutes since midnight, local time.
    fn minute_of_day(&self) -> u32;
}

/// The real machine.
pub struct Machine;

impl Probe for Machine {
    fn command_succeeds(&self, command: &str, env: &[(String, String)]) -> Result<bool> {
        Command::new("sh")
            .args(["-c", command])
            .envs(env.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .map_err(|e| Error::io("Could not run the command", e))
    }

    fn open_classes(&self) -> Vec<String> {
        let output = Command::new("hyprctl")
            .args(["-j", "clients"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output();
        let Ok(output) = output else {
            return Vec::new();
        };
        serde_json::from_slice::<serde_json::Value>(&output.stdout)
            .ok()
            .and_then(|clients| clients.as_array().cloned())
            .unwrap_or_default()
            .iter()
            .flat_map(|client| {
                ["class", "initialClass"]
                    .into_iter()
                    .filter_map(|key| client.get(key)?.as_str().map(str::to_string))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn on_battery(&self) -> bool {
        on_battery_in(std::path::Path::new("/sys/class/power_supply"))
    }

    fn minute_of_day(&self) -> u32 {
        use chrono::Timelike;
        let now = chrono::Local::now();
        now.hour() * 60 + now.minute()
    }
}

/// On battery means there is a battery and no mains supply is online. A
/// desktop without a battery is never on battery.
pub fn on_battery_in(dir: &std::path::Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    let read = |path: std::path::PathBuf| {
        std::fs::read_to_string(path)
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };
    let (mut battery, mut mains_online) = (false, false);
    for entry in entries.flatten() {
        let path = entry.path();
        match read(path.join("type")).as_str() {
            "Battery" => battery = true,
            "Mains" | "USB" | "USB_C" | "USB_PD" | "Wireless"
                if read(path.join("online")) == "1" =>
            {
                mains_online = true;
            }
            _ => {}
        }
    }
    battery && !mains_online
}

/// `HH:MM` as minutes since midnight.
pub fn parse_time(text: &str) -> Option<u32> {
    let (hours, minutes) = text.trim().split_once(':')?;
    let hours: u32 = hours.parse().ok()?;
    let minutes: u32 = minutes.parse().ok()?;
    (hours < 24 && minutes < 60 && minutes.to_string().len() <= 2).then_some(hours * 60 + minutes)
}

impl Condition {
    /// Every `{{name}}` the condition uses.
    pub fn references(&self) -> Vec<String> {
        match self {
            Condition::Equals { value, to } => {
                let mut names = vars::names_in(value);
                names.extend(vars::names_in(to));
                names
            }
            Condition::Contains { value, text } => {
                let mut names = vars::names_in(value);
                names.extend(vars::names_in(text));
                names
            }
            Condition::Empty { value } => vars::names_in(value),
            Condition::Command { command } => vars::names_in(command),
            Condition::AppOpen { class } => vars::names_in(class),
            Condition::OnBattery | Condition::TimeBetween { .. } => Vec::new(),
        }
    }

    /// The check in a few plain words, for the command line and logs.
    pub fn text(&self) -> String {
        match self {
            Condition::Equals { value, to } => format!("{value} is {to}"),
            Condition::Contains { value, text } => format!("{value} contains {text}"),
            Condition::Empty { value } => format!("{value} is empty"),
            Condition::Command { command } => format!("`{command}` succeeds"),
            Condition::AppOpen { class } => format!("{class} is open"),
            Condition::OnBattery => "on battery".to_string(),
            Condition::TimeBetween { from, to } => format!("the time is between {from} and {to}"),
        }
    }

    /// What is missing before the condition can be checked.
    pub fn validate(&self) -> std::result::Result<(), String> {
        match self {
            Condition::Equals { value, .. }
            | Condition::Contains { value, .. }
            | Condition::Empty { value }
                if value.trim().is_empty() =>
            {
                Err("there is nothing to check".to_string())
            }
            Condition::Contains { text, .. } if text.is_empty() => {
                Err("the text to look for is missing".to_string())
            }
            Condition::Command { command } if command.trim().is_empty() => {
                Err("the command to check is missing".to_string())
            }
            Condition::AppOpen { class } if class.trim().is_empty() => {
                Err("the app to look for is missing".to_string())
            }
            Condition::TimeBetween { from, to }
                if parse_time(from).is_none() || parse_time(to).is_none() =>
            {
                Err("times are written as 09:00".to_string())
            }
            _ => Ok(()),
        }
    }

    /// Whether the condition holds now.
    pub fn evaluate(&self, vars: &mut Vars, probe: &dyn Probe) -> Result<bool> {
        Ok(match self {
            Condition::Equals { value, to } => {
                vars.text(value)?.trim().to_lowercase() == vars.text(to)?.trim().to_lowercase()
            }
            Condition::Contains { value, text } => vars
                .text(value)?
                .to_lowercase()
                .contains(&vars.text(text)?.to_lowercase()),
            Condition::Empty { value } => vars.text_or_empty(value).trim().is_empty(),
            Condition::Command { command } => {
                let (command, env) = vars.shell(command)?;
                probe.command_succeeds(&command, &env)?
            }
            Condition::AppOpen { class } => {
                let wanted = vars.text(class)?.trim().to_lowercase();
                probe
                    .open_classes()
                    .iter()
                    .any(|open| open.to_lowercase() == wanted)
            }
            Condition::OnBattery => probe.on_battery(),
            Condition::TimeBetween { from, to } => {
                let (Some(from), Some(to)) = (parse_time(from), parse_time(to)) else {
                    return Err(Error::Invalid("Times are written as 09:00".to_string()));
                };
                let now = probe.minute_of_day();
                if from <= to {
                    (from..to).contains(&now)
                } else {
                    now >= from || now < to
                }
            }
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A machine tests describe.
    #[derive(Default)]
    pub(crate) struct Fake {
        pub classes: Vec<String>,
        pub battery: bool,
        pub minute: u32,
    }

    impl Probe for Fake {
        fn command_succeeds(&self, command: &str, env: &[(String, String)]) -> Result<bool> {
            Machine.command_succeeds(command, env)
        }

        fn open_classes(&self) -> Vec<String> {
            self.classes.clone()
        }

        fn on_battery(&self) -> bool {
            self.battery
        }

        fn minute_of_day(&self) -> u32 {
            self.minute
        }
    }

    fn holds(condition: Condition, vars: &mut Vars, probe: &Fake) -> bool {
        condition.evaluate(vars, probe).unwrap()
    }

    #[test]
    fn text_checks_ignore_case_and_outer_spaces() {
        let fake = Fake::default();
        let mut vars = Vars::new();
        vars.set("answer", "  Yes ");
        assert!(holds(
            Condition::Equals {
                value: "{{answer}}".into(),
                to: "yes".into()
            },
            &mut vars,
            &fake
        ));
        assert!(holds(
            Condition::Contains {
                value: "see HTTPS://omarchist.com".into(),
                text: "https://".into()
            },
            &mut vars,
            &fake
        ));
        assert!(!holds(
            Condition::Empty {
                value: "{{answer}}".into()
            },
            &mut vars,
            &fake
        ));
        // A name nothing has set yet is empty, not an error.
        assert!(holds(
            Condition::Empty {
                value: "{{later}}".into()
            },
            &mut vars,
            &fake
        ));
    }

    #[test]
    fn a_command_check_passes_values_as_data() {
        let fake = Fake::default();
        let mut vars = Vars::new();
        vars.set("name", "a b; exit 1");
        assert!(holds(
            Condition::Command {
                command: "test {{name}} = 'a b; exit 1'".into()
            },
            &mut vars,
            &fake
        ));
        assert!(!holds(
            Condition::Command {
                command: "false".into()
            },
            &mut vars,
            &fake
        ));
    }

    #[test]
    fn machine_checks_read_the_probe() {
        let fake = Fake {
            classes: vec!["Firefox".into(), "org.gnome.Nautilus".into()],
            battery: true,
            minute: 23 * 60 + 30,
        };
        let mut vars = Vars::new();
        assert!(holds(
            Condition::AppOpen {
                class: "firefox".into()
            },
            &mut vars,
            &fake
        ));
        assert!(!holds(
            Condition::AppOpen {
                class: "spotify".into()
            },
            &mut vars,
            &fake
        ));
        assert!(holds(Condition::OnBattery, &mut vars, &fake));
        let between = |from: &str, to: &str| Condition::TimeBetween {
            from: from.into(),
            to: to.into(),
        };
        assert!(holds(between("22:00", "06:00"), &mut vars, &fake));
        assert!(!holds(between("09:00", "17:00"), &mut vars, &fake));
        assert!(holds(between("23:00", "23:31"), &mut vars, &fake));
        assert!(!holds(between("23:00", "23:30"), &mut vars, &fake));
    }

    #[test]
    fn times_and_missing_parts_are_checked() {
        assert_eq!(parse_time("09:05"), Some(545));
        assert_eq!(parse_time(" 23:59 "), Some(1439));
        assert_eq!(parse_time("24:00"), None);
        assert_eq!(parse_time("9"), None);
        assert_eq!(parse_time("9:5x"), None);
        assert!(
            Condition::TimeBetween {
                from: "9".into(),
                to: "17:00".into()
            }
            .validate()
            .is_err()
        );
        assert!(Condition::default().validate().is_err());
        assert!(Condition::OnBattery.validate().is_ok());
    }

    #[test]
    fn on_battery_needs_a_battery_and_no_mains() {
        let dir = std::env::temp_dir().join(format!("omarchist-power-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let supply = |name: &str, kind: &str, online: Option<&str>| {
            let path = dir.join(name);
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(path.join("type"), format!("{kind}\n")).unwrap();
            if let Some(online) = online {
                std::fs::write(path.join("online"), format!("{online}\n")).unwrap();
            }
        };
        assert!(!on_battery_in(&dir), "no supplies at all");
        supply("AC", "Mains", Some("0"));
        assert!(!on_battery_in(&dir), "a desktop has no battery");
        supply("BAT0", "Battery", None);
        assert!(on_battery_in(&dir));
        supply("AC", "Mains", Some("1"));
        assert!(!on_battery_in(&dir));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
