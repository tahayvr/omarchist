//! Flows: named sequences of actions that run one after another, triggered
//! from a keybind, the app launcher, startup, or `omarchist flow run`.
//!
//! A flow is one TOML file under `~/.config/omarchist/flows/`. Its steps
//! reuse the keybind dispatcher vocabulary (`Exec` and `Lua` map onto
//! [`Dispatcher`]) and add the flow-only kinds `Wait`, `Notify`, and `Flow`.
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::system::flows::condition::Condition;
use crate::system::keybinds::Dispatcher;
use crate::system::keybinds::overrides::is_dsp_call;
use crate::system::themes::theme_management::lifecycle::slugify_theme_name;

pub mod actions;
pub mod condition;
pub mod launcher;
pub mod prompt;
pub mod requirements;
pub mod runner;
pub mod share;
pub mod store;
pub mod templates;
pub mod vars;

pub const DEFAULT_ICON: &str = "workflow";
/// The newest flow file format this build reads. A file declaring a higher
/// one is refused; a lower one goes through [`migrate`] on read. A flow is
/// written with the lowest format that can hold it ([`Flow::required_format`]),
/// so flows that use nothing new stay readable by older builds.
pub const FORMAT: u32 = 2;
/// Format 2 added saved outputs, `{{variable}}` references, the steps
/// that ask (`ask`, `choose`, `confirm`, `pick`, a notification's click),
/// and the steps that hold other steps (`if`, `repeat`, `each`, `menu`)
/// with `stop`.
pub const FORMAT_VARIABLES: u32 = 2;
/// Nesting deeper than this is treated as a mistake rather than run.
pub const MAX_DEPTH: usize = 8;
/// How deep steps can sit inside `if`, `repeat`, `each` and `menu` steps.
pub const MAX_BLOCK_DEPTH: usize = 6;
/// The most rounds a `repeat` or `each` step runs.
pub const MAX_ROUNDS: u32 = 1000;

/// Where a step sits in a flow. A top-level step is `[index]`; a step
/// inside another adds the branch it is in and its index there, so
/// `[2, 1, 0]` is the first step of the second branch (an `if`'s
/// "otherwise") of the third step. A path of even length names a list of
/// steps instead: `[]` is the flow's own, `[2, 1]` that "otherwise".
pub type StepPath = Vec<usize>;

/// The comment at the top of every flow file Omarchist writes (saved
/// flows, exports, and the built-in templates), so a file found on its own
/// says what it is and where it is documented.
pub const FILE_HEADER: &str = "# This is an Omarchist flow: https://omarchist.com/flows/\n\n";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flow {
    /// Declared first so it is the first key of the file, under
    /// [`FILE_HEADER`].
    #[serde(default = "default_format")]
    pub format: u32,
    /// Stable slug that keybinds, desktop entries, and the CLI refer to.
    /// Absent in templates and shared files, which get one when saved.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// A Lucide icon name from [`ICONS`].
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default)]
    pub on_error: OnError,
    /// Who made the flow and what it needs; what a shared file carries.
    #[serde(default, skip_serializing_if = "Meta::is_empty")]
    pub meta: Meta,
    /// Where `{{input}}` comes from when the flow is started without any.
    #[serde(default, skip_serializing_if = "InputFallback::is_none")]
    pub input: InputFallback,
    /// Declared before `steps` so the TOML file lists it before the
    /// `[[step]]` tables rather than after them.
    #[serde(default, skip_serializing_if = "Triggers::is_empty")]
    pub triggers: Triggers,
    /// Serialized as `step`, so each `[[step]]` table in the file is one step.
    #[serde(default, rename = "step")]
    pub steps: Vec<Step>,
}

fn default_icon() -> String {
    DEFAULT_ICON.to_string()
}

fn default_format() -> u32 {
    FORMAT
}

/// Metadata about a flow rather than what it does. Every field is optional;
/// a flow written in the editor has none of them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub author: String,
    /// The author's version of the flow, compared as a plain string.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub homepage: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Programs the flow expects to find on the machine.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires: Vec<String>,
    /// The URL the flow was imported from, when it came from one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source: String,
}

impl Meta {
    pub fn is_empty(&self) -> bool {
        *self == Meta::default()
    }
}

/// Parses a flow file, refusing a newer format and migrating an older one.
pub fn parse_flow(content: &str) -> Result<Flow> {
    #[derive(Deserialize)]
    struct Header {
        #[serde(default = "default_format")]
        format: u32,
    }
    // Read the format alone first: a newer file may use keys this build does
    // not know, and the strict parse below would report those instead.
    let header: Header = toml::from_str(content)
        .map_err(|e| Error::Invalid(format!("Failed to parse flow: {e}")))?;
    if header.format > FORMAT {
        return Err(Error::Invalid(format!(
            "This flow uses format {} and needs a newer Omarchist (this one reads up to {FORMAT})",
            header.format
        )));
    }
    let flow: Flow = toml::from_str(content)
        .map_err(|e| Error::Invalid(format!("Failed to parse flow: {e}")))?;
    Ok(migrate(flow))
}

/// Brings a flow read from an older format up to [`FORMAT`]. Format 2 only
/// added fields, so this only stamps the current format; writing picks the
/// lowest format again ([`Flow::required_format`]).
fn migrate(mut flow: Flow) -> Flow {
    flow.format = FORMAT;
    flow
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

/// What `{{input}}` holds when a flow is started with nothing: no
/// arguments, no piped text, no files, no flow that hands it something.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputFallback {
    /// Nothing: `{{input}}` is empty.
    #[default]
    None,
    /// The text selected anywhere on screen.
    Selection,
    /// What is on the clipboard.
    Clipboard,
    /// What the person types when asked.
    Ask,
}

impl InputFallback {
    pub const ALL: [InputFallback; 4] = [
        InputFallback::None,
        InputFallback::Selection,
        InputFallback::Clipboard,
        InputFallback::Ask,
    ];

    pub fn label(self) -> &'static str {
        match self {
            InputFallback::None => "Nothing",
            InputFallback::Selection => "Selected text",
            InputFallback::Clipboard => "Clipboard",
            InputFallback::Ask => "Ask",
        }
    }

    pub fn is_none(&self) -> bool {
        *self == InputFallback::None
    }
}

/// Where a flow can be started from, besides the command line and a keybind
/// (which lives in the keybind overrides, not here).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Triggers {
    /// A `.desktop` entry, so the flow appears in the app launcher.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub launcher: bool,
    /// Omarchy's `post-boot` hook, run once per session start.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub startup: bool,
    /// An entry in the file manager's Scripts menu, which runs the flow
    /// with the selected files as its input.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub files: bool,
}

impl Triggers {
    pub fn is_empty(&self) -> bool {
        !self.launcher && !self.startup && !self.files
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    #[serde(flatten)]
    pub kind: StepKind,
    #[serde(default = "enabled_default", skip_serializing_if = "is_true")]
    pub enabled: bool,
    /// Saves the step's output under this name for later steps, as
    /// `{{name}}`. Only steps that produce output take one: a command the
    /// flow waits for (its stdout), a nested flow (its last output), and
    /// the steps that ask (the answer).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Tells steps apart while they are being edited, wherever they move
    /// to. Never written to the file, and not part of what makes two steps
    /// equal.
    #[serde(skip, default = "next_uid")]
    pub uid: u64,
}

impl PartialEq for Step {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.enabled == other.enabled && self.output == other.output
    }
}

impl Eq for Step {}

fn enabled_default() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

fn next_uid() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

impl Step {
    pub fn new(kind: StepKind) -> Self {
        Self {
            kind,
            enabled: true,
            output: None,
            uid: next_uid(),
        }
    }

    /// The step, saving its output under `name`.
    pub fn saving(mut self, name: &str) -> Self {
        self.output = Some(name.to_string());
        self
    }

    /// The step, turned off.
    pub fn off(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Whether the step produces output a later step can use.
    pub fn has_output(&self) -> bool {
        self.kind.has_output()
    }

    /// A copy that is a step of its own: it and the steps inside it get
    /// new identities.
    pub fn duplicate(&self) -> Self {
        let mut copy = self.clone();
        copy.renew_uids();
        copy
    }

    fn renew_uids(&mut self) {
        self.uid = next_uid();
        for branch in self.kind.branches_mut() {
            for step in branch {
                step.renew_uids();
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepKind {
    /// A shell command. Started in its own session and left to run, unless
    /// `wait` is set, in which case the flow waits for it to exit and treats
    /// a non-zero status as a failure.
    Exec {
        command: String,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        wait: bool,
    },
    /// A Hyprland dispatcher call, `hl.dsp.*(...)`, sent through `hyprctl`.
    Lua { expr: String },
    /// Pauses the flow.
    Wait { ms: u64 },
    /// A desktop notification. With `on_click`, clicking it copies or opens
    /// `target` (the message when there is no target, else the title).
    Notify {
        title: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        body: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        on_click: Option<OnClick>,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        target: String,
    },
    /// Runs another flow to completion, handing it `input` as its
    /// `{{input}}`.
    Flow {
        id: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        input: String,
    },
    /// Asks for a line of text in Omarchy's menu; the answer is the output.
    Ask { prompt: String },
    /// Offers a list in Omarchy's menu; the pick is the output. The list is
    /// `options`, or the lines of `from` when there are none.
    Choose {
        prompt: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        options: Vec<String>,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        from: String,
    },
    /// Continue or Cancel in Omarchy's menu; Cancel ends the flow quietly.
    Confirm { prompt: String },
    /// A file chooser; the path picked is the output.
    Pick {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        prompt: String,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        folder: bool,
    },
    /// Runs `then` when the condition holds (or does not, with `not`),
    /// `otherwise` when it is the other way.
    If {
        #[serde(flatten)]
        condition: Condition,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        not: bool,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        then: Vec<Step>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        otherwise: Vec<Step>,
    },
    /// Runs its steps `times` times. `{{index}}` is the round, from 1.
    Repeat {
        times: u32,
        #[serde(default, rename = "do", skip_serializing_if = "Vec::is_empty")]
        steps: Vec<Step>,
    },
    /// Runs its steps once per line of `items`, with the line as
    /// `{{item}}` and the round as `{{index}}`.
    Each {
        items: String,
        #[serde(default, rename = "do", skip_serializing_if = "Vec::is_empty")]
        steps: Vec<Step>,
    },
    /// Offers the choices in Omarchy's menu and runs the steps of the one
    /// picked. The pick's label is the output.
    Menu {
        prompt: String,
        #[serde(default, rename = "choice", skip_serializing_if = "Vec::is_empty")]
        choices: Vec<MenuChoice>,
    },
    /// Ends the flow here, as finished.
    Stop,
    /// A ready-made action from [`actions::ACTIONS`], with its fields.
    Action {
        action: String,
        #[serde(flatten)]
        args: actions::Args,
    },
}

/// One entry of a `menu` step with the steps it runs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuChoice {
    pub label: String,
    #[serde(default, rename = "do", skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<Step>,
}

/// What clicking a notification does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnClick {
    /// Puts the target on the clipboard.
    Copy,
    /// Opens the target: a link, a file, or a folder.
    Open,
}

impl OnClick {
    pub const ALL: [OnClick; 2] = [OnClick::Copy, OnClick::Open];

    pub fn label(self) -> &'static str {
        match self {
            OnClick::Copy => "Copy",
            OnClick::Open => "Open",
        }
    }
}

/// The most options a `choose` step offers; a longer list is cut.
pub const MAX_OPTIONS: usize = 500;

impl StepKind {
    /// A plain notification.
    pub fn notify(title: impl Into<String>, body: impl Into<String>) -> Self {
        StepKind::Notify {
            title: title.into(),
            body: body.into(),
            on_click: None,
            target: String::new(),
        }
    }

    /// A step that runs the flow `id` with no input of its own.
    pub fn flow(id: impl Into<String>) -> Self {
        StepKind::Flow {
            id: id.into(),
            input: String::new(),
        }
    }

    /// Every `{{name}}` the step uses, in the texts it fills in.
    pub fn references(&self) -> Vec<String> {
        match self {
            StepKind::Exec { command, .. } => vars::names_in(command),
            StepKind::Lua { expr } => vars::names_in(expr),
            StepKind::Notify {
                title,
                body,
                target,
                ..
            } => {
                let mut names = vars::names_in(title);
                names.extend(vars::names_in(body));
                names.extend(vars::names_in(target));
                names
            }
            StepKind::Ask { prompt } | StepKind::Confirm { prompt } => vars::names_in(prompt),
            StepKind::Pick { prompt, .. } => vars::names_in(prompt),
            StepKind::Choose {
                prompt,
                options,
                from,
            } => {
                let mut names = vars::names_in(prompt);
                for option in options {
                    names.extend(vars::names_in(option));
                }
                names.extend(vars::names_in(from));
                names
            }
            StepKind::If { condition, .. } => condition.references(),
            StepKind::Each { items, .. } => vars::names_in(items),
            StepKind::Menu { prompt, choices } => {
                let mut names = vars::names_in(prompt);
                for choice in choices {
                    names.extend(vars::names_in(&choice.label));
                }
                names
            }
            StepKind::Action { action, args } => actions::find(action)
                .map(|def| def.references(args))
                .unwrap_or_default(),
            StepKind::Flow { input, .. } => vars::names_in(input),
            StepKind::Wait { .. } | StepKind::Repeat { .. } | StepKind::Stop => Vec::new(),
        }
    }

    /// Whether the step produces output a later step can use: a command
    /// the flow waits for (its stdout), a nested flow (its last output),
    /// and the steps that ask (the answer).
    pub fn has_output(&self) -> bool {
        match self {
            StepKind::Exec { wait: true, .. }
            | StepKind::Flow { .. }
            | StepKind::Ask { .. }
            | StepKind::Choose { .. }
            | StepKind::Pick { .. }
            | StepKind::Menu { .. } => true,
            StepKind::Action { action, .. } => {
                actions::find(action).is_some_and(|def| def.has_output())
            }
            _ => false,
        }
    }

    /// Whether the step exists only since format 2.
    fn needs_format_2(&self) -> bool {
        match self {
            StepKind::Ask { .. }
            | StepKind::Choose { .. }
            | StepKind::Confirm { .. }
            | StepKind::Pick { .. }
            | StepKind::If { .. }
            | StepKind::Repeat { .. }
            | StepKind::Each { .. }
            | StepKind::Menu { .. }
            | StepKind::Stop
            | StepKind::Action { .. } => true,
            StepKind::Notify { on_click, .. } => on_click.is_some(),
            StepKind::Flow { input, .. } => !input.is_empty(),
            _ => false,
        }
    }

    /// Whether the step holds other steps.
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            StepKind::If { .. }
                | StepKind::Repeat { .. }
                | StepKind::Each { .. }
                | StepKind::Menu { .. }
        )
    }

    /// The lists of steps inside this one, in order: an `if`'s "then" and
    /// "otherwise", a loop's body, a menu's choices.
    pub fn branches(&self) -> Vec<&Vec<Step>> {
        match self {
            StepKind::If {
                then, otherwise, ..
            } => vec![then, otherwise],
            StepKind::Repeat { steps, .. } | StepKind::Each { steps, .. } => vec![steps],
            StepKind::Menu { choices, .. } => choices.iter().map(|c| &c.steps).collect(),
            _ => Vec::new(),
        }
    }

    pub fn branches_mut(&mut self) -> Vec<&mut Vec<Step>> {
        match self {
            StepKind::If {
                then, otherwise, ..
            } => vec![then, otherwise],
            StepKind::Repeat { steps, .. } | StepKind::Each { steps, .. } => vec![steps],
            StepKind::Menu { choices, .. } => choices.iter_mut().map(|c| &mut c.steps).collect(),
            _ => Vec::new(),
        }
    }

    /// Takes over the steps inside `old`. Editing a block in the step
    /// dialog changes what it checks or how often it runs, never what it
    /// holds. A menu's choices keep their steps by name, so removing or
    /// reordering choices never hands one choice another's steps; a
    /// renamed choice keeps the steps of the place it sits in.
    pub fn adopt_branches(&mut self, old: &StepKind) {
        if let (
            StepKind::Menu { choices, .. },
            StepKind::Menu {
                choices: before, ..
            },
        ) = (&mut *self, old)
        {
            let mut left: Vec<Option<&MenuChoice>> = before.iter().map(Some).collect();
            for choice in choices.iter_mut() {
                let same = left
                    .iter()
                    .position(|c| c.is_some_and(|c| c.label.trim() == choice.label.trim()));
                if let Some(ix) = same {
                    choice.steps = left[ix].take().map(|c| c.steps.clone()).unwrap_or_default();
                }
            }
            for (ix, choice) in choices.iter_mut().enumerate() {
                let renamed = !before.iter().any(|c| c.label.trim() == choice.label.trim());
                if renamed && let Some(old) = left.get_mut(ix).and_then(Option::take) {
                    choice.steps = old.steps.clone();
                }
            }
            return;
        }
        let old: Vec<Vec<Step>> = old.branches().into_iter().cloned().collect();
        for (branch, steps) in self.branches_mut().into_iter().zip(old) {
            *branch = steps;
        }
    }

    /// The names a loop sets for the steps inside it.
    pub fn loop_names(&self) -> &'static [&'static str] {
        match self {
            StepKind::Repeat { .. } => &["index"],
            StepKind::Each { .. } => &["item", "index"],
            _ => &[],
        }
    }

    /// The literal thing the step does: the command, the dispatcher
    /// expression, the pause, the notification title, or the flow id.
    pub fn text(&self) -> String {
        match self {
            StepKind::Exec { command, .. } => command.clone(),
            StepKind::Lua { expr } => expr.clone(),
            StepKind::Wait { ms } => format!("wait {}", format_duration(*ms)),
            StepKind::Notify { title, .. } => format!("notify \"{title}\""),
            StepKind::Flow { id, .. } => run_command(id),
            StepKind::Ask { prompt } => format!("ask \"{prompt}\""),
            StepKind::Choose {
                prompt,
                options,
                from,
            } => {
                if options.is_empty() {
                    format!("choose \"{prompt}\" from {from}")
                } else {
                    format!("choose \"{prompt}\": {}", options.join(", "))
                }
            }
            StepKind::Confirm { prompt } => format!("confirm \"{prompt}\""),
            StepKind::Pick { folder, .. } => {
                format!("pick a {}", if *folder { "folder" } else { "file" })
            }
            StepKind::If { condition, not, .. } => {
                format!("if {}{}", if *not { "not " } else { "" }, condition.text())
            }
            StepKind::Repeat { times, .. } => format!("repeat {times} times"),
            StepKind::Each { items, .. } => format!("repeat with each line of {items}"),
            StepKind::Menu { prompt, choices } => format!(
                "menu \"{prompt}\": {}",
                choices
                    .iter()
                    .map(|c| c.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            StepKind::Stop => "stop".to_string(),
            StepKind::Action { action, args } => match actions::find(action) {
                Some(def) => def.title(args),
                None => format!("action {action}"),
            },
        }
    }

    /// The dispatcher form of an `Exec`, `Lua`, or `Flow` step, which is
    /// what the action builder edits.
    pub fn dispatcher(&self) -> Option<Dispatcher> {
        match self {
            StepKind::Exec { command, .. } => Some(Dispatcher::Exec(command.clone())),
            StepKind::Lua { expr } => Some(Dispatcher::Lua(expr.clone())),
            StepKind::Flow { id, .. } => Some(Dispatcher::Exec(run_command(id))),
            _ => None,
        }
    }

    /// The inverse of [`dispatcher`](Self::dispatcher): a command that runs
    /// a flow becomes a `Flow` step, so nesting is checked in process.
    pub fn from_dispatcher(dispatcher: Dispatcher, wait: bool) -> Option<Self> {
        match dispatcher {
            Dispatcher::Exec(command) => Some(match run_command_id(&command) {
                Some(id) => StepKind::flow(id),
                None => StepKind::Exec { command, wait },
            }),
            Dispatcher::Lua(expr) => Some(StepKind::Lua { expr }),
            Dispatcher::Function => None,
        }
    }
}

impl Flow {
    pub fn new(id: String, name: String) -> Self {
        Self {
            format: FORMAT,
            id,
            name,
            description: String::new(),
            icon: DEFAULT_ICON.to_string(),
            on_error: OnError::Stop,
            meta: Meta::default(),
            input: InputFallback::None,
            triggers: Triggers::default(),
            steps: Vec::new(),
        }
    }

    /// The flow as a file, in the layout the store and exports use, under
    /// a comment that says what the file is.
    pub fn to_toml(&self) -> Result<String> {
        let mut flow = self.clone();
        flow.format = self.required_format();
        let body = toml::to_string_pretty(&flow)
            .map_err(|e| Error::Invalid(format!("Failed to serialize flow: {e}")))?;
        Ok(format!("{FILE_HEADER}{body}"))
    }

    /// The lowest format that holds this flow: 2 once a step saves an
    /// output, uses a variable, or is one of the kinds format 2 added, 1
    /// otherwise.
    pub fn required_format(&self) -> u32 {
        let uses_format_2 = self.walk().iter().any(|(_, s)| {
            s.output.is_some() || !s.kind.references().is_empty() || s.kind.needs_format_2()
        });
        if uses_format_2 || !self.input.is_none() || self.triggers.files {
            FORMAT_VARIABLES
        } else {
            1
        }
    }

    /// Every step, including the ones inside other steps, in the order
    /// they are written, each with its path.
    pub fn walk(&self) -> Vec<(StepPath, &Step)> {
        walk(&self.steps)
    }

    pub fn step_at(&self, path: &[usize]) -> Option<&Step> {
        step_at(&self.steps, path)
    }

    /// The number messages and the editor show for the step at `path`:
    /// its place among all steps as written, counting from 1.
    pub fn step_number(&self, path: &[usize]) -> usize {
        self.walk()
            .iter()
            .position(|(p, _)| p == path)
            .map_or(0, |ix| ix + 1)
    }

    /// The names a step at position `index` of the list `list` can use:
    /// what the steps written before that place save, in order and
    /// without repeats, then what the loops around it set.
    pub fn names_at(&self, list: &[usize], index: usize) -> Vec<String> {
        let mut slot = list.to_vec();
        slot.push(index);
        let mut names: Vec<String> = Vec::new();
        let mut add = |name: String| {
            if !names.contains(&name) {
                names.push(name);
            }
        };
        for (path, step) in self.walk() {
            if path < slot
                && let Some(name) = &step.output
            {
                add(vars::normalize(name));
            }
        }
        // Each pair of the list's path is a step and the branch inside it.
        for depth in (2..=list.len()).step_by(2) {
            if let Some(step) = self.step_at(&list[..depth - 1]) {
                for name in step.kind.loop_names() {
                    add(name.to_string());
                }
            }
        }
        names
    }

    /// Names saved by the top-level steps before `index` and the steps
    /// inside them.
    pub fn outputs_before(&self, index: usize) -> Vec<String> {
        self.names_at(&[], index)
    }

    /// The command that runs this flow from anywhere.
    pub fn command(&self) -> String {
        run_command(&self.id)
    }

    /// Steps that are on, counting the ones inside other steps unless the
    /// step around them is off.
    pub fn enabled_steps(&self) -> usize {
        fn count(steps: &[Step]) -> usize {
            steps
                .iter()
                .filter(|s| s.enabled)
                .map(|s| {
                    1 + s
                        .kind
                        .branches()
                        .into_iter()
                        .map(|b| count(b))
                        .sum::<usize>()
                })
                .sum()
        }
        count(&self.steps)
    }

    /// Every step, including the ones inside other steps.
    pub fn step_count(&self) -> usize {
        self.walk().len()
    }

    /// Rejects what could not be run or written back safely: a malformed
    /// id, an empty name, a Lua step that is not a plain `hl.dsp.*(...)`
    /// call, an empty command, or a step that runs the flow itself.
    pub fn validate(&self) -> Result<()> {
        if !is_slug(&self.id) {
            return Err(Error::Invalid(format!("Invalid flow id '{}'", self.id)));
        }
        self.validate_content()
    }

    /// [`validate`](Self::validate) without the id check, for templates and
    /// shared files, which have no id yet.
    pub fn validate_content(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::Invalid("A flow needs a name".to_string()));
        }
        if self.name.chars().any(char::is_control) {
            return Err(Error::Invalid(
                "A flow's name cannot contain line breaks".to_string(),
            ));
        }
        let mut scope = Scope::default();
        self.validate_steps(&self.steps, 0, &mut scope)
    }

    fn validate_steps(&self, steps: &[Step], depth: usize, scope: &mut Scope) -> Result<()> {
        for step in steps {
            scope.number += 1;
            let number = scope.number;
            let fail = |message: String| -> Result<()> {
                Err(Error::Invalid(format!("Step {number}: {message}")))
            };
            if let Some(name) = &step.output {
                if !vars::is_name(name) {
                    return fail(format!(
                        "'{name}' cannot be a variable name (a letter, then letters, digits, \
                         spaces, - or _)"
                    ));
                }
                if vars::is_reserved(name) {
                    return fail(format!(
                        "{{{{{}}}}} is set by Omarchist; pick another name",
                        vars::normalize(name)
                    ));
                }
                if !step.has_output() {
                    return fail(
                        "this step has no output to save (a command needs `wait = true`)"
                            .to_string(),
                    );
                }
            }
            for name in step.kind.references() {
                if !vars::is_builtin(&name) && !scope.knows(&name) {
                    return Err(Error::Invalid(format!(
                        "Step {number} uses {{{{{name}}}}}, which no earlier step saves"
                    )));
                }
            }
            if let StepKind::Lua { expr } = &step.kind
                && !step.kind.references().is_empty()
            {
                // The guard below checks the text as written; a reference must
                // also sit inside a string so its value is escaped there.
                let mut probe = vars::Vars::new();
                for name in step.kind.references() {
                    probe.set(&name, "x");
                }
                if let Err(e) = probe.lua(expr) {
                    return fail(e.to_string());
                }
            }
            match &step.kind {
                StepKind::Exec { command, .. } if command.trim().is_empty() => {
                    return fail("the command is empty".to_string());
                }
                StepKind::Lua { expr } if !is_dsp_call(expr) => {
                    return fail(format!("unsupported dispatcher '{expr}'"));
                }
                StepKind::Notify { title, .. } if title.trim().is_empty() => {
                    return fail("a notification needs a title".to_string());
                }
                StepKind::Flow { id, .. } if id.is_empty() => {
                    return fail("pick the flow to run".to_string());
                }
                StepKind::Flow { id, .. } if id == &self.id => {
                    return fail("a flow cannot run itself".to_string());
                }
                StepKind::Ask { prompt } | StepKind::Confirm { prompt }
                    if prompt.trim().is_empty() =>
                {
                    return fail("a question is missing".to_string());
                }
                StepKind::Choose {
                    prompt,
                    options,
                    from,
                } => {
                    if prompt.trim().is_empty() {
                        return fail("a question is missing".to_string());
                    }
                    let listed = options.iter().any(|o| !o.trim().is_empty());
                    if !listed && from.trim().is_empty() {
                        return fail("there is nothing to choose from".to_string());
                    }
                    if options.len() > MAX_OPTIONS {
                        return fail(format!("at most {MAX_OPTIONS} options"));
                    }
                }
                StepKind::If { condition, .. } => {
                    if let Err(message) = condition.validate() {
                        return fail(message);
                    }
                }
                StepKind::Repeat { times, .. } if *times == 0 || *times > MAX_ROUNDS => {
                    return fail(format!("repeat between 1 and {MAX_ROUNDS} times"));
                }
                StepKind::Each { items, .. } if items.trim().is_empty() => {
                    return fail("there is nothing to repeat with".to_string());
                }
                StepKind::Action { action, args } => match actions::find(action) {
                    Some(def) => {
                        if let Err(message) = def.validate(args) {
                            return fail(message);
                        }
                    }
                    None => {
                        return fail(format!(
                            "this Omarchist has no action '{action}'; a newer one may"
                        ));
                    }
                },
                StepKind::Menu { prompt, choices } => {
                    if prompt.trim().is_empty() {
                        return fail("a question is missing".to_string());
                    }
                    if choices.is_empty() {
                        return fail("a menu needs at least one choice".to_string());
                    }
                    if choices.len() > MAX_CHOICES {
                        return fail(format!("at most {MAX_CHOICES} choices"));
                    }
                    for (ix, choice) in choices.iter().enumerate() {
                        let label = choice.label.trim();
                        if label.is_empty() {
                            return fail("a choice has no name".to_string());
                        }
                        if choices[..ix].iter().any(|c| c.label.trim() == label) {
                            return fail(format!("'{label}' is there twice"));
                        }
                    }
                }
                _ => {}
            }
            if step.kind.is_block() {
                if depth >= MAX_BLOCK_DEPTH {
                    return fail(format!("steps are nested more than {MAX_BLOCK_DEPTH} deep"));
                }
                // What a menu's pick is called is known to its branches.
                if let (StepKind::Menu { .. }, Some(name)) = (&step.kind, &step.output) {
                    scope.save(name);
                }
                let loops = step.kind.loop_names();
                scope.loops.extend(loops);
                for branch in step.kind.branches() {
                    self.validate_steps(branch, depth + 1, scope)?;
                }
                scope.loops.truncate(scope.loops.len() - loops.len());
            }
            if let Some(name) = &step.output {
                scope.save(name);
            }
        }
        Ok(())
    }
}

/// What validation knows at one point of a flow, walking it as written.
#[derive(Default)]
struct Scope {
    /// How many steps have been seen; the next one's number minus one.
    number: usize,
    /// Names saved so far.
    saved: Vec<String>,
    /// Names set by the loops around this point.
    loops: Vec<&'static str>,
}

impl Scope {
    fn knows(&self, name: &str) -> bool {
        self.saved.iter().any(|n| n == name) || self.loops.contains(&name)
    }

    fn save(&mut self, name: &str) {
        let name = vars::normalize(name);
        if !self.saved.contains(&name) {
            self.saved.push(name);
        }
    }
}

/// The most choices a `menu` step offers.
pub const MAX_CHOICES: usize = 30;

/// Every step of `steps` and the steps inside them, in the order they are
/// written, each with its path.
pub fn walk(steps: &[Step]) -> Vec<(StepPath, &Step)> {
    fn visit<'a>(steps: &'a [Step], base: &[usize], out: &mut Vec<(StepPath, &'a Step)>) {
        for (index, step) in steps.iter().enumerate() {
            let mut path = base.to_vec();
            path.push(index);
            out.push((path.clone(), step));
            for (branch, inner) in step.kind.branches().into_iter().enumerate() {
                let mut list = path.clone();
                list.push(branch);
                visit(inner, &list, out);
            }
        }
    }
    let mut out = Vec::new();
    visit(steps, &[], &mut out);
    out
}

/// The list of steps a list path names: `[]` is `steps` itself, `[2, 1]`
/// the second branch of the third step.
pub fn list_at<'a>(steps: &'a Vec<Step>, list: &[usize]) -> Option<&'a Vec<Step>> {
    match list {
        [] => Some(steps),
        [index, branch, rest @ ..] => {
            let inner = steps
                .get(*index)?
                .kind
                .branches()
                .into_iter()
                .nth(*branch)?;
            list_at(inner, rest)
        }
        [_] => None,
    }
}

pub fn list_at_mut<'a>(steps: &'a mut Vec<Step>, list: &[usize]) -> Option<&'a mut Vec<Step>> {
    match list {
        [] => Some(steps),
        [index, branch, rest @ ..] => {
            let inner = steps
                .get_mut(*index)?
                .kind
                .branches_mut()
                .into_iter()
                .nth(*branch)?;
            list_at_mut(inner, rest)
        }
        [_] => None,
    }
}

pub fn step_at<'a>(steps: &'a [Step], path: &[usize]) -> Option<&'a Step> {
    let (index, list) = path.split_last()?;
    match list {
        [] => steps.get(*index),
        [first, branch, rest @ ..] => {
            let inner = steps
                .get(*first)?
                .kind
                .branches()
                .into_iter()
                .nth(*branch)?;
            let mut inner_path = rest.to_vec();
            inner_path.push(*index);
            step_at(inner, &inner_path)
        }
        [_] => None,
    }
}

pub fn step_at_mut<'a>(steps: &'a mut Vec<Step>, path: &[usize]) -> Option<&'a mut Step> {
    let (index, list) = path.split_last()?;
    list_at_mut(steps, list)?.get_mut(*index)
}

/// Every place a step can sit, in the order the flow is written: before
/// each step of a list, inside each branch of a step that holds steps,
/// and at the end of the list. Each is a list path and an index in it.
fn slots(steps: &[Step], list: &[usize], out: &mut Vec<(StepPath, usize)>) {
    for (index, step) in steps.iter().enumerate() {
        out.push((list.to_vec(), index));
        for (branch, inner) in step.kind.branches().into_iter().enumerate() {
            let mut inner_list = list.to_vec();
            inner_list.extend([index, branch]);
            slots(inner, &inner_list, out);
        }
    }
    out.push((list.to_vec(), steps.len()));
}

/// Moves the step at `path` one place down or up through the flow as it
/// is written: past the step next to it, into a block it meets, or out of
/// the block it is in. Returns where it is now, or `None` at either end.
pub fn move_step(steps: &mut Vec<Step>, path: &[usize], down: bool) -> Option<StepPath> {
    let (index, list) = path.split_last()?;
    let step = {
        let from = list_at_mut(steps, list)?;
        if *index >= from.len() {
            return None;
        }
        from.remove(*index)
    };
    let mut places = Vec::new();
    slots(steps, &[], &mut places);
    let here = places
        .iter()
        .position(|(l, i)| l.as_slice() == list && i == index);
    let target = here.and_then(|here| {
        if down {
            places.get(here + 1)
        } else {
            here.checked_sub(1).and_then(|ix| places.get(ix))
        }
    });
    let (target_list, target_index, moved) = match target {
        Some((l, i)) => (l.clone(), *i, true),
        None => (list.to_vec(), *index, false),
    };
    let into = list_at_mut(steps, &target_list).expect("a place that was just listed");
    into.insert(target_index.min(into.len()), step);
    moved.then(|| {
        let mut path = target_list;
        path.push(target_index);
        path
    })
}

/// `omarchist flow run <id>`. Ids are slugs, so the command needs no
/// quoting and works as a desktop entry's `Exec` line, which only defines
/// double quotes, as well as in a shell.
pub fn run_command(id: &str) -> String {
    format!("omarchist flow run {id}")
}

/// The flow id a command line runs, if it is a `run_command`.
pub fn run_command_id(command: &str) -> Option<String> {
    let words = crate::system::keybinds::action::shell_split(command);
    let words = match words.as_slice() {
        // The launcher's form of the same command.
        [uwsm, dashes, rest @ ..] if uwsm == "uwsm-app" && dashes == "--" => rest,
        rest => rest,
    };
    match words {
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

/// The slug themes use for their directories: letters and digits kept
/// (lowercased), runs of anything else collapsed to one hyphen, and empty
/// when the name has neither.
fn slug(name: &str) -> String {
    let slug = slugify_theme_name(name);
    if slug == "custom-theme" && !name.to_lowercase().contains("custom") {
        String::new()
    } else {
        slug
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

/// The Material Design Nerd Font glyph for each of [`ICONS`], for places
/// that draw with the bar's font (the bar widget) so the theme colors them.
const ICON_GLYPHS: &[(&str, char)] = &[
    ("workflow", '\u{f04aa}'),
    ("zap", '\u{f140b}'),
    ("rocket", '\u{f14de}'),
    ("sparkles", '\u{f0674}'),
    ("play", '\u{f040a}'),
    ("sun", '\u{f0599}'),
    ("moon", '\u{f0594}'),
    ("coffee", '\u{f0176}'),
    ("briefcase", '\u{f00d6}'),
    ("code", '\u{f0174}'),
    ("terminal", '\u{f018d}'),
    ("globe", '\u{f059f}'),
    ("monitor", '\u{f0379}'),
    ("music", '\u{f075a}'),
    ("headphones", '\u{f02cb}'),
    ("camera", '\u{f0100}'),
    ("video", '\u{f0567}'),
    ("message-square", '\u{f0361}'),
    ("mail", '\u{f01ee}'),
    ("bell", '\u{f009a}'),
    ("clock", '\u{f0150}'),
    ("calendar", '\u{f00ed}'),
    ("book", '\u{f14f7}'),
    ("pen-tool", '\u{f0d13}'),
    ("palette", '\u{f03d8}'),
    ("gamepad-2", '\u{f0297}'),
    ("heart", '\u{f02d1}'),
    ("star", '\u{f04ce}'),
    ("flame", '\u{f0238}'),
    ("leaf", '\u{f032a}'),
    ("house", '\u{f02dc}'),
    ("lock", '\u{f033e}'),
    ("power", '\u{f0425}'),
    ("wrench", '\u{f05b7}'),
    ("shield", '\u{f0498}'),
    ("target", '\u{f04fe}'),
];

/// The Nerd Font glyph for a flow icon, the default icon's for an unknown one.
pub fn icon_glyph(icon: &str) -> char {
    let find = |name: &str| {
        ICON_GLYPHS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, g)| *g)
    };
    find(icon)
        .or_else(|| find(DEFAULT_ICON))
        .unwrap_or('\u{f04aa}')
}

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

    #[test]
    fn every_icon_is_embedded() {
        for icon in super::ICONS {
            assert!(
                crate::assets::OmarchistAssets::get(&format!("icons/{icon}.svg")).is_some(),
                "{icon}"
            );
        }
    }

    #[test]
    fn every_icon_has_a_glyph() {
        for icon in super::ICONS {
            assert!(
                super::ICON_GLYPHS.iter().any(|(name, _)| name == icon),
                "{icon} has no Nerd Font glyph"
            );
        }
    }

    use super::*;

    #[test]
    fn slugs_and_unique_ids() {
        assert_eq!(slug("Morning Start!"), "morning-start");
        assert_eq!(slug("  Déjà vu  "), "d-j-vu");
        assert_eq!(slug("***"), "");
        assert_eq!(slug("Custom"), "custom");
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
        assert_eq!(StepKind::flow("x").text(), "omarchist flow run x");
    }

    #[test]
    fn flow_commands_become_flow_steps() {
        let step =
            StepKind::from_dispatcher(Dispatcher::Exec("omarchist flow run 'other'".into()), false);
        assert_eq!(step, Some(StepKind::flow("other")));
        assert_eq!(
            StepKind::flow("other").dispatcher(),
            Some(Dispatcher::Exec("omarchist flow run other".into()))
        );
        assert_eq!(
            StepKind::from_dispatcher(Dispatcher::Exec("ls".into()), true),
            Some(StepKind::Exec {
                command: "ls".into(),
                wait: true
            })
        );
    }

    #[test]
    fn run_command_round_trips() {
        assert_eq!(run_command("morning"), "omarchist flow run morning");
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
        // The launcher entry's Exec line, as the App picker would hand it in.
        assert_eq!(
            run_command_id("uwsm-app -- omarchist flow run morning").as_deref(),
            Some("morning")
        );
    }

    #[test]
    fn flows_round_trip_through_toml() {
        let mut flow = Flow::new("focus-mode".into(), "Focus mode".into());
        flow.triggers.launcher = true;
        flow.steps = vec![
            Step::new(StepKind::Lua {
                expr: "hl.dsp.focus({ workspace = \"2\" })".into(),
            }),
            Step::new(StepKind::Wait { ms: 500 }).off(),
            Step::new(StepKind::Exec {
                command: "omarchy-launch-editor".into(),
                wait: true,
            }),
        ];
        let text = toml::to_string_pretty(&flow).unwrap();
        assert!(
            text.contains("[[step]]\ntype = \"lua\"\nexpr = 'hl.dsp.focus({ workspace = \"2\" })'")
        );
        assert!(text.contains("type = \"wait\"\nms = 500\nenabled = false"));
        assert!(text.contains("[triggers]\nlauncher = true\n\n[[step]]"));
        let back: Flow = toml::from_str(&text).unwrap();
        assert_eq!(back, flow);

        let minimal: Flow = toml::from_str("id = \"x\"\nname = \"X\"\n").unwrap();
        assert_eq!(minimal.icon, DEFAULT_ICON);
        assert!(minimal.steps.is_empty());
        assert_eq!(minimal.on_error, OnError::Stop);
    }

    #[test]
    fn steps_serialize_with_a_type_tag() {
        let flow = Flow {
            steps: vec![
                Step::new(StepKind::Exec {
                    command: "omarchy-launch-browser".into(),
                    wait: false,
                }),
                Step::new(StepKind::Wait { ms: 500 }).off(),
                Step::new(StepKind::Lua {
                    expr: "hl.dsp.focus({ workspace = \"2\" })".into(),
                }),
                Step::new(StepKind::notify("Ready", "")),
                Step::new(StepKind::flow("other")),
            ],
            ..Flow::new("morning".into(), "Morning".into())
        };
        let json = serde_json::to_value(&flow).unwrap();
        assert_eq!(json["step"][0]["type"], "exec");
        assert_eq!(json["step"][0]["command"], "omarchy-launch-browser");
        assert!(json["step"][0].get("enabled").is_none());
        assert!(json["step"][0].get("wait").is_none());
        assert_eq!(json["step"][1]["enabled"], false);
        assert_eq!(json["step"][1]["ms"], 500);
        assert_eq!(json["icon"], "workflow");
        assert!(json.get("triggers").is_none(), "no triggers, no table");

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

        flow.steps.push(Step::new(StepKind::flow("ok")));
        assert!(flow.validate().is_err());
        flow.steps.clear();

        flow.steps.push(Step::new(StepKind::Exec {
            command: "  ".into(),
            wait: false,
        }));
        assert!(flow.validate().is_err());
        flow.steps.clear();

        flow.name = "Two\nlines".into();
        assert!(flow.validate().is_err());
        flow.name = "Ok".into();

        assert!(
            toml::from_str::<Flow>(
                "id = \"x\"\nname = \"X\"\n[[steps]]\ntype = \"wait\"\nms = 1\n"
            )
            .is_err(),
            "a misspelled table must not load as an empty flow"
        );

        let bad_id = Flow::new("Bad Id".into(), "Bad".into());
        assert!(bad_id.validate().is_err());
        let no_name = Flow::new("ok".into(), " ".into());
        assert!(no_name.validate().is_err());
    }
}

#[cfg(test)]
mod variable_tests {
    use super::{Flow, Step, StepKind, parse_flow};

    fn command(text: &str, wait: bool) -> Step {
        Step::new(StepKind::Exec {
            command: text.into(),
            wait,
        })
    }

    fn saving(step: Step, name: &str) -> Step {
        Step {
            output: Some(name.into()),
            ..step
        }
    }

    fn flow(steps: Vec<Step>) -> Flow {
        let mut flow = Flow::new("f".into(), "F".into());
        flow.steps = steps;
        flow
    }

    #[test]
    fn a_reference_needs_an_earlier_step_or_a_builtin() {
        let ok = flow(vec![
            saving(command("echo hi", true), "greeting"),
            command("notify-send {{greeting}} {{date}}", false),
        ]);
        assert!(ok.validate().is_ok(), "{:?}", ok.validate());

        let later = flow(vec![
            command("echo {{greeting}}", false),
            saving(command("echo hi", true), "greeting"),
        ]);
        let error = later.validate().unwrap_err().to_string();
        assert!(
            error.contains("Step 1") && error.contains("{{greeting}}"),
            "{error}"
        );
    }

    #[test]
    fn only_steps_with_output_can_save_it_and_names_are_checked() {
        assert!(
            flow(vec![saving(command("echo hi", false), "x")])
                .validate()
                .is_err()
        );
        assert!(
            flow(vec![saving(command("echo hi", true), "2x")])
                .validate()
                .is_err()
        );
        assert!(
            flow(vec![saving(command("echo hi", true), "clipboard")])
                .validate()
                .is_err()
        );
        assert!(
            flow(vec![saving(Step::new(StepKind::flow("other")), "x")])
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn a_hyprland_reference_must_sit_inside_a_string() {
        let quoted = flow(vec![Step::new(StepKind::Lua {
            expr: "hl.dsp.focus({ workspace = \"{{workspace}}\" })".into(),
        })]);
        assert!(quoted.validate().is_ok(), "{:?}", quoted.validate());
        let bare = flow(vec![Step::new(StepKind::Lua {
            expr: "hl.dsp.focus({ workspace = {{workspace}} })".into(),
        })]);
        assert!(bare.validate().is_err());
    }

    #[test]
    fn a_flow_without_variables_stays_format_1() {
        let plain = flow(vec![command("echo hi", false)]);
        assert!(plain.to_toml().unwrap().contains("format = 1\n"));
        let with = flow(vec![saving(command("echo hi", true), "x")]);
        let text = with.to_toml().unwrap();
        assert!(text.contains("format = 2\n"), "{text}");
        assert!(text.contains("output = \"x\""), "{text}");
        assert_eq!(
            parse_flow(&text).unwrap().steps[0].output.as_deref(),
            Some("x")
        );
    }
}

#[cfg(test)]
mod file_tests {
    use super::{FILE_HEADER, FORMAT, Flow, Meta, StepKind, parse_flow};

    #[test]
    fn a_file_without_a_format_is_the_current_format() {
        let flow = parse_flow("name = \"Old\"\n").unwrap();
        assert_eq!(flow.format, FORMAT);
        assert!(flow.id.is_empty());
    }

    #[test]
    fn a_newer_format_is_refused_before_its_keys_are_checked() {
        let error = parse_flow("format = 99\nname = \"Future\"\nnovelty = true\n")
            .unwrap_err()
            .to_string();
        assert!(error.contains("newer Omarchist"), "{error}");
    }

    #[test]
    fn an_unknown_key_at_the_current_format_is_an_error() {
        assert!(parse_flow("format = 1\nname = \"X\"\nnovelty = true\n").is_err());
    }

    #[test]
    fn metadata_round_trips_and_is_omitted_when_empty() {
        let mut flow = Flow::new("demo".into(), "Demo".into());
        let plain = flow.to_toml().unwrap();
        let body = plain
            .strip_prefix(FILE_HEADER)
            .expect("the header comes first");
        assert!(body.starts_with("format = 1\n"), "{plain}");
        assert!(!plain.contains("[meta]"));

        flow.meta = Meta {
            author: "Taha".into(),
            version: "1.2".into(),
            tags: vec!["morning".into()],
            requires: vec!["spotify".into()],
            ..Meta::default()
        };
        flow.steps.push(super::Step::new(StepKind::Wait { ms: 10 }));
        let text = flow.to_toml().unwrap();
        assert!(text.contains("[meta]"), "{text}");
        assert!(text.find("[meta]").unwrap() < text.find("[[step]]").unwrap());
        assert_eq!(parse_flow(&text).unwrap(), flow);
    }
}

#[cfg(test)]
mod block_tests {
    use super::condition::Condition;
    use super::{
        Flow, MAX_BLOCK_DEPTH, MenuChoice, Step, StepKind, list_at, move_step, parse_flow, step_at,
    };

    fn wait(ms: u64) -> Step {
        Step::new(StepKind::Wait { ms })
    }

    fn command(text: &str) -> Step {
        Step::new(StepKind::Exec {
            command: text.into(),
            wait: true,
        })
    }

    fn when(condition: Condition, then: Vec<Step>, otherwise: Vec<Step>) -> Step {
        Step::new(StepKind::If {
            condition,
            not: false,
            then,
            otherwise,
        })
    }

    fn repeat(times: u32, steps: Vec<Step>) -> Step {
        Step::new(StepKind::Repeat { times, steps })
    }

    fn each(items: &str, steps: Vec<Step>) -> Step {
        Step::new(StepKind::Each {
            items: items.into(),
            steps,
        })
    }

    fn menu(choices: Vec<(&str, Vec<Step>)>) -> Step {
        Step::new(StepKind::Menu {
            prompt: "Pick".into(),
            choices: choices
                .into_iter()
                .map(|(label, steps)| MenuChoice {
                    label: label.into(),
                    steps,
                })
                .collect(),
        })
    }

    fn flow(steps: Vec<Step>) -> Flow {
        let mut flow = Flow::new("f".into(), "F".into());
        flow.steps = steps;
        flow
    }

    fn waits(steps: &[Step]) -> Vec<u64> {
        super::walk(steps)
            .into_iter()
            .filter_map(|(_, s)| match s.kind {
                StepKind::Wait { ms } => Some(ms),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn nested_steps_are_tables_under_their_step() {
        let nested = flow(vec![
            when(
                Condition::Contains {
                    value: "{{clipboard}}".into(),
                    text: "https://".into(),
                },
                vec![command("xdg-open {{clipboard}}")],
                vec![
                    Step::new(StepKind::notify("No link", "")),
                    Step::new(StepKind::Stop),
                ],
            ),
            repeat(2, vec![wait(5)]),
            each("{{clipboard}}", vec![command("echo {{item}} {{index}}")]),
            menu(vec![
                ("Lock", vec![command("omarchy-system-lock")]),
                ("Nothing", vec![]),
            ])
            .saving("pick"),
        ]);
        assert!(nested.validate().is_ok(), "{:?}", nested.validate());
        let text = nested.to_toml().unwrap();
        for expected in [
            "format = 2\n",
            "[[step]]\ntype = \"if\"\ncheck = \"contains\"\nvalue = \"{{clipboard}}\"\ntext = \"https://\"\n",
            "[[step.then]]\ntype = \"exec\"\n",
            "[[step.otherwise]]\ntype = \"notify\"\n",
            "[[step.otherwise]]\ntype = \"stop\"\n",
            "[[step]]\ntype = \"repeat\"\ntimes = 2\n",
            "[[step.do]]\ntype = \"wait\"\nms = 5\n",
            "[[step]]\ntype = \"each\"\nitems = \"{{clipboard}}\"\n",
            "[[step]]\ntype = \"menu\"\nprompt = \"Pick\"\noutput = \"pick\"\n",
            "[[step.choice]]\nlabel = \"Lock\"\n",
            "[[step.choice.do]]\ntype = \"exec\"\n",
            "[[step.choice]]\nlabel = \"Nothing\"\n",
        ] {
            assert!(text.contains(expected), "missing {expected:?} in:\n{text}");
        }
        assert_eq!(parse_flow(&text).unwrap().steps, nested.steps);
    }

    #[test]
    fn a_negated_check_and_a_keyless_one_round_trip() {
        let mut step = when(Condition::OnBattery, vec![wait(1)], vec![]);
        if let StepKind::If { not, .. } = &mut step.kind {
            *not = true;
        }
        let text = flow(vec![step.clone()]).to_toml().unwrap();
        assert!(
            text.contains("type = \"if\"\ncheck = \"on_battery\"\nnot = true\n"),
            "{text}"
        );
        assert_eq!(parse_flow(&text).unwrap().steps, vec![step]);
    }

    #[test]
    fn steps_are_numbered_and_found_as_written() {
        let nested = flow(vec![
            wait(1),
            when(
                Condition::OnBattery,
                vec![wait(2), repeat(2, vec![wait(3)])],
                vec![wait(4)],
            ),
            wait(5),
        ]);
        assert_eq!(waits(&nested.steps), vec![1, 2, 3, 4, 5]);
        assert_eq!(nested.step_count(), 7);
        assert_eq!(nested.step_number(&[0]), 1);
        assert_eq!(nested.step_number(&[1, 0, 1, 0, 0]), 5);
        assert_eq!(nested.step_number(&[1, 1, 0]), 6);
        assert_eq!(nested.step_number(&[2]), 7);
        assert_eq!(nested.step_number(&[9]), 0);
        assert_eq!(
            step_at(&nested.steps, &[1, 0, 1, 0, 0]).map(|s| &s.kind),
            Some(&StepKind::Wait { ms: 3 })
        );
        assert!(step_at(&nested.steps, &[1, 2, 0]).is_none());
        assert_eq!(list_at(&nested.steps, &[1, 1]).map(Vec::len), Some(1));
        assert!(
            list_at(&nested.steps, &[0, 0]).is_none(),
            "a wait holds no steps"
        );
    }

    #[test]
    fn enabled_steps_leave_out_what_a_switched_off_block_holds() {
        let mut nested = flow(vec![wait(1), repeat(2, vec![wait(2), wait(3).off()])]);
        assert_eq!(nested.enabled_steps(), 3);
        nested.steps[1].enabled = false;
        assert_eq!(nested.enabled_steps(), 1);
    }

    #[test]
    fn names_follow_the_order_steps_are_written_in() {
        let nested = flow(vec![
            command("echo a").saving("first"),
            when(
                Condition::OnBattery,
                vec![command("echo b").saving("inside")],
                vec![],
            ),
            each("{{first}}", vec![wait(1)]),
            command("echo c").saving("last"),
        ]);
        assert_eq!(nested.names_at(&[], 0), Vec::<String>::new());
        assert_eq!(nested.names_at(&[], 1), vec!["first"]);
        // Inside the If, before and after its own step.
        assert_eq!(nested.names_at(&[1, 0], 0), vec!["first"]);
        assert_eq!(nested.names_at(&[1, 0], 1), vec!["first", "inside"]);
        // The other branch is written after the first one.
        assert_eq!(nested.names_at(&[1, 1], 0), vec!["first", "inside"]);
        // Inside the loop, the loop's own names are there too.
        assert_eq!(
            nested.names_at(&[2, 0], 0),
            vec!["first", "inside", "item", "index"]
        );
        assert_eq!(nested.names_at(&[], 3), vec!["first", "inside"]);
        assert_eq!(nested.outputs_before(4), vec!["first", "inside", "last"]);
    }

    #[test]
    fn loop_names_only_exist_inside_their_loop() {
        let inside = flow(vec![each("a", vec![command("echo {{item}} {{index}}")])]);
        assert!(inside.validate().is_ok(), "{:?}", inside.validate());
        let round = flow(vec![repeat(2, vec![command("echo {{index}}")])]);
        assert!(round.validate().is_ok());

        let outside = flow(vec![each("a", vec![wait(1)]), command("echo {{item}}")]);
        let error = outside.validate().unwrap_err().to_string();
        assert!(
            error.contains("Step 3") && error.contains("{{item}}"),
            "{error}"
        );
        // A plain repeat has rounds but no items.
        assert!(
            flow(vec![repeat(2, vec![command("echo {{item}}")])])
                .validate()
                .is_err()
        );
        // The names are Omarchist's.
        assert!(
            flow(vec![command("echo").saving("item")])
                .validate()
                .is_err()
        );
        assert!(
            flow(vec![command("echo").saving("Index")])
                .validate()
                .is_err()
        );
    }

    #[test]
    fn a_menus_pick_is_known_to_the_steps_it_runs() {
        let ok = flow(vec![
            menu(vec![("A", vec![command("echo {{pick}}")])]).saving("pick"),
        ]);
        assert!(ok.validate().is_ok(), "{:?}", ok.validate());
    }

    #[test]
    fn blocks_are_checked() {
        let invalid = [
            repeat(0, vec![]),
            repeat(1001, vec![]),
            each("  ", vec![]),
            menu(vec![]),
            menu(vec![("A", vec![]), ("A", vec![])]),
            menu(vec![(" ", vec![])]),
            when(
                Condition::Contains {
                    value: "x".into(),
                    text: String::new(),
                },
                vec![],
                vec![],
            ),
            when(
                Condition::TimeBetween {
                    from: "9".into(),
                    to: "17:00".into(),
                },
                vec![],
                vec![],
            ),
            // A mistake deep inside is still found, under its own number.
            repeat(2, vec![wait(1), command("  ")]),
        ];
        for step in invalid {
            let text = step.kind.text();
            assert!(flow(vec![step]).validate().is_err(), "{text} passed");
        }
        let deep_error = flow(vec![repeat(2, vec![wait(1), command("  ")])])
            .validate()
            .unwrap_err()
            .to_string();
        assert!(deep_error.starts_with("Step 3:"), "{deep_error}");

        let mut nest = wait(1);
        for _ in 0..MAX_BLOCK_DEPTH {
            nest = repeat(1, vec![nest]);
        }
        assert!(flow(vec![nest.clone()]).validate().is_ok());
        assert!(flow(vec![repeat(1, vec![nest])]).validate().is_err());
    }

    #[test]
    fn a_step_moves_through_the_flow_one_place_at_a_time() {
        // 1, If [2 | 3], 4
        let mut steps = vec![
            wait(1),
            when(Condition::OnBattery, vec![wait(2)], vec![wait(3)]),
            wait(4),
        ];
        let mut path = vec![0];
        let mut seen = vec![path.clone()];
        while let Some(next) = move_step(&mut steps, &path, true) {
            path = next;
            seen.push(path.clone());
        }
        assert_eq!(
            seen,
            vec![
                vec![0],
                // Into the If: first of "then", then after its step.
                vec![0, 0, 0],
                vec![0, 0, 1],
                // Into "otherwise".
                vec![0, 1, 0],
                vec![0, 1, 1],
                // Out, after the If, then past the last step.
                vec![1],
                vec![2],
            ]
        );
        assert_eq!(waits(&steps), vec![2, 3, 4, 1]);
        // And all the way back up.
        while let Some(next) = move_step(&mut steps, &path, false) {
            path = next;
        }
        assert_eq!(path, vec![0]);
        assert_eq!(waits(&steps), vec![1, 2, 3, 4]);
        assert!(move_step(&mut steps, &[7], true).is_none());
    }

    #[test]
    fn a_block_moves_with_what_it_holds() {
        let mut steps = vec![repeat(2, vec![wait(1), wait(2)]), wait(3)];
        assert_eq!(move_step(&mut steps, &[0], true), Some(vec![1]));
        assert_eq!(waits(&steps), vec![3, 1, 2]);
        assert!(steps[1].kind.is_block());
    }

    #[test]
    fn editing_a_block_keeps_its_steps() {
        let old = when(Condition::OnBattery, vec![wait(1)], vec![wait(2)]);
        let mut edited = StepKind::If {
            condition: Condition::Command {
                command: "true".into(),
            },
            not: true,
            then: Vec::new(),
            otherwise: Vec::new(),
        };
        edited.adopt_branches(&old.kind);
        assert_eq!(waits(&[Step::new(edited)]), vec![1, 2]);

        // A menu's choices keep their steps by name; a renamed one keeps
        // the steps of its place.
        let before = menu(vec![
            ("Lock", vec![wait(1)]),
            ("Sleep", vec![wait(2)]),
            ("Off", vec![wait(3)]),
        ]);
        let mut after = menu(vec![("Off", vec![]), ("Suspend", vec![]), ("New", vec![])]).kind;
        after.adopt_branches(&before.kind);
        let StepKind::Menu { choices, .. } = &after else {
            unreachable!()
        };
        assert_eq!(waits(&choices[0].steps), vec![3], "Off kept its steps");
        assert_eq!(
            waits(&choices[1].steps),
            vec![2],
            "Sleep was renamed in place"
        );
        assert!(choices[2].steps.is_empty(), "a new choice starts empty");
    }

    #[test]
    fn an_actions_fields_sit_next_to_its_name() {
        use super::actions::{Arg, Args};
        let args: Args = [("level".to_string(), Arg::Number(40))].into();
        let volume = Step::new(StepKind::Action {
            action: "volume.set".into(),
            args,
        });
        let args: Args = [
            ("text".to_string(), Arg::Text("{{clipboard}}".into())),
            ("to".to_string(), Arg::Text("upper".into())),
        ]
        .into();
        let case = Step::new(StepKind::Action {
            action: "text.case".into(),
            args,
        })
        .saving("loud")
        .off();
        let with_actions = flow(vec![volume, case]);
        assert!(
            with_actions.validate().is_ok(),
            "{:?}",
            with_actions.validate()
        );
        let text = with_actions.to_toml().unwrap();
        assert!(
            text.contains("[[step]]\ntype = \"action\"\naction = \"volume.set\"\nlevel = 40\n"),
            "{text}"
        );
        assert!(
            text.contains(
                "type = \"action\"\naction = \"text.case\"\ntext = \"{{clipboard}}\"\nto = \"upper\"\nenabled = false\noutput = \"loud\"\n"
            ),
            "{text}"
        );
        assert_eq!(parse_flow(&text).unwrap().steps, with_actions.steps);
        assert!(with_actions.steps[1].has_output());
        assert!(!with_actions.steps[0].has_output());

        // A field the action does not have, or a value out of range.
        for bad in [
            "type = \"action\"\naction = \"volume.set\"\nlevel = 400\n",
            "type = \"action\"\naction = \"volume.set\"\nloudness = 4\n",
            "type = \"action\"\naction = \"no.such.action\"\n",
            "type = \"action\"\naction = \"clipboard.set\"\n",
        ] {
            let parsed = parse_flow(&format!("name = \"X\"\n[[step]]\n{bad}")).unwrap();
            assert!(parsed.validate_content().is_err(), "{bad} passed");
        }
    }

    #[test]
    fn input_settings_are_written_only_when_set() {
        let plain = flow(vec![command("echo hi")]);
        let text = plain.to_toml().unwrap();
        assert!(!text.contains("input"), "{text}");

        let mut with = flow(vec![
            command("echo {{input}}"),
            Step::new(StepKind::Flow {
                id: "other".into(),
                input: "{{input}}".into(),
            }),
        ]);
        with.input = super::InputFallback::Selection;
        with.triggers.files = true;
        assert!(with.validate().is_ok(), "{:?}", with.validate());
        let text = with.to_toml().unwrap();
        assert!(text.contains("format = 2\n"), "{text}");
        assert!(text.contains("input = \"selection\"\n"), "{text}");
        assert!(text.contains("[triggers]\nfiles = true\n"), "{text}");
        assert!(
            text.contains("type = \"flow\"\nid = \"other\"\ninput = \"{{input}}\"\n"),
            "{text}"
        );
        let back = parse_flow(&text).unwrap();
        assert_eq!(back.input, super::InputFallback::Selection);
        assert!(back.triggers.files);
        assert_eq!(back.steps, with.steps);
        // `input` is Omarchist's name.
        assert!(
            flow(vec![command("echo").saving("input")])
                .validate()
                .is_err()
        );
    }

    #[test]
    fn a_duplicate_is_equal_but_a_step_of_its_own() {
        let original = repeat(2, vec![wait(1)]);
        let copy = original.duplicate();
        assert_eq!(copy, original);
        assert_ne!(copy.uid, original.uid);
        assert_ne!(
            copy.kind.branches()[0][0].uid,
            original.kind.branches()[0][0].uid
        );
    }
}
