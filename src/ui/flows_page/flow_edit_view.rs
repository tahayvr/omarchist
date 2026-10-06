// The flow editor: details and triggers on the left, the ordered steps on
// the right. The step list is one tab stop with a roving index.
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::ui::app_events::{AppEvent, emit};
use crate::ui::heading;
use crate::ui::notify;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, Icon, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    clipboard::Clipboard,
    h_flex,
    input::{Input, InputEvent, InputState},
    menu::DropdownMenu,
    switch::Switch,
    v_flex,
};

use crate::system::apps::{DesktopApp, installed_apps};
use crate::system::config::config_setup::settings;
use crate::system::config::hypr_setup::HOOK_RESTORED_MESSAGE;
use crate::system::flows::automations::Event;
use crate::system::flows::catalog;
use crate::system::flows::history::{self, Recorder};
use crate::system::flows::requirements::missing_programs;
use crate::system::flows::risks;
use crate::system::flows::runner::{Cancel, Outcome, RunEvent, Runner, StepStatus};
use crate::system::flows::service;
use crate::system::flows::share::Imported;
use crate::system::flows::store::{
    existing_ids, load_flow, load_flows, runs_flow, save_flow, save_new_flow,
};
use crate::system::flows::templates::{save_user_template, template};
use crate::system::flows::{
    Flow, InputFallback, OnError, Step, StepKind, StepPath, unique_id, vars,
};
use crate::system::keybinds::chord::Chord;
use crate::system::keybinds::overrides::Override;
use crate::system::keybinds::replay::scan_keybinds;
use crate::system::keybinds::store::{load_overrides, save_overrides};
use crate::system::keybinds::{BindStatus, Dispatcher, Keybind, Origin};
use crate::ui::app_view::ActivePage;
use crate::ui::editable_title::{Title, TitleState};
use crate::ui::flows_page::automation_dialog::{
    AutomationDialog, AutomationDialogEvent, EventKind, open_automation_dialog,
};
use crate::ui::flows_page::flow_card::icon_tile;
use crate::ui::flows_page::gallery_detail::{Installed, open_gallery_detail};
use crate::ui::flows_page::history_dialog::open_history_dialog;
use crate::ui::flows_page::icon_dialog::{IconDialog, IconDialogEvent, open_icon_dialog};
use crate::ui::flows_page::publish_dialog::open_publish_dialog;
use crate::ui::flows_page::share_ui::{export_flow, warning_banner};
use crate::ui::flows_page::step_dialog::{
    StepDialog, StepDialogEvent, StepDialogMode, open_step_dialog,
};
use crate::ui::flows_page::step_list::RowKey;
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::flows_page::test_step_dialog::{
    TestStepDialog, TestStepEvent, open_test_step_dialog,
};
use crate::ui::flows_page::undo::UndoStack;
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
            TestStep,
            Undo,
            Redo,
            SaveAsTemplate,
            Publish,
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
    /// The saved flow with this id, taking the steps of a newer version
    /// from the gallery, shown for review before it is saved over.
    Update(String, Box<Imported>),
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
    /// The changes that can be taken back.
    undo: UndoStack,
    name: Entity<InputState>,
    description: Entity<InputState>,
    /// The name in the header, a title until it is clicked.
    title: TitleState,
    icon_focus: FocusHandle,
    icon_dialog: Option<(Entity<IconDialog>, Subscription)>,
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
    pub(super) running: bool,
    /// The step being run alone, when the run is a test of one step.
    testing: Option<StepPath>,
    /// What each variable last held: saved by a run in this editor or
    /// typed into a test. A step run alone starts from these.
    sample_values: HashMap<String, String>,
    test_dialog: Option<(Entity<TestStepDialog>, Subscription)>,
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
    /// What an update from the gallery brought, shown until it is saved.
    update_note: Option<String>,
    /// What the gallery says about this flow, when it came from there:
    /// its entry, its install count, and whether its author is verified.
    gallery: Option<(catalog::Entry, u64, bool)>,
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
                    notify::error(window, format!("Could not read the flow: {e}"), cx);
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
            FlowEditSource::Update(id, _) => match load_flow(id) {
                Ok(flow) => flow,
                Err(e) => {
                    notify::error(window, format!("Could not read the flow: {e}"), cx);
                    emit(cx, AppEvent::Navigate(ActivePage::Flows));
                    Flow::new(String::new(), String::new())
                }
            },
        };
        // What is saved stays the baseline, so an update shows as unsaved
        // and is one change to undo.
        let baseline = flow.clone();
        let undo = UndoStack::new(&flow);
        let mut flow = flow;
        let mut update_note = None;
        if let FlowEditSource::Update(_, imported) = &source
            && !flow.id.is_empty()
        {
            // The new version brings what the flow does; the name, the
            // icon and what starts it are this machine's and stay.
            let new = &imported.flow;
            flow.steps = new.steps.clone();
            flow.description = new.description.clone();
            flow.on_error = new.on_error;
            flow.input = new.input;
            flow.meta = new.meta.clone();
            update_note = Some(match catalog::source_of(new) {
                Some((_, version)) => format!(
                    "These are the steps of version {version} from {}. Check them before you save.",
                    imported.origin
                ),
                None => "These steps are new. Check them before you save.".to_string(),
            });
        }
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
            // Leaving the field ends the rename (Enter is a binding).
            cx.subscribe_in(&name, window, |this, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::Blur) && TitleState::blur_ends(window) {
                    this.title.renaming = false;
                    cx.notify();
                }
            }),
            cx.subscribe(&description, |_, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
            // Every change to the flow ends in a notify, so looking at the
            // flow here records each as one step to undo.
            cx.observe_self(|this, _| this.undo.track(&this.flow)),
        ];

        let mut page = Self {
            focus_handle: cx.focus_handle(),
            step_states: HashMap::new(),
            rounds: HashMap::new(),
            collapsed: HashSet::new(),
            selected_bounds: Rc::new(Cell::new(Bounds::default())),
            undo,
            flow,
            baseline,
            discarded: false,
            update_note,
            gallery: None,
            title: TitleState::new(name.clone(), cx),
            name,
            description,
            icon_focus: focus::tab_stop(cx),
            icon_dialog: None,
            input_focus: focus::tab_stop(cx),
            steps_focus: focus::tab_stop(cx),
            selected: None,
            running: false,
            testing: None,
            sample_values: HashMap::new(),
            test_dialog: None,
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

    /// A flow without a name starts by asking for one; a named one lands
    /// on its title.
    pub fn focus_entry(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.name.read(cx).value().trim().is_empty() {
            self.start_rename(window, cx);
        } else {
            self.title.focus.focus(window, cx);
        }
    }

    fn start_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.title.start(&self.focus_handle, window, cx);
        cx.notify();
    }

    fn stop_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.title.stop(&self.focus_handle, window, cx);
        cx.notify();
    }

    fn choose_icon(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.icon_dialog.is_some() {
            return;
        }
        let dialog = open_icon_dialog(&self.flow.icon, window, cx);
        let subscription = cx.subscribe_in(
            &dialog,
            window,
            |this, _, event: &IconDialogEvent, _window, cx| {
                this.icon_dialog = None;
                if let IconDialogEvent::Picked(icon) = event {
                    this.set_icon(icon, cx);
                }
            },
        );
        self.icon_dialog = Some((dialog, subscription));
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
                        catalog::cached(),
                    )
                })
                .await;
            this.update(cx, |this, cx| {
                let (apps, flows, scan, service_on, gallery) = loaded;
                this.service_on = service_on;
                // The copy of the gallery kept from the last visit says
                // whether this flow has an update or was pulled.
                this.gallery = gallery.and_then(|gallery| {
                    let (slug, _) = catalog::source_of(&this.flow)?;
                    let entry = gallery.index.entry(&slug)?.clone();
                    let installs = gallery.installs.get(&slug).copied().unwrap_or(0);
                    let verified = gallery.index.is_verified(&entry.author);
                    Some((entry, installs, verified))
                });
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
            notify::warning(window, "Give the flow a name first", cx);
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
                notify::success(window, format!("Saved '{}'", flow.name), cx);
                // Clean before anything navigates, or the reopen below would
                // ask to discard the flow that was just saved.
                self.flow = flow.clone();
                self.baseline = flow.clone();
                self.update_note = None;
                // A first save of a flow from the gallery is an install.
                if was_new
                    && let Some((slug, version)) = catalog::source_of(&flow)
                    && settings().gallery_count_installs
                {
                    cx.background_spawn(async move { catalog::count_install(&slug, version) })
                        .detach();
                }
                cx.notify();
                if was_new {
                    // Reopen under the new id so triggers can refer to it.
                    emit(cx, AppEvent::Navigate(ActivePage::FlowEdit(flow.id)));
                }
            }
            Err(e) => notify::error(window, format!("Could not save the flow: {e}"), cx),
        }
    }

    fn run(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.running {
            return;
        }
        let flow = self.current(cx);
        if flow.enabled_steps() == 0 {
            notify::warning(window, "Add a step to run", cx);
            return;
        }
        if let Err(e) = flow.validate_content() {
            notify::error(window, format!("Cannot run the flow: {e}"), cx);
            return;
        }
        self.start_run(flow, None, window, cx);
    }

    /// Runs the step at `path` alone, to try it out. The variables it
    /// uses are asked for first, filled with what they last held.
    pub(super) fn test_step(
        &mut self,
        path: &[usize],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.running || self.test_dialog.is_some() {
            return;
        }
        let flow = self.current(cx);
        if let Err(e) = flow.validate_step(path) {
            notify::error(window, format!("Cannot run the step: {e}"), cx);
            return;
        }
        let needs = flow.needs_at(path);
        if needs.is_empty() {
            self.start_run(flow, Some((path.to_vec(), Vec::new())), window, cx);
            return;
        }
        let Some(step) = flow.step_at(path) else {
            return;
        };
        let summary = SummaryContext {
            apps: &self.apps,
            flows: &self.flows,
        }
        .summarize(&step.kind);
        let values: Vec<(String, String)> = needs
            .into_iter()
            .map(|name| {
                let value = self.sample_values.get(&name).cloned().unwrap_or_default();
                (name, value)
            })
            .collect();
        let dialog = open_test_step_dialog(flow.step_number(path), summary, &values, window, cx);
        let path = path.to_vec();
        let subscription = cx.subscribe_in(
            &dialog,
            window,
            move |this, _, event: &TestStepEvent, window, cx| {
                this.test_dialog = None;
                this.steps_focus.focus(window, cx);
                if let TestStepEvent::Run(values) = event {
                    for (name, value) in values {
                        this.sample_values.insert(name.clone(), value.clone());
                    }
                    let flow = this.current(cx);
                    this.start_run(flow, Some((path.clone(), values.clone())), window, cx);
                }
            },
        );
        self.test_dialog = Some((dialog, subscription));
    }

    /// Starts `flow` on a thread of its own, or only the step `only` names
    /// with the values it is given.
    fn start_run(
        &mut self,
        flow: Flow,
        only: Option<(StepPath, Vec<(String, String)>)>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.running = true;
        self.run_id += 1;
        let run_id = self.run_id;
        let cancel = Cancel::new();
        self.cancel = Some(cancel.clone());
        match &only {
            // The other steps keep what the last run showed under them:
            // those are the values this one starts from.
            Some((path, _)) => {
                self.step_states.retain(|step, _| !step.starts_with(path));
                self.rounds.retain(|step, _| !step.starts_with(path));
            }
            None => {
                self.step_states.clear();
                self.rounds.clear();
            }
        }
        self.testing = only.as_ref().map(|(path, _)| path.clone());
        cx.notify();

        let (tx, rx) = smol::channel::unbounded::<RunMessage>();
        // A thread of its own: a run blocks (waits, waited commands) for as
        // long as the flow takes, which is not what the executor's pool is
        // for.
        std::thread::spawn(move || {
            let runner = Runner::new(true).cancellable(cancel.clone());
            let outcome = match only {
                // A test of one step is not a run of the flow: it is not
                // kept in the history.
                Some((path, values)) => runner.run_only(&flow, &path, &values, &mut |event| {
                    let _ = tx.send_blocking(RunMessage::Event(run_id, event));
                }),
                None => {
                    let mut recorder = Recorder::new(&flow, "Editor");
                    let outcome = runner.run(&flow, &mut |event| {
                        recorder.event(&event);
                        let _ = tx.send_blocking(RunMessage::Event(run_id, event));
                    });
                    // A flow that was never saved has no history to add to.
                    let run = recorder.finish(&flow, &outcome, cancel.is_cancelled());
                    if let Err(e) = history::record(&flow.id, &run) {
                        eprintln!("{e}");
                    }
                    outcome
                }
            };
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

    /// Whether there is a change to take back, and one to make again.
    pub fn can_undo_redo(&self) -> (bool, bool) {
        (self.undo.can_undo(), self.undo.can_redo())
    }

    /// Takes the last change to the flow back, or makes it again.
    fn undo(&mut self, redo: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.refuse_while_running(window, cx) {
            return;
        }
        let changed = if redo {
            self.undo.redo(&mut self.flow)
        } else {
            self.undo.undo(&mut self.flow)
        };
        if changed {
            self.touch_steps(cx);
        }
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
                    let flow = self.current(cx);
                    let message = match self.testing.take() {
                        Some(path) => {
                            let number = flow.step_number(&path);
                            if outcome.cancelled {
                                format!("Step {number} cancelled")
                            } else if outcome.is_ok() {
                                format!("Step {number} finished")
                            } else {
                                format!("Step {number} failed")
                            }
                        }
                        None => outcome.summary(&flow),
                    };
                    notify::result(window, outcome.is_ok(), message, cx);
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
                // What a step saved is what a later step, run alone, is
                // offered as its value.
                if let StepStatus::Done(Some(output)) = &status
                    && let Some(name) = self.flow.step_at(&path).and_then(|s| s.output.as_ref())
                {
                    self.sample_values
                        .insert(vars::normalize(name), output.clone());
                }
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
            notify::warning(window, "Stop the flow before changing its steps", cx);
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
            notify::warning(window, "Save the flow to keep a history of its runs", cx);
            return;
        }
        let name = self.name.read(cx).value().trim().to_string();
        open_history_dialog(&self.flow.id, &name, window, cx);
    }

    /// Keeps the flow as it is in the editor, saved or not, as a template
    /// of the user's own.
    fn save_as_template(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let flow = self.current(cx);
        if flow.name.is_empty() {
            notify::warning(window, "Give the flow a name first", cx);
            self.focus_entry(window, cx);
            return;
        }
        match save_user_template(&flow) {
            Ok(false) => {
                notify::success(window, format!("Saved '{}' as a template", flow.name), cx)
            }
            Ok(true) => {
                notify::success(window, format!("Updated the template '{}'", flow.name), cx)
            }
            Err(e) => notify::error(window, format!("Could not save the template: {e}"), cx),
        }
    }

    // MARK: Sharing

    fn export(&self, window: &mut Window, cx: &mut Context<Self>) {
        let mut flow = self.current(cx);
        if flow.name.is_empty() {
            notify::warning(window, "Give the flow a name first", cx);
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
            notify::warning(window, "This step needs a newer Omarchist to edit", cx);
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

    fn assign_keybind(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.keybind_dialog.is_some() {
            return;
        }
        if self.is_new() {
            notify::warning(window, "Save the flow first, then assign a keybind", cx);
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
                notify::success(
                    window,
                    if removing {
                        "Keybind removed"
                    } else {
                        "Keybind saved"
                    },
                    cx,
                );
                if hook_restored {
                    notify::info(window, HOOK_RESTORED_MESSAGE, cx);
                }
                self.load_context(cx);
            }
            Err(e) => notify::error(window, format!("Could not save the keybind: {e}"), cx),
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
                match result {
                    Ok(()) => notify::success(window, "Automations are on", cx),
                    Err(e) => {
                        notify::error(window, format!("Could not turn automations on: {e}"), cx)
                    }
                }

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
            .child(Self::row(
                "Automations",
                Button::new("flow-add-automation")
                    .outline()
                    .xsmall()
                    .icon(Icon::new(Icon::empty()).path("icons/plus.svg"))
                    .label("Add automation")
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.open_automation_dialog(None, window, cx)
                    })),
            ))
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
    }

    // MARK: Render

    pub(super) fn section_title(text: &'static str, cx: &App) -> Div {
        heading::section(text, cx)
    }

    fn label(text: &'static str) -> Div {
        div().text_sm().child(text)
    }

    /// A setting as one line: its label on the left, its control on the
    /// right; on a narrow card the control wraps under the label.
    fn row(label: &'static str, control: impl IntoElement) -> Div {
        h_flex()
            .gap_x_3()
            .gap_y_1()
            .items_center()
            .justify_between()
            .flex_wrap()
            // The same inset as a switch row, so the labels line up.
            .px_1()
            .child(Self::label(label))
            .child(control)
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
        match catalog::source_of(&self.flow) {
            Some(_) => parts.insert(0, "From the gallery".to_string()),
            None if !meta.source.is_empty() => {
                parts.push(format!("imported from {}", meta.source));
            }
            None => {}
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
        let text = match (&self.import_origin, &self.update_note) {
            (Some(origin), _) => {
                format!("Imported from {origin}. Check every step before you save.")
            }
            (None, Some(note)) => note.clone(),
            (None, None) => return None,
        };
        Some(warning_banner("import-banner", text, cx))
    }

    /// Whether somebody else's steps are on screen, waiting to be saved.
    pub(super) fn reviewing(&self) -> bool {
        self.import_origin.is_some() || self.update_note.is_some()
    }

    /// What a reader should know about the steps under review: the ones
    /// that run as administrator, delete, download code, and the like.
    fn render_risks(&self, cx: &App) -> Option<impl IntoElement> {
        if !self.reviewing() {
            return None;
        }
        let found = risks::risks(&self.flow);
        if found.is_empty() {
            return None;
        }
        let theme = cx.theme();
        Some(
            v_flex()
                .id("import-risks")
                .test_support()
                .gap_1()
                .px_3()
                .py_2()
                .rounded(theme.radius)
                .border_1()
                .border_color(theme.border)
                .children(found.into_iter().enumerate().map(|(ix, risk)| {
                    h_flex()
                        .gap_2()
                        .items_center()
                        .text_sm()
                        .child(
                            Icon::new(Icon::empty())
                                .path("icons/shield-alert.svg")
                                .size_4()
                                .flex_shrink_0()
                                .text_color(if risk.level == risks::Level::Danger {
                                    theme.danger
                                } else {
                                    theme.warning
                                }),
                        )
                        .child(selectable(
                            ("import-risk", ix),
                            format!("Step {}: {}", risk.step, risk.what),
                        ))
                })),
        )
    }

    /// For a flow from the gallery: that a newer version is there, or that
    /// it was pulled.
    fn render_gallery_standing(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let (entry, installs, verified) = self.gallery.clone()?;
        // Under review, the update is what is on screen already.
        if self.reviewing() {
            return None;
        }
        let index = catalog::Index {
            flows: vec![entry.clone()],
            ..catalog::Index::default()
        };
        let (_, version) = catalog::source_of(&self.flow)?;
        match catalog::standing(&self.flow, &index)? {
            catalog::Standing::Pulled(reason) => Some(
                warning_banner(
                    "gallery-pulled",
                    if reason.is_empty() {
                        "This flow was pulled from the gallery.".to_string()
                    } else {
                        format!("This flow was pulled from the gallery: {reason}")
                    },
                    cx,
                )
                .into_any_element(),
            ),
            catalog::Standing::Update(newer) => Some(
                h_flex()
                    .gap_3()
                    .items_center()
                    .px_3()
                    .py_2()
                    .rounded(cx.theme().radius)
                    .border_1()
                    .border_color(cx.theme().border)
                    .text_sm()
                    .child(
                        Icon::new(Icon::empty())
                            .path("icons/circle-arrow-up.svg")
                            .size_4()
                            .text_color(cx.theme().primary),
                    )
                    .child(div().flex_1().child(selectable(
                        "gallery-update-text",
                        format!("Version {newer} is in the gallery"),
                    )))
                    .child(
                        Button::new("flow-gallery-update")
                            .outline()
                            .small()
                            .label("See what changes")
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, window, cx| {
                                open_gallery_detail(
                                    entry.clone(),
                                    installs,
                                    verified,
                                    Some(Installed {
                                        id: this.flow.id.clone(),
                                        version,
                                    }),
                                    window,
                                    cx,
                                );
                            })),
                    )
                    .into_any_element(),
            ),
            _ => None,
        }
    }

    /// Opens the dialog that makes the flow ready for the gallery.
    fn publish(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let flow = self.current(cx);
        if flow.name.is_empty() {
            notify::warning(window, "Give the flow a name first", cx);
            self.focus_entry(window, cx);
            return;
        }
        cx.spawn_in(window, async move |this, cx| {
            // The gallery as last seen tells a new flow from a new version.
            let index = cx
                .background_spawn(async { catalog::cached().map(|gallery| gallery.index) })
                .await;
            this.update_in(cx, |_, window, cx| {
                open_publish_dialog(flow, index, window, cx);
            })
            .ok();
        })
        .detach();
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

    /// The flow's icon as a button that opens the icons to choose from.
    fn render_icon_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Button::new("flow-icon-button")
            .ghost()
            .compact()
            .p_0p5()
            .track_focus(&self.icon_focus)
            .tooltip("Change the icon")
            .cursor_pointer()
            .child(icon_tile(&self.flow.icon, px(28.), cx))
            .on_click(cx.listener(|this, _, window, cx| this.choose_icon(window, cx)))
    }

    fn render_header(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let icon_button = self.render_icon_button(cx).into_any_element();
        let view = cx.entity();
        let on_start = {
            let view = view.clone();
            move |window: &mut Window, cx: &mut App| {
                view.update(cx, |this, cx| this.start_rename(window, cx))
            }
        };
        let on_stop = move |window: &mut Window, cx: &mut App| {
            view.update(cx, |this, cx| this.stop_rename(window, cx))
        };
        let name = self.name.read(cx).value().trim().to_string();
        let title = self.title.render(
            Title {
                id: "flow-title",
                field_id: "flow-name",
                text: &name,
                placeholder: "New flow",
                on_start: Box::new(on_start),
                on_stop: Box::new(on_stop),
            },
            window,
            cx,
        );
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
                    .gap_1()
                    .items_center()
                    .child(icon_button)
                    .child(title),
            )
            .child(
                h_flex()
                    .gap_0p5()
                    .child(
                        Button::new("flow-undo")
                            .ghost()
                            .compact()
                            .disabled(!self.undo.can_undo())
                            .icon(Icon::new(Icon::empty()).path("icons/undo-2.svg"))
                            .tooltip_with_action(
                                if self.undo.can_undo() {
                                    "Undo"
                                } else {
                                    "Nothing to undo"
                                },
                                &Undo,
                                Some(KEY_CONTEXT),
                            )
                            .cursor_pointer()
                            .on_click(
                                cx.listener(|this, _, window, cx| this.undo(false, window, cx)),
                            ),
                    )
                    .child(
                        Button::new("flow-redo")
                            .ghost()
                            .compact()
                            .disabled(!self.undo.can_redo())
                            .icon(Icon::new(Icon::empty()).path("icons/redo-2.svg"))
                            .tooltip_with_action(
                                if self.undo.can_redo() {
                                    "Redo"
                                } else {
                                    "Nothing to redo"
                                },
                                &Redo,
                                Some(KEY_CONTEXT),
                            )
                            .cursor_pointer()
                            .on_click(
                                cx.listener(|this, _, window, cx| this.undo(true, window, cx)),
                            ),
                    ),
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
                    .dropdown_menu(|menu, _, _| {
                        menu.menu("Save as template", Box::new(SaveAsTemplate))
                            .menu("Export…", Box::new(Export))
                            .menu("Publish to the gallery…", Box::new(Publish))
                    }),
            )
    }

    fn render_details(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Self::card(cx)
            .child(Self::section_title("DETAILS", cx))
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
                        .between()
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
        Some(Self::row(
            "Started without input, use",
            h_flex()
                .id("flow-input-fallback")
                .test_support()
                .key_context(FILTERS_CONTEXT)
                .track_focus(&self.input_focus)
                .on_action(
                    cx.listener(move |this, _: &keybinds_nav::FilterPrev, _, cx| {
                        cycle(this, -1, cx)
                    }),
                )
                .on_action(
                    cx.listener(move |this, _: &keybinds_nav::FilterNext, _, cx| {
                        cycle(this, 1, cx)
                    }),
                )
                .rounded(theme.radius)
                .border_1()
                .border_color(ring)
                .p_0p5()
                .gap_1()
                .flex_wrap()
                .children(
                    InputFallback::ALL
                        .into_iter()
                        .enumerate()
                        .map(|(ix, fallback)| {
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
                        }),
                ),
        ))
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
            None => h_flex().child(
                Button::new("flow-keybind-assign")
                    .outline()
                    .xsmall()
                    .icon(Icon::new(Icon::empty()).path("icons/keyboard.svg"))
                    .label("Assign a keybind")
                    .disabled(is_new)
                    .when(is_new, |this| this.tooltip("Save the flow first"))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| this.assign_keybind(window, cx))),
            ),
        };

        let switch = |id: &'static str, label: &'static str, checked: bool| {
            FocusableSwitch::new(id)
                .label(label)
                .between()
                .checked(checked)
        };
        let command_box = div()
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
            .child(selectable("flow-command", command));

        Self::card(cx)
            .child(Self::section_title("RUN IT FROM", cx))
            .child(Self::row("Keybind", keybind_row))
            .child(
                v_flex()
                    .gap_1()
                    .text_sm()
                    .child(
                        switch(
                            "flow-trigger-launcher",
                            "App launcher",
                            self.flow.triggers.launcher,
                        )
                        .on_change(cx.listener(|this, checked, _, cx| {
                            this.set_trigger(Some(*checked), None, cx)
                        })),
                    )
                    .child(
                        switch(
                            "flow-trigger-startup",
                            "At startup",
                            self.flow.triggers.startup,
                        )
                        .on_change(cx.listener(|this, checked, _, cx| {
                            this.set_trigger(None, Some(*checked), cx)
                        })),
                    )
                    .child(
                        switch("flow-trigger-files", "Files menu", self.flow.triggers.files)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.flow.triggers.files = *checked;
                                cx.notify();
                            })),
                    ),
            )
            .children(input_fallback)
            .child(automations)
            .child(
                v_flex()
                    .gap_1()
                    .child(Self::row(
                        "Command line",
                        div().when(!is_new, |this| {
                            this.child(
                                Clipboard::new("flow-copy-command")
                                    .value(self.flow.command())
                                    .tooltip("Copy the command")
                                    .on_copied(|_, window, cx| {
                                        notify::success(window, "Command copied", cx)
                                    }),
                            )
                        }),
                    ))
                    .child(command_box),
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
            .child(self.render_details(cx))
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
            .on_action(
                cx.listener(|this, _: &SaveAsTemplate, window, cx| {
                    this.save_as_template(window, cx)
                }),
            )
            .on_action(cx.listener(|this, _: &Publish, window, cx| this.publish(window, cx)))
            .on_action(cx.listener(|this, _: &Undo, window, cx| this.undo(false, window, cx)))
            .on_action(cx.listener(|this, _: &Redo, window, cx| this.undo(true, window, cx)))
            .on_action(cx.listener(|this, _: &Run, window, cx| this.run(window, cx)))
            .on_action(cx.listener(|this, _: &AddStep, window, cx| this.add_step(window, cx)))
            .child(self.render_header(window, cx))
            .children(self.render_import_banner(cx))
            .children(self.render_risks(cx))
            .children(self.render_gallery_standing(cx))
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
