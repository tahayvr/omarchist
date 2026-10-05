// The flow editor: details and triggers on the left, the ordered steps on
// the right. The step list is one tab stop with a roving index.
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, Icon, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    clipboard::Clipboard,
    h_flex,
    input::{Input, InputEvent, InputState},
    menu::DropdownMenu,
    popover::Popover,
    switch::Switch,
    v_flex,
};

use crate::system::apps::{DesktopApp, installed_apps};
use crate::system::config::hypr_setup::HOOK_RESTORED_MESSAGE;
use crate::system::flows::automations::Event;
use crate::system::flows::history::{self, Recorder};
use crate::system::flows::requirements::missing_programs;
use crate::system::flows::runner::{Cancel, Outcome, RunEvent, Runner, StepStatus};
use crate::system::flows::service;
use crate::system::flows::share::Imported;
use crate::system::flows::store::{
    existing_ids, load_flow, load_flows, runs_flow, save_flow, save_new_flow,
};
use crate::system::flows::templates::template;
use crate::system::flows::{
    Flow, ICONS, InputFallback, OnError, Step, StepKind, StepPath, unique_id,
};
use crate::system::keybinds::chord::Chord;
use crate::system::keybinds::overrides::Override;
use crate::system::keybinds::replay::scan_keybinds;
use crate::system::keybinds::store::{load_overrides, save_overrides};
use crate::system::keybinds::{BindStatus, Dispatcher, Keybind, Origin};
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::flows_page::automation_dialog::{
    AutomationDialog, AutomationDialogEvent, EventKind, open_automation_dialog,
};
use crate::ui::flows_page::flow_card::icon_tile;
use crate::ui::flows_page::history_dialog::open_history_dialog;
use crate::ui::flows_page::share_ui::{export_flow, warning_banner};
use crate::ui::flows_page::step_dialog::{
    StepDialog, StepDialogEvent, StepDialogMode, open_step_dialog,
};
use crate::ui::flows_page::step_list::RowKey;
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::focus::{self, FocusableSwitch};
use crate::ui::keybinds_page::chord_chips::chord_chips;
use crate::ui::keybinds_page::keybind_dialog::{
    DialogMode, KeybindDialog, KeybindDialogEvent, open_keybind_dialog,
};
use crate::ui::keybinds_page::keybinds_view::{FILTERS_CONTEXT, keybinds_nav};
use crate::ui::menu::app_menu;
use crate::ui::text::selectable;
use gpui_kit::TestSupportExt;

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
            CollapseStep,
            ExpandStep,
            ShowHistory,
            Export,
        ]
    );
}
use flow_edit_nav::*;

/// What the editor opens on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowEditSource {
    Existing(String),
    New(Option<String>),
    /// A flow from a file or URL, shown for review before its first save.
    Imported(Box<Imported>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum StepState {
    Idle,
    Running,
    /// With what the step produced, shown under it until the next run or
    /// edit.
    Done(Option<String>),
    /// Why, shown under the step until the next run or edit.
    Failed(String),
    /// A prompt was dismissed here, which ended the run.
    Cancelled,
}

/// Progress from the runner thread, tagged with the run it belongs to so
/// a run left behind by an earlier press cannot mark the wrong rows.
enum RunMessage {
    Event(u64, RunEvent),
    Done(u64, Outcome),
}

pub struct FlowEditPage {
    pub focus_handle: FocusHandle,
    /// Everything but the name and description, which live in the inputs.
    pub(super) flow: Flow,
    /// What the editor opened with or last saved; the dirty check compares
    /// against it, so a template or an import is not "unsaved" until it is
    /// edited.
    baseline: Flow,
    /// The user chose to leave without saving; nothing counts as unsaved.
    discarded: bool,
    name: Entity<InputState>,
    description: Entity<InputState>,
    icon_focus: FocusHandle,
    input_focus: FocusHandle,
    pub(super) steps_focus: FocusHandle,
    /// The line of the step list the keyboard is on.
    pub(super) selected: Option<RowKey>,
    /// Where that line was drawn, for scrolling it into view.
    pub(super) selected_bounds: Rc<Cell<Bounds<Pixels>>>,
    /// How each step of the last run ended, by its path.
    pub(super) step_states: HashMap<StepPath, StepState>,
    /// The round a loop step is on in the current run, and of how many.
    pub(super) rounds: HashMap<StepPath, (u32, u32)>,
    /// Blocks whose steps are hidden, by the block's `uid`.
    pub(super) collapsed: HashSet<u64>,
    running: bool,
    /// Counts runs; a message from an older run is ignored.
    run_id: u64,
    /// Stops the current run from the Stop button.
    cancel: Option<Cancel>,
    pub(super) apps: Vec<DesktopApp>,
    pub(super) flows: Vec<Flow>,
    binds: Rc<Vec<Keybind>>,
    /// The chord that runs this flow, and whether Omarchist owns that bind.
    chord: Option<(Chord, bool)>,
    step_dialog: Option<(Entity<StepDialog>, Subscription)>,
    automation_dialog: Option<(Entity<AutomationDialog>, Subscription)>,
    /// Whether the background service that runs automations is on.
    service_on: bool,
    /// The service is being turned on.
    service_pending: bool,
    keybind_dialog: Option<(Entity<KeybindDialog>, Subscription)>,
    pub(super) scroll: ScrollHandle,
    /// Where an imported flow came from, shown until it is saved.
    pub(super) import_origin: Option<String>,
    /// Programs the flow needs that are not on this machine.
    pub(super) missing: Vec<String>,
    _subscriptions: Vec<Subscription>,
}

impl FlowEditPage {
    pub fn new(source: FlowEditSource, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let flow = match &source {
            FlowEditSource::Existing(id) => match load_flow(id) {
                Ok(flow) => flow,
                Err(e) => {
                    // The file is on the page as broken; there is nothing
                    // to edit, and a new flow under its name would replace
                    // it. Back to the list.
                    window.push_notification(format!("Could not read the flow: {e}"), cx);
                    emit(cx, AppEvent::Navigate(ActivePage::Flows));
                    Flow::new(String::new(), String::new())
                }
            },
            FlowEditSource::New(template_key) => template_key
                .as_deref()
                .and_then(template)
                .map(|t| t.flow)
                .unwrap_or_else(|| Flow::new(String::new(), String::new())),
            FlowEditSource::Imported(imported) => imported.flow.clone(),
        };
        let baseline = flow.clone();
        let import_origin = match &source {
            FlowEditSource::Imported(imported) => Some(imported.origin.clone()),
            _ => None,
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
            step_states: HashMap::new(),
            rounds: HashMap::new(),
            collapsed: HashSet::new(),
            selected_bounds: Rc::new(Cell::new(Bounds::default())),
            flow,
            baseline,
            discarded: false,
            name,
            description,
            icon_focus: focus::tab_stop(cx),
            input_focus: focus::tab_stop(cx),
            steps_focus: focus::tab_stop(cx),
            selected: None,
            running: false,
            run_id: 0,
            cancel: None,
            apps: Vec::new(),
            flows: Vec::new(),
            binds: Rc::new(Vec::new()),
            chord: None,
            step_dialog: None,
            automation_dialog: None,
            service_on: false,
            service_pending: false,
            keybind_dialog: None,
            scroll: ScrollHandle::new(),
            import_origin,
            missing: Vec::new(),
            _subscriptions: subscriptions,
        };
        page.missing = missing_programs(&page.flow);
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
                .background_spawn(async {
                    (
                        installed_apps(),
                        load_flows(),
                        scan_keybinds(),
                        service::is_enabled(),
                    )
                })
                .await;
            this.update(cx, |this, cx| {
                let (apps, flows, scan, service_on) = loaded;
                this.service_on = service_on;
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
    pub fn current(&self, cx: &App) -> Flow {
        let mut flow = self.flow.clone();
        flow.name = self.name.read(cx).value().trim().to_string();
        flow.description = self.description.read(cx).value().trim().to_string();
        flow
    }

    /// Whether the editor holds changes that are not saved.
    pub fn is_dirty(&self, cx: &App) -> bool {
        if self.discarded {
            return false;
        }
        self.current(cx) != self.baseline
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
        let result = if was_new {
            flow.id = unique_id(&flow.name, &existing_ids());
            save_new_flow(&flow)
        } else {
            save_flow(&flow)
        };
        match result {
            Ok(()) => {
                window.push_notification(format!("Saved '{}'", flow.name), cx);
                // Clean before anything navigates, or the reopen below would
                // ask to discard the flow that was just saved.
                self.flow = flow.clone();
                self.baseline = flow.clone();
                cx.notify();
                if was_new {
                    // Reopen under the new id so triggers can refer to it.
                    emit(cx, AppEvent::Navigate(ActivePage::FlowEdit(flow.id)));
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
        if let Err(e) = flow.validate_content() {
            window.push_notification(format!("Cannot run the flow: {e}"), cx);
            return;
        }
        self.running = true;
        self.run_id += 1;
        let run_id = self.run_id;
        let cancel = Cancel::new();
        self.cancel = Some(cancel.clone());
        self.step_states.clear();
        self.rounds.clear();
        cx.notify();

        let (tx, rx) = smol::channel::unbounded::<RunMessage>();
        // A thread of its own: a run blocks (waits, waited commands) for as
        // long as the flow takes, which is not what the executor's pool is
        // for.
        std::thread::spawn(move || {
            let mut recorder = Recorder::new(&flow, "Editor");
            let outcome = Runner::new(true)
                .cancellable(cancel.clone())
                .run(&flow, &mut |event| {
                    recorder.event(&event);
                    let _ = tx.send_blocking(RunMessage::Event(run_id, event));
                });
            // A flow that was never saved has no history to add to.
            let run = recorder.finish(&flow, &outcome, cancel.is_cancelled());
            if let Err(e) = history::record(&flow.id, &run) {
                eprintln!("{e}");
            }
            let _ = tx.send_blocking(RunMessage::Done(run_id, outcome));
        });

        cx.spawn_in(window, async move |this, cx| {
            while let Ok(message) = rx.recv().await {
                let done = matches!(message, RunMessage::Done(..));
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

    /// Stops the run after the current step; a command being waited for
    /// is killed.
    fn stop(&mut self, cx: &mut Context<Self>) {
        if let Some(cancel) = &self.cancel {
            cancel.cancel();
        }
        cx.notify();
    }

    fn on_run_message(&mut self, message: RunMessage, window: &mut Window, cx: &mut Context<Self>) {
        let (run_id, event) = match message {
            RunMessage::Event(run_id, event) => (run_id, Some(event)),
            RunMessage::Done(run_id, outcome) => {
                if run_id == self.run_id {
                    self.running = false;
                    self.cancel = None;
                    window.push_notification(outcome.summary(&self.current(cx)), cx);
                    cx.notify();
                }
                return;
            }
        };
        if run_id != self.run_id {
            return;
        }
        match event {
            Some(RunEvent::Started { path }) => {
                self.step_states.insert(path, StepState::Running);
            }
            Some(RunEvent::Finished { path, status }) => {
                let state = match status {
                    StepStatus::Done(output) => StepState::Done(output),
                    StepStatus::Failed(error) => StepState::Failed(error),
                    StepStatus::Cancelled => StepState::Cancelled,
                };
                self.rounds.remove(&path);
                self.step_states.insert(path, state);
            }
            Some(RunEvent::Round { path, round, of }) => {
                // The steps inside start over, so their marks do too.
                self.step_states
                    .retain(|step, _| !(step.len() > path.len() && step.starts_with(&path)));
                self.rounds.insert(path, (round, of));
            }
            None => {}
        }
        cx.notify();
    }

    /// Editing the steps while they run would desynchronise the live
    /// states (they are kept by position).
    pub(super) fn refuse_while_running(&self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.running {
            window.push_notification("Stop the flow before changing its steps", cx);
        }
        self.running
    }

    /// Lets the next navigation leave without asking again.
    pub fn discard(&mut self) {
        self.discarded = true;
    }

    /// The app asks before discarding unsaved changes on any navigation
    /// away from the editor (`MainWindowView::navigate_to`).
    fn navigate_back(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        emit(cx, AppEvent::Navigate(ActivePage::Flows));
    }

    /// Opens the flow's run history. A flow that was never saved has none.
    fn show_history(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_new() {
            window.push_notification("Save the flow to keep a history of its runs", cx);
            return;
        }
        let name = self.name.read(cx).value().trim().to_string();
        open_history_dialog(&self.flow.id, &name, window, cx);
    }

    // MARK: Sharing

    fn export(&self, window: &mut Window, cx: &mut Context<Self>) {
        let mut flow = self.current(cx);
        if flow.name.is_empty() {
            window.push_notification("Give the flow a name first", cx);
            return;
        }
        if flow.id.is_empty() {
            flow.id = unique_id(&flow.name, &[]);
        }
        export_flow(&flow, window, cx);
    }

    // MARK: Steps (the list itself lives in `step_list.rs`)

    pub(super) fn open_step_dialog(
        &mut self,
        mode: StepDialogMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.step_dialog.is_some() || self.refuse_while_running(window, cx) {
            return;
        }
        let (initial, output) = match &mode {
            StepDialogMode::Edit(path) => match self.flow.step_at(path) {
                Some(step) => (Some(step.kind.clone()), step.output.clone()),
                None => return,
            },
            StepDialogMode::Add { .. } => (None, None),
        };
        // An action from a newer Omarchist has no form here.
        if let Some(StepKind::Action { action, .. }) = &initial
            && crate::system::flows::actions::find(action).is_none()
        {
            window.push_notification("This step needs a newer Omarchist to edit", cx);
            return;
        }
        // A step can use what the steps written before it save, and what
        // the loops around it set.
        let saved = match &mode {
            StepDialogMode::Edit(path) => match path.split_last() {
                Some((index, list)) => self.flow.names_at(list, *index),
                None => Vec::new(),
            },
            StepDialogMode::Add { list, index } => self.flow.names_at(list, *index),
        };
        let exclude = (!self.is_new()).then(|| self.flow.id.clone());
        let dialog = open_step_dialog(
            mode,
            initial.as_ref(),
            output.as_deref(),
            &saved,
            exclude.as_deref(),
            window,
            cx,
        );
        let subscription = cx.subscribe_in(
            &dialog,
            window,
            |this, _, event: &StepDialogEvent, window, cx| {
                this.step_dialog = None;
                this.steps_focus.focus(window, cx);
                match event {
                    StepDialogEvent::Save(StepDialogMode::Add { list, index }, kind, output) => {
                        let mut step = Step::new(kind.clone());
                        step.output = output.clone();
                        this.insert_step(list, *index, step, window, cx);
                    }
                    StepDialogEvent::Save(StepDialogMode::Edit(path), kind, output) => {
                        this.replace_step(path, kind.clone(), output.clone(), cx);
                    }
                    StepDialogEvent::Cancel => {}
                }
            },
        );
        self.step_dialog = Some((dialog, subscription));
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
                if let KeybindDialogEvent::Save { override_, .. } = event {
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
        let removing = override_.is_none();
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
            Ok(hook_restored) => {
                crate::system::hyprland_config::manager::reload_hyprland();
                window.push_notification(
                    if removing {
                        "Keybind removed"
                    } else {
                        "Keybind saved"
                    },
                    cx,
                );
                if hook_restored {
                    window.push_notification(HOOK_RESTORED_MESSAGE, cx);
                }
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

    // MARK: Automations

    fn open_automation_dialog(
        &mut self,
        index: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.automation_dialog.is_some() {
            return;
        }
        let initial = index
            .and_then(|ix| self.flow.triggers.automations.get(ix))
            .cloned();
        let dialog = open_automation_dialog(index, initial.as_ref(), window, cx);
        let subscription = cx.subscribe_in(
            &dialog,
            window,
            |this, _, event: &AutomationDialogEvent, window, cx| {
                this.automation_dialog = None;
                if let AutomationDialogEvent::Save(index, automation) = event {
                    let automations = &mut this.flow.triggers.automations;
                    match index.and_then(|ix| automations.get_mut(ix)) {
                        Some(slot) => *slot = automation.clone(),
                        None => automations.push(automation.clone()),
                    }
                    cx.notify();
                }
                this.focus_handle.focus(window, cx);
            },
        );
        self.automation_dialog = Some((dialog, subscription));
    }

    /// Installs and starts the background service, off the UI thread.
    fn turn_on_service(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.service_pending {
            return;
        }
        self.service_pending = true;
        cx.notify();
        let task = cx.background_spawn(async { service::enable() });
        cx.spawn_in(window, async move |this, cx| {
            let result = task.await;
            this.update_in(cx, |this, window, cx| {
                this.service_pending = false;
                this.service_on = result.is_ok();
                window.push_notification(
                    match result {
                        Ok(()) => "Automations are on".to_string(),
                        Err(e) => format!("Could not turn automations on: {e}"),
                    },
                    cx,
                );
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// An automation as a sentence, with an app by its name.
    fn describe_automation(&self, event: &Event) -> String {
        let summaries = SummaryContext {
            apps: &self.apps,
            flows: &self.flows,
        };
        match event {
            Event::AppOpened { class } => {
                format!("When {} opens", summaries.app_name_for_class(class))
            }
            Event::AppClosed { class } => {
                format!("When {} closes", summaries.app_name_for_class(class))
            }
            other => other.describe(),
        }
    }

    fn render_automations(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let automations = &self.flow.triggers.automations;
        let rows = automations.iter().enumerate().map(|(ix, automation)| {
            let kind = EventKind::of(&automation.event);
            h_flex()
                .id(("flow-automation", ix))
                .test_support()
                .gap_2()
                .items_center()
                .child(
                    Icon::new(Icon::empty())
                        .path(kind.icon())
                        .size_4()
                        .flex_shrink_0()
                        .text_color(theme.muted_foreground),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_sm()
                        .when(!automation.enabled, |this| {
                            this.text_color(theme.muted_foreground)
                        })
                        .child(selectable(
                            ("automation-text", ix),
                            self.describe_automation(&automation.event),
                        )),
                )
                .when(automation.ask, |this| {
                    this.child(
                        div()
                            .flex_shrink_0()
                            .px_1p5()
                            .rounded(theme.radius)
                            .bg(theme.secondary)
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("asks"),
                    )
                })
                .child(
                    Button::new(("automation-edit", ix))
                        .ghost()
                        .xsmall()
                        .icon(Icon::new(Icon::empty()).path("icons/pencil.svg"))
                        .tooltip("Edit")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.open_automation_dialog(Some(ix), window, cx)
                        })),
                )
                .child(
                    Button::new(("automation-remove", ix))
                        .ghost()
                        .xsmall()
                        .icon(Icon::new(Icon::empty()).path("icons/trash.svg"))
                        .tooltip("Remove")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if ix < this.flow.triggers.automations.len() {
                                this.flow.triggers.automations.remove(ix);
                                cx.notify();
                            }
                        })),
                )
                .child(
                    Switch::new(("automation-enabled", ix))
                        .small()
                        .checked(automation.enabled)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(automation) = this.flow.triggers.automations.get_mut(ix) {
                                automation.enabled = !automation.enabled;
                                cx.notify();
                            }
                        })),
                )
        });
        let service_off = !automations.is_empty() && !self.service_on;
        v_flex()
            .gap_2()
            .child(Self::label("Automations"))
            .children(rows)
            .when(service_off, |this| {
                this.child(
                    h_flex()
                        .id("automations-service-off")
                        .test_support()
                        .gap_2()
                        .items_center()
                        .flex_wrap()
                        .p_2()
                        .rounded(theme.radius)
                        .bg(theme.warning.opacity(0.1))
                        .text_xs()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(selectable("service-off", "The background service is off")),
                        )
                        .child(
                            Button::new("automations-turn-on")
                                .xsmall()
                                .outline()
                                .label("Turn on")
                                .loading(self.service_pending)
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.turn_on_service(window, cx)
                                })),
                        ),
                )
            })
            .child(
                h_flex().child(
                    Button::new("flow-add-automation")
                        .outline()
                        .xsmall()
                        .icon(Icon::new(Icon::empty()).path("icons/plus.svg"))
                        .label("Add automation")
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.open_automation_dialog(None, window, cx)
                        })),
                ),
            )
    }

    // MARK: Render

    pub(super) fn section_title(text: &'static str, cx: &App) -> Div {
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

    /// Who made the flow and where it came from, when the file says.
    fn render_meta(&self, cx: &App) -> Option<impl IntoElement> {
        let meta = &self.flow.meta;
        let mut parts: Vec<String> = Vec::new();
        if !meta.author.is_empty() {
            parts.push(format!("By {}", meta.author));
        }
        if !meta.version.is_empty() {
            parts.push(format!("version {}", meta.version));
        }
        if !meta.homepage.is_empty() {
            parts.push(meta.homepage.clone());
        }
        if !meta.source.is_empty() {
            parts.push(format!("imported from {}", meta.source));
        }
        if parts.is_empty() {
            return None;
        }
        Some(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(selectable("flow-meta", parts.join(" · "))),
        )
    }

    fn render_import_banner(&self, cx: &App) -> Option<impl IntoElement> {
        let origin = self.import_origin.as_ref()?;
        Some(warning_banner(
            "import-banner",
            format!("Imported from {origin}. Check every step before you save."),
            cx,
        ))
    }

    /// Programs from `meta.requires` that are missing; command steps show
    /// their own warning on the card.
    fn render_requirements_banner(&self, cx: &App) -> Option<impl IntoElement> {
        let missing: Vec<&str> = self
            .flow
            .meta
            .requires
            .iter()
            .filter(|r| self.missing.contains(r))
            .map(String::as_str)
            .collect();
        if missing.is_empty() {
            return None;
        }
        let verb = if missing.len() == 1 { "is" } else { "are" };
        Some(warning_banner(
            "requirements-banner",
            format!(
                "This flow needs {}, which {verb} not installed.",
                missing.join(", ")
            ),
            cx,
        ))
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
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .truncate()
                            .child(selectable(
                                "flow-title",
                                if name.is_empty() {
                                    "New flow".to_string()
                                } else {
                                    name
                                },
                            )),
                    )
                    .when(dirty, |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(selectable("flow-unsaved", "Unsaved")),
                        )
                    }),
            )
            .when(!self.is_new(), |this| {
                this.child(
                    Button::new("flow-history")
                        .ghost()
                        .compact()
                        .icon(Icon::new(Icon::empty()).path("icons/rotate-ccw-clock.svg"))
                        .tooltip_with_action("Run history", &ShowHistory, Some(KEY_CONTEXT))
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _, window, cx| this.show_history(window, cx))),
                )
            })
            .child(if self.running {
                Button::new("flow-stop")
                    .compact()
                    .danger()
                    .icon(Icon::new(Icon::empty()).path("icons/square.svg"))
                    .label("Stop")
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| this.stop(cx)))
            } else {
                Button::new("flow-run")
                    .compact()
                    .icon(Icon::new(Icon::empty()).path("icons/play.svg"))
                    .label("Run")
                    .tooltip_with_action("Run the flow as it is now", &Run, Some(KEY_CONTEXT))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| this.run(window, cx)))
            })
            .child(
                Button::new("flow-save")
                    .primary()
                    .compact()
                    .label("Save")
                    .tooltip_with_action("Save the flow", &Save, Some(KEY_CONTEXT))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
            )
            .child(
                Button::new("flow-more")
                    .ghost()
                    .compact()
                    .icon(Icon::new(Icon::empty()).path("icons/ellipsis-vertical.svg"))
                    .tooltip("More")
                    .cursor_pointer()
                    .dropdown_menu(|menu, _, _| menu.menu("Export…", Box::new(Export))),
            )
    }

    /// The flow's icon as a button that opens the icons to choose from.
    /// With the keyboard on it, Left and Right step through them.
    fn render_icon_picker(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let focused = self.icon_focus.is_focused(window);
        let ring = focus::focus_border(focused, theme.transparent, cx);
        let editor = cx.entity();
        let current = self.flow.icon.clone();
        h_flex()
            .id("flow-icons")
            .test_support()
            .key_context(FILTERS_CONTEXT)
            .track_focus(&self.icon_focus)
            .on_action(
                cx.listener(|this, _: &keybinds_nav::FilterPrev, _, cx| this.cycle_icon(-1, cx)),
            )
            .on_action(
                cx.listener(|this, _: &keybinds_nav::FilterNext, _, cx| this.cycle_icon(1, cx)),
            )
            .flex_shrink_0()
            .rounded(theme.radius)
            .border_1()
            .border_color(ring)
            .child(
                Popover::new("flow-icon-popover")
                    .trigger(
                        Button::new("flow-icon-button")
                            .icon(
                                Icon::new(Icon::empty())
                                    .path(crate::system::flows::icon_path(&self.flow.icon)),
                            )
                            .small()
                            .tab_stop(false)
                            .tooltip("Choose an icon")
                            .cursor_pointer(),
                    )
                    .content(move |_, _, cx| {
                        let editor = editor.clone();
                        let current = current.clone();
                        h_flex().w(px(264.)).flex_wrap().gap_1().children(
                            ICONS.iter().enumerate().map(|(ix, &icon)| {
                                let editor = editor.clone();
                                let button = Button::new(("flow-icon", ix))
                                    .icon(
                                        Icon::new(Icon::empty()).path(format!("icons/{icon}.svg")),
                                    )
                                    .small()
                                    .cursor_pointer();
                                let button = if current == icon {
                                    button.primary()
                                } else {
                                    button.ghost()
                                };
                                button.on_click(cx.listener(move |popover, _, window, cx| {
                                    editor.update(cx, |this, cx| this.set_icon(icon, cx));
                                    popover.dismiss(window, cx);
                                }))
                            }),
                        )
                    }),
            )
    }

    fn render_details(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        Self::card(cx)
            .child(Self::section_title("DETAILS", cx))
            .child(
                h_flex()
                    .gap_2()
                    .items_end()
                    .child(self.render_icon_picker(window, cx))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_1()
                            .child(Self::label("Name"))
                            .child(Input::new(&self.name).id("flow-name").small()),
                    ),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(Self::label("Description"))
                    .child(Input::new(&self.description).id("flow-description").small()),
            )
            .children(self.render_meta(cx))
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

    /// Whether the flow reads `{{input}}` anywhere, or is set up to get one.
    fn uses_input(&self) -> bool {
        !self.flow.input.is_none()
            || self.flow.triggers.files
            || self
                .flow
                .walk()
                .iter()
                .any(|(_, step)| step.kind.references().iter().any(|name| name == "input"))
    }

    /// Where the input comes from when the flow is started with none. Only
    /// shown for a flow that uses its input.
    fn render_input_fallback(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement + use<>> {
        if !self.uses_input() {
            return None;
        }
        let theme = cx.theme();
        let focused = self.input_focus.is_focused(window);
        let ring = focus::focus_border(focused, theme.transparent, cx);
        let cycle = |this: &mut Self, delta: isize, cx: &mut Context<Self>| {
            let all = InputFallback::ALL;
            let ix = all.iter().position(|f| *f == this.flow.input).unwrap_or(0) as isize;
            this.flow.input = all[(ix + delta).rem_euclid(all.len() as isize) as usize];
            cx.notify();
        };
        Some(
            v_flex()
                .gap_1()
                .child(Self::label("Started without input, use"))
                .child(
                    h_flex().child(
                        h_flex()
                            .id("flow-input-fallback")
                            .test_support()
                            .key_context(FILTERS_CONTEXT)
                            .track_focus(&self.input_focus)
                            .on_action(cx.listener(
                                move |this, _: &keybinds_nav::FilterPrev, _, cx| {
                                    cycle(this, -1, cx)
                                },
                            ))
                            .on_action(cx.listener(
                                move |this, _: &keybinds_nav::FilterNext, _, cx| cycle(this, 1, cx),
                            ))
                            .rounded(theme.radius)
                            .border_1()
                            .border_color(ring)
                            .p_0p5()
                            .gap_1()
                            .flex_wrap()
                            .children(InputFallback::ALL.into_iter().enumerate().map(
                                |(ix, fallback)| {
                                    let button = Button::new(("flow-input-fallback", ix))
                                        .label(fallback.label())
                                        .small()
                                        .tab_stop(false)
                                        .cursor_pointer();
                                    let button = if self.flow.input == fallback {
                                        button.primary()
                                    } else {
                                        button.ghost()
                                    };
                                    button.on_click(cx.listener(move |this, _, _, cx| {
                                        this.flow.input = fallback;
                                        cx.notify();
                                    }))
                                },
                            )),
                    ),
                ),
        )
    }

    fn render_triggers(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input_fallback = self
            .render_input_fallback(window, cx)
            .map(|e| e.into_any_element());
        let automations = self.render_automations(cx).into_any_element();
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
                            .child(selectable("keybind-external", "Set in your bindings.lua")),
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
                            .child(selectable("keybind-after-saving", "after saving")),
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
                h_flex()
                    .gap_x_4()
                    .gap_y_1()
                    .flex_wrap()
                    .text_sm()
                    .child(
                        FocusableSwitch::new("flow-trigger-launcher")
                            .label("App launcher")
                            .checked(self.flow.triggers.launcher)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.set_trigger(Some(*checked), None, cx)
                            })),
                    )
                    .child(
                        FocusableSwitch::new("flow-trigger-startup")
                            .label("At startup")
                            .checked(self.flow.triggers.startup)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.set_trigger(None, Some(*checked), cx)
                            })),
                    )
                    .child(
                        FocusableSwitch::new("flow-trigger-files")
                            .label("Files menu")
                            .checked(self.flow.triggers.files)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.flow.triggers.files = *checked;
                                cx.notify();
                            })),
                    ),
            )
            .children(input_fallback)
            .child(automations)
            .child(
                v_flex().gap_1().child(Self::label("Command line")).child(
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
                                .text_xs()
                                .truncate()
                                .text_color(if is_new {
                                    theme.muted_foreground
                                } else {
                                    theme.foreground
                                })
                                .child(selectable("flow-command", command)),
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
                ),
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
            .child(self.render_triggers(window, cx));
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
            .on_action(cx.listener(|this, _: &Export, window, cx| this.export(window, cx)))
            .on_action(
                cx.listener(|this, _: &ShowHistory, window, cx| this.show_history(window, cx)),
            )
            .on_action(cx.listener(|this, _: &Run, window, cx| this.run(window, cx)))
            .on_action(cx.listener(|this, _: &AddStep, window, cx| this.add_step(window, cx)))
            .child(self.render_header(cx))
            .children(self.render_import_banner(cx))
            .children(self.render_requirements_banner(cx))
            .child(
                div()
                    .id("flow-edit-content")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .pb_8()
                    .child(crate::ui::focus::scroll_area(&self.scroll).child(columns)),
            )
    }
}
