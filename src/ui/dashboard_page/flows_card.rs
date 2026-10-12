//! The Dashboard's Flows card: on the left the latest runs of every flow,
//! however they were started, with how each ended and a button to run it
//! again; on the right the automations that are on and what each is doing,
//! with the switch for the service that starts them.
use std::time::Duration;

use chrono::{Datelike, Local, NaiveDateTime, Timelike};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, Icon, Sizable,
    button::{Button, ButtonVariants},
    h_flex, v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::flows::automations::{Automation, Day, Event};
use crate::system::flows::condition::{parse_time, power};
use crate::system::flows::history::{self, Run, RunResult, StepResult};
use crate::system::flows::runner::run_in_thread;
use crate::system::flows::running::{self, Running};
use crate::system::flows::service;
use crate::system::flows::store::load_flows_with_broken;
use crate::system::flows::{Flow, icon_path};
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::flows_page::automation_dialog::EventKind;
use crate::ui::flows_page::history_dialog::open_history_dialog;
use crate::ui::focus::FocusableSwitch;
use crate::ui::heading;
use crate::ui::notify;
use crate::ui::palette::Area;
use crate::ui::text::{selectable, title_case};

/// How many runs and how many automations the card lists.
const ROWS: usize = 6;

/// One finished run and the flow it belongs to.
#[derive(Debug, Clone)]
struct PastRun {
    flow_id: String,
    run: Run,
}

/// What the card reads off the UI thread.
struct Loaded {
    flows: crate::error::Result<Vec<Flow>>,
    /// Every flow's runs, newest first.
    runs: Vec<PastRun>,
    runs_today: usize,
    service_on: bool,
    battery: Option<u8>,
}

fn midnight() -> i64 {
    Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|time| time.and_local_timezone(Local).earliest())
        .map_or(0, |time| time.timestamp())
}

fn load() -> Loaded {
    let flows = load_flows_with_broken().map(|(flows, _)| flows);
    let since = midnight();
    let mut runs: Vec<PastRun> = flows
        .iter()
        .flatten()
        .flat_map(|flow| {
            history::load(&flow.id).into_iter().map(|run| PastRun {
                flow_id: flow.id.clone(),
                run,
            })
        })
        .collect();
    runs.sort_by_key(|past| std::cmp::Reverse(past.run.started));
    let runs_today = runs.iter().filter(|r| r.run.started >= since).count();
    Loaded {
        flows,
        runs,
        runs_today,
        service_on: service::is_enabled(),
        battery: power().percent,
    }
}

/// What a finished run says on the right of its row: how it ended, and
/// the line under that (how long it took, or why it did not finish).
fn outcome(run: &Run) -> (&'static str, String) {
    let failed_step = run
        .steps
        .iter()
        .find(|step| step.result == StepResult::Failed);
    match run.result {
        RunResult::Finished => ("Finished", format!("took {}", history::took(run.ms))),
        RunResult::Failed => {
            let why = match failed_step {
                Some(step) if !step.detail.is_empty() => {
                    format!("Step {}: {}", step.number, step.detail)
                }
                Some(step) => format!("At step {}", step.number),
                None => run.summary.clone(),
            };
            ("Failed", why)
        }
        RunResult::Stopped => ("Stopped", format!("after {}", history::took(run.ms))),
        RunResult::Cancelled => ("Cancelled", "A question was dismissed".to_string()),
    }
}

/// When a time automation next fires after `now`, within the next week.
fn next_time(at: &str, days: &[Day], now: NaiveDateTime) -> Option<NaiveDateTime> {
    let minutes = parse_time(at)?;
    let (hour, minute) = (minutes / 60, minutes % 60);
    (0..8).find_map(|ahead| {
        let date = now.date() + chrono::Days::new(ahead);
        let day = Day::ALL[date.weekday().num_days_from_monday() as usize];
        let time = date.and_hms_opt(hour, minute, 0)?;
        (time > now && (days.is_empty() || days.contains(&day))).then_some(time)
    })
}

/// "Today 09:00", "Tomorrow 09:00", "Tue 09:00".
fn when_text(time: NaiveDateTime, now: NaiveDateTime) -> String {
    let clock = format!("{:02}:{:02}", time.hour(), time.minute());
    match (time.date() - now.date()).num_days() {
        0 => format!("Today {clock}"),
        1 => format!("Tomorrow {clock}"),
        _ => format!("{} {clock}", time.format("%a")),
    }
}

/// How an automation is doing, as its row shows it.
#[derive(Debug, Clone, PartialEq)]
enum AutoState {
    /// The automations service is off.
    Paused,
    /// Its flow is running now, started by it.
    Running,
    /// The last run it started failed.
    Failed,
    /// It waits for a value it follows (the battery).
    Watching,
    Waiting,
}

/// One automation that is on, with its flow and what it is doing.
#[derive(Debug, Clone)]
struct Shown {
    flow_id: String,
    flow_name: String,
    automation: Automation,
    state: AutoState,
    detail: String,
}

/// The automations to list: those running first, then those whose last
/// run failed, then the rest by flow name; at most `ROWS`.
fn automations(
    flows: &[Flow],
    runs: &[PastRun],
    running: &[Running],
    service_on: bool,
    battery: Option<u8>,
    now: NaiveDateTime,
    now_ts: i64,
) -> (Vec<Shown>, usize) {
    let mut shown = Vec::new();
    for flow in flows {
        for automation in flow.triggers.automations.iter().filter(|a| a.enabled) {
            // The service passes the event's sentence as the trigger, which
            // is how a run is traced back to the automation that started it.
            let label = automation.event.describe();
            let live = running
                .iter()
                .find(|run| run.id == flow.id && run.trigger == label);
            let last = runs
                .iter()
                .find(|past| past.flow_id == flow.id && past.run.trigger == label);
            let last_text = last.map(|past| history::ago(past.run.started, now_ts));
            let (state, detail) = if !service_on {
                (AutoState::Paused, "Automations are off".to_string())
            } else if let Some(run) = live {
                let at = run
                    .step
                    .as_ref()
                    .map(|path| flow.step_number(path))
                    .unwrap_or(0);
                let detail = if at > 0 {
                    format!("Step {at} of {}", flow.step_count())
                } else {
                    "Starting".to_string()
                };
                (AutoState::Running, detail)
            } else if last.is_some_and(|past| past.run.result == RunResult::Failed) {
                (
                    AutoState::Failed,
                    format!("Last run {}", last_text.clone().unwrap_or_default()),
                )
            } else {
                match &automation.event {
                    Event::Time { at, days } => {
                        let next = next_time(at, days, now)
                            .map(|time| format!("Next: {}", when_text(time, now)));
                        (AutoState::Waiting, next.unwrap_or_default())
                    }
                    Event::BatteryBelow { .. } => (
                        AutoState::Watching,
                        battery.map_or(String::new(), |percent| format!("Now at {percent}%")),
                    ),
                    _ => (
                        AutoState::Waiting,
                        last_text
                            .map_or("Not run yet".to_string(), |ago| format!("Last run {ago}")),
                    ),
                }
            };
            shown.push(Shown {
                flow_id: flow.id.clone(),
                flow_name: title_case(&flow.name),
                automation: automation.clone(),
                state,
                detail,
            });
        }
    }
    let total = shown.len();
    let rank = |state: &AutoState| match state {
        AutoState::Running => 0,
        AutoState::Failed => 1,
        _ => 2,
    };
    shown.sort_by(|a, b| {
        rank(&a.state)
            .cmp(&rank(&b.state))
            .then_with(|| a.flow_name.to_lowercase().cmp(&b.flow_name.to_lowercase()))
    });
    shown.truncate(ROWS);
    (shown, total)
}

pub struct FlowsCard {
    flows: Vec<Flow>,
    load_error: Option<String>,
    loaded: bool,
    runs: Vec<PastRun>,
    runs_today: usize,
    service_on: bool,
    service_pending: bool,
    battery: Option<u8>,
    /// Every flow running anywhere, read from the run registry every second.
    running: Vec<Running>,
    /// The flow this card started, until its run ends.
    starting: Option<String>,
}

impl FlowsCard {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut card = Self {
            flows: Vec::new(),
            load_error: None,
            loaded: false,
            runs: Vec::new(),
            runs_today: 0,
            service_on: false,
            service_pending: false,
            battery: None,
            running: Vec::new(),
            starting: None,
        };
        card.refresh(cx);
        card
    }

    /// Reads the flows, their history and the service again.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let loaded = cx.background_spawn(async { load() }).await;
            this.update(cx, |this, cx| {
                match loaded.flows {
                    Ok(flows) => {
                        this.flows = flows;
                        this.load_error = None;
                    }
                    Err(e) => this.load_error = Some(e.to_string()),
                }
                this.runs = loaded.runs;
                this.runs_today = loaded.runs_today;
                if !this.service_pending {
                    this.service_on = loaded.service_on;
                }
                this.battery = loaded.battery;
                this.loaded = true;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Follows the run registry: a run that starts shows its step, and one
    /// that ends brings its row. Started from `main.rs` through
    /// `MainWindowView::watch_runs`, so a headless test window has no loop
    /// that never ends.
    pub fn watch_runs(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                let now = cx.background_spawn(async { running::list() }).await;
                let alive = this
                    .update(cx, |this, cx| {
                        if this.running != now {
                            let ended = this
                                .running
                                .iter()
                                .any(|run| !now.iter().any(|n| n.id == run.id));
                            this.running = now;
                            if ended {
                                this.refresh(cx);
                            }
                            cx.notify();
                        }
                    })
                    .is_ok();
                if !alive {
                    break;
                }
                cx.background_executor().timer(Duration::from_secs(1)).await;
            }
        })
        .detach();
    }

    fn is_running(&self, id: &str) -> bool {
        self.starting.as_deref() == Some(id) || self.running.iter().any(|run| run.id == id)
    }

    fn run(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(flow) = self.flows.iter().find(|flow| flow.id == id).cloned() else {
            notify::warning(window, "This flow no longer exists", cx);
            return;
        };
        if self.is_running(&flow.id) || self.starting.is_some() {
            notify::warning(window, "A flow is already running", cx);
            return;
        }
        if flow.enabled_steps() == 0 {
            notify::warning(window, "This flow has no steps to run", cx);
            return;
        }
        self.starting = Some(flow.id.clone());
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let outcome = run_in_thread(flow.clone()).await;
            this.update_in(cx, |this, window, cx| {
                this.starting = None;
                match outcome {
                    Ok(outcome) => {
                        notify::result(window, outcome.is_ok(), outcome.summary(&flow), cx)
                    }
                    Err(e) => notify::error(window, format!("Could not run the flow: {e}"), cx),
                }
                this.refresh(cx);
            })
            .ok();
        })
        .detach();
    }

    /// Stops the running flow wherever it was started.
    fn stop(&self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let id = id.to_string();
        cx.spawn_in(window, async move |_, cx| {
            let stopped = cx.background_spawn(async move { running::stop(&id) }).await;
            if stopped == 0 {
                cx.update(|window, cx| notify::info(window, "The flow already ended", cx))
                    .ok();
            }
        })
        .detach();
    }

    /// Turns the automations service on or off, in the background.
    fn set_service(&mut self, on: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.service_pending {
            return;
        }
        self.service_pending = true;
        self.service_on = on;
        cx.notify();
        let task = cx.background_spawn(async move {
            if on {
                service::enable()
            } else {
                service::disable()
            }
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = task.await;
            let now_on = cx.background_spawn(async { service::is_enabled() }).await;
            this.update_in(cx, |this, window, cx| {
                this.service_pending = false;
                this.service_on = now_on;
                match result {
                    Ok(()) if on => notify::success(window, "Automations are on", cx),
                    Ok(()) => notify::success(window, "Automations are off", cx),
                    Err(e) => {
                        notify::error(window, format!("Could not change automations: {e}"), cx)
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn flow(&self, id: &str) -> Option<&Flow> {
        self.flows.iter().find(|flow| flow.id == id)
    }

    /// A square with an icon on a tint of `color`.
    fn tile(icon: impl Into<SharedString>, color: Hsla) -> Div {
        div()
            .size_8()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .bg(color.opacity(0.14))
            .child(
                Icon::new(Icon::empty())
                    .path(icon)
                    .size_3p5()
                    .text_color(color),
            )
    }

    /// The name and the line under it, as one button that opens `open`.
    fn label_button(
        id: impl Into<ElementId>,
        name_id: (&'static str, usize),
        name: String,
        line: String,
        tooltip: &'static str,
        cx: &App,
        open: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Button {
        let muted = cx.theme().muted_foreground;
        Button::new(id)
            .ghost()
            .flex_1()
            .min_w_0()
            .h_auto()
            .py_1()
            .cursor_pointer()
            .tooltip(tooltip)
            .child(
                v_flex()
                    .w_full()
                    .min_w_0()
                    .items_start()
                    .text_left()
                    .child(
                        div()
                            .w_full()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .truncate()
                            .child(selectable(name_id, name)),
                    )
                    .child(
                        div()
                            .w_full()
                            .text_xs()
                            .font_weight(FontWeight::NORMAL)
                            .text_color(muted)
                            .truncate()
                            .child(line),
                    ),
            )
            .on_click(move |_, window, cx| open(window, cx))
    }

    /// What a row says on its right: a state in its colour over a line.
    fn state_cell(state: String, color: Hsla, line: String, line_color: Hsla) -> Div {
        v_flex()
            .flex_none()
            .max_w(relative(0.45))
            .items_end()
            .child(
                h_flex()
                    .gap_1p5()
                    .items_center()
                    .text_sm()
                    .text_color(color)
                    .child(div().size_2().flex_none().bg(color))
                    .child(state),
            )
            .child(
                div()
                    .max_w_full()
                    .text_xs()
                    .text_color(line_color)
                    .truncate()
                    .child(line),
            )
    }

    fn render_live(&self, ix: usize, run: &Running, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let accent = Area::Flows.accent(cx);
        let flow = self.flow(&run.id);
        let total = flow.map_or(1, |flow| flow.step_count().max(1));
        let at = match (flow, run.step.as_ref()) {
            (Some(flow), Some(path)) => flow.step_number(path),
            _ => 0,
        };
        let line = if at > 0 {
            format!("Step {at} of {total}")
        } else {
            "Starting".to_string()
        };
        let icon = flow.map_or("workflow".to_string(), |flow| flow.icon.clone());
        let id = run.id.clone();
        let stop_id = run.id.clone();
        let name = title_case(&run.name);
        h_flex()
            .id(("dashboard-live-run", ix))
            .test_support()
            .relative()
            .gap_3()
            .px_4()
            .py_1p5()
            .bg(accent.opacity(0.06))
            .child(Self::tile(icon_path(&icon), accent))
            .child(Self::label_button(
                ("dashboard-live-open", ix),
                ("dashboard-live-name", ix),
                name,
                format!("{} · now", run.trigger),
                "Edit",
                cx,
                move |_, cx| emit(cx, AppEvent::Navigate(ActivePage::FlowEdit(id.clone()))),
            ))
            .child(Self::state_cell(
                "Running".into(),
                accent,
                line,
                theme.muted_foreground,
            ))
            .child(
                Button::new(("dashboard-stop-run", ix))
                    .secondary()
                    .small()
                    .icon(Icon::new(Icon::empty()).path("icons/square.svg"))
                    .tooltip("Stop")
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.stop(&stop_id, window, cx);
                    })),
            )
            .when(at > 0, |row| {
                row.child(
                    div()
                        .absolute()
                        .left_0()
                        .bottom_0()
                        .h(px(2.))
                        .w(relative(at as f32 / total as f32))
                        .bg(accent),
                )
            })
            .into_any_element()
    }

    fn render_past(&self, ix: usize, past: &PastRun, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let color = match past.run.result {
            RunResult::Finished => theme.success,
            RunResult::Failed => theme.danger,
            RunResult::Cancelled | RunResult::Stopped => theme.muted_foreground,
        };
        let icon = match past.run.result {
            RunResult::Finished => "icons/circle-check.svg",
            RunResult::Failed => "icons/circle-x.svg",
            RunResult::Cancelled | RunResult::Stopped => "icons/ban.svg",
        };
        let (state, why) = outcome(&past.run);
        let why_color = if past.run.result == RunResult::Failed {
            theme.danger
        } else {
            theme.muted_foreground
        };
        let flow = self.flow(&past.flow_id);
        let name = flow.map_or(past.flow_id.clone(), |flow| title_case(&flow.name));
        let busy = self.starting.is_some() || self.is_running(&past.flow_id);
        let ago = history::ago(past.run.started, Local::now().timestamp());
        let (history_id, history_name) = (past.flow_id.clone(), name.clone());
        let run_id = past.flow_id.clone();
        h_flex()
            .id(("dashboard-run", ix))
            .test_support()
            .gap_3()
            .px_4()
            .py_1p5()
            .child(Self::tile(icon, color))
            .child(Self::label_button(
                ("dashboard-run-open", ix),
                ("dashboard-run-name", ix),
                name,
                format!("{ago} · {}", past.run.trigger),
                "Run history",
                cx,
                move |window, cx| {
                    open_history_dialog(&history_id, &history_name, window, cx);
                },
            ))
            .child(Self::state_cell(state.into(), color, why, why_color))
            .child(
                Button::new(("dashboard-run-again", ix))
                    .secondary()
                    .small()
                    .icon(Icon::new(Icon::empty()).path("icons/play.svg"))
                    .tooltip(if busy {
                        "A flow is running"
                    } else {
                        "Run again"
                    })
                    .disabled(busy || flow.is_none())
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.run(&run_id, window, cx);
                    })),
            )
            .into_any_element()
    }

    fn render_automation(&self, ix: usize, shown: &Shown, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let accent = Area::Flows.accent(cx);
        let (state, color) = match shown.state {
            AutoState::Paused => ("Paused", theme.muted_foreground),
            AutoState::Running => ("Running", accent),
            AutoState::Failed => ("Last run failed", theme.danger),
            AutoState::Watching => ("Watching", theme.info),
            AutoState::Waiting => ("Waiting", theme.muted_foreground),
        };
        let mut line = format!("Runs {}", shown.flow_name);
        if shown.automation.ask {
            line.push_str(" · asks first");
        }
        let id = shown.flow_id.clone();
        h_flex()
            .id(("dashboard-automation", ix))
            .test_support()
            .gap_3()
            .px_4()
            .py_1p5()
            .when(shown.state == AutoState::Paused, |row| row.opacity(0.6))
            .child(Self::tile(
                EventKind::of(&shown.automation.event).icon(),
                accent,
            ))
            .child(Self::label_button(
                ("dashboard-automation-open", ix),
                ("dashboard-automation-name", ix),
                shown.automation.event.describe(),
                line,
                "Edit the flow",
                cx,
                move |_, cx| emit(cx, AppEvent::Navigate(ActivePage::FlowEdit(id.clone()))),
            ))
            .child(Self::state_cell(
                state.into(),
                color,
                shown.detail.clone(),
                theme.muted_foreground,
            ))
            .into_any_element()
    }

    fn empty_line(id: &'static str, text: String, cx: &App) -> Div {
        div()
            .px_4()
            .py_2()
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .child(selectable(id, text))
    }

    fn render_runs(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let live: Vec<Running> = self.running.iter().take(ROWS).cloned().collect();
        let past: Vec<PastRun> = self
            .runs
            .iter()
            .take(ROWS.saturating_sub(live.len()))
            .cloned()
            .collect();
        let mut column = v_flex().id("dashboard-runs").min_w_0().pb_2().child(
            div()
                .px_4()
                .pt_3()
                .pb_1()
                .child(heading::section("Recent runs", cx)),
        );
        if let Some(e) = &self.load_error {
            column = column.child(Self::empty_line(
                "dashboard-runs-error",
                format!("Could not read the flows: {e}"),
                cx,
            ));
        } else if self.loaded && live.is_empty() && past.is_empty() {
            column = column.child(Self::empty_line(
                "dashboard-runs-empty",
                "No runs yet".into(),
                cx,
            ));
        }
        for (ix, run) in live.iter().enumerate() {
            column = column.child(self.render_live(ix, run, cx));
        }
        for (ix, run) in past.iter().enumerate() {
            column = column.child(self.render_past(ix, run, cx));
        }
        column
    }

    fn render_automations(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let now = Local::now();
        let (shown, total) = automations(
            &self.flows,
            &self.runs,
            &self.running,
            self.service_on,
            self.battery,
            now.naive_local(),
            now.timestamp(),
        );
        let title = match total {
            0 => "Automations".to_string(),
            n => format!("Automations · {n} on"),
        };
        let mut column = v_flex().id("dashboard-automations").min_w_0().pb_2().child(
            h_flex()
                .px_4()
                .pt_2()
                .pb_1()
                .gap_2()
                .justify_between()
                .child(heading::section(&title, cx))
                .child(
                    div().flex_none().child(
                        FocusableSwitch::new("dashboard-automations-switch")
                            .label(if self.service_on { "On" } else { "Off" })
                            .checked(self.service_on)
                            .disabled(self.service_pending)
                            .on_change(cx.listener(|this, value, window, cx| {
                                this.set_service(*value, window, cx);
                            })),
                    ),
                ),
        );
        if self.loaded && shown.is_empty() {
            column = column.child(Self::empty_line(
                "dashboard-automations-empty",
                "No flow starts on its own yet".into(),
                cx,
            ));
        }
        for (ix, item) in shown.iter().enumerate() {
            column = column.child(self.render_automation(ix, item, cx));
        }
        column
    }
}

impl Render for FlowsCard {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let accent = Area::Flows.accent(cx);
        let border = theme.border;
        let today = match self.runs_today {
            0 => None,
            1 => Some("· 1 run today".to_string()),
            n => Some(format!("· {n} runs today")),
        };
        // Side by side when there is room for two columns of rows, one
        // above the other otherwise.
        let stacked = window.viewport_size().width < px(1100.);
        let runs = self.render_runs(cx).flex_1();
        let automations = self.render_automations(cx).flex_1();

        v_flex()
            .id("dashboard-flows")
            .test_support()
            .w_full()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(border)
            .child(
                h_flex()
                    .px_4()
                    .py_3()
                    .gap_2()
                    .justify_between()
                    .border_b_1()
                    .border_color(border)
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Icon::new(Icon::empty())
                                    .path("icons/workflow.svg")
                                    .size_3p5()
                                    .text_color(accent),
                            )
                            .child(heading::section("Flows", cx))
                            .children(today.map(|today| {
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(today)
                            })),
                    )
                    .child(
                        Button::new("dashboard-all-flows")
                            .ghost()
                            .small()
                            .cursor_pointer()
                            .child(
                                h_flex().gap_1().items_center().child("All flows").child(
                                    Icon::new(Icon::empty())
                                        .path("icons/chevron-right.svg")
                                        .size_3(),
                                ),
                            )
                            .on_click(|_, _, cx| emit(cx, AppEvent::Navigate(ActivePage::Flows))),
                    ),
            )
            .child(if stacked {
                v_flex()
                    .child(runs)
                    .child(automations.border_t_1().border_color(border))
            } else {
                div()
                    .flex()
                    .flex_row()
                    .child(runs)
                    .child(automations.border_l_1().border_color(border))
            })
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{AutoState, PastRun, automations, next_time, outcome, when_text};
    use crate::system::flows::Flow;
    use crate::system::flows::automations::{Automation, Day, Event};
    use crate::system::flows::history::{Run, RunResult, StepResult, StepRun};
    use crate::system::flows::running::Running;

    fn at(day: u32, hour: u32, minute: u32) -> chrono::NaiveDateTime {
        // 2026-10-12 is a Monday.
        NaiveDate::from_ymd_opt(2026, 10, day)
            .unwrap()
            .and_hms_opt(hour, minute, 0)
            .unwrap()
    }

    fn run(trigger: &str, result: RunResult, started: i64) -> Run {
        Run {
            started,
            ms: 1200,
            trigger: trigger.into(),
            result,
            summary: String::new(),
            steps: Vec::new(),
        }
    }

    #[test]
    fn a_time_automation_fires_next_on_one_of_its_days() {
        let monday_noon = at(12, 12, 0);
        let weekdays = &Day::ALL[..5];
        let next = next_time("09:00", weekdays, monday_noon).unwrap();
        assert_eq!(when_text(next, monday_noon), "Tomorrow 09:00");
        let later = next_time("18:30", &[], monday_noon).unwrap();
        assert_eq!(when_text(later, monday_noon), "Today 18:30");
        let friday = at(16, 12, 0);
        let next = next_time("09:00", weekdays, friday).unwrap();
        assert_eq!(when_text(next, friday), "Mon 09:00");
        assert!(next_time("9am", &[], friday).is_none());
    }

    #[test]
    fn a_failed_run_names_the_step_and_its_error() {
        let mut failed = run("Keybind", RunResult::Failed, 0);
        failed.steps.push(StepRun {
            number: 3,
            depth: 0,
            title: "Lock".into(),
            result: StepResult::Failed,
            detail: "hyprlock: command not found".into(),
            ms: 4,
        });
        assert_eq!(
            outcome(&failed),
            ("Failed", "Step 3: hyprlock: command not found".to_string())
        );
        assert_eq!(
            outcome(&run("Bar", RunResult::Finished, 0)),
            ("Finished", "took 1.2 s".to_string())
        );
    }

    #[test]
    fn automations_show_what_they_are_doing_and_running_ones_come_first() {
        let mut lock = Flow::new("leave".into(), "Leave".into());
        lock.triggers
            .automations
            .push(Automation::new(Event::Locked));
        let mut morning = Flow::new("morning".into(), "Morning".into());
        morning
            .triggers
            .automations
            .push(Automation::new(Event::Time {
                at: "09:00".into(),
                days: Vec::new(),
            }));
        let mut off = Automation::new(Event::Woke);
        off.enabled = false;
        morning.triggers.automations.push(off);
        let flows = vec![morning.clone(), lock.clone()];
        let runs = vec![PastRun {
            flow_id: "leave".into(),
            run: run(&Event::Locked.describe(), RunResult::Failed, 100),
        }];
        let now = at(12, 12, 0);

        let (shown, total) = automations(&flows, &runs, &[], true, None, now, 160);
        assert_eq!(total, 2, "a switched-off automation is not listed");
        assert_eq!(shown[0].state, AutoState::Failed);
        assert_eq!(shown[0].detail, "Last run 1 min ago");
        assert_eq!(shown[1].state, AutoState::Waiting);
        assert_eq!(shown[1].detail, "Next: Tomorrow 09:00");

        let live = Running {
            id: "morning".into(),
            name: "Morning".into(),
            pid: 1,
            started: 150,
            trigger: "At 09:00".into(),
            in_app: false,
            step: None,
        };
        let (shown, _) = automations(&flows, &runs, &[live], true, None, now, 160);
        assert_eq!(shown[0].flow_id, "morning");
        assert_eq!(shown[0].state, AutoState::Running);

        let (shown, _) = automations(&flows, &runs, &[], false, None, now, 160);
        assert!(shown.iter().all(|s| s.state == AutoState::Paused));
    }
}
