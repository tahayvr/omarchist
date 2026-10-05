//! The controls that assemble one flow step: the keybind action builder's
//! kinds plus the flow-only Wait and Notify.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState, NumberInput},
    v_flex,
};

use crate::system::flows::{StepKind, format_duration, vars};
use crate::system::keybinds::action::{Action, ActionKind};
use crate::ui::flows_page::var_token;
use crate::ui::focus::{self, FocusableSwitch};
use crate::ui::keybinds_page::action_builder::{ActionBuilder, ActionBuilderEvent};
use crate::ui::keybinds_page::keybinds_view::{FILTERS_CONTEXT, keybinds_nav};
use crate::ui::text::selectable;

pub enum StepBuilderEvent {
    Changed,
}

/// The row of variables a step can use: one tab stop, arrows pick, Enter
/// or Space inserts.
pub const VARIABLES_CONTEXT: &str = "StepVariables";

pub mod step_vars {
    gpui::actions!(step_vars, [Prev, Next, Insert]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepChoice {
    Action(ActionKind),
    Wait,
    Notify,
}

impl StepChoice {
    pub const ALL: [StepChoice; 9] = [
        StepChoice::Action(ActionKind::App),
        StepChoice::Action(ActionKind::WebApp),
        StepChoice::Action(ActionKind::Terminal),
        StepChoice::Action(ActionKind::Omarchy),
        StepChoice::Action(ActionKind::Window),
        StepChoice::Action(ActionKind::Flow),
        StepChoice::Action(ActionKind::Command),
        StepChoice::Wait,
        StepChoice::Notify,
    ];

    fn label(self) -> &'static str {
        match self {
            StepChoice::Action(kind) => kind.label(),
            StepChoice::Wait => "Wait",
            StepChoice::Notify => "Notify",
        }
    }

    fn icon_path(self) -> &'static str {
        match self {
            StepChoice::Action(kind) => kind.icon_path(),
            StepChoice::Wait => "icons/hourglass.svg",
            StepChoice::Notify => "icons/bell.svg",
        }
    }
}

/// Ten minutes: long enough for anything a flow waits for.
const MAX_WAIT_MS: u64 = 600_000;

const WAIT_PRESETS: [u64; 4] = [500, 1000, 2000, 5000];

pub struct StepBuilder {
    choice: StepChoice,
    kind_focus: FocusHandle,
    action: Entity<ActionBuilder>,
    /// Wait for the command to exit before the next step.
    wait: bool,
    wait_ms: Entity<InputState>,
    notify_title: Entity<InputState>,
    notify_body: Entity<InputState>,
    /// "Save output as": the name later steps use as `{{name}}`.
    output_name: Entity<InputState>,
    /// What this step can use: built-ins, then names earlier steps save.
    variables: Vec<String>,
    /// The notification field a variable goes into: the one last focused.
    notify_body_last: bool,
    variables_focus: FocusHandle,
    /// The variable the keyboard is on in the row.
    variable_ix: usize,
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
        let choice = match initial {
            Some(StepKind::Wait { .. }) => StepChoice::Wait,
            Some(StepKind::Notify { .. }) => StepChoice::Notify,
            _ => StepChoice::Action(action.read(cx).kind()),
        };
        let wait = matches!(initial, Some(StepKind::Exec { wait: true, .. }));
        let (wait_value, title_value, body_value) = match initial {
            Some(StepKind::Wait { ms }) => (ms.to_string(), String::new(), String::new()),
            Some(StepKind::Notify { title, body }) => {
                (String::from("1000"), title.clone(), body.clone())
            }
            _ => (String::from("1000"), String::new(), String::new()),
        };
        let text = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str, value: &str| {
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
        let notify_title = text(window, cx, "Title", &title_value);
        let notify_body = text(window, cx, "Message (optional)", &body_value);
        let output_name = text(window, cx, "Name, such as url", output.unwrap_or_default());
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
        for input in [&wait_ms, &notify_title, &notify_body, &output_name] {
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

        for (input, body) in [(&notify_title, false), (&notify_body, true)] {
            subscriptions.push(cx.subscribe_in(
                input,
                window,
                move |this, _, event: &InputEvent, _window, _cx| {
                    if matches!(event, InputEvent::Focus) {
                        this.notify_body_last = body;
                    }
                },
            ));
        }

        Self {
            choice,
            kind_focus: focus::tab_stop(cx),
            action,
            wait,
            wait_ms,
            notify_title,
            notify_body,
            output_name,
            variables,
            notify_body_last: false,
            variables_focus: focus::tab_stop(cx),
            variable_ix: 0,
            _subscriptions: subscriptions,
        }
    }

    /// Whether the step being built produces output to save: a command
    /// the flow waits for, or another flow.
    fn produces_output(&self) -> bool {
        match self.choice {
            StepChoice::Action(ActionKind::Command) => self.wait,
            StepChoice::Action(ActionKind::Flow) => true,
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
                "{{{{{}}}}} is a built-in variable; pick another name",
                vars::normalize(&name)
            ));
        }
        Ok(Some(vars::normalize(&name)))
    }

    /// Whether the current kind has a text field a variable can go into.
    fn takes_variables(&self, cx: &App) -> bool {
        match self.choice {
            StepChoice::Notify => true,
            StepChoice::Action(_) => self.action.read(cx).accepts_variables(),
            StepChoice::Wait => false,
        }
    }

    fn insert_variable(&mut self, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        let text = format!("{{{{{name}}}}}");
        match self.choice {
            StepChoice::Notify => {
                let input = if self.notify_body_last {
                    self.notify_body.clone()
                } else {
                    self.notify_title.clone()
                };
                input.update(cx, |input, cx| {
                    let current = input.value().to_string();
                    let sep = if current.is_empty() || current.ends_with(' ') {
                        ""
                    } else {
                        " "
                    };
                    input.set_value(format!("{current}{sep}{text}"), window, cx);
                    input.focus(window, cx);
                });
                self.changed(cx);
            }
            StepChoice::Action(_) => self
                .action
                .update(cx, |builder, cx| builder.append_to_field(&text, window, cx)),
            StepChoice::Wait => {}
        }
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        cx.emit(StepBuilderEvent::Changed);
        cx.notify();
    }

    fn set_choice(&mut self, choice: StepChoice, cx: &mut Context<Self>) {
        if self.choice == choice {
            return;
        }
        self.choice = choice;
        if let StepChoice::Action(kind) = choice {
            self.action
                .update(cx, |builder, cx| builder.set_kind(kind, cx));
        }
        self.changed(cx);
    }

    fn cycle_choice(&mut self, delta: isize, cx: &mut Context<Self>) {
        let all = StepChoice::ALL;
        let ix = all.iter().position(|c| *c == self.choice).unwrap_or(0) as isize;
        let next = (ix + delta).rem_euclid(all.len() as isize) as usize;
        self.set_choice(all[next], cx);
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
    /// field that is not filled in, or a `{{name}}` this step cannot use.
    pub fn step(&self, cx: &App) -> Result<StepKind, String> {
        let kind = self.kind(cx)?;
        for name in kind.references() {
            if !self.variables.contains(&name) {
                return Err(format!(
                    "{{{{{name}}}}} is not a variable here: no earlier step saves it"
                ));
            }
        }
        Ok(kind)
    }

    fn kind(&self, cx: &App) -> Result<StepKind, String> {
        match self.choice {
            StepChoice::Action(_) => {
                let builder = self.action.read(cx);
                if let Ok(Action::Flow(id)) = builder.action(cx) {
                    return Ok(StepKind::Flow { id });
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
                })
            }
        }
    }

    // MARK: Render

    fn render_choices(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.kind_focus.is_focused(window);
        let ring = focus::focus_border(focused, cx.theme().transparent, cx);
        let border = cx.theme().border;
        h_flex()
            .id("step-kinds")
            .key_context(FILTERS_CONTEXT)
            .track_focus(&self.kind_focus)
            .on_action(
                cx.listener(|this, _: &keybinds_nav::FilterPrev, _, cx| this.cycle_choice(-1, cx)),
            )
            .on_action(
                cx.listener(|this, _: &keybinds_nav::FilterNext, _, cx| this.cycle_choice(1, cx)),
            )
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(ring)
            .p_0p5()
            .gap_1()
            .flex_wrap()
            .children(
                StepChoice::ALL
                    .iter()
                    .enumerate()
                    .flat_map(|(ix, &choice)| {
                        let button = Button::new(("step-kind", ix))
                            .icon(Icon::new(Icon::empty()).path(choice.icon_path()))
                            .label(choice.label())
                            .small()
                            .tab_stop(false)
                            .cursor_pointer();
                        let button = if self.choice == choice {
                            button.primary()
                        } else {
                            button.ghost()
                        };
                        let button =
                            button
                                .on_click(cx.listener(move |this, _, _window, cx| {
                                    this.set_choice(choice, cx)
                                }))
                                .into_any_element();
                        // A thin divider separates the actions from the flow-only kinds.
                        let divider =
                            (choice == StepChoice::Action(ActionKind::Command)).then(|| {
                                div()
                                    .w(px(1.))
                                    .h_5()
                                    .mx_1()
                                    .self_center()
                                    .bg(border)
                                    .into_any_element()
                            });
                        std::iter::once(button).chain(divider)
                    }),
            )
    }

    fn render_body(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.choice {
            StepChoice::Action(kind) => v_flex()
                .gap_2()
                .child(self.action.clone())
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
            StepChoice::Notify => v_flex()
                .gap_2()
                .child(Input::new(&self.notify_title).small())
                .child(Input::new(&self.notify_body).small())
                .into_any_element(),
        }
    }

    fn render_preview(&self, cx: &App) -> Option<AnyElement> {
        if matches!(self.choice, StepChoice::Action(_)) {
            // The action builder draws its own preview.
            return None;
        }
        let theme = cx.theme();
        let row = h_flex().gap_2().items_start().text_xs();
        Some(match self.step(cx) {
            Ok(step) => row
                .child(
                    div()
                        .flex_shrink_0()
                        .text_color(theme.muted_foreground)
                        .child("Step"),
                )
                .child(
                    div()
                        .min_w_0()
                        .px_2()
                        .py_0p5()
                        .rounded(theme.radius)
                        .bg(theme.secondary)
                        .child(var_token::rich_text("step-preview", &step.text(), cx)),
                )
                .into_any_element(),
            Err(message) => row
                .child(
                    div()
                        .text_color(theme.muted_foreground)
                        .child(selectable("step-preview-message", message)),
                )
                .into_any_element(),
        })
    }
}

impl StepBuilder {
    /// The variables this step can use, as buttons that insert them.
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
                        .id(("step-variable", ix))
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
                .child(div().w_48().child(Input::new(&self.output_name).small()))
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
            .gap_2()
            .child(self.render_choices(window, cx))
            .child(self.render_body(cx))
            .children(self.render_variables(window, cx))
            .children(self.render_output(cx))
            .children(self.render_preview(cx))
    }
}
