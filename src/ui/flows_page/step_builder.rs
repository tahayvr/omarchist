//! The controls that assemble one flow step: the keybind action builder's
//! kinds plus the flow-only Wait and Notify.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

use crate::system::flows::{StepKind, format_duration};
use crate::system::keybinds::action::{Action, ActionKind};
use crate::ui::focus::{self, FocusableSwitch};
use crate::ui::keybinds_page::action_builder::{ActionBuilder, ActionBuilderEvent};
use crate::ui::keybinds_page::keybinds_view::{FILTERS_CONTEXT, keybinds_nav};

pub enum StepBuilderEvent {
    Changed,
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
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<StepBuilderEvent> for StepBuilder {}

impl StepBuilder {
    /// `exclude_flow` keeps the flow being edited out of the Flow picker.
    pub fn new(
        initial: Option<&StepKind>,
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
        let wait_ms = text(window, cx, "Milliseconds", &wait_value);
        let notify_title = text(window, cx, "Title", &title_value);
        let notify_body = text(window, cx, "Message (optional)", &body_value);

        let mut subscriptions = vec![cx.subscribe_in(
            &action,
            window,
            |this, _, event: &ActionBuilderEvent, _window, cx| {
                let ActionBuilderEvent::Changed = event;
                this.changed(cx);
            },
        )];
        for input in [&wait_ms, &notify_title, &notify_body] {
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

        Self {
            choice,
            kind_focus: focus::tab_stop(cx),
            action,
            wait,
            wait_ms,
            notify_title,
            notify_body,
            _subscriptions: subscriptions,
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
            Ok(ms) if ms > 0 => Ok(ms),
            _ => Err("Enter how long to wait, in milliseconds".to_string()),
        }
    }

    /// The step the controls currently describe, or why they don't.
    pub fn step(&self, cx: &App) -> Result<StepKind, String> {
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

    fn hint(text: &'static str, cx: &App) -> Div {
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(text)
    }

    fn render_body(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.choice {
            StepChoice::Action(kind) => v_flex()
                .gap_2()
                .child(self.action.clone())
                .when(kind != ActionKind::Window && kind != ActionKind::Flow, |this| {
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
                    .child(Self::hint(
                        "Off, the command is started and the flow moves on, which is what \
                         opening an app needs. On, the flow waits for it to exit and treats \
                         a failure as the step failing.",
                        cx,
                    ))
                })
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
                            .child(div().w_32().child(Input::new(&self.wait_ms).small()))
                            .child(Self::hint("ms", cx))
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
                    .child(Self::hint(
                        "Gives the previous step time to finish, such as a window appearing \
                         before the next step moves it.",
                        cx,
                    ))
                    .into_any_element()
            }
            StepChoice::Notify => v_flex()
                .gap_2()
                .child(Input::new(&self.notify_title).small())
                .child(Input::new(&self.notify_body).small())
                .child(Self::hint(
                    "Shows a desktop notification, handy as the last step so you know the flow ran.",
                    cx,
                ))
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
                        .font_family("monospace")
                        .child(step.text()),
                )
                .into_any_element(),
            Err(message) => row
                .child(div().text_color(theme.muted_foreground).child(message))
                .into_any_element(),
        })
    }
}

impl Render for StepBuilder {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(self.render_choices(window, cx))
            .child(self.render_body(cx))
            .children(self.render_preview(cx))
    }
}
