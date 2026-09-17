//! Flows: named sequences of actions that run one after another, triggered
//! from a keybind, the app launcher, startup, or `omarchist flow run`.
//!
//! A flow is one JSON file under `~/.config/omarchist/flows/`. Its steps
//! reuse the keybind dispatcher vocabulary (`Exec` and `Lua` map onto
//! [`Dispatcher`]) and add the flow-only kinds `Wait`, `Notify`, and `Flow`.
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::system::keybinds::Dispatcher;
use crate::system::keybinds::overrides::is_dsp_call;

pub mod launcher;
pub mod runner;
pub mod store;
pub mod templates;

pub const DEFAULT_ICON: &str = "workflow";
/// Nesting deeper than this is treated as a mistake rather than run.
pub const MAX_DEPTH: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flow {
    /// Stable slug that keybinds, desktop entries, and the CLI refer to.
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// A Lucide icon name from [`ICONS`].
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default)]
    pub steps: Vec<Step>,
    #[serde(default)]
    pub on_error: OnError,
    #[serde(default)]
    pub triggers: Triggers,
}

fn default_icon() -> String {
    DEFAULT_ICON.to_string()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnError {
    /// Stop at the first failing step.
    #[default]
    Stop,
    /// Keep going and report the failures at the end.
    Continue,
}

impl OnError {
    pub const ALL: [OnError; 2] = [OnError::Stop, OnError::Continue];

    pub fn label(self) -> &'static str {
        match self {
            OnError::Stop => "Stop the flow",
            OnError::Continue => "Keep going",
        }
    }
}

/// Where a flow can be started from, besides the command line and a keybind
/// (which lives in the keybind overrides, not here).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Triggers {
    /// A `.desktop` entry, so the flow appears in the app launcher.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub launcher: bool,
    /// Omarchy's `post-boot` hook, run once per session start.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub startup: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    #[serde(flatten)]
    pub kind: StepKind,
    #[serde(default = "enabled_default", skip_serializing_if = "is_true")]
    pub enabled: bool,
}

fn enabled_default() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

impl Step {
    pub fn new(kind: StepKind) -> Self {
        Self {
            kind,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepKind {
    /// A shell command. Waited for unless `detach` is set, in which case it
    /// is started in its own session and the flow moves on.
    Exec {
        command: String,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        detach: bool,
    },
    /// A Hyprland dispatcher call, `hl.dsp.*(...)`, sent through `hyprctl`.
    Lua { expr: String },
    /// Pauses the flow.
    Wait { ms: u64 },
    /// A desktop notification.
    Notify {
        title: String,
        #[serde(default)]
        body: String,
    },
    /// Runs another flow to completion.
    Flow { id: String },
}

impl StepKind {
    /// The literal thing the step does: the command, the dispatcher
    /// expression, the pause, the notification title, or the flow id.
    pub fn text(&self) -> String {
        match self {
            StepKind::Exec { command, .. } => command.clone(),
            StepKind::Lua { expr } => expr.clone(),
            StepKind::Wait { ms } => format!("wait {}", format_duration(*ms)),
            StepKind::Notify { title, .. } => format!("notify \"{title}\""),
            StepKind::Flow { id } => run_command(id),
        }
    }

    /// The dispatcher form of an `Exec` or `Lua` step, for the action builder.
    pub fn dispatcher(&self) -> Option<Dispatcher> {
        match self {
            StepKind::Exec { command, .. } => Some(Dispatcher::Exec(command.clone())),
            StepKind::Lua { expr } => Some(Dispatcher::Lua(expr.clone())),
            _ => None,
        }
    }

    pub fn from_dispatcher(dispatcher: Dispatcher, detach: bool) -> Option<Self> {
        match dispatcher {
            Dispatcher::Exec(command) => Some(StepKind::Exec { command, detach }),
            Dispatcher::Lua(expr) => Some(StepKind::Lua { expr }),
            Dispatcher::Function => None,
        }
    }
}

impl Flow {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            icon: DEFAULT_ICON.to_string(),
            steps: Vec::new(),
            on_error: OnError::Stop,
            triggers: Triggers::default(),
        }
    }

    /// The command that runs this flow from anywhere.
    pub fn command(&self) -> String {
        run_command(&self.id)
    }

    pub fn enabled_steps(&self) -> usize {
        self.steps.iter().filter(|s| s.enabled).count()
    }

    /// Rejects what could not be run or written back safely: a malformed
    /// id, an empty name, a Lua step that is not a plain `hl.dsp.*(...)`
    /// call, an empty command, or a step that runs the flow itself.
    pub fn validate(&self) -> Result<()> {
        if !is_slug(&self.id) {
            return Err(Error::Invalid(format!("Invalid flow id '{}'", self.id)));
        }
        if self.name.trim().is_empty() {
            return Err(Error::Invalid("A flow needs a name".to_string()));
        }
        for step in &self.steps {
            match &step.kind {
                StepKind::Exec { command, .. } if command.trim().is_empty() => {
                    return Err(Error::Invalid("A command step is empty".to_string()));
                }
                StepKind::Lua { expr } if !is_dsp_call(expr) => {
                    return Err(Error::Invalid(format!("Unsupported dispatcher '{expr}'")));
                }
                StepKind::Notify { title, .. } if title.trim().is_empty() => {
                    return Err(Error::Invalid("A notification needs a title".to_string()));
                }
                StepKind::Flow { id } if id == &self.id => {
                    return Err(Error::Invalid("A flow cannot run itself".to_string()));
                }
                _ => {}
            }
        }
        Ok(())
    }
}

/// `omarchist flow run '<id>'`, quoted the way keybinds quote arguments.
pub fn run_command(id: &str) -> String {
    format!(
        "omarchist flow run {}",
        crate::system::keybinds::action::shell_quote(id)
    )
}

/// The flow id a command line runs, if it is a `run_command`.
pub fn run_command_id(command: &str) -> Option<String> {
    let words = crate::system::keybinds::action::shell_split(command);
    match words.as_slice() {
        [omarchist, flow, run, id]
            if omarchist == "omarchist" && flow == "flow" && run == "run" =>
        {
            Some(id.clone())
        }
        _ => None,
    }
}

/// "250 ms", "1.5 s", "2 s".
pub fn format_duration(ms: u64) -> String {
    if ms < 1000 {
        format!("{ms} ms")
    } else if ms.is_multiple_of(1000) {
        format!("{} s", ms / 1000)
    } else {
        format!("{:.1} s", ms as f64 / 1000.0)
    }
}

/// Lowercase ASCII letters, digits, and single hyphens between them.
pub fn is_slug(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('-')
        && !id.ends_with('-')
        && !id.contains("--")
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// A slug for a name: letters and digits kept (lowercased), runs of anything
/// else collapsed to one hyphen. Empty when the name has neither.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut pending_hyphen = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            if pending_hyphen && !out.is_empty() {
                out.push('-');
            }
            pending_hyphen = false;
            out.push(c.to_ascii_lowercase());
        } else {
            pending_hyphen = true;
        }
    }
    out
}

/// A slug for `name` that no flow in `taken` uses: the plain slug, then
/// `-2`, `-3`, and so on.
pub fn unique_id(name: &str, taken: &[String]) -> String {
    let base = match slug(name) {
        s if s.is_empty() => "flow".to_string(),
        s => s,
    };
    if !taken.iter().any(|t| t == &base) {
        return base;
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|candidate| !taken.iter().any(|t| t == candidate))
        .expect("an unused suffix exists")
}

/// Lucide icons a flow can carry, embedded under `assets/icons/`.
pub const ICONS: &[&str] = &[
    "workflow",
    "zap",
    "rocket",
    "sparkles",
    "play",
    "sun",
    "moon",
    "coffee",
    "briefcase",
    "code",
    "terminal",
    "globe",
    "monitor",
    "music",
    "headphones",
    "camera",
    "video",
    "message-square",
    "mail",
    "bell",
    "clock",
    "calendar",
    "book",
    "pen-tool",
    "palette",
    "gamepad-2",
    "heart",
    "star",
    "flame",
    "leaf",
    "house",
    "lock",
    "power",
    "wrench",
    "shield",
    "target",
];

pub fn icon_path(icon: &str) -> String {
    let icon = if ICONS.contains(&icon) {
        icon
    } else {
        DEFAULT_ICON
    };
    format!("icons/{icon}.svg")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_and_unique_ids() {
        assert_eq!(slug("Morning Start!"), "morning-start");
        assert_eq!(slug("  Déjà vu  "), "d-j-vu");
        assert_eq!(slug("***"), "");
        assert!(is_slug("focus-mode-2"));
        assert!(!is_slug("Focus"));
        assert!(!is_slug("-a"));
        assert!(!is_slug("a--b"));

        let taken = vec!["focus".to_string(), "focus-2".to_string()];
        assert_eq!(unique_id("Focus", &taken), "focus-3");
        assert_eq!(unique_id("Other", &taken), "other");
        assert_eq!(unique_id("!!!", &[]), "flow");
    }

    #[test]
    fn durations_and_step_text() {
        assert_eq!(format_duration(250), "250 ms");
        assert_eq!(format_duration(1000), "1 s");
        assert_eq!(format_duration(1500), "1.5 s");
        assert_eq!(StepKind::Wait { ms: 2000 }.text(), "wait 2 s");
        assert_eq!(
            StepKind::Flow { id: "x".into() }.text(),
            "omarchist flow run 'x'"
        );
    }

    #[test]
    fn run_command_round_trips() {
        assert_eq!(run_command("morning"), "omarchist flow run 'morning'");
        assert_eq!(
            run_command_id("omarchist flow run 'morning'").as_deref(),
            Some("morning")
        );
        assert_eq!(
            run_command_id("omarchist flow run morning").as_deref(),
            Some("morning")
        );
        assert_eq!(run_command_id("omarchist flow list"), None);
        assert_eq!(run_command_id("omarchy-launch-terminal"), None);
    }

    #[test]
    fn steps_serialize_with_a_type_tag() {
        let flow = Flow {
            steps: vec![
                Step::new(StepKind::Exec {
                    command: "omarchy-launch-browser".into(),
                    detach: false,
                }),
                Step {
                    kind: StepKind::Wait { ms: 500 },
                    enabled: false,
                },
                Step::new(StepKind::Lua {
                    expr: "hl.dsp.focus({ workspace = \"2\" })".into(),
                }),
                Step::new(StepKind::Notify {
                    title: "Ready".into(),
                    body: String::new(),
                }),
                Step::new(StepKind::Flow { id: "other".into() }),
            ],
            ..Flow::new("morning".into(), "Morning".into())
        };
        let json = serde_json::to_value(&flow).unwrap();
        assert_eq!(json["steps"][0]["type"], "exec");
        assert_eq!(json["steps"][0]["command"], "omarchy-launch-browser");
        assert!(json["steps"][0].get("enabled").is_none());
        assert!(json["steps"][0].get("detach").is_none());
        assert_eq!(json["steps"][1]["enabled"], false);
        assert_eq!(json["steps"][1]["ms"], 500);
        assert_eq!(json["icon"], "workflow");
        assert!(json.get("triggers").is_some());

        let back: Flow = serde_json::from_value(json).unwrap();
        assert_eq!(back, flow);

        let minimal: Flow = serde_json::from_str(r#"{"id":"x","name":"X"}"#).unwrap();
        assert_eq!(minimal.icon, DEFAULT_ICON);
        assert_eq!(minimal.on_error, OnError::Stop);
        assert!(minimal.steps.is_empty());
    }

    #[test]
    fn validation_rejects_unsafe_and_incomplete_flows() {
        let mut flow = Flow::new("ok".into(), "Ok".into());
        assert!(flow.validate().is_ok());

        flow.steps.push(Step::new(StepKind::Lua {
            expr: "os.execute('rm -rf ~')".into(),
        }));
        assert!(flow.validate().is_err());
        flow.steps.clear();

        flow.steps
            .push(Step::new(StepKind::Flow { id: "ok".into() }));
        assert!(flow.validate().is_err());
        flow.steps.clear();

        flow.steps.push(Step::new(StepKind::Exec {
            command: "  ".into(),
            detach: false,
        }));
        assert!(flow.validate().is_err());

        let bad_id = Flow::new("Bad Id".into(), "Bad".into());
        assert!(bad_id.validate().is_err());
        let no_name = Flow::new("ok".into(), " ".into());
        assert!(no_name.validate().is_err());
    }
}
