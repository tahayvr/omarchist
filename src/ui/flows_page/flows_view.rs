// The Flows page: every flow as a card, with search, run, and the way into
// the editor. Cards form one tab stop with a roving index.
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::ui::app_events::{AppEvent, emit};
use crate::ui::notify;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, Sizable,
    button::{Button, ButtonVariants, DropdownButton},
    h_flex,
    input::{InputEvent, InputState},
    menu::DropdownMenu,
    v_flex,
};

use crate::system::apps::{DesktopApp, installed_apps};
use crate::system::flows::catalog::{self, Catalog, Standing};
use crate::system::flows::history;
use crate::system::flows::runner::run_in_thread;
use crate::system::flows::running;
use crate::system::flows::store::{
    BrokenFlow, delete_flow, existing_ids, load_flows_with_broken, save_new_flow,
};
use crate::system::flows::templates::{Template, save_user_template, templates};
use crate::system::flows::{Flow, run_command_id, unique_id};
use crate::system::keybinds::chord::Chord;
use crate::system::keybinds::replay::scan_keybinds;
use crate::system::keybinds::{BindStatus, Dispatcher};
use crate::ui::app_view::ActivePage;
use crate::ui::dialogs::confirm_dialog::{ConfirmDialog, open_confirm_dialog};
use crate::ui::flows_page::flow_card::{
    LastRun, icon_tile, last_run, running_mark, step_count_label, step_strip, template_card,
    trigger_chips,
};
use crate::ui::flows_page::gallery_detail::{Installed, open_gallery_detail};
use crate::ui::flows_page::history_dialog::open_history_dialog;
use crate::ui::flows_page::share_ui::{export_flow, import_flow_from_dialog, import_flow_path};
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::focus;
use crate::ui::menu::app_menu;
use crate::ui::text::{selectable, title_case};
use crate::ui::toolbar;

const KEY_CONTEXT: &str = "FlowsPage";
/// Wraps the search box so Down and Escape hand off from inside the input.
pub const SEARCH_CONTEXT: &str = "FlowsSearch";
/// Wraps the cards: arrows move, Enter edits, Ctrl+Enter runs.
pub const GRID_CONTEXT: &str = "FlowsGrid";

pub mod flows_nav {
    gpui::actions!(
        flows,
        [
            FocusSearch,
            ClearSearch,
            FocusGrid,
            NewFlow,
            BrowseTemplates,
            ImportFlow,
            RunSelected,
            EditSelected,
            DeleteSelected,
            DuplicateSelected,
            GridUp,
            GridDown,
            GridLeft,
            GridRight,
            GridFirst,
            GridLast,
        ]
    );
}
use flows_nav::*;

#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = flows, no_json)]
pub struct RunFlow(pub usize);

#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = flows, no_json)]
pub struct EditFlow(pub usize);

#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = flows, no_json)]
pub struct DuplicateFlow(pub usize);

#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = flows, no_json)]
pub struct DeleteFlow(pub usize);

#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = flows, no_json)]
pub struct ExportFlow(pub usize);

#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = flows, no_json)]
pub struct FlowHistory(pub usize);

#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = flows, no_json)]
pub struct FlowAsTemplate(pub usize);

#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = flows, no_json)]
pub struct FlowInGallery(pub usize);

/// When each flow last ran and how it ended, keyed by flow id.
fn last_runs(flows: &[Flow]) -> HashMap<String, LastRun> {
    flows
        .iter()
        .filter_map(|flow| {
            let run = history::last(&flow.id)?;
            Some((flow.id.clone(), (run.started, run.result)))
        })
        .collect()
}

/// The chord of every bind that runs a flow, keyed by flow id.
fn flow_chords(
    scan: crate::error::Result<crate::system::keybinds::replay::ScanResult>,
) -> HashMap<String, Chord> {
    let mut chords = HashMap::new();
    if let Ok(scan) = scan {
        for bind in scan.binds.iter().filter(|b| b.status == BindStatus::Active) {
            if let Dispatcher::Exec(command) = &bind.dispatcher
                && let Some(id) = run_command_id(command)
            {
                chords.entry(id).or_insert_with(|| bind.chord.clone());
            }
        }
    }
    chords
}

pub struct FlowsView {
    pub focus_handle: FocusHandle,
    search: Entity<InputState>,
    query: String,
    flows: Vec<Flow>,
    /// Indices into `flows` that match the query.
    filtered: Vec<usize>,
    chords: HashMap<String, Chord>,
    /// Read with the flows, and again after a run from this page.
    last_runs: HashMap<String, LastRun>,
    /// The copy of the gallery kept from the last look, for the flows
    /// installed from it: an update, or word that one was pulled.
    gallery: Option<Catalog>,
    apps: Vec<DesktopApp>,
    /// For the empty state's cards; reloaded with the flows.
    templates: Vec<Template>,
    /// Files in the flows folder that could not be read.
    broken: Vec<BrokenFlow>,
    /// The folder itself could not be read.
    load_error: Option<String>,
    loaded: bool,
    /// The cards are one tab stop; `focused` is the card with the keyboard.
    grid_focus: FocusHandle,
    focused: Option<usize>,
    columns: usize,
    /// Id of the flow running from this page, if any.
    running: Option<String>,
    /// Every flow running anywhere (a keybind, the bar, an automation,
    /// the editor), read from the run registry every other second.
    running_ids: HashSet<String>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl FlowsView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search flows"));
        let subscriptions = vec![
            cx.subscribe(&search, |this, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.query = input.read(cx).value().to_string();
                    this.apply_filter(cx);
                }
            }),
        ];
        let mut view = Self {
            focus_handle: cx.focus_handle(),
            search,
            query: String::new(),
            flows: Vec::new(),
            filtered: Vec::new(),
            chords: HashMap::new(),
            last_runs: HashMap::new(),
            gallery: None,
            apps: Vec::new(),
            templates: Vec::new(),
            broken: Vec::new(),
            load_error: None,
            loaded: false,
            grid_focus: focus::tab_stop(cx),
            focused: None,
            columns: 1,
            running: None,
            running_ids: HashSet::new(),
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        };
        view.refresh(cx);
        view
    }

    /// Keeps `running_ids` current: the registry is a handful of small
    /// files, read off the UI thread, and the cards only redraw on a change.
    /// Started from `main.rs`, like every loop that never ends: a headless
    /// test window has none, so its scheduler sees only the test's own work.
    pub fn watch_runs(&self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                let ids: HashSet<String> = cx
                    .background_spawn(async {
                        running::list().into_iter().map(|run| run.id).collect()
                    })
                    .await;
                let alive = this
                    .update(cx, |this, cx| {
                        if this.running_ids != ids {
                            this.running_ids = ids;
                            cx.notify();
                        }
                    })
                    .is_ok();
                if !alive {
                    break;
                }
                cx.background_executor().timer(Duration::from_secs(2)).await;
            }
        })
        .detach();
    }

    /// Asks every run of the flow to stop, wherever it was started.
    fn stop(&mut self, filtered_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(flow) = self.flow_at(filtered_ix).cloned() else {
            return;
        };
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
    }

    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.search.update(cx, |input, cx| input.focus(window, cx));
    }

    /// Reloads the flows, the keybinds that run them, and the installed
    /// apps (for step icons), all off the UI thread.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let loaded = cx
                .background_spawn(async {
                    let flows = load_flows_with_broken();
                    let runs = flows
                        .as_ref()
                        .map(|(flows, _)| last_runs(flows))
                        .unwrap_or_default();
                    // Only someone who installed from the gallery has a
                    // reason to hear from it.
                    let from_gallery = flows.as_ref().is_ok_and(|(flows, _)| {
                        flows.iter().any(|flow| catalog::source_of(flow).is_some())
                    });
                    let gallery = from_gallery.then(catalog::cached).flatten();
                    let look = from_gallery && catalog::is_stale();
                    (
                        flows,
                        runs,
                        scan_keybinds(),
                        installed_apps(),
                        templates(),
                        gallery,
                        look,
                    )
                })
                .await;
            let look = loaded.6;
            this.update(cx, |this, cx| {
                let (flows, runs, scan, apps, templates, gallery, _) = loaded;
                this.templates = templates;
                this.last_runs = runs;
                this.gallery = gallery;
                match flows {
                    Ok((flows, broken)) => {
                        this.flows = flows;
                        this.broken = broken;
                        this.load_error = None;
                    }
                    Err(e) => this.load_error = Some(e.to_string()),
                }
                this.chords = flow_chords(scan);
                this.apps = apps;
                this.loaded = true;
                this.apply_filter(cx);
            })
            .ok();
            // At most once a day, and after the page is up: the network
            // may take its time.
            if look && let Ok(gallery) = cx.background_spawn(async { catalog::load() }).await {
                this.update(cx, |this, cx| {
                    this.gallery = Some(gallery);
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    /// What the gallery says about a flow installed from it.
    fn standing(&self, flow: &Flow) -> Option<Standing> {
        catalog::standing(flow, &self.gallery.as_ref()?.index)
    }

    /// Opens the gallery's page of a flow installed from it.
    fn show_in_gallery(&mut self, filtered_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(flow) = self.flow_at(filtered_ix) else {
            return;
        };
        let Some((slug, version)) = catalog::source_of(flow) else {
            return;
        };
        // Without a copy of the gallery there is nothing to show yet; its
        // page fetches one.
        let Some(gallery) = &self.gallery else {
            emit(cx, AppEvent::Navigate(ActivePage::FlowGallery));
            return;
        };
        let Some(entry) = gallery.index.entry(&slug) else {
            notify::info(window, "The gallery no longer lists this flow", cx);
            return;
        };
        open_gallery_detail(
            entry.clone(),
            gallery.installs.get(&slug).copied().unwrap_or(0),
            gallery.index.is_verified(&entry.author),
            Some(Installed {
                id: flow.id.clone(),
                version,
            }),
            window,
            cx,
        );
    }

    fn apply_filter(&mut self, cx: &mut Context<Self>) {
        let query = self.query.trim().to_lowercase();
        self.filtered = self
            .flows
            .iter()
            .enumerate()
            .filter(|(_, flow)| {
                query.is_empty()
                    || flow.name.to_lowercase().contains(&query)
                    || flow.description.to_lowercase().contains(&query)
                    || flow.id.contains(&query)
            })
            .map(|(ix, _)| ix)
            .collect();
        self.focused = match self.focused {
            Some(ix) if ix < self.filtered.len() => Some(ix),
            Some(_) if !self.filtered.is_empty() => Some(self.filtered.len() - 1),
            _ => None,
        };
        cx.notify();
    }

    fn flow_at(&self, filtered_ix: usize) -> Option<&Flow> {
        self.filtered.get(filtered_ix).map(|&ix| &self.flows[ix])
    }

    // MARK: Grid focus

    fn focus_grid(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.filtered.is_empty() {
            return;
        }
        if self.focused.is_none() {
            self.focused = Some(0);
        }
        self.grid_focus.focus(window, cx);
        cx.notify();
    }

    fn set_focused(&mut self, ix: usize, cx: &mut Context<Self>) {
        if self.filtered.is_empty() {
            return;
        }
        let ix = ix.min(self.filtered.len() - 1);
        self.focused = Some(ix);
        // Rows are the scroll container's children.
        self.scroll.scroll_to_item(ix / self.columns.max(1));
        cx.notify();
    }

    fn move_focused(&mut self, delta: isize, cx: &mut Context<Self>) {
        let Some(current) = self.focused else {
            return self.set_focused(0, cx);
        };
        let target = (current as isize + delta).clamp(0, self.filtered.len() as isize - 1);
        self.set_focused(target as usize, cx);
    }

    fn move_row(&mut self, down: bool, cx: &mut Context<Self>) {
        let Some(current) = self.focused else {
            return self.set_focused(0, cx);
        };
        let columns = self.columns.max(1);
        if down {
            if current + columns < self.filtered.len() {
                self.set_focused(current + columns, cx);
            } else if current + 1 < self.filtered.len() {
                self.set_focused(self.filtered.len() - 1, cx);
            }
        } else if current >= columns {
            self.set_focused(current - columns, cx);
        }
    }

    // MARK: Commands

    fn new_flow(&self, cx: &mut Context<Self>) {
        emit(cx, AppEvent::Navigate(ActivePage::FlowNew(None)));
    }

    // MARK: Sharing

    fn export(&self, filtered_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(flow) = self.flow_at(filtered_ix).cloned() {
            export_flow(&flow, window, cx);
        }
    }

    fn edit(&self, filtered_ix: usize, cx: &mut Context<Self>) {
        if let Some(flow) = self.flow_at(filtered_ix) {
            emit(
                cx,
                AppEvent::Navigate(ActivePage::FlowEdit(flow.id.clone())),
            );
        }
    }

    fn run(&mut self, filtered_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(flow) = self.flow_at(filtered_ix).cloned() else {
            return;
        };
        if self.running.is_some() {
            notify::warning(window, "A flow is already running", cx);
            return;
        }
        if flow.enabled_steps() == 0 {
            notify::warning(window, "This flow has no steps to run", cx);
            return;
        }
        self.running = Some(flow.id.clone());
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let outcome = run_in_thread(flow.clone()).await;
            let id = flow.id.clone();
            let last = cx.background_spawn(async move { history::last(&id) }).await;
            this.update_in(cx, |this, window, cx| {
                this.running = None;
                if let Some(run) = last {
                    this.last_runs
                        .insert(flow.id.clone(), (run.started, run.result));
                }
                match outcome {
                    Ok(outcome) => {
                        notify::result(window, outcome.is_ok(), outcome.summary(&flow), cx)
                    }
                    Err(e) => notify::error(window, format!("Could not run the flow: {e}"), cx),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn duplicate(&mut self, filtered_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(flow) = self.flow_at(filtered_ix).cloned() else {
            return;
        };
        let name = format!("{} copy", flow.name);
        let mut copy = Flow {
            id: unique_id(&name, &existing_ids()),
            name,
            ..flow
        };
        // Triggers point at one flow each; the copy starts with none.
        copy.triggers = Default::default();
        match save_new_flow(&copy) {
            Ok(()) => {
                notify::success(window, format!("Created '{}'", copy.name), cx);
                self.refresh(cx);
            }
            Err(e) => notify::error(window, format!("Could not duplicate the flow: {e}"), cx),
        }
    }

    fn confirm_delete(&mut self, filtered_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(flow) = self.flow_at(filtered_ix).cloned() else {
            return;
        };
        let view = cx.entity();
        open_confirm_dialog(
            ConfirmDialog {
                title: "Delete this flow?",
                message: format!(
                    "Delete '{}'? Its keybind, launcher entry, and startup hook are removed with it.",
                    flow.name
                ),
                confirm_label: "Delete",
                danger: true,
            },
            move |window, cx| {
                let id = flow.id.clone();
                let name = flow.name.clone();
                view.update(cx, |this, cx| match delete_flow(&id) {
                    Ok(()) => {
                        notify::success(window, format!("Deleted '{name}'"), cx);
                        this.refresh(cx);
                    }
                    Err(e) => notify::error(window, format!("Could not delete the flow: {e}"), cx),
                });
            },
            window,
            cx,
        );
    }

    fn use_template(&self, id: &str, cx: &mut Context<Self>) {
        emit(
            cx,
            AppEvent::Navigate(ActivePage::FlowNew(Some(id.to_string()))),
        );
    }

    // MARK: Render

    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // With no flows yet, the empty state offers the templates and a
        // create button, so the header does not repeat it.
        let show_new = !(self.loaded && self.flows.is_empty());
        toolbar::bar()
            .child(
                div()
                    .id("flows-search")
                    .key_context(SEARCH_CONTEXT)
                    .w(px(toolbar::SEARCH_WIDTH))
                    .max_w_full()
                    .child(toolbar::search_input(&self.search)),
            )
            .child(toolbar::spacer())
            .child(
                Button::new("browse-gallery")
                    .icon(Icon::new(Icon::empty()).path("icons/store.svg"))
                    .label("Gallery")
                    .outline()
                    .small()
                    .cursor_pointer()
                    .on_click(|_, window, cx| {
                        window.dispatch_action(Box::new(app_menu::OpenGallery), cx)
                    }),
            )
            .when(show_new, |this| {
                this.child(
                    Button::new("browse-templates")
                        .label("Templates")
                        .outline()
                        .small()
                        .tooltip_with_action("Templates", &BrowseTemplates, Some(KEY_CONTEXT))
                        .cursor_pointer()
                        .on_click(|_, window, cx| {
                            window.dispatch_action(Box::new(BrowseTemplates), cx)
                        }),
                )
            })
            .when(show_new, |this| {
                // A split button: the main half starts a blank flow, the
                // arrow offers the other ways in.
                this.child(
                    DropdownButton::new("new-flow")
                        .primary()
                        .small()
                        .button(
                            Button::new("new-flow-main")
                                .icon(Icon::new(Icon::empty()).path("icons/plus.svg"))
                                .label("New flow")
                                .tooltip_with_action("New flow", &NewFlow, Some(KEY_CONTEXT))
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, _, cx| this.new_flow(cx))),
                        )
                        .dropdown_menu(|menu, _, _| {
                            menu.menu("From scratch", Box::new(NewFlow))
                                .menu("From template", Box::new(BrowseTemplates))
                                .menu("From the gallery", Box::new(app_menu::OpenGallery))
                                .menu("Import flow", Box::new(ImportFlow))
                        }),
                )
            })
    }

    fn render_card(
        &self,
        filtered_ix: usize,
        flow: &Flow,
        summaries: &SummaryContext,
        grid_focused: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let focused = grid_focused && self.focused == Some(filtered_ix);
        let running =
            self.running.as_deref() == Some(&flow.id) || self.running_ids.contains(&flow.id);
        let description = if flow.description.trim().is_empty() {
            step_count_label(flow)
        } else {
            flow.description.trim().to_string()
        };
        let on_click_ix = filtered_ix;
        let from_gallery = catalog::source_of(flow).is_some();

        v_flex()
            .id(("flow-card", filtered_ix))
            .group("flow-card")
            .flex_1()
            .min_w_0()
            .gap_3()
            .p_4()
            .rounded(theme.radius)
            .border_1()
            .border_color(if focused { theme.ring } else { theme.border })
            .bg(if focused {
                theme.secondary
            } else {
                theme.background
            })
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                this.focused = Some(on_click_ix);
                this.grid_focus.focus(window, cx);
                if event.click_count() >= 2 {
                    this.edit(on_click_ix, cx);
                } else {
                    cx.notify();
                }
            }))
            .child(
                h_flex()
                    .gap_3()
                    .items_start()
                    .child(icon_tile(&flow.icon, px(40.), cx))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_0p5()
                            .child(div().font_weight(FontWeight::SEMIBOLD).truncate().child(
                                selectable(("flow-name", filtered_ix), title_case(&flow.name)),
                            ))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    // Two lines before an ellipsis: the
                                    // action buttons leave this column
                                    // little width.
                                    .line_clamp(2)
                                    .text_ellipsis()
                                    .child(selectable(
                                        ("flow-description", filtered_ix),
                                        description,
                                    )),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_0p5()
                            .flex_shrink_0()
                            .child(if running {
                                // The play button becomes Stop while the
                                // flow runs, wherever it was started.
                                Button::new(("stop-flow", filtered_ix))
                                    .ghost()
                                    .xsmall()
                                    .tab_stop(false)
                                    .icon(Icon::new(Icon::empty()).path("icons/square.svg"))
                                    .tooltip("Stop")
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        cx.stop_propagation();
                                        this.stop(filtered_ix, window, cx);
                                    }))
                            } else {
                                Button::new(("run-flow", filtered_ix))
                                    .ghost()
                                    .xsmall()
                                    .tab_stop(false)
                                    .icon(Icon::new(Icon::empty()).path("icons/play.svg"))
                                    .tooltip_with_action("Run", &RunSelected, Some(GRID_CONTEXT))
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        cx.stop_propagation();
                                        this.run(filtered_ix, window, cx);
                                    }))
                            })
                            .child(
                                Button::new(("edit-flow", filtered_ix))
                                    .ghost()
                                    .xsmall()
                                    .tab_stop(false)
                                    .icon(Icon::new(Icon::empty()).path("icons/pencil.svg"))
                                    .tooltip_with_action("Edit", &EditSelected, Some(GRID_CONTEXT))
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        cx.stop_propagation();
                                        this.edit(filtered_ix, cx);
                                    })),
                            )
                            .child(
                                Button::new(("more-flow", filtered_ix))
                                    .ghost()
                                    .xsmall()
                                    .tab_stop(false)
                                    .icon(
                                        Icon::new(Icon::empty())
                                            .path("icons/ellipsis-vertical.svg"),
                                    )
                                    .cursor_pointer()
                                    .dropdown_menu(move |menu, _, _| {
                                        let menu = menu.menu(
                                            "Run history",
                                            Box::new(FlowHistory(filtered_ix)),
                                        );
                                        let menu = if from_gallery {
                                            menu.menu(
                                                "Show in the gallery",
                                                Box::new(FlowInGallery(filtered_ix)),
                                            )
                                        } else {
                                            menu
                                        };
                                        menu.menu("Duplicate", Box::new(DuplicateFlow(filtered_ix)))
                                            .menu(
                                                "Save as template",
                                                Box::new(FlowAsTemplate(filtered_ix)),
                                            )
                                            .menu("Export…", Box::new(ExportFlow(filtered_ix)))
                                            .separator()
                                            .menu("Delete", Box::new(DeleteFlow(filtered_ix)))
                                    }),
                            ),
                    ),
            )
            .child(step_strip(flow, summaries, cx))
            .child(
                h_flex()
                    .gap_2()
                    .items_end()
                    .justify_between()
                    .child(trigger_chips(
                        flow,
                        self.chords.get(&flow.id),
                        self.standing(flow).as_ref(),
                        cx,
                    ))
                    .child(if running {
                        running_mark(filtered_ix, cx).into_any_element()
                    } else {
                        div()
                            .children(
                                self.last_runs
                                    .get(&flow.id)
                                    .map(|last| last_run(filtered_ix, *last, cx)),
                            )
                            .into_any_element()
                    }),
            )
    }

    fn render_grid(&self, window: &Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let grid_focused = self.grid_focus.is_focused(window);
        let summaries = SummaryContext {
            apps: &self.apps,
            flows: &self.flows,
        };
        let columns = self.columns.max(1);
        let rows: Vec<AnyElement> = self
            .filtered
            .chunks(columns)
            .enumerate()
            .map(|(row_ix, chunk)| {
                let mut row = h_flex().gap_4().items_stretch();
                for (col_ix, &flow_ix) in chunk.iter().enumerate() {
                    let filtered_ix = row_ix * columns + col_ix;
                    row = row.child(self.render_card(
                        filtered_ix,
                        &self.flows[flow_ix],
                        &summaries,
                        grid_focused,
                        cx,
                    ));
                }
                for _ in chunk.len()..columns {
                    row = row.child(div().flex_1().min_w_0());
                }
                row.into_any_element()
            })
            .collect();
        rows
    }

    /// Files that could not be read, named above the grid so a typo in a
    /// hand-edited flow is not a flow that silently vanished.
    fn render_problems(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let theme = cx.theme();
        if self.load_error.is_none() && self.broken.is_empty() {
            return None;
        }
        Some(
            v_flex()
                .gap_1()
                .text_sm()
                .text_color(theme.warning)
                .children(self.load_error.as_ref().map(|e| {
                    selectable(
                        "flows-load-error",
                        format!("Could not read the flows folder: {e}"),
                    )
                }))
                .children(self.broken.iter().enumerate().map(|(ix, broken)| {
                    let name = broken
                        .path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    selectable(
                        ("flows-broken", ix),
                        format!("Could not read {name}: {}", broken.error),
                    )
                })),
        )
    }

    fn render_empty(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        if !self.query.trim().is_empty() {
            return v_flex()
                .items_center()
                .py_12()
                .gap_1()
                .child(div().text_color(theme.muted_foreground).child(selectable(
                    "no-match",
                    format!("No flows match \"{}\"", self.query.trim()),
                )))
                .into_any_element();
        }
        if !self.loaded {
            return div().into_any_element();
        }

        let templates = &self.templates;
        let summaries = SummaryContext {
            apps: &self.apps,
            flows: &self.flows,
        };
        v_flex()
            .items_center()
            .py_8()
            .gap_6()
            .child(
                v_flex()
                    .items_center()
                    .gap_2()
                    .max_w(px(520.))
                    .child(icon_tile("workflow", px(56.), cx))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Automate Omarchy with flows"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_left()
                            .text_color(theme.muted_foreground)
                            .child(selectable(
                                "empty-what",
                                "A flow strings actions together: open apps, switch workspaces, \
                                 send a notification, wait a moment.",
                            )),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_left()
                            .text_color(theme.muted_foreground)
                            .child(selectable(
                                "empty-how",
                                "Run it from a keybind, the app launcher, at startup, or with \
                                 one command.",
                            )),
                    ),
            )
            .child(
                Button::new("new-flow-empty")
                    .primary()
                    .icon(Icon::new(Icon::empty()).path("icons/plus.svg"))
                    .label("New flow")
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| this.new_flow(cx))),
            )
            .child(
                v_flex()
                    .gap_3()
                    .w_full()
                    .max_w(px(960.))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.muted_foreground)
                            .child("OR START FROM A TEMPLATE"),
                    )
                    .child(h_flex().gap_4().flex_wrap().items_stretch().children(
                        templates.iter().enumerate().map(|(ix, template)| {
                            let key = template.key.clone();
                            template_card(("template", ix), &template.flow, &summaries, cx)
                                .on_click(
                                    cx.listener(move |this, _, _, cx| this.use_template(&key, cx)),
                                )
                        }),
                    )),
            )
            .into_any_element()
    }
}

/// The page is centred and never wider than this, so a card's description
/// and its step strip stay within a glance of its name on a wide screen.
const PAGE_WIDTH: f32 = 1400.;

impl Render for FlowsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Columns from the width the grid gets, never more than the page.
        let width = window.viewport_size().width.min(px(PAGE_WIDTH));
        self.columns = if width < px(700.) {
            1
        } else if width < px(1100.) {
            2
        } else if width < px(1500.) {
            3
        } else {
            4
        };

        v_flex()
            .id("flows-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .items_center()
            .on_action(cx.listener(|this, _: &FocusSearch, window, cx| {
                this.focus_entry(window, cx);
            }))
            .on_action(cx.listener(|this, _: &ClearSearch, window, cx| {
                this.search
                    .update(cx, |input, cx| input.set_value("", window, cx));
                this.query.clear();
                this.apply_filter(cx);
            }))
            .on_action(cx.listener(|this, _: &FocusGrid, window, cx| {
                this.focus_grid(window, cx);
            }))
            .on_action(cx.listener(|this, _: &NewFlow, _, cx| this.new_flow(cx)))
            .on_action(cx.listener(|_, _: &ImportFlow, window, cx| {
                import_flow_from_dialog(window, cx);
            }))
            .on_action(cx.listener(|_, _: &BrowseTemplates, _, cx| {
                emit(cx, AppEvent::Navigate(ActivePage::FlowTemplates));
            }))
            .on_action(cx.listener(|this, action: &ExportFlow, window, cx| {
                this.export(action.0, window, cx);
            }))
            .on_action(cx.listener(|this, action: &FlowAsTemplate, window, cx| {
                if let Some(flow) = this.flow_at(action.0) {
                    match save_user_template(flow) {
                        Ok(false) => notify::success(
                            window,
                            format!("Saved '{}' as a template", flow.name),
                            cx,
                        ),
                        Ok(true) => notify::success(
                            window,
                            format!("Updated the template '{}'", flow.name),
                            cx,
                        ),
                        Err(e) => {
                            notify::error(window, format!("Could not save the template: {e}"), cx)
                        }
                    }
                    this.refresh(cx);
                }
            }))
            .on_action(cx.listener(|this, action: &FlowInGallery, window, cx| {
                this.show_in_gallery(action.0, window, cx);
            }))
            .on_action(cx.listener(|this, action: &FlowHistory, window, cx| {
                if let Some(flow) = this.flow_at(action.0) {
                    open_history_dialog(&flow.id, &flow.name, window, cx);
                }
            }))
            .on_drop(cx.listener(|_, paths: &ExternalPaths, window, cx| {
                if let Some(path) = paths.paths().first() {
                    import_flow_path(path.clone(), window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &RunSelected, window, cx| {
                if let Some(ix) = this.focused {
                    this.run(ix, window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &EditSelected, _, cx| {
                if let Some(ix) = this.focused {
                    this.edit(ix, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &DeleteSelected, window, cx| {
                if let Some(ix) = this.focused {
                    this.confirm_delete(ix, window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &DuplicateSelected, window, cx| {
                if let Some(ix) = this.focused {
                    this.duplicate(ix, window, cx);
                }
            }))
            .on_action(cx.listener(|this, action: &RunFlow, window, cx| {
                this.run(action.0, window, cx);
            }))
            .on_action(cx.listener(|this, action: &EditFlow, _, cx| this.edit(action.0, cx)))
            .on_action(cx.listener(|this, action: &DuplicateFlow, window, cx| {
                this.duplicate(action.0, window, cx);
            }))
            .on_action(cx.listener(|this, action: &DeleteFlow, window, cx| {
                this.confirm_delete(action.0, window, cx);
            }))
            .on_action(cx.listener(|this, _: &GridLeft, _, cx| this.move_focused(-1, cx)))
            .on_action(cx.listener(|this, _: &GridRight, _, cx| this.move_focused(1, cx)))
            .on_action(cx.listener(|this, _: &GridUp, _, cx| this.move_row(false, cx)))
            .on_action(cx.listener(|this, _: &GridDown, _, cx| this.move_row(true, cx)))
            .on_action(cx.listener(|this, _: &GridFirst, _, cx| this.set_focused(0, cx)))
            .on_action(cx.listener(|this, _: &GridLast, _, cx| this.set_focused(usize::MAX, cx)))
            .child(
                v_flex()
                    .w_full()
                    .max_w(px(PAGE_WIDTH))
                    .flex_1()
                    .min_h_0()
                    .gap_4()
                    .child(self.render_toolbar(cx))
                    .children(self.render_problems(cx))
                    .map(|this| {
                        let scroll = div()
                            .id("flows-grid")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .track_scroll(&self.scroll)
                            .pb_8();
                        if self.filtered.is_empty() {
                            // The empty state's buttons are ordinary tab stops,
                            // so it stays outside the grid's roving focus
                            // container.
                            this.child(scroll.child(self.render_empty(cx)))
                        } else {
                            this.child(
                                scroll
                                    .key_context(GRID_CONTEXT)
                                    .track_focus(&self.grid_focus)
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .children(self.render_grid(window, cx)),
                            )
                        }
                    }),
            )
    }
}
