//! The Dashboard's Flows card: the flows used most recently, how each one
//! starts and when it last ran, a button to run it, and the step a running
//! flow is at, wherever it was started.
use std::collections::HashMap;
use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, Sizable,
    button::{Button, ButtonVariants},
    h_flex, v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::apps::{DesktopApp, installed_apps};
use crate::system::flows::history;
use crate::system::flows::runner::run_in_thread;
use crate::system::flows::running;
use crate::system::flows::store::load_flows_with_broken;
use crate::system::flows::{Flow, StepPath, icon_path};
use crate::system::keybinds::chord::Chord;
use crate::system::keybinds::replay::scan_keybinds;
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::flows_page::flow_card::{LastRun, last_run, running_mark};
use crate::ui::flows_page::flows_view::flow_chords;
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::flows_page::var_token::rich_text;
use crate::ui::heading;
use crate::ui::keybinds_page::chord_chips::chord_chips;
use crate::ui::notify;
use crate::ui::palette::Area;
use crate::ui::text::{selectable, title_case};

/// How many flows the card lists.
const ROWS: usize = 5;

/// What the card reads off the UI thread.
struct Loaded {
    flows: crate::error::Result<Vec<Flow>>,
    last_runs: HashMap<String, LastRun>,
    runs_today: usize,
    chords: HashMap<String, Chord>,
    apps: Vec<DesktopApp>,
}

fn load() -> Loaded {
    let flows = load_flows_with_broken().map(|(flows, _)| flows);
    let midnight = chrono::Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|time| time.and_local_timezone(chrono::Local).earliest())
        .map_or(0, |time| time.timestamp());
    let mut last_runs = HashMap::new();
    let mut runs_today = 0;
    for flow in flows.iter().flatten() {
        let runs = history::load(&flow.id);
        runs_today += runs.iter().filter(|run| run.started >= midnight).count();
        if let Some(run) = runs.iter().max_by_key(|run| run.started) {
            last_runs.insert(flow.id.clone(), (run.started, run.result));
        }
    }
    Loaded {
        flows,
        last_runs,
        runs_today,
        chords: flow_chords(scan_keybinds()),
        apps: installed_apps(),
    }
}

/// The flows to list: those running now, then the most recently run, then
/// the others by name.
fn shown(
    flows: &[Flow],
    last_runs: &HashMap<String, LastRun>,
    running: impl Fn(&str) -> bool,
) -> Vec<Flow> {
    let mut flows = flows.to_vec();
    flows.sort_by(|a, b| {
        let started = |flow: &Flow| last_runs.get(&flow.id).map(|(started, _)| *started);
        running(&b.id)
            .cmp(&running(&a.id))
            .then_with(|| started(b).cmp(&started(a)))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    flows.truncate(ROWS);
    flows
}

/// How a flow starts besides its keybind, in words: "At startup · App
/// launcher", or that only the command line runs it.
fn trigger_text(flow: &Flow, has_chord: bool) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if flow.triggers.startup {
        parts.push("At startup".into());
    }
    if flow.triggers.launcher {
        parts.push("App launcher".into());
    }
    if flow.triggers.files {
        parts.push("Files menu".into());
    }
    match flow
        .triggers
        .automations
        .iter()
        .filter(|a| a.enabled)
        .count()
    {
        0 => {}
        1 => parts.push("Automation".into()),
        n => parts.push(format!("{n} automations")),
    }
    if parts.is_empty() {
        return (!has_chord).then(|| "Runs from the command line".into());
    }
    Some(parts.join(" · "))
}

pub struct FlowsCard {
    flows: Vec<Flow>,
    load_error: Option<String>,
    loaded: bool,
    last_runs: HashMap<String, LastRun>,
    runs_today: usize,
    chords: HashMap<String, Chord>,
    apps: Vec<DesktopApp>,
    /// Every flow running anywhere, with the step it is at, read from the
    /// run registry every second.
    running: HashMap<String, Option<StepPath>>,
    /// The flow this card started, until its run ends.
    starting: Option<String>,
}

impl FlowsCard {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut card = Self {
            flows: Vec::new(),
            load_error: None,
            loaded: false,
            last_runs: HashMap::new(),
            runs_today: 0,
            chords: HashMap::new(),
            apps: Vec::new(),
            running: HashMap::new(),
            starting: None,
        };
        card.refresh(cx);
        card
    }

    /// Reads the flows, their history, keybinds and apps again.
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
                this.last_runs = loaded.last_runs;
                this.runs_today = loaded.runs_today;
                this.chords = loaded.chords;
                this.apps = loaded.apps;
                this.loaded = true;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Follows the run registry: a run that starts shows its step, and one
    /// that ends brings its new last run and count. Started from `main.rs`
    /// through `MainWindowView::watch_runs`, so a headless test window has
    /// no loop that never ends.
    pub fn watch_runs(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                let now: HashMap<String, Option<StepPath>> = cx
                    .background_spawn(async {
                        running::list()
                            .into_iter()
                            .map(|run| (run.id, run.step))
                            .collect()
                    })
                    .await;
                let alive = this
                    .update(cx, |this, cx| {
                        if this.running != now {
                            let ended = this.running.keys().any(|id| !now.contains_key(id));
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
        self.starting.as_deref() == Some(id) || self.running.contains_key(id)
    }

    fn run(&mut self, flow: Flow, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_running(&flow.id) {
            let id = flow.id.clone();
            cx.spawn_in(window, async move |_, cx| {
                let stopped = cx.background_spawn(async move { running::stop(&id) }).await;
                if stopped == 0 {
                    cx.update(|window, cx| {
                        notify::info(window, format!("'{}' is not running", flow.name), cx)
                    })
                    .ok();
                }
            })
            .detach();
            return;
        }
        if self.starting.is_some() {
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

    /// What a running flow is doing: "Step 2 of 5 · Open Firefox".
    fn progress(&self, ix: usize, flow: &Flow, cx: &App) -> (AnyElement, f32) {
        let accent = Area::Flows.accent(cx);
        let total = flow.step_count().max(1);
        let at = self
            .running
            .get(&flow.id)
            .cloned()
            .flatten()
            .and_then(|path| Some((flow.step_number(&path), flow.step_at(&path)?)));
        let Some((number, step)) = at else {
            let line = div().text_color(accent).child("Starting…");
            return (line.into_any_element(), 0.);
        };
        let summaries = SummaryContext {
            apps: &self.apps,
            flows: &self.flows,
        };
        let title = summaries.summarize(&step.kind).title;
        let line = h_flex()
            .gap_1()
            .min_w_0()
            .text_color(accent)
            .child(
                div()
                    .flex_shrink_0()
                    .child(format!("Step {number} of {total} ·")),
            )
            .child(rich_text(&format!("dashboard-flow-step-{ix}"), &title, cx));
        (line.into_any_element(), number as f32 / total as f32)
    }

    fn render_row(&self, ix: usize, flow: &Flow, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let accent = Area::Flows.accent(cx);
        let running = self.is_running(&flow.id);
        let chord = self.chords.get(&flow.id);

        let (subtitle, done) = if running {
            self.progress(ix, flow, cx)
        } else {
            let line = h_flex()
                .gap_2()
                .min_w_0()
                .text_color(theme.muted_foreground)
                .children(chord.map(|chord| chord_chips(chord, false, cx)))
                .children(
                    trigger_text(flow, chord.is_some()).map(|text| div().truncate().child(text)),
                );
            (line.into_any_element(), 0.)
        };

        let when = if running {
            running_mark(ix, cx).into_any_element()
        } else if let Some(last) = self.last_runs.get(&flow.id) {
            last_run(ix, *last, cx).into_any_element()
        } else {
            h_flex()
                .flex_shrink_0()
                .gap_1()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(Icon::new(Icon::empty()).path("icons/circle.svg").size_3())
                .child("Never run")
                .into_any_element()
        };

        let open_id = flow.id.clone();
        let run_flow = flow.clone();
        h_flex()
            .id(("dashboard-flow", ix))
            .relative()
            .gap_3()
            .px_4()
            .py_2()
            .when(running, |row| row.bg(accent.opacity(0.06)))
            .child(
                div()
                    .size_9()
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        Icon::new(Icon::empty())
                            .path(icon_path(&flow.icon))
                            .size_4()
                            .text_color(accent),
                    ),
            )
            .child(
                Button::new(("dashboard-open-flow", ix))
                    .ghost()
                    .flex_1()
                    .min_w_0()
                    .h_auto()
                    .py_1()
                    .cursor_pointer()
                    .tooltip("Edit")
                    .child(
                        v_flex()
                            .w_full()
                            .min_w_0()
                            .gap_1()
                            .items_start()
                            .text_left()
                            .child(
                                div()
                                    .w_full()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .truncate()
                                    .child(selectable(
                                        ("dashboard-flow-name", ix),
                                        title_case(&flow.name),
                                    )),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .text_xs()
                                    .font_weight(FontWeight::NORMAL)
                                    .child(subtitle),
                            ),
                    )
                    .on_click(move |_, _, cx| {
                        emit(
                            cx,
                            AppEvent::Navigate(ActivePage::FlowEdit(open_id.clone())),
                        );
                    }),
            )
            .child(when)
            .child(
                Button::new(("dashboard-run-flow", ix))
                    .secondary()
                    .small()
                    .icon(Icon::new(Icon::empty()).path(if running {
                        "icons/square.svg"
                    } else {
                        "icons/play.svg"
                    }))
                    .tooltip(if running { "Stop" } else { "Run" })
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.run(run_flow.clone(), window, cx);
                    })),
            )
            .when(running && done > 0., |row| {
                row.child(
                    div()
                        .absolute()
                        .left_0()
                        .bottom_0()
                        .h(px(2.))
                        .w(relative(done))
                        .bg(accent),
                )
            })
    }

    fn render_empty(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let message = match &self.load_error {
            Some(e) => format!("Could not read the flows: {e}"),
            None => "No flows yet".to_string(),
        };
        h_flex()
            .px_4()
            .pb_4()
            .gap_3()
            .flex_wrap()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(selectable("dashboard-flows-empty", message)),
            )
            .when(self.load_error.is_none(), |row| {
                row.child(
                    Button::new("dashboard-new-flow")
                        .small()
                        .outline()
                        .icon(Icon::new(Icon::empty()).path("icons/plus.svg"))
                        .label("New flow")
                        .cursor_pointer()
                        .on_click(|_, _, cx| {
                            emit(cx, AppEvent::Navigate(ActivePage::FlowNew(None)))
                        }),
                )
            })
    }
}

impl Render for FlowsCard {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let accent = Area::Flows.accent(cx);
        let rows = shown(&self.flows, &self.last_runs, |id| self.is_running(id));
        let today = match self.runs_today {
            0 => None,
            1 => Some("· 1 run today".to_string()),
            n => Some(format!("· {n} runs today")),
        };

        v_flex()
            .id("dashboard-flows")
            .test_support()
            .w_full()
            .rounded(theme.radius)
            .border_1()
            .border_color(theme.border)
            .child(
                h_flex()
                    .px_4()
                    .py_3()
                    .gap_2()
                    .justify_between()
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
                                    .text_color(theme.muted_foreground)
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
            .when(self.loaded && rows.is_empty(), |card| {
                card.child(self.render_empty(cx))
            })
            .children(
                rows.iter()
                    .enumerate()
                    .map(|(ix, flow)| self.render_row(ix, flow, cx).into_any_element())
                    .collect::<Vec<_>>(),
            )
            .when(!rows.is_empty(), |card| card.pb_2())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{shown, trigger_text};
    use crate::system::flows::Flow;
    use crate::system::flows::history::RunResult;

    fn flow(id: &str) -> Flow {
        Flow::new(id.into(), id.into())
    }

    #[test]
    fn running_then_most_recently_run_then_the_rest_by_name() {
        let flows = ["b", "a", "c", "d", "e", "f"].map(flow).to_vec();
        let mut last = HashMap::new();
        last.insert("c".to_string(), (10, RunResult::Finished));
        last.insert("f".to_string(), (20, RunResult::Failed));
        let ids: Vec<String> = shown(&flows, &last, |_| false)
            .into_iter()
            .map(|f| f.id)
            .collect();
        assert_eq!(ids, ["f", "c", "a", "b", "d"]);
        let ids: Vec<String> = shown(&flows, &last, |id| id == "e")
            .into_iter()
            .map(|f| f.id)
            .collect();
        assert_eq!(ids, ["e", "f", "c", "a", "b"], "a running flow comes first");
    }

    #[test]
    fn triggers_read_as_words_and_a_keybind_needs_none() {
        let mut f = flow("x");
        assert_eq!(
            trigger_text(&f, false).as_deref(),
            Some("Runs from the command line")
        );
        assert_eq!(trigger_text(&f, true), None);
        f.triggers.startup = true;
        f.triggers.launcher = true;
        assert_eq!(
            trigger_text(&f, true).as_deref(),
            Some("At startup · App launcher")
        );
    }
}
