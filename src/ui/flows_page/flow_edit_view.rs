// The flow editor: details and triggers on the left, the ordered steps on
// the right. The step list is one tab stop with a roving index.
use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, Icon, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    clipboard::Clipboard,
    h_flex,
    input::{Input, InputEvent, InputState},
    spinner::Spinner,
    switch::Switch,
    v_flex,
};

use crate::system::apps::{DesktopApp, installed_apps};
use crate::system::flows::runner::{Outcome, RunEvent, Runner};
use crate::system::flows::store::{existing_ids, load_flow, load_flows, runs_flow, save_flow};
use crate::system::flows::templates::template;
use crate::system::flows::{Flow, ICONS, OnError, Step, unique_id};
use crate::system::keybinds::chord::Chord;
use crate::system::keybinds::overrides::Override;
use crate::system::keybinds::replay::scan_keybinds;
use crate::system::keybinds::store::{load_overrides, save_overrides};
use crate::system::keybinds::{BindStatus, Dispatcher, Keybind, Origin};
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::dialogs::confirm_dialog::{ConfirmDialog, open_confirm_dialog};
use crate::ui::flows_page::flow_card::icon_tile;
use crate::ui::flows_page::step_dialog::{
    StepDialog, StepDialogEvent, StepDialogMode, open_step_dialog,
};
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::focus::{self, FocusableSwitch};
use crate::ui::keybinds_page::chord_chips::chord_chips;
use crate::ui::keybinds_page::keybind_dialog::{
    DialogMode, KeybindDialog, KeybindDialogEvent, open_keybind_dialog,
};
use crate::ui::keybinds_page::keybinds_view::{FILTERS_CONTEXT, keybinds_nav};
use crate::ui::menu::app_menu;

const KEY_CONTEXT: &str = "FlowEditPage";
/// Wraps the step list: up/down select, Enter edits, Alt+arrows reorder.
pub const STEPS_CONTEXT: &str = "FlowSteps";

pub mod flow_edit_nav {
    gpui::actions!(
        flow_edit,
        [
            Save,
            Run,
            AddStep,
            StepUp,
            StepDown,
            StepFirst,
            StepLast,
            EditStep,
            RemoveStep,
            MoveStepUp,
            MoveStepDown,
            ToggleStep,
            DuplicateStep,
        ]
    );
}
use flow_edit_nav::*;

/// What the editor opens on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowEditSource {
    Existing(String),
    New(Option<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StepState {
    Idle,
    Running,
    Done,
    Failed,
}

enum RunMessage {
    Event(RunEvent),
    Done(Outcome),
}

pub struct FlowEditPage {
    pub focus_handle: FocusHandle,
    /// Everything but the name and description, which live in the inputs.
    flow: Flow,
    /// The flow as last saved; `None` until a new flow is saved.
    saved: Option<Flow>,
    name: Entity<InputState>,
    description: Entity<InputState>,
    icon_focus: FocusHandle,
    steps_focus: FocusHandle,
    selected_step: Option<usize>,
    step_states: Vec<StepState>,
    running: bool,
    apps: Vec<DesktopApp>,
    flows: Vec<Flow>,
    binds: Rc<Vec<Keybind>>,
    /// The chord that runs this flow, and whether Omarchist owns that bind.
    chord: Option<(Chord, bool)>,
    step_dialog: Option<(Entity<StepDialog>, Subscription)>,
    keybind_dialog: Option<(Entity<KeybindDialog>, Subscription)>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl FlowEditPage {
    pub fn new(source: FlowEditSource, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (flow, saved) = match &source {
            FlowEditSource::Existing(id) => match load_flow(id) {
                Ok(flow) => (flow.clone(), Some(flow)),
                Err(e) => {
                    // Opened as a new flow so Save can never replace the
                    // unreadable file; it gets a fresh id.
                    window.push_notification(format!("Could not read the flow: {e}"), cx);
                    (Flow::new(String::new(), id.clone()), None)
                }
            },
            FlowEditSource::New(template_key) => {
                let flow = template_key
                    .as_deref()
                    .and_then(template)
                    .map(|t| t.flow)
                    .unwrap_or_else(|| Flow::new(String::new(), String::new()));
                (flow, None)
            }
        };

        let name = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Name, e.g. Morning start")
                .default_value(flow.name.clone())
        });
        let description = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("What it does (optional)")
                .default_value(flow.description.clone())
        });
        let subscriptions = vec![
            cx.subscribe(&name, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
            cx.subscribe(&description, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
        ];

        let mut page = Self {
            focus_handle: cx.focus_handle(),
            step_states: vec![StepState::Idle; flow.steps.len()],
            flow,
            saved,
            name,
            description,
            icon_focus: focus::tab_stop(cx),
            steps_focus: focus::tab_stop(cx),
            selected_step: None,
            running: false,
            apps: Vec::new(),
            flows: Vec::new(),
            binds: Rc::new(Vec::new()),
            chord: None,
            step_dialog: None,
            keybind_dialog: None,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        };
        page.load_context(cx);
        page
    }

    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.name.update(cx, |input, cx| input.focus(window, cx));
    }

    fn is_new(&self) -> bool {
        self.flow.id.is_empty()
    }

    /// Installed apps and other flows (for step summaries) and the keybinds
    /// (for the trigger row and the keybind dialog), off the UI thread.
    fn load_context(&mut self, cx: &mut Context<Self>) {
        let id = self.flow.id.clone();
        cx.spawn(async move |this, cx| {
            let loaded = cx
                .background_spawn(async { (installed_apps(), load_flows(), scan_keybinds()) })
                .await;
            this.update(cx, |this, cx| {
                let (apps, flows, scan) = loaded;
                this.apps = apps;
                this.flows = flows.unwrap_or_default();
                if let Ok(scan) = scan {
                    this.chord = scan
                        .binds
                        .iter()
                        .find(|b| b.status == BindStatus::Active && runs_flow(&b.dispatcher, &id))
                        .map(|b| (b.chord.clone(), b.origin == Origin::Omarchist));
                    this.binds = Rc::new(scan.binds);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// The flow with the current name and description.
    fn current(&self, cx: &App) -> Flow {
        let mut flow = self.flow.clone();
        flow.name = self.name.read(cx).value().trim().to_string();
        flow.description = self.description.read(cx).value().trim().to_string();
        flow
    }

    fn is_dirty(&self, cx: &App) -> bool {
        match &self.saved {
            Some(saved) => self.current(cx) != *saved,
            None => {
                let current = self.current(cx);
                !current.name.is_empty() || !current.steps.is_empty()
            }
        }
    }

    // MARK: Commands

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut flow = self.current(cx);
        if flow.name.is_empty() {
            window.push_notification("Give the flow a name first", cx);
            self.focus_entry(window, cx);
            return;
        }
        let was_new = self.is_new();
        if was_new {
            flow.id = unique_id(&flow.name, &existing_ids());
        }
        match save_flow(&flow) {
            Ok(()) => {
                window.push_notification(format!("Saved '{}'", flow.name), cx);
                if was_new {
                    // Reopen under the new id so triggers can refer to it.
                    emit(cx, AppEvent::Navigate(ActivePage::FlowEdit(flow.id)));
                } else {
                    self.flow = flow.clone();
                    self.saved = Some(flow);
                    cx.notify();
                }
            }
            Err(e) => window.push_notification(format!("Could not save the flow: {e}"), cx),
        }
    }

    fn run(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.running {
            return;
        }
        let flow = self.current(cx);
        if flow.enabled_steps() == 0 {
            window.push_notification("Add a step to run", cx);
            return;
        }
        self.running = true;
        self.step_states = vec![StepState::Idle; flow.steps.len()];
        cx.notify();

        let (tx, rx) = smol::channel::unbounded::<RunMessage>();
        cx.background_spawn(async move {
            let outcome = Runner::new(true).run(&flow, &mut |event| {
                let _ = tx.send_blocking(RunMessage::Event(event));
            });
            let _ = tx.send_blocking(RunMessage::Done(outcome));
        })
        .detach();

        cx.spawn_in(window, async move |this, cx| {
            while let Ok(message) = rx.recv().await {
                let done = matches!(message, RunMessage::Done(_));
                this.update_in(cx, |this, window, cx| {
                    this.on_run_message(message, window, cx)
                })
                .ok();
                if done {
                    break;
                }
            }
        })
        .detach();
    }

    fn on_run_message(&mut self, message: RunMessage, window: &mut Window, cx: &mut Context<Self>) {
        match message {
            RunMessage::Event(RunEvent::Started { index }) => {
                if let Some(state) = self.step_states.get_mut(index) {
                    *state = StepState::Running;
                }
            }
            RunMessage::Event(RunEvent::Finished { index, error }) => {
                if let Some(state) = self.step_states.get_mut(index) {
                    *state = if error.is_some() {
                        StepState::Failed
                    } else {
                        StepState::Done
                    };
                }
            }
            RunMessage::Done(outcome) => {
                self.running = false;
                window.push_notification(outcome.summary(&self.current(cx)), cx);
            }
        }
        cx.notify();
    }

    fn navigate_back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_dirty(cx) {
            emit(cx, AppEvent::Navigate(ActivePage::Flows));
            return;
        }
        open_confirm_dialog(
            ConfirmDialog {
                title: "Discard changes?",
                message: "This flow has changes that are not saved.".to_string(),
                confirm_label: "Discard",
                danger: true,
            },
            |_, cx| emit(cx, AppEvent::Navigate(ActivePage::Flows)),
            window,
            cx,
        );
    }

    // MARK: Steps

    fn touch_steps(&mut self, cx: &mut Context<Self>) {
        self.step_states = vec![StepState::Idle; self.flow.steps.len()];
        self.selected_step = match self.selected_step {
            Some(ix) if ix < self.flow.steps.len() => Some(ix),
            Some(_) if !self.flow.steps.is_empty() => Some(self.flow.steps.len() - 1),
            _ => None,
        };
        cx.notify();
    }

    fn select_step(&mut self, ix: usize, cx: &mut Context<Self>) {
        if self.flow.steps.is_empty() {
            return;
        }
        self.selected_step = Some(ix.min(self.flow.steps.len() - 1));
        cx.notify();
    }

    fn open_step_dialog(
        &mut self,
        mode: StepDialogMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.step_dialog.is_some() {
            return;
        }
        let initial = match mode {
            StepDialogMode::Edit(ix) => self.flow.steps.get(ix).map(|s| s.kind.clone()),
            StepDialogMode::Add => None,
        };
        let exclude = (!self.is_new()).then(|| self.flow.id.clone());
        let dialog = open_step_dialog(mode, initial.as_ref(), exclude.as_deref(), window, cx);
        let subscription = cx.subscribe_in(
            &dialog,
            window,
            |this, _, event: &StepDialogEvent, window, cx| {
                this.step_dialog = None;
                match event {
                    StepDialogEvent::Save(StepDialogMode::Add, kind) => {
                        this.flow.steps.push(Step::new(kind.clone()));
                        this.selected_step = Some(this.flow.steps.len() - 1);
                        this.touch_steps(cx);
                    }
                    StepDialogEvent::Save(StepDialogMode::Edit(ix), kind) => {
                        if let Some(step) = this.flow.steps.get_mut(*ix) {
                            step.kind = kind.clone();
                        }
                        this.touch_steps(cx);
                    }
                    StepDialogEvent::Cancel => {}
                }
                this.steps_focus.focus(window, cx);
            },
        );
        self.step_dialog = Some((dialog, subscription));
    }

    fn remove_step(&mut self, ix: usize, cx: &mut Context<Self>) {
        if ix < self.flow.steps.len() {
            self.flow.steps.remove(ix);
            self.touch_steps(cx);
        }
    }

    fn move_step(&mut self, ix: usize, delta: isize, cx: &mut Context<Self>) {
        let len = self.flow.steps.len() as isize;
        let target = ix as isize + delta;
        if ix as isize >= len || target < 0 || target >= len {
            return;
        }
        self.flow.steps.swap(ix, target as usize);
        self.selected_step = Some(target as usize);
        self.touch_steps(cx);
    }

    fn toggle_step(&mut self, ix: usize, cx: &mut Context<Self>) {
        if let Some(step) = self.flow.steps.get_mut(ix) {
            step.enabled = !step.enabled;
            self.touch_steps(cx);
        }
    }

    fn duplicate_step(&mut self, ix: usize, cx: &mut Context<Self>) {
        if let Some(step) = self.flow.steps.get(ix).cloned() {
            self.flow.steps.insert(ix + 1, step);
            self.selected_step = Some(ix + 1);
            self.touch_steps(cx);
        }
    }

    // MARK: Triggers

    fn set_icon(&mut self, icon: &str, cx: &mut Context<Self>) {
        self.flow.icon = icon.to_string();
        cx.notify();
    }

    fn cycle_icon(&mut self, delta: isize, cx: &mut Context<Self>) {
        let ix = ICONS.iter().position(|i| *i == self.flow.icon).unwrap_or(0) as isize;
        let next = (ix + delta).rem_euclid(ICONS.len() as isize) as usize;
        self.set_icon(ICONS[next], cx);
    }

    fn assign_keybind(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.keybind_dialog.is_some() {
            return;
        }
        if self.is_new() {
            window.push_notification("Save the flow first, then assign a keybind", cx);
            return;
        }
        let name = self.name.read(cx).value().trim().to_string();
        let dialog = open_keybind_dialog(
            DialogMode::AddPreset {
                dispatcher: Dispatcher::Exec(self.flow.command()),
                description: if name.is_empty() {
                    "Run flow".to_string()
                } else {
                    name
                },
            },
            self.binds.clone(),
            window,
            cx,
        );
        let subscription = cx.subscribe_in(
            &dialog,
            window,
            |this, _, event: &KeybindDialogEvent, window, cx| {
                this.keybind_dialog = None;
                if let KeybindDialogEvent::Save(override_) = event {
                    this.commit_keybind(Some(override_.clone()), window, cx);
                    window.close_dialog(cx);
                }
                // The dialog took focus with it; without a focused element the
                // page's shortcuts would not fire until the next click.
                this.focus_entry(window, cx);
            },
        );
        self.keybind_dialog = Some((dialog, subscription));
    }

    /// Replaces every Omarchist bind that runs this flow with `override_`
    /// (or removes them when `None`), then rescans.
    fn commit_keybind(
        &mut self,
        override_: Option<Override>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = self.flow.id.clone();
        let result = load_overrides().and_then(|mut overrides| {
            overrides
                .overrides
                .retain(|o| o.bind().is_none_or(|b| !runs_flow(&b.dispatcher, &id)));
            if let Some(override_) = override_ {
                overrides.upsert(override_);
            }
            save_overrides(&overrides)
        });
        match result {
            Ok(()) => {
                window.push_notification("Keybind saved", cx);
                self.load_context(cx);
            }
            Err(e) => window.push_notification(format!("Could not save the keybind: {e}"), cx),
        }
    }

    fn set_trigger(
        &mut self,
        launcher: Option<bool>,
        startup: Option<bool>,
        cx: &mut Context<Self>,
    ) {
        if let Some(launcher) = launcher {
            self.flow.triggers.launcher = launcher;
        }
        if let Some(startup) = startup {
            self.flow.triggers.startup = startup;
        }
        cx.notify();
    }

    // MARK: Render

    fn section_title(text: &'static str, cx: &App) -> Div {
        div()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(cx.theme().muted_foreground)
            .child(text)
    }

    fn label(text: &'static str) -> Div {
        div().text_sm().child(text)
    }

    fn card(cx: &App) -> Div {
        let theme = cx.theme();
        v_flex()
            .gap_4()
            .p_4()
            .rounded(theme.radius)
            .border_1()
            .border_color(theme.border)
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let name = self.name.read(cx).value().trim().to_string();
        let dirty = self.is_dirty(cx);
        h_flex()
            .gap_3()
            .items_center()
            .flex_wrap()
            .child(
                Button::new("flow-back")
                    .label("Back")
                    .compact()
                    .tooltip_with_action(
                        "Back to Flows",
                        &app_menu::NavigateBack,
                        Some(KEY_CONTEXT),
                    )
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| this.navigate_back(window, cx))),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_2()
                    .items_center()
                    .child(icon_tile(&self.flow.icon, px(28.), cx))
                    .child(div().font_weight(FontWeight::SEMIBOLD).truncate().child(
                        if name.is_empty() {
                            "New flow".to_string()
                        } else {
                            name
                        },
                    ))
                    .when(dirty, |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child("Unsaved"),
                        )
                    }),
            )
            .child(
                Button::new("flow-run")
                    .compact()
                    .icon(Icon::new(Icon::empty()).path("icons/play.svg"))
                    .label("Run")
                    .loading(self.running)
                    .tooltip_with_action("Run the flow as it is now", &Run, Some(KEY_CONTEXT))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| this.run(window, cx))),
            )
            .child(
                Button::new("flow-save")
                    .primary()
                    .compact()
                    .label("Save")
                    .tooltip_with_action("Save the flow", &Save, Some(KEY_CONTEXT))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
            )
    }

    fn render_icon_picker(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let focused = self.icon_focus.is_focused(window);
        let ring = focus::focus_border(focused, theme.transparent, cx);
        h_flex()
            .id("flow-icons")
            .key_context(FILTERS_CONTEXT)
            .track_focus(&self.icon_focus)
            .on_action(
                cx.listener(|this, _: &keybinds_nav::FilterPrev, _, cx| this.cycle_icon(-1, cx)),
            )
            .on_action(
                cx.listener(|this, _: &keybinds_nav::FilterNext, _, cx| this.cycle_icon(1, cx)),
            )
            .flex_wrap()
            .gap_1()
            .p_1()
            .rounded(theme.radius)
            .border_1()
            .border_color(ring)
            .children(ICONS.iter().enumerate().map(|(ix, &icon)| {
                let selected = self.flow.icon == icon;
                let button = Button::new(("flow-icon", ix))
                    .icon(Icon::new(Icon::empty()).path(format!("icons/{icon}.svg")))
                    .small()
                    .tab_stop(false)
                    .cursor_pointer();
                let button = if selected {
                    button.primary()
                } else {
                    button.ghost()
                };
                button.on_click(cx.listener(move |this, _, _, cx| this.set_icon(icon, cx)))
            }))
    }

    fn render_details(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        Self::card(cx)
            .child(Self::section_title("DETAILS", cx))
            .child(
                v_flex()
                    .gap_1()
                    .child(Self::label("Icon"))
                    .child(self.render_icon_picker(window, cx)),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(Self::label("Name"))
                    .child(Input::new(&self.name).small()),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(Self::label("Description"))
                    .child(Input::new(&self.description).small()),
            )
            .child(
                div().text_sm().child(
                    FocusableSwitch::new("flow-on-error")
                        .label("Keep going when a step fails")
                        .checked(self.flow.on_error == OnError::Continue)
                        .on_change(cx.listener(|this, checked, _, cx| {
                            this.flow.on_error = if *checked {
                                OnError::Continue
                            } else {
                                OnError::Stop
                            };
                            cx.notify();
                        })),
                ),
            )
    }

    fn render_triggers(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_new = self.is_new();
        let command = if is_new {
            "Save the flow to get its command".to_string()
        } else {
            self.flow.command()
        };
        let keybind_row = match &self.chord {
            Some((chord, owned)) => h_flex()
                .gap_2()
                .items_center()
                .flex_wrap()
                .child(chord_chips(chord, false, cx))
                .when(*owned, |this| {
                    this.child(
                        Button::new("flow-keybind-change")
                            .outline()
                            .xsmall()
                            .label("Change")
                            .cursor_pointer()
                            .on_click(
                                cx.listener(|this, _, window, cx| this.assign_keybind(window, cx)),
                            ),
                    )
                    .child(
                        Button::new("flow-keybind-remove")
                            .ghost()
                            .xsmall()
                            .label("Remove")
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.commit_keybind(None, window, cx)
                            })),
                    )
                })
                .when(!*owned, |this| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Set in your bindings.lua"),
                    )
                }),
            None => h_flex()
                .gap_2()
                .items_center()
                .child(
                    Button::new("flow-keybind-assign")
                        .outline()
                        .xsmall()
                        .icon(Icon::new(Icon::empty()).path("icons/keyboard.svg"))
                        .label("Assign a keybind")
                        .disabled(is_new)
                        .cursor_pointer()
                        .on_click(
                            cx.listener(|this, _, window, cx| this.assign_keybind(window, cx)),
                        ),
                )
                .when(is_new, |this| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("after saving"),
                    )
                }),
        };

        Self::card(cx)
            .child(Self::section_title("RUN IT FROM", cx))
            .child(
                v_flex()
                    .gap_1()
                    .child(Self::label("Keybind"))
                    .child(keybind_row),
            )
            .child(
                div().text_sm().child(
                    FocusableSwitch::new("flow-trigger-launcher")
                        .label("App launcher")
                        .checked(self.flow.triggers.launcher)
                        .on_change(cx.listener(|this, checked, _, cx| {
                            this.set_trigger(Some(*checked), None, cx)
                        })),
                ),
            )
            .child(
                div().text_sm().child(
                    FocusableSwitch::new("flow-trigger-startup")
                        .label("At startup")
                        .checked(self.flow.triggers.startup)
                        .on_change(cx.listener(|this, checked, _, cx| {
                            this.set_trigger(None, Some(*checked), cx)
                        })),
                ),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(Self::label("Command line"))
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .px_2()
                                    .py_1()
                                    .rounded(theme.radius)
                                    .bg(theme.secondary)
                                    .font_family("monospace")
                                    .text_xs()
                                    .truncate()
                                    .text_color(if is_new {
                                        theme.muted_foreground
                                    } else {
                                        theme.foreground
                                    })
                                    .child(command),
                            )
                            .when(!is_new, |this| {
                                this.child(
                                    Clipboard::new("flow-copy-command")
                                        .value(self.flow.command())
                                        .tooltip("Copy the command")
                                        .on_copied(|_, window, cx| {
                                            window.push_notification("Command copied", cx)
                                        }),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Works from any script, terminal, or keybind."),
                    ),
            )
    }

    fn render_step(
        &self,
        ix: usize,
        step: &Step,
        summaries: &SummaryContext,
        list_focused: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let summary = summaries.summarize(&step.kind);
        let selected = list_focused && self.selected_step == Some(ix);
        let state = self.step_states.get(ix).copied().unwrap_or(StepState::Idle);
        let count = self.flow.steps.len();
        let state_icon: Option<AnyElement> = match state {
            StepState::Idle => None,
            StepState::Running => Some(Spinner::new().small().into_any_element()),
            StepState::Done => Some(
                Icon::new(Icon::empty())
                    .path("icons/circle-check.svg")
                    .size_4()
                    .text_color(theme.success)
                    .into_any_element(),
            ),
            StepState::Failed => Some(
                Icon::new(Icon::empty())
                    .path("icons/circle-x.svg")
                    .size_4()
                    .text_color(theme.danger)
                    .into_any_element(),
            ),
        };

        h_flex()
            .id(("flow-step", ix))
            .gap_3()
            .items_center()
            .p_3()
            .rounded(theme.radius)
            .border_1()
            .border_color(if selected { theme.ring } else { theme.border })
            .bg(if selected {
                theme.secondary
            } else {
                theme.background
            })
            .hover(|s| s.bg(theme.secondary))
            .cursor_pointer()
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                this.selected_step = Some(ix);
                this.steps_focus.focus(window, cx);
                if event.click_count() >= 2 {
                    this.open_step_dialog(StepDialogMode::Edit(ix), window, cx);
                } else {
                    cx.notify();
                }
            }))
            .child(
                div()
                    .size_6()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .bg(theme.secondary)
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.muted_foreground)
                    .child((ix + 1).to_string()),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .opacity(if step.enabled { 1. } else { 0.5 })
                    .child(summary.icon.render(px(20.))),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_0p5()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .truncate()
                            .text_color(if step.enabled {
                                theme.foreground
                            } else {
                                theme.muted_foreground
                            })
                            .child(summary.title),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_family("monospace")
                            .text_color(theme.muted_foreground)
                            .truncate()
                            .child(summary.detail),
                    ),
            )
            .children(state_icon)
            .child(
                h_flex()
                    .gap_0p5()
                    .flex_shrink_0()
                    .child(
                        Button::new(("step-up", ix))
                            .ghost()
                            .xsmall()
                            .tab_stop(false)
                            .disabled(ix == 0)
                            .icon(Icon::new(Icon::empty()).path("icons/arrow-up.svg"))
                            .tooltip("Move up")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.move_step(ix, -1, cx);
                            })),
                    )
                    .child(
                        Button::new(("step-down", ix))
                            .ghost()
                            .xsmall()
                            .tab_stop(false)
                            .disabled(ix + 1 >= count)
                            .icon(Icon::new(Icon::empty()).path("icons/arrow-down.svg"))
                            .tooltip("Move down")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.move_step(ix, 1, cx);
                            })),
                    )
                    .child(
                        Button::new(("step-edit", ix))
                            .ghost()
                            .xsmall()
                            .tab_stop(false)
                            .icon(Icon::new(Icon::empty()).path("icons/pencil.svg"))
                            .tooltip("Edit")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.open_step_dialog(StepDialogMode::Edit(ix), window, cx);
                            })),
                    )
                    .child(
                        Button::new(("step-remove", ix))
                            .ghost()
                            .xsmall()
                            .tab_stop(false)
                            .icon(Icon::new(Icon::empty()).path("icons/trash.svg"))
                            .tooltip("Remove")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.remove_step(ix, cx);
                            })),
                    )
                    .child(
                        div().ml_1().child(
                            Switch::new(("step-enabled", ix))
                                .small()
                                .checked(step.enabled)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    cx.stop_propagation();
                                    this.toggle_step(ix, cx);
                                })),
                        ),
                    ),
            )
    }

    fn render_steps(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let list_focused = self.steps_focus.is_focused(window);
        let ring = focus::focus_border(list_focused, theme.transparent, cx);
        let summaries = SummaryContext {
            apps: &self.apps,
            flows: &self.flows,
        };
        let count = self.flow.steps.len();
        let mut list = v_flex()
            .id("flow-steps")
            .key_context(STEPS_CONTEXT)
            .track_focus(&self.steps_focus)
            .rounded(theme.radius)
            .border_1()
            .border_color(ring)
            .p_0p5();
        for (ix, step) in self.flow.steps.iter().enumerate() {
            list = list.child(self.render_step(ix, step, &summaries, list_focused, cx));
            if ix + 1 < count {
                // Connector under the step number.
                list = list.child(div().ml(px(25.)).w(px(2.)).h(px(12.)).bg(theme.border));
            }
        }
        if count == 0 {
            list = list.child(
                v_flex()
                    .items_center()
                    .py_6()
                    .gap_1()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("No steps yet")
                    .child(
                        div()
                            .text_xs()
                            .child("Add the first thing this flow should do."),
                    ),
            );
        }

        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(Self::section_title("STEPS", cx))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Run in order, top to bottom"),
                    ),
            )
            .child(list)
            .child(
                Button::new("flow-add-step")
                    .outline()
                    .w_full()
                    .icon(Icon::new(Icon::empty()).path("icons/plus.svg"))
                    .label("Add step")
                    .tooltip_with_action("Add a step", &AddStep, Some(KEY_CONTEXT))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.open_step_dialog(StepDialogMode::Add, window, cx)
                    })),
            )
    }
}

impl Render for FlowEditPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let background = cx.theme().background;
        let wide = window.viewport_size().width >= px(1024.);

        let details = v_flex()
            .gap_4()
            .when(wide, |this| this.w(px(380.)).flex_shrink_0())
            .child(self.render_details(window, cx))
            .child(self.render_triggers(cx));
        let steps = div()
            .flex_1()
            .min_w_0()
            .child(self.render_steps(window, cx));
        let columns = if wide {
            h_flex().items_start().gap_6().child(details).child(steps)
        } else {
            v_flex().gap_6().child(details).child(steps)
        };

        v_flex()
            .id("flow-edit-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(background)
            .gap_4()
            .on_action(cx.listener(|this, _: &app_menu::NavigateBack, window, cx| {
                this.navigate_back(window, cx);
            }))
            .on_action(cx.listener(|this, _: &Save, window, cx| this.save(window, cx)))
            .on_action(cx.listener(|this, _: &Run, window, cx| this.run(window, cx)))
            .on_action(cx.listener(|this, _: &AddStep, window, cx| {
                this.open_step_dialog(StepDialogMode::Add, window, cx);
            }))
            .on_action(cx.listener(|this, _: &StepUp, _, cx| {
                let ix = this.selected_step.unwrap_or(0);
                this.select_step(ix.saturating_sub(1), cx);
            }))
            .on_action(cx.listener(|this, _: &StepDown, _, cx| {
                let ix = this.selected_step.map(|ix| ix + 1).unwrap_or(0);
                this.select_step(ix, cx);
            }))
            .on_action(cx.listener(|this, _: &StepFirst, _, cx| this.select_step(0, cx)))
            .on_action(cx.listener(|this, _: &StepLast, _, cx| this.select_step(usize::MAX, cx)))
            .on_action(cx.listener(|this, _: &EditStep, window, cx| {
                if let Some(ix) = this.selected_step {
                    this.open_step_dialog(StepDialogMode::Edit(ix), window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &RemoveStep, _, cx| {
                if let Some(ix) = this.selected_step {
                    this.remove_step(ix, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &MoveStepUp, _, cx| {
                if let Some(ix) = this.selected_step {
                    this.move_step(ix, -1, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &MoveStepDown, _, cx| {
                if let Some(ix) = this.selected_step {
                    this.move_step(ix, 1, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &ToggleStep, _, cx| {
                if let Some(ix) = this.selected_step {
                    this.toggle_step(ix, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &DuplicateStep, _, cx| {
                if let Some(ix) = this.selected_step {
                    this.duplicate_step(ix, cx);
                }
            }))
            .child(self.render_header(cx))
            .child(
                div()
                    .id("flow-edit-content")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .pb_8()
                    .child(columns),
            )
    }
}
