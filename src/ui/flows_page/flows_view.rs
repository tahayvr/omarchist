// The Flows page: every flow as a card, with search, run, and the way into
// the editor. Cards form one tab stop with a roving index.
use std::collections::HashMap;
use std::path::PathBuf;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    menu::DropdownMenu,
    v_flex,
};

use crate::system::apps::{DesktopApp, installed_apps};
use crate::system::flows::runner::Runner;
use crate::system::flows::share::{ImportSource, export_file_name, export_toml, read_import};
use crate::system::flows::store::{delete_flow, existing_ids, load_flows, save_flow};
use crate::system::flows::templates::templates;
use crate::system::flows::{Flow, run_command_id, unique_id};
use crate::system::keybinds::chord::Chord;
use crate::system::keybinds::replay::scan_keybinds;
use crate::system::keybinds::{BindStatus, Dispatcher};
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::dialogs::confirm_dialog::{ConfirmDialog, open_confirm_dialog};
use crate::ui::flows_page::flow_card::{
    icon_tile, step_count_label, step_strip, template_card, trigger_chips,
};
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::focus;

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
    apps: Vec<DesktopApp>,
    loaded: bool,
    /// The cards are one tab stop; `focused` is the card with the keyboard.
    grid_focus: FocusHandle,
    focused: Option<usize>,
    columns: usize,
    /// Id of the flow running from this page, if any.
    running: Option<String>,
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
            apps: Vec::new(),
            loaded: false,
            grid_focus: focus::tab_stop(cx),
            focused: None,
            columns: 1,
            running: None,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        };
        view.refresh(cx);
        view
    }

    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.search.update(cx, |input, cx| input.focus(window, cx));
    }

    /// Reloads the flows, the keybinds that run them, and the installed
    /// apps (for step icons), all off the UI thread.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let loaded = cx
                .background_spawn(async { (load_flows(), scan_keybinds(), installed_apps()) })
                .await;
            this.update(cx, |this, cx| {
                let (flows, scan, apps) = loaded;
                match flows {
                    Ok(flows) => this.flows = flows,
                    Err(e) => eprintln!("Failed to load flows: {e}"),
                }
                this.chords = flow_chords(scan);
                this.apps = apps;
                this.loaded = true;
                this.apply_filter(cx);
            })
            .ok();
        })
        .detach();
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
        self.focused = Some(ix.min(self.filtered.len() - 1));
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

    fn import_from_dialog(&self, window: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Import".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(paths))) = receiver.await
                && let Some(path) = paths.into_iter().next()
            {
                this.update_in(cx, |this, window, cx| this.import_path(path, window, cx))
                    .ok();
            }
        })
        .detach();
    }

    /// Reads the file off the UI thread and opens it in the editor for
    /// review; nothing is saved until the user does.
    fn import_path(&self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move { read_import(&ImportSource::File(path)) })
                .await;
            this.update_in(cx, |_, window, cx| match result {
                Ok(imported) => emit(
                    cx,
                    AppEvent::Navigate(ActivePage::FlowImport(Box::new(imported))),
                ),
                Err(e) => window.push_notification(format!("Could not import the flow: {e}"), cx),
            })
            .ok();
        })
        .detach();
    }

    fn export(&self, filtered_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(flow) = self.flow_at(filtered_ix).cloned() else {
            return;
        };
        let text = match export_toml(&flow) {
            Ok(text) => text,
            Err(e) => {
                window.push_notification(format!("Could not export the flow: {e}"), cx);
                return;
            }
        };
        let dir = dirs::download_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_default();
        let receiver = cx.prompt_for_new_path(&dir, Some(&export_file_name(&flow)));
        cx.spawn_in(window, async move |this, cx| {
            if let Ok(Ok(Some(path))) = receiver.await {
                let written = std::fs::write(&path, text);
                this.update_in(cx, |_, window, cx| match written {
                    Ok(()) => {
                        window.push_notification(format!("Exported to {}", path.display()), cx)
                    }
                    Err(e) => {
                        window.push_notification(format!("Could not write the file: {e}"), cx)
                    }
                })
                .ok();
            }
        })
        .detach();
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
            window.push_notification("A flow is already running", cx);
            return;
        }
        if flow.enabled_steps() == 0 {
            window.push_notification("This flow has no steps to run", cx);
            return;
        }
        self.running = Some(flow.id.clone());
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let outcome = cx
                .background_spawn(async move {
                    let outcome = Runner::new(true).run(&flow, &mut |_| {});
                    (flow, outcome)
                })
                .await;
            this.update_in(cx, |this, window, cx| {
                let (flow, outcome) = outcome;
                this.running = None;
                window.push_notification(outcome.summary(&flow), cx);
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
        match save_flow(&copy) {
            Ok(()) => {
                window.push_notification(format!("Created '{}'", copy.name), cx);
                self.refresh(cx);
            }
            Err(e) => window.push_notification(format!("Could not duplicate the flow: {e}"), cx),
        }
    }

    fn confirm_delete(&mut self, filtered_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(flow) = self.flow_at(filtered_ix).cloned() else {
            return;
        };
        let view = cx.entity();
        open_confirm_dialog(
            ConfirmDialog {
                title: "Delete flow",
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
                        window.push_notification(format!("Deleted '{name}'"), cx);
                        this.refresh(cx);
                    }
                    Err(e) => {
                        window.push_notification(format!("Could not delete the flow: {e}"), cx)
                    }
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

    fn render_toolbar(&self) -> impl IntoElement {
        // With no flows yet, the empty state offers the templates and a
        // create button, so the header does not repeat it.
        let show_new = !(self.loaded && self.flows.is_empty());
        h_flex()
            .gap_3()
            .items_center()
            .flex_wrap()
            .child(
                div()
                    .id("flows-search")
                    .key_context(SEARCH_CONTEXT)
                    .w(px(320.))
                    .child(
                        Input::new(&self.search)
                            .cleanable(true)
                            .small()
                            .prefix(Icon::new(IconName::Search).size_4()),
                    ),
            )
            .child(div().flex_1())
            .when(show_new, |this| {
                this.child(
                    Button::new("new-flow")
                        .primary()
                        .small()
                        .icon(Icon::new(Icon::empty()).path("icons/plus.svg"))
                        .label("New flow")
                        .cursor_pointer()
                        .dropdown_menu(|menu, _, _| {
                            menu.menu("From scratch", Box::new(NewFlow))
                                .menu("From template", Box::new(BrowseTemplates))
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
        let running = self.running.as_deref() == Some(&flow.id);
        let description = if flow.description.trim().is_empty() {
            step_count_label(flow)
        } else {
            flow.description.trim().to_string()
        };
        let on_click_ix = filtered_ix;

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
            .hover(|s| s.bg(theme.secondary))
            .cursor_pointer()
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
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .truncate()
                                    .child(flow.name.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .truncate()
                                    .child(description),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_0p5()
                            .flex_shrink_0()
                            .child(
                                Button::new(("run-flow", filtered_ix))
                                    .ghost()
                                    .xsmall()
                                    .tab_stop(false)
                                    .loading(running)
                                    .icon(Icon::new(Icon::empty()).path("icons/play.svg"))
                                    .tooltip_with_action("Run", &RunSelected, Some(GRID_CONTEXT))
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        cx.stop_propagation();
                                        this.run(filtered_ix, window, cx);
                                    })),
                            )
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
                                        menu.menu("Duplicate", Box::new(DuplicateFlow(filtered_ix)))
                                            .menu("Export…", Box::new(ExportFlow(filtered_ix)))
                                            .separator()
                                            .menu("Delete", Box::new(DeleteFlow(filtered_ix)))
                                    }),
                            ),
                    ),
            )
            .child(step_strip(flow, summaries, cx))
            .child(trigger_chips(flow, self.chords.get(&flow.id), cx))
    }

    fn render_grid(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
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
        v_flex().gap_4().children(rows)
    }

    fn render_empty(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        if !self.query.trim().is_empty() {
            return v_flex()
                .items_center()
                .py_12()
                .gap_1()
                .child(
                    div()
                        .text_color(theme.muted_foreground)
                        .child(format!("No flows match \"{}\"", self.query.trim())),
                )
                .into_any_element();
        }
        if !self.loaded {
            return div().into_any_element();
        }

        let templates = templates();
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
                            .child(
                                "A flow strings actions together: open apps, switch workspaces, \
                                 send a notification, wait a moment.",
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_left()
                            .text_color(theme.muted_foreground)
                            .child(
                                "Run it from a keybind, the app launcher, at startup, or with \
                                 one command.",
                            ),
                    ),
            )
            .child(
                Button::new("new-flow-empty")
                    .primary()
                    .icon(Icon::new(Icon::empty()).path("icons/plus.svg"))
                    .label("Create a flow")
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

impl Render for FlowsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let width = window.viewport_size().width;
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
            .gap_4()
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
            .on_action(cx.listener(|this, _: &ImportFlow, window, cx| {
                this.import_from_dialog(window, cx);
            }))
            .on_action(cx.listener(|_, _: &BrowseTemplates, _, cx| {
                emit(cx, AppEvent::Navigate(ActivePage::FlowTemplates));
            }))
            .on_action(cx.listener(|this, action: &ExportFlow, window, cx| {
                this.export(action.0, window, cx);
            }))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, window, cx| {
                if let Some(path) = paths.paths().first() {
                    this.import_path(path.clone(), window, cx);
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
            .child(self.render_toolbar())
            .map(|this| {
                let scroll = div()
                    .id("flows-grid")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .pb_8();
                if self.filtered.is_empty() {
                    // The empty state's buttons are ordinary tab stops, so it
                    // stays outside the grid's roving focus container.
                    this.child(scroll.child(self.render_empty(cx)))
                } else {
                    this.child(
                        scroll
                            .key_context(GRID_CONTEXT)
                            .track_focus(&self.grid_focus)
                            .child(self.render_grid(window, cx)),
                    )
                }
            })
    }
}
