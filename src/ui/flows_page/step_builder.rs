//! The form that assembles one flow step. The keybind action builder
//! edits the kinds a keybind can also run (apps, Omarchy and window
//! actions, flows, commands); the flow-only kinds have their forms here.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState, NumberInput, Position},
    v_flex,
};

use gpui_kit::TestSupportExt;
use std::collections::BTreeMap;

use crate::system::flows::actions::{ActionDef, Args, FieldKind};
use crate::system::flows::condition::Condition;
use crate::system::flows::{MAX_ROUNDS, MenuChoice, OnClick, StepKind, format_duration, vars};
use crate::system::keybinds::action::{Action, ActionKind};
use crate::ui::flows_page::app_picker::{AppPicker, AppPickerEvent};
use crate::ui::flows_page::option_picker::{OptionPicker, OptionPickerEvent, installed_themes};
use crate::ui::flows_page::step_picker::icon_tile;
use crate::ui::flows_page::step_types::StepChoice;
use crate::ui::flows_page::var_token;
use crate::ui::focus::{self, FocusableSwitch};
use crate::ui::keybinds_page::action_builder::{ActionBuilder, ActionBuilderEvent};
use crate::ui::keybinds_page::keybinds_view::{FILTERS_CONTEXT, keybinds_nav};
use crate::ui::text::selectable;

pub enum StepBuilderEvent {
    Changed,
    /// Back to the list of step types.
    ChangeType,
}

/// The row of variables a step can use: one tab stop, arrows pick, Enter
/// or Space inserts.
pub const VARIABLES_CONTEXT: &str = "StepVariables";

pub mod step_vars {
    gpui::actions!(step_vars, [Prev, Next, Insert]);
}

pub mod step_lines {
    gpui::actions!(step_lines, [AddLine]);
}

/// Around a list of one-line fields: Enter in one adds the next.
pub const LINES_CONTEXT: &str = "StepLines";

/// Ten minutes: long enough for anything a flow waits for.
const MAX_WAIT_MS: u64 = 600_000;

const WAIT_PRESETS: [u64; 4] = [500, 1000, 2000, 5000];

/// What an If step looks at; each has its own ways to compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IfKind {
    Text,
    Command,
    App,
    Power,
    Time,
}

impl IfKind {
    const ALL: [(IfKind, &'static str); 5] = [
        (IfKind::Text, "Text"),
        (IfKind::Command, "Command"),
        (IfKind::App, "App"),
        (IfKind::Power, "Power"),
        (IfKind::Time, "Time"),
    ];

    /// The comparisons of this kind, in the order the form offers them.
    /// Every second one is the opposite of the one before it.
    fn ops(self) -> &'static [&'static str] {
        match self {
            IfKind::Text => &[
                "is",
                "is not",
                "contains",
                "does not contain",
                "is empty",
                "is not empty",
            ],
            IfKind::Command => &["succeeds", "fails"],
            IfKind::App => &["is open", "is not open"],
            IfKind::Power => &["on battery", "plugged in"],
            IfKind::Time => &["is between", "is outside"],
        }
    }

    /// The kind and comparison that edit `condition`.
    fn of(condition: &Condition, not: bool) -> (IfKind, usize) {
        let not = usize::from(not);
        match condition {
            Condition::Equals { .. } => (IfKind::Text, not),
            Condition::Contains { .. } => (IfKind::Text, 2 + not),
            Condition::Empty { .. } => (IfKind::Text, 4 + not),
            Condition::Command { .. } => (IfKind::Command, not),
            Condition::AppOpen { .. } => (IfKind::App, not),
            Condition::OnBattery => (IfKind::Power, not),
            Condition::TimeBetween { .. } => (IfKind::Time, not),
        }
    }
}

/// One field of a ready-made action's form.
enum ActionControl {
    Text(Entity<InputState>),
    Number(Entity<InputState>),
    /// The index of the choice, and the row's tab stop.
    Choice(usize, FocusHandle),
    App(Entity<AppPicker>),
    Theme(Entity<OptionPicker>),
}

/// One [`StepBuilder::segmented`] row: its choices, the current one, and
/// what picking another does.
struct Segmented<'a, T> {
    id: SharedString,
    handle: &'a FocusHandle,
    choices: Vec<(T, &'static str)>,
    current: T,
    set: fn(&mut StepBuilder, T, &mut Context<StepBuilder>),
}

/// A text field of the form that takes `{{variables}}`.
#[derive(Clone)]
struct Field(Entity<InputState>);

impl Field {
    /// Puts `text` where the cursor was and gives the field the keyboard.
    fn insert(&self, text: &str, window: &mut Window, cx: &mut App) {
        self.0.update(cx, |input, cx| {
            input.insert(text.to_string(), window, cx);
            input.focus(window, cx);
        })
    }
}

/// Which list of one-line fields of the form is meant.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lines {
    /// A Choose step's options.
    Options,
    /// A menu's choices.
    Choices,
}

impl Lines {
    fn placeholder(self) -> &'static str {
        match self {
            Lines::Options => "Option",
            Lines::Choices => "Choice",
        }
    }

    fn row_id(self) -> &'static str {
        match self {
            Lines::Options => "step-option",
            Lines::Choices => "step-choice",
        }
    }

    fn remove_id(self) -> &'static str {
        match self {
            Lines::Options => "step-option-remove",
            Lines::Choices => "step-choice-remove",
        }
    }

    fn add_id(self) -> &'static str {
        match self {
            Lines::Options => "step-option-add",
            Lines::Choices => "step-choice-add",
        }
    }

    fn add_label(self) -> &'static str {
        match self {
            Lines::Options => "Add option",
            Lines::Choices => "Add choice",
        }
    }
}

/// A list of short values, one field each, laid out as a form builder
/// does: Enter in a field adds the next one, and a button adds one at the
/// end.
#[derive(Default)]
struct LineList {
    rows: Vec<(Entity<InputState>, Subscription)>,
}

impl LineList {
    /// The values typed, trimmed, without the empty fields.
    fn values(&self, cx: &App) -> Vec<String> {
        self.rows
            .iter()
            .map(|(input, _)| input.read(cx).value().trim().to_string())
            .filter(|value| !value.is_empty())
            .collect()
    }
}

pub struct StepBuilder {
    choice: StepChoice,
    action: Entity<ActionBuilder>,
    /// Wait for the command to exit before the next step.
    wait: bool,
    wait_ms: Entity<InputState>,
    notify_title: Entity<InputState>,
    notify_body: Entity<InputState>,
    notify_target: Entity<InputState>,
    on_click: Option<OnClick>,
    click_focus: FocusHandle,
    /// The question of an Ask, Choose or Confirm step, and the title of a
    /// file chooser.
    prompt: Entity<InputState>,
    /// A Choose step's options, one field each.
    options: LineList,
    /// A Choose step's list as text to split into lines.
    from: Entity<InputState>,
    /// Whether the Choose step takes its list from `from`.
    from_variable: bool,
    source_focus: FocusHandle,
    /// What a Run a flow step hands the flow as its input.
    flow_input: Entity<InputState>,
    /// What an If step checks, and how it compares.
    if_kind: IfKind,
    if_op: usize,
    if_kind_focus: FocusHandle,
    if_op_focus: FocusHandle,
    /// The text an If step looks at, and what it compares it with.
    if_value: Entity<InputState>,
    if_other: Entity<InputState>,
    if_command: Entity<InputState>,
    if_from: Entity<InputState>,
    if_to: Entity<InputState>,
    if_app: Entity<AppPicker>,
    /// How often a Repeat step runs.
    times: Entity<InputState>,
    /// What a Repeat with each step goes through, one line at a time.
    items: Entity<InputState>,
    /// A menu's choices, one field each.
    choices: LineList,
    /// The controls of a ready-made action's form, one per field, and
    /// what listens to them.
    action_controls: Vec<ActionControl>,
    _action_subscriptions: Vec<Subscription>,
    /// "Save output as": the name later steps use as `{{name}}`.
    output_name: Entity<InputState>,
    /// The name the form filled in by itself, replaced when the kind
    /// changes unless the person typed their own.
    auto_output: Option<String>,
    /// What this step can use: built-ins, then names earlier steps save.
    variables: Vec<String>,
    /// Names earlier steps save, which a default name must not repeat.
    saved: Vec<String>,
    /// The field a picked variable goes into: the one focused last.
    target: Option<Field>,
    variables_focus: FocusHandle,
    /// The variable the keyboard is on in the row.
    variable_ix: usize,
    back_focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<StepBuilderEvent> for StepBuilder {}

impl StepBuilder {
    /// `exclude_flow` keeps the flow being edited out of the Flow picker.
    /// `output` is the name the step saves its output under; `saved` the
    /// names earlier steps save, which this one may use.
    pub fn new(
        initial: Option<&StepKind>,
        output: Option<&str>,
        saved: &[String],
        exclude_flow: Option<&str>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let dispatcher = initial.and_then(StepKind::dispatcher);
        let action = cx.new(|cx| ActionBuilder::new(dispatcher.as_ref(), window, cx));
        action.update(cx, |builder, cx| {
            builder.set_kind_strip(false, cx);
            if let Some(id) = exclude_flow {
                builder.exclude_flow(id, window, cx);
            }
        });
        let choice = initial
            .and_then(StepChoice::of)
            .unwrap_or_else(|| StepChoice::Action(action.read(cx).kind()));
        let wait = matches!(initial, Some(StepKind::Exec { wait: true, .. }));

        let mut wait_value = String::from("1000");
        let (mut title, mut body, mut target, mut on_click) =
            (String::new(), String::new(), String::new(), None);
        let (mut prompt, mut options, mut from) = (String::new(), Vec::new(), String::new());
        match initial {
            Some(StepKind::Wait { ms }) => wait_value = ms.to_string(),
            Some(StepKind::Notify {
                title: t,
                body: b,
                on_click: c,
                target: g,
            }) => {
                (title, body, target, on_click) = (t.clone(), b.clone(), g.clone(), *c);
            }
            Some(StepKind::Ask { prompt: p }) | Some(StepKind::Confirm { prompt: p }) => {
                prompt = p.clone()
            }
            Some(StepKind::Pick { prompt: p, .. }) => prompt = p.clone(),
            Some(StepKind::Choose {
                prompt: p,
                options: o,
                from: f,
            }) => {
                (prompt, options, from) = (p.clone(), o.clone(), f.clone());
            }
            _ => {}
        }
        let from_variable = options.is_empty() && !from.trim().is_empty();
        let (mut if_kind, mut if_op) = (IfKind::Text, 0);
        let (mut if_value, mut if_other, mut if_command) =
            (String::new(), String::new(), String::new());
        let (mut if_from, mut if_to, mut if_class) =
            (String::from("09:00"), String::from("17:00"), String::new());
        let (mut times, mut items, mut choices) = (String::from("3"), String::new(), Vec::new());
        match initial {
            Some(StepKind::If { condition, not, .. }) => {
                (if_kind, if_op) = IfKind::of(condition, *not);
                match condition {
                    Condition::Equals { value, to } => {
                        (if_value, if_other) = (value.clone(), to.clone())
                    }
                    Condition::Contains { value, text } => {
                        (if_value, if_other) = (value.clone(), text.clone())
                    }
                    Condition::Empty { value } => if_value = value.clone(),
                    Condition::Command { command } => if_command = command.clone(),
                    Condition::AppOpen { class } => if_class = class.clone(),
                    Condition::OnBattery => {}
                    Condition::TimeBetween { from, to } => {
                        (if_from, if_to) = (from.clone(), to.clone())
                    }
                }
            }
            Some(StepKind::Repeat { times: n, .. }) => times = n.to_string(),
            Some(StepKind::Each { items: i, .. }) => items = i.clone(),
            Some(StepKind::Menu {
                prompt: p,
                choices: c,
            }) => {
                prompt = p.clone();
                choices = c.iter().map(|choice| choice.label.clone()).collect();
            }
            _ => {}
        }

        let line = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str, value: &str| {
            cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(placeholder.to_string())
                    .default_value(value.to_string())
            })
        };
        // A number field with a ceiling: a stray digit must not become a
        // pause of centuries.
        let wait_ms = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Milliseconds")
                .default_value(wait_value.clone())
                .step(100.)
                .min(1.)
                .max(MAX_WAIT_MS as f64)
        });
        let notify_title = line(window, cx, "Title", &title);
        let notify_body = line(window, cx, "Message (optional)", &body);
        let notify_target = line(window, cx, "The message", &target);
        let prompt = line(window, cx, "", &prompt);
        let from = line(window, cx, "Text with one option per line", &from);
        let flow_input = line(
            window,
            cx,
            "Nothing",
            match initial {
                Some(StepKind::Flow { input, .. }) => input,
                _ => "",
            },
        );
        let if_value = line(window, cx, "Text, usually a variable", &if_value);
        let if_other = line(window, cx, "", &if_other);
        let if_command = line(window, cx, "Command, e.g. pgrep -x spotify", &if_command);
        let if_from = line(window, cx, "09:00", &if_from);
        let if_to = line(window, cx, "17:00", &if_to);
        let if_app = cx.new(|cx| AppPicker::new(&if_class, window, cx));
        let times = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(times)
                .step(1.)
                .min(1.)
                .max(MAX_ROUNDS as f64)
        });
        let items = line(window, cx, "Text with one item per line", &items);
        let output_name = line(window, cx, "Name, such as url", output.unwrap_or_default());
        let variables: Vec<String> = vars::BUILTINS
            .iter()
            .map(|(name, _)| name.to_string())
            .chain(saved.iter().cloned())
            .collect();

        let mut subscriptions = vec![cx.subscribe_in(
            &action,
            window,
            |this, _, event: &ActionBuilderEvent, _window, cx| {
                let ActionBuilderEvent::Changed = event;
                this.changed(cx);
            },
        )];
        subscriptions.push(cx.subscribe_in(
            &if_app,
            window,
            |this, _, event: &AppPickerEvent, _window, cx| {
                let AppPickerEvent::Changed = event;
                this.changed(cx);
            },
        ));
        for input in [&wait_ms, &output_name, &times, &if_from, &if_to] {
            subscriptions.push(cx.subscribe_in(
                input,
                window,
                |this, _, event: &InputEvent, _window, cx| {
                    if matches!(event, InputEvent::Change) {
                        this.changed(cx);
                    }
                },
            ));
        }
        // Fields that take variables remember being focused, so a variable
        // picked from the row lands in the one the person was typing in.
        for input in [
            &notify_title,
            &notify_body,
            &notify_target,
            &prompt,
            &from,
            &if_value,
            &if_other,
            &if_command,
            &items,
            &flow_input,
        ] {
            let field = Field(input.clone());
            subscriptions.push(cx.subscribe_in(
                input,
                window,
                move |this, _, event: &InputEvent, _window, cx| match event {
                    InputEvent::Change => this.changed(cx),
                    InputEvent::Focus => this.target = Some(field.clone()),
                    _ => {}
                },
            ));
        }

        let mut this = Self {
            choice,
            action,
            wait,
            wait_ms,
            notify_title,
            notify_body,
            notify_target,
            on_click,
            click_focus: focus::tab_stop(cx),
            prompt,
            options: LineList::default(),
            from,
            from_variable,
            source_focus: focus::tab_stop(cx),
            flow_input,
            if_kind,
            if_op,
            if_kind_focus: focus::tab_stop(cx),
            if_op_focus: focus::tab_stop(cx),
            if_value,
            if_other,
            if_command,
            if_from,
            if_to,
            if_app,
            times,
            items,
            choices: LineList::default(),
            action_controls: Vec::new(),
            _action_subscriptions: Vec::new(),
            auto_output: None,
            output_name,
            variables,
            saved: saved.to_vec(),
            target: None,
            variables_focus: focus::tab_stop(cx),
            variable_ix: 0,
            back_focus: focus::tab_stop(cx),
            _subscriptions: subscriptions,
        };
        if let (StepChoice::Do(def), Some(StepKind::Action { args, .. })) = (choice, initial) {
            this.build_action_controls(def, args, window, cx);
        }
        // A new list starts with two fields to fill in.
        for (which, values) in [(Lines::Options, options), (Lines::Choices, choices)] {
            let values = if values.is_empty() {
                vec![String::new(), String::new()]
            } else {
                values
            };
            for value in &values {
                this.add_line(which, usize::MAX, value, window, cx);
            }
        }
        this
    }

    fn lines_mut(&mut self, which: Lines) -> &mut LineList {
        match which {
            Lines::Options => &mut self.options,
            Lines::Choices => &mut self.choices,
        }
    }

    /// Adds a field to `which` at `at` (or the end) holding `value`, and
    /// returns it so the caller can give it the keyboard.
    fn add_line(
        &mut self,
        which: Lines,
        at: usize,
        value: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(which.placeholder())
                .default_value(value.to_string())
        });
        let field = Field(input.clone());
        let subscription =
            cx.subscribe(&input, move |this, _, event: &InputEvent, cx| match event {
                InputEvent::Change => this.changed(cx),
                InputEvent::Focus => this.target = Some(field.clone()),
                _ => {}
            });
        let rows = &mut self.lines_mut(which).rows;
        let at = at.min(rows.len());
        rows.insert(at, (input.clone(), subscription));
        input
    }

    /// Adds an empty field after the one at `ix` (or at the end) and gives
    /// it the keyboard.
    fn add_line_after(
        &mut self,
        which: Lines,
        ix: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let at = ix.map_or(usize::MAX, |ix| ix + 1);
        let next = self.add_line(which, at, "", window, cx);
        // Once it is on screen; focused before that, it loses the focus.
        window.on_next_frame(move |window, cx| {
            next.update(cx, |next, cx| next.focus(window, cx));
        });
        self.changed(cx);
    }

    fn remove_line(&mut self, which: Lines, ix: usize, cx: &mut Context<Self>) {
        let rows = &mut self.lines_mut(which).rows;
        if rows.len() > 1 && ix < rows.len() {
            drop(rows.remove(ix));
            self.target = None;
            self.changed(cx);
        }
    }

    pub fn choice(&self) -> StepChoice {
        self.choice
    }

    /// Switches the form to `choice`, as picked from the list of step
    /// types. A kind whose answer is its whole point starts with a name to
    /// save it under.
    pub fn set_choice(&mut self, choice: StepChoice, window: &mut Window, cx: &mut Context<Self>) {
        self.choice = choice;
        self.target = None;
        match choice {
            StepChoice::Action(kind) => self
                .action
                .update(cx, |builder, cx| builder.set_kind(kind, cx)),
            StepChoice::Do(def) => self.build_action_controls(def, &Args::new(), window, cx),
            _ => {}
        }
        // The name the form suggested for the last kind gives way to the
        // new kind's; one the person typed stays.
        let current = self.output_name.read(cx).value().trim().to_string();
        if current.is_empty() || Some(&current) == self.auto_output.as_ref() {
            let name = choice.default_output().map(|base| {
                (1..)
                    .map(|n| {
                        if n == 1 {
                            base.to_string()
                        } else {
                            format!("{base} {n}")
                        }
                    })
                    .find(|name| !self.saved.contains(name))
                    .unwrap_or_else(|| base.to_string())
            });
            self.output_name.update(cx, |input, cx| {
                input.set_value(name.clone().unwrap_or_default(), window, cx)
            });
            self.auto_output = name;
        }
        self.changed(cx);
    }

    /// Builds the controls of a ready-made action's form from its fields,
    /// starting with `args` (or each field's default).
    fn build_action_controls(
        &mut self,
        def: &'static ActionDef,
        args: &Args,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut controls = Vec::new();
        let mut subscriptions = Vec::new();
        for field in def.fields {
            let value = def.raw(field, args);
            let control = match field.kind {
                FieldKind::Text => {
                    let input = cx.new(|cx| {
                        InputState::new(window, cx)
                            .placeholder(field.placeholder)
                            .default_value(value)
                    });
                    let target = Field(input.clone());
                    subscriptions.push(cx.subscribe_in(
                        &input,
                        window,
                        move |this, _, event: &InputEvent, _window, cx| match event {
                            InputEvent::Change => this.changed(cx),
                            InputEvent::Focus => this.target = Some(target.clone()),
                            _ => {}
                        },
                    ));
                    ActionControl::Text(input)
                }
                FieldKind::Number { min, max, step, .. } => {
                    let input = cx.new(|cx| {
                        InputState::new(window, cx)
                            .default_value(value)
                            .step(step as f64)
                            .min(min as f64)
                            .max(max as f64)
                    });
                    subscriptions.push(cx.subscribe_in(
                        &input,
                        window,
                        |this, _, event: &InputEvent, _window, cx| {
                            if matches!(event, InputEvent::Change) {
                                this.changed(cx);
                            }
                        },
                    ));
                    ActionControl::Number(input)
                }
                FieldKind::Choice(choices) => {
                    let current = choices.iter().position(|(v, _)| *v == value).unwrap_or(0);
                    ActionControl::Choice(current, focus::tab_stop(cx))
                }
                FieldKind::App => {
                    let picker = cx.new(|cx| AppPicker::new(&value, window, cx));
                    subscriptions.push(cx.subscribe_in(
                        &picker,
                        window,
                        |this, _, _: &AppPickerEvent, _window, cx| this.changed(cx),
                    ));
                    ActionControl::App(picker)
                }
                FieldKind::Theme => {
                    let picker = cx.new(|cx| {
                        OptionPicker::new(&value, "Choose a theme", installed_themes, window, cx)
                    });
                    subscriptions.push(cx.subscribe_in(
                        &picker,
                        window,
                        |this, _, _: &OptionPickerEvent, _window, cx| this.changed(cx),
                    ));
                    ActionControl::Theme(picker)
                }
            };
            controls.push(control);
        }
        self.action_controls = controls;
        self._action_subscriptions = subscriptions;
    }

    /// What the action form holds, by field.
    fn action_values(&self, def: &'static ActionDef, cx: &App) -> BTreeMap<&'static str, String> {
        def.fields
            .iter()
            .zip(&self.action_controls)
            .map(|(field, control)| {
                let value = match control {
                    ActionControl::Text(input) | ActionControl::Number(input) => {
                        input.read(cx).value().trim().to_string()
                    }
                    ActionControl::Choice(current, _) => match field.kind {
                        FieldKind::Choice(choices) => choices
                            .get(*current)
                            .map(|(value, _)| value.to_string())
                            .unwrap_or_default(),
                        _ => String::new(),
                    },
                    ActionControl::App(picker) => picker.read(cx).class().to_string(),
                    ActionControl::Theme(picker) => picker.read(cx).value().to_string(),
                };
                (field.key, value)
            })
            .collect()
    }

    /// Puts the keyboard on the form's first control, past the Change
    /// button, with the caret after any text already there.
    pub fn focus_first(&self, window: &mut Window, cx: &mut Context<Self>) {
        let field = match self.choice {
            StepChoice::Notify => &self.notify_title,
            StepChoice::Ask
            | StepChoice::Choose
            | StepChoice::Confirm
            | StepChoice::PickFile
            | StepChoice::PickFolder => &self.prompt,
            StepChoice::Wait => &self.wait_ms,
            StepChoice::Menu => &self.prompt,
            StepChoice::Repeat => &self.times,
            StepChoice::Each => &self.items,
            StepChoice::If => match self.if_kind {
                IfKind::Text => &self.if_value,
                IfKind::Command => &self.if_command,
                IfKind::Time => &self.if_from,
                IfKind::App | IfKind::Power => {
                    self.if_kind_focus.focus(window, cx);
                    return;
                }
            },
            StepChoice::Stop => {
                self.back_focus.focus(window, cx);
                return;
            }
            StepChoice::Do(_) => match self.action_controls.first() {
                Some(ActionControl::Text(input) | ActionControl::Number(input)) => input,
                _ => {
                    // Past the Change button, onto the first control.
                    self.back_focus.focus(window, cx);
                    window.on_next_frame(|window, cx| window.focus_next(cx));
                    return;
                }
            },
            StepChoice::Action(_) => {
                self.back_focus.focus(window, cx);
                window.on_next_frame(|window, cx| window.focus_next(cx));
                return;
            }
        };
        field.update(cx, |input, cx| {
            let end = input.value().encode_utf16().count() as u32;
            input.set_cursor_position(Position::new(0, end), window, cx);
        });
    }

    /// Whether the step being built produces output to save.
    fn produces_output(&self) -> bool {
        match self.choice {
            StepChoice::Action(ActionKind::Command) => self.wait,
            StepChoice::Action(ActionKind::Flow) => true,
            StepChoice::Ask
            | StepChoice::Choose
            | StepChoice::PickFile
            | StepChoice::PickFolder
            | StepChoice::Menu => true,
            StepChoice::Do(def) => def.has_output(),
            _ => false,
        }
    }

    /// The name to save the output under, if any, or why it cannot be one.
    pub fn output_name(&self, cx: &App) -> Result<Option<String>, String> {
        if !self.produces_output() {
            return Ok(None);
        }
        let name = self.output_name.read(cx).value().trim().to_string();
        if name.is_empty() {
            return Ok(None);
        }
        if !vars::is_name(&name) {
            return Err("Use a letter first, then letters, digits, spaces, - or _".to_string());
        }
        if vars::is_builtin(&name) {
            return Err(format!(
                "{} is a built-in variable; pick another name",
                var_token::describe(&name).0
            ));
        }
        Ok(Some(vars::normalize(&name)))
    }

    /// Whether the current kind has a text field a variable can go into.
    fn takes_variables(&self, cx: &App) -> bool {
        match self.choice {
            StepChoice::Action(ActionKind::Flow) => true,
            StepChoice::Action(_) => self.action.read(cx).accepts_variables(),
            StepChoice::Wait | StepChoice::Repeat | StepChoice::Stop => false,
            StepChoice::If => matches!(self.if_kind, IfKind::Text | IfKind::Command),
            StepChoice::Do(_) => self
                .action_controls
                .iter()
                .any(|control| matches!(control, ActionControl::Text(_))),
            _ => true,
        }
    }

    /// The field a variable goes into when none was focused yet.
    fn default_target(&self) -> Option<Field> {
        match self.choice {
            StepChoice::Notify => Some(Field(self.notify_title.clone())),
            StepChoice::Choose if self.from_variable => Some(Field(self.from.clone())),
            StepChoice::Ask
            | StepChoice::Choose
            | StepChoice::Confirm
            | StepChoice::PickFile
            | StepChoice::PickFolder
            | StepChoice::Menu => Some(Field(self.prompt.clone())),
            StepChoice::Each => Some(Field(self.items.clone())),
            StepChoice::Do(_) => self
                .action_controls
                .iter()
                .find_map(|control| match control {
                    ActionControl::Text(input) => Some(Field(input.clone())),
                    _ => None,
                }),
            StepChoice::If => match self.if_kind {
                IfKind::Text => Some(Field(self.if_value.clone())),
                IfKind::Command => Some(Field(self.if_command.clone())),
                _ => None,
            },
            _ => None,
        }
    }

    fn insert_variable(&mut self, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        let text = format!("{{{{{name}}}}}");
        match self.choice {
            StepChoice::Action(ActionKind::Flow) => {
                Field(self.flow_input.clone()).insert(&text, window, cx);
                self.changed(cx);
                return;
            }
            StepChoice::Action(_) => {
                self.action
                    .update(cx, |builder, cx| builder.insert_in_field(&text, window, cx));
                return;
            }
            _ => {}
        }
        if let Some(field) = self.target.clone().or_else(|| self.default_target()) {
            field.insert(&text, window, cx);
            self.changed(cx);
        }
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        cx.emit(StepBuilderEvent::Changed);
        cx.notify();
    }

    fn wait_value(&self, cx: &App) -> Result<u64, String> {
        let text = self.wait_ms.read(cx).value().trim().to_string();
        match text.parse::<u64>() {
            Ok(ms) if ms > MAX_WAIT_MS => Err(format!(
                "A wait can be at most {} (use several steps for longer)",
                format_duration(MAX_WAIT_MS)
            )),
            Ok(ms) if ms > 0 => Ok(ms),
            _ => Err("Enter how long to wait, in milliseconds".to_string()),
        }
    }

    /// The step the controls currently describe, or why they don't: a
    /// field that is not filled in, or a variable this step cannot use.
    pub fn step(&self, cx: &App) -> Result<StepKind, String> {
        let kind = self.kind(cx)?;
        for name in kind.references() {
            if !self.variables.contains(&name) {
                return Err(format!(
                    "Nothing before this step saves {}",
                    var_token::describe(&name).0
                ));
            }
        }
        Ok(kind)
    }

    fn question(&self, cx: &App) -> Result<String, String> {
        let prompt = self.prompt.read(cx).value().trim().to_string();
        if prompt.is_empty() {
            return Err("Enter the question".into());
        }
        Ok(prompt)
    }

    fn kind(&self, cx: &App) -> Result<StepKind, String> {
        match self.choice {
            StepChoice::Action(_) => {
                let builder = self.action.read(cx);
                if let Ok(Action::Flow(id)) = builder.action(cx) {
                    return Ok(StepKind::Flow {
                        id,
                        input: self.flow_input.read(cx).value().trim().to_string(),
                    });
                }
                let dispatcher = builder.dispatcher(cx)?;
                StepKind::from_dispatcher(dispatcher, self.wait)
                    .ok_or_else(|| "This action cannot be a step".to_string())
            }
            StepChoice::Wait => Ok(StepKind::Wait {
                ms: self.wait_value(cx)?,
            }),
            StepChoice::Notify => {
                let title = self.notify_title.read(cx).value().trim().to_string();
                if title.is_empty() {
                    return Err("Enter the notification's title".into());
                }
                Ok(StepKind::Notify {
                    title,
                    body: self.notify_body.read(cx).value().trim().to_string(),
                    on_click: self.on_click,
                    target: match self.on_click {
                        Some(_) => self.notify_target.read(cx).value().trim().to_string(),
                        None => String::new(),
                    },
                })
            }
            StepChoice::Ask => Ok(StepKind::Ask {
                prompt: self.question(cx)?,
            }),
            StepChoice::Confirm => Ok(StepKind::Confirm {
                prompt: self.question(cx)?,
            }),
            StepChoice::Choose => {
                let prompt = self.question(cx)?;
                if self.from_variable {
                    let from = self.from.read(cx).value().trim().to_string();
                    if from.is_empty() {
                        return Err("Pick the variable that holds the options".into());
                    }
                    return Ok(StepKind::Choose {
                        prompt,
                        options: Vec::new(),
                        from,
                    });
                }
                let options = self.options.values(cx);
                if options.is_empty() {
                    return Err("Enter at least one option".into());
                }
                Ok(StepKind::Choose {
                    prompt,
                    options,
                    from: String::new(),
                })
            }
            StepChoice::PickFile | StepChoice::PickFolder => Ok(StepKind::Pick {
                prompt: self.prompt.read(cx).value().trim().to_string(),
                folder: self.choice == StepChoice::PickFolder,
            }),
            StepChoice::If => {
                let (condition, not) = self.condition(cx);
                condition.validate().map_err(capitalize)?;
                Ok(StepKind::If {
                    condition,
                    not,
                    then: Vec::new(),
                    otherwise: Vec::new(),
                })
            }
            StepChoice::Repeat => {
                let text = self.times.read(cx).value().trim().to_string();
                match text.parse::<u32>() {
                    Ok(times) if (1..=MAX_ROUNDS).contains(&times) => Ok(StepKind::Repeat {
                        times,
                        steps: Vec::new(),
                    }),
                    _ => Err(format!("Repeat between 1 and {MAX_ROUNDS} times")),
                }
            }
            StepChoice::Each => {
                let items = self.items.read(cx).value().trim().to_string();
                if items.is_empty() {
                    return Err("Pick the variable that holds the items".into());
                }
                Ok(StepKind::Each {
                    items,
                    steps: Vec::new(),
                })
            }
            StepChoice::Menu => {
                let prompt = self.question(cx)?;
                let mut choices: Vec<MenuChoice> = Vec::new();
                for label in self.choices.values(cx) {
                    if choices.iter().any(|c| c.label == label) {
                        return Err(format!("'{label}' is there twice"));
                    }
                    choices.push(MenuChoice {
                        label,
                        steps: Vec::new(),
                    });
                }
                if choices.is_empty() {
                    return Err("Enter at least one choice".into());
                }
                Ok(StepKind::Menu { prompt, choices })
            }
            StepChoice::Stop => Ok(StepKind::Stop),
            StepChoice::Do(def) => {
                let args = def.args(&self.action_values(def, cx));
                def.validate(&args).map_err(capitalize)?;
                Ok(StepKind::Action {
                    action: def.id.to_string(),
                    args,
                })
            }
        }
    }

    /// The condition the If form describes, and whether it is negated.
    fn condition(&self, cx: &App) -> (Condition, bool) {
        let read = |input: &Entity<InputState>| input.read(cx).value().trim().to_string();
        let not = self.if_op % 2 == 1;
        let condition = match self.if_kind {
            IfKind::Text => match self.if_op / 2 {
                0 => Condition::Equals {
                    value: read(&self.if_value),
                    to: read(&self.if_other),
                },
                1 => Condition::Contains {
                    value: read(&self.if_value),
                    text: read(&self.if_other),
                },
                _ => Condition::Empty {
                    value: read(&self.if_value),
                },
            },
            IfKind::Command => Condition::Command {
                command: read(&self.if_command),
            },
            IfKind::App => Condition::AppOpen {
                class: self.if_app.read(cx).class().to_string(),
            },
            IfKind::Power => Condition::OnBattery,
            IfKind::Time => Condition::TimeBetween {
                from: read(&self.if_from),
                to: read(&self.if_to),
            },
        };
        (condition, not)
    }

    // MARK: Render

    fn label(text: &'static str) -> Div {
        div().text_sm().child(text)
    }

    fn field(label: &'static str, control: impl IntoElement) -> Div {
        v_flex().gap_1().child(Self::label(label)).child(control)
    }

    /// The step's kind, with the way back to the list of kinds.
    fn render_header(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let info = self.choice.info();
        let _ = window;
        h_flex()
            .gap_2()
            .items_center()
            .child(icon_tile(info.icon, info.group.accent(cx), px(28.), cx))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .font_weight(FontWeight::MEDIUM)
                    .truncate()
                    .child(info.label),
            )
            .child(
                Button::new("step-change-type")
                    .ghost()
                    .xsmall()
                    .label("Change")
                    .track_focus(&self.back_focus)
                    .cursor_pointer()
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(StepBuilderEvent::ChangeType))),
            )
    }

    /// A row of exclusive choices that is one tab stop; the arrow keys
    /// move the choice.
    fn segmented<T: Copy + PartialEq + 'static>(
        &self,
        row: Segmented<T>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Segmented {
            id,
            handle,
            choices,
            current,
            set,
        } = row;
        let focused = handle.is_focused(window);
        let ring = focus::focus_border(focused, cx.theme().transparent, cx);
        let values: Vec<T> = choices.iter().map(|(value, _)| *value).collect();
        let cycle = move |this: &mut Self, delta: isize, cx: &mut Context<Self>| {
            let ix = values.iter().position(|v| *v == current).unwrap_or(0) as isize;
            let next = (ix + delta).rem_euclid(values.len() as isize) as usize;
            set(this, values[next], cx);
        };
        let (prev, next) = (cycle.clone(), cycle);
        // Wrapped so the row is as wide as its choices, not the form.
        h_flex()
            .child(
                h_flex()
                    .id(ElementId::Name(id.clone()))
                    .test_support()
                    .key_context(FILTERS_CONTEXT)
                    .track_focus(handle)
                    .on_action(
                        cx.listener(move |this, _: &keybinds_nav::FilterPrev, _, cx| {
                            prev(this, -1, cx)
                        }),
                    )
                    .on_action(
                        cx.listener(move |this, _: &keybinds_nav::FilterNext, _, cx| {
                            next(this, 1, cx)
                        }),
                    )
                    .rounded(cx.theme().radius)
                    .border_1()
                    .border_color(ring)
                    .p_0p5()
                    .gap_1()
                    .flex_wrap()
                    .children(choices.into_iter().enumerate().map(|(ix, (value, label))| {
                        let button = Button::new(ElementId::NamedInteger(id.clone(), ix as u64))
                            .label(label)
                            .small()
                            .tab_stop(false)
                            .cursor_pointer();
                        let button = if value == current {
                            button.primary()
                        } else {
                            button.ghost()
                        };
                        button.on_click(cx.listener(move |this, _, _, cx| set(this, value, cx)))
                    })),
            )
            .into_any_element()
    }

    fn render_body(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        match self.choice {
            StepChoice::Action(kind) => v_flex()
                .gap_2()
                .child(self.action.clone())
                .when(kind == ActionKind::Flow, |this| {
                    this.child(Self::field(
                        "Input",
                        Input::new(&self.flow_input).id("flow-input").small(),
                    ))
                })
                .when(
                    kind != ActionKind::Window && kind != ActionKind::Flow,
                    |this| {
                        this.child(
                            div().text_sm().child(
                                FocusableSwitch::new("step-wait")
                                    .label("Wait until it finishes")
                                    .checked(self.wait)
                                    .on_change(cx.listener(|this, checked, _window, cx| {
                                        this.wait = *checked;
                                        this.changed(cx);
                                    })),
                            ),
                        )
                    },
                )
                .into_any_element(),
            StepChoice::Wait => {
                let current = self.wait_value(cx).ok();
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .flex_wrap()
                            .child(div().w_40().child(NumberInput::new(&self.wait_ms).small()))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("ms"),
                            )
                            .children(WAIT_PRESETS.iter().enumerate().map(|(ix, &ms)| {
                                let button = Button::new(("wait-preset", ix))
                                    .label(format_duration(ms))
                                    .xsmall()
                                    .tab_stop(false)
                                    .cursor_pointer();
                                let button = if current == Some(ms) {
                                    button.primary()
                                } else {
                                    button.outline()
                                };
                                button.on_click(cx.listener(move |this, _, window, cx| {
                                    this.wait_ms.update(cx, |input, cx| {
                                        input.set_value(ms.to_string(), window, cx)
                                    });
                                    this.changed(cx);
                                }))
                            })),
                    )
                    .into_any_element()
            }
            StepChoice::Notify => {
                let clicked = self.on_click;
                v_flex()
                    .gap_3()
                    .child(Self::field(
                        "Title",
                        Input::new(&self.notify_title).id("notify-title").small(),
                    ))
                    .child(Self::field(
                        "Message",
                        Input::new(&self.notify_body).id("notify-body").small(),
                    ))
                    .child(Self::field(
                        "When clicked",
                        self.segmented(
                            Segmented {
                                id: "notify-click".into(),
                                handle: &self.click_focus,
                                choices: vec![
                                    (None, "Nothing"),
                                    (Some(OnClick::Copy), "Copy"),
                                    (Some(OnClick::Open), "Open"),
                                ],
                                current: clicked,
                                set: |this, value, cx| {
                                    this.on_click = value;
                                    this.changed(cx);
                                },
                            },
                            window,
                            cx,
                        ),
                    ))
                    .when_some(clicked, |this, action| {
                        this.child(Self::field(
                            match action {
                                OnClick::Copy => "What to copy",
                                OnClick::Open => "What to open",
                            },
                            Input::new(&self.notify_target).id("notify-target").small(),
                        ))
                    })
                    .into_any_element()
            }
            StepChoice::Ask | StepChoice::Confirm => v_flex()
                .gap_3()
                .child(Self::field(
                    "Question",
                    Input::new(&self.prompt).id("step-prompt").small(),
                ))
                .into_any_element(),
            StepChoice::PickFile | StepChoice::PickFolder => v_flex()
                .gap_3()
                .child(Self::field(
                    "Title",
                    Input::new(&self.prompt).id("step-prompt").small(),
                ))
                .into_any_element(),
            StepChoice::Choose => v_flex()
                .gap_3()
                .child(Self::field(
                    "Question",
                    Input::new(&self.prompt).id("step-prompt").small(),
                ))
                .child(Self::field(
                    "Options",
                    self.segmented(
                        Segmented {
                            id: "choose-source".into(),
                            handle: &self.source_focus,
                            choices: vec![(false, "A list"), (true, "From a variable")],
                            current: self.from_variable,
                            set: |this, value, cx| {
                                this.from_variable = value;
                                this.target = None;
                                this.changed(cx);
                            },
                        },
                        window,
                        cx,
                    ),
                ))
                .child(if self.from_variable {
                    Input::new(&self.from)
                        .id("step-from")
                        .small()
                        .into_any_element()
                } else {
                    self.render_lines(Lines::Options, cx)
                })
                .into_any_element(),
            StepChoice::If => self.render_if(window, cx),
            StepChoice::Repeat => v_flex()
                .gap_3()
                .child(Self::field(
                    "Times",
                    div()
                        .id("step-times")
                        .test_support()
                        .w_40()
                        .child(NumberInput::new(&self.times).small()),
                ))
                .into_any_element(),
            StepChoice::Each => v_flex()
                .gap_3()
                .child(Self::field(
                    "Items",
                    Input::new(&self.items).id("step-items").small(),
                ))
                .into_any_element(),
            StepChoice::Menu => v_flex()
                .gap_3()
                .child(Self::field(
                    "Question",
                    Input::new(&self.prompt).id("step-prompt").small(),
                ))
                .child(Self::field(
                    "Choices",
                    self.render_lines(Lines::Choices, cx),
                ))
                .into_any_element(),
            StepChoice::Stop => div().into_any_element(),
            StepChoice::Do(def) => self.render_action(def, window, cx),
        }
    }

    /// The fields of `which`, each with a button that removes it, and one
    /// that adds another.
    fn render_lines(&self, which: Lines, cx: &mut Context<Self>) -> AnyElement {
        let rows = match which {
            Lines::Options => &self.options.rows,
            Lines::Choices => &self.choices.rows,
        };
        let only_one = rows.len() <= 1;
        v_flex()
            .key_context(LINES_CONTEXT)
            .gap_1()
            .children(rows.iter().enumerate().map(|(ix, (input, _))| {
                h_flex()
                    .on_action(
                        cx.listener(move |this, _: &step_lines::AddLine, window, cx| {
                            this.add_line_after(which, Some(ix), window, cx)
                        }),
                    )
                    .gap_1()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .child(Input::new(input).id((which.row_id(), ix)).small()),
                    )
                    .child(
                        Button::new((which.remove_id(), ix))
                            .ghost()
                            .xsmall()
                            .icon(Icon::new(IconName::Close))
                            .tooltip(if only_one {
                                "The last one stays"
                            } else {
                                "Remove"
                            })
                            .disabled(only_one)
                            .cursor_pointer()
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.remove_line(which, ix, cx)),
                            ),
                    )
            }))
            .child(
                h_flex().child(
                    Button::new(which.add_id())
                        .ghost()
                        .xsmall()
                        .icon(Icon::new(Icon::empty()).path("icons/plus.svg"))
                        .label(which.add_label())
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.add_line_after(which, None, window, cx)
                        })),
                ),
            )
            .into_any_element()
    }

    /// A ready-made action's form: one control per field.
    fn render_action(
        &self,
        def: &'static ActionDef,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut form = v_flex().gap_3();
        for (ix, (field, control)) in def.fields.iter().zip(&self.action_controls).enumerate() {
            let id: SharedString = format!("action-{}", field.key).into();
            let element: AnyElement = match control {
                ActionControl::Text(input) => Input::new(input)
                    .id(ElementId::Name(id))
                    .small()
                    .into_any_element(),
                ActionControl::Number(input) => {
                    let unit = match field.kind {
                        FieldKind::Number { unit, .. } => unit,
                        _ => "",
                    };
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .id(ElementId::Name(id))
                                .test_support()
                                .w_32()
                                .child(NumberInput::new(input).small()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(unit),
                        )
                        .into_any_element()
                }
                ActionControl::Choice(current, handle) => {
                    let choices = match field.kind {
                        FieldKind::Choice(choices) => choices,
                        _ => &[],
                    };
                    self.segmented(
                        Segmented {
                            id,
                            handle,
                            choices: choices
                                .iter()
                                .enumerate()
                                .map(|(choice, (_, label))| ((ix, choice), *label))
                                .collect(),
                            current: (ix, *current),
                            set: |this, (field, choice), cx| {
                                if let Some(ActionControl::Choice(current, _)) =
                                    this.action_controls.get_mut(field)
                                {
                                    *current = choice;
                                }
                                this.changed(cx);
                            },
                        },
                        window,
                        cx,
                    )
                }
                ActionControl::App(picker) => picker.clone().into_any_element(),
                ActionControl::Theme(picker) => picker.clone().into_any_element(),
            };
            form = form.child(Self::field(field.label, element));
        }
        form.into_any_element()
    }

    /// The If form: what to look at, how to compare, and with what.
    fn render_if(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let kind = self.if_kind;
        let ops: Vec<(usize, &'static str)> = kind.ops().iter().copied().enumerate().collect();
        let op_row = self.segmented(
            Segmented {
                id: "if-op".into(),
                handle: &self.if_op_focus,
                choices: ops,
                current: self.if_op,
                set: |this, value, cx| {
                    this.if_op = value;
                    this.changed(cx);
                },
            },
            window,
            cx,
        );
        let form = v_flex().gap_3().child(Self::field(
            "Check",
            self.segmented(
                Segmented {
                    id: "if-kind".into(),
                    handle: &self.if_kind_focus,
                    choices: IfKind::ALL.to_vec(),
                    current: kind,
                    set: |this, value, cx| {
                        this.if_kind = value;
                        this.if_op = 0;
                        this.target = None;
                        this.changed(cx);
                    },
                },
                window,
                cx,
            ),
        ));
        match kind {
            IfKind::Text => form
                .child(Input::new(&self.if_value).id("if-value").small())
                .child(op_row)
                // "is empty" and "is not empty" compare with nothing.
                .when(self.if_op < 4, |this| {
                    this.child(Input::new(&self.if_other).id("if-other").small())
                }),
            IfKind::Command => form
                .child(Input::new(&self.if_command).id("if-command").small())
                .child(op_row),
            IfKind::App => form.child(self.if_app.clone()).child(op_row),
            IfKind::Power => form.child(op_row),
            IfKind::Time => form.child(op_row).child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .w_24()
                            .child(Input::new(&self.if_from).id("if-from").small()),
                    )
                    .child(div().text_sm().child("and"))
                    .child(
                        div()
                            .w_24()
                            .child(Input::new(&self.if_to).id("if-to").small()),
                    ),
            ),
        }
        .into_any_element()
    }

    /// Why the form is not a step yet. The action builder shows its own.
    fn render_problem(&self, cx: &App) -> Option<AnyElement> {
        if matches!(self.choice, StepChoice::Action(_)) {
            return None;
        }
        let message = self.step(cx).err()?;
        Some(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(selectable("step-problem", message))
                .into_any_element(),
        )
    }

    /// The variables this step can use, as tokens that insert them.
    fn render_variables(&self, window: &Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        if !self.takes_variables(cx) {
            return None;
        }
        let theme = cx.theme();
        let row_focused = self.variables_focus.is_focused(window);
        let ring = focus::focus_border(row_focused, theme.transparent, cx);
        let count = self.variables.len();
        Some(
            h_flex()
                .id("step-variables")
                .key_context(VARIABLES_CONTEXT)
                .track_focus(&self.variables_focus)
                .on_action(cx.listener(move |this, _: &step_vars::Prev, _, cx| {
                    this.variable_ix = (this.variable_ix + count - 1) % count.max(1);
                    cx.notify();
                }))
                .on_action(cx.listener(move |this, _: &step_vars::Next, _, cx| {
                    this.variable_ix = (this.variable_ix + 1) % count.max(1);
                    cx.notify();
                }))
                .on_action(cx.listener(|this, _: &step_vars::Insert, window, cx| {
                    if let Some(name) = this.variables.get(this.variable_ix).cloned() {
                        this.insert_variable(&name, window, cx);
                    }
                }))
                .rounded(theme.radius)
                .border_1()
                .border_color(ring)
                .p_0p5()
                .gap_1()
                .flex_wrap()
                .items_center()
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .mr_1()
                        .child("Insert"),
                )
                .children(self.variables.iter().enumerate().map(|(ix, name)| {
                    let name = name.clone();
                    let current = row_focused && ix == self.variable_ix;
                    let ring = focus::focus_border(current, theme.transparent, cx);
                    div()
                        .id(ElementId::Name(format!("step-variable-{name}").into()))
                        .test_support()
                        .rounded(theme.radius)
                        .border_1()
                        .border_color(ring)
                        .p_px()
                        .cursor_pointer()
                        .hover(|this| this.opacity(0.8))
                        .child(var_token::token(&name, cx).text_xs())
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.insert_variable(&name, window, cx)
                        }))
                }))
                .into_any_element(),
        )
    }

    /// "Save output as", for a step that produces output.
    fn render_output(&self, cx: &App) -> Option<AnyElement> {
        if !self.produces_output() {
            return None;
        }
        let theme = cx.theme();
        Some(
            h_flex()
                .gap_2()
                .items_center()
                .flex_wrap()
                .child(div().text_sm().child("Save output as"))
                .child(
                    div()
                        .w_48()
                        .child(Input::new(&self.output_name).id("step-output-name").small()),
                )
                .when_some(self.output_name(cx).err(), |this, error| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(theme.danger)
                            .child(selectable("output-name-error", error)),
                    )
                })
                .into_any_element(),
        )
    }
}

impl Render for StepBuilder {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(self.render_header(window, cx))
            .child(self.render_body(window, cx))
            .children(self.render_variables(window, cx))
            .children(self.render_output(cx))
            .children(self.render_problem(cx))
    }
}

/// A validation message as a sentence.
fn capitalize(message: String) -> String {
    let mut chars = message.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => message,
    }
}
