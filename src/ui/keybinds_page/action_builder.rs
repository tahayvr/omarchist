//! The "what it does" part of the keybind dialog: a kind selector and,
//! per kind, the controls that assemble an [`Action`].
use std::path::PathBuf;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, IndexPath, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    v_flex,
};

use crate::system::apps::{DesktopApp, installed_apps};
use crate::system::flows::Flow;
use crate::system::flows::store::load_flows;
use crate::system::keybinds::Dispatcher;
use crate::system::keybinds::action::{
    Action, ActionKind, AppLaunch, Direction, OMARCHY_ACTIONS, WindowAction, WindowActionKind,
    WindowParam, WorkspaceTarget, omarchy_entry, program_name,
};
use crate::ui::focus::{self, FocusableSwitch};
use crate::ui::keybinds_page::keybinds_view::{FILTERS_CONTEXT, keybinds_nav};

pub enum ActionBuilderEvent {
    Changed,
}

// MARK: Picker items

#[derive(Clone)]
struct AppItem {
    id: String,
    name: SharedString,
    exec: SharedString,
    icon: Option<PathBuf>,
}

impl SelectItem for AppItem {
    type Value = String;

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn value(&self) -> &String {
        &self.id
    }

    fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.name.to_lowercase().contains(&query)
            || self.exec.to_lowercase().contains(&query)
            || self.id.to_lowercase().contains(&query)
    }

    fn render(&self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .gap_2()
            .min_w_0()
            .child(app_icon(self.icon.as_ref()))
            .child(div().flex_shrink_0().child(self.name.clone()))
            .child(
                div()
                    .min_w_0()
                    .overflow_hidden()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.exec.clone()),
            )
    }
}

fn app_icon(icon: Option<&PathBuf>) -> AnyElement {
    match icon {
        Some(path) => img(path.clone())
            .size(px(18.))
            .flex_shrink_0()
            .into_any_element(),
        None => Icon::new(Icon::empty())
            .path("icons/app-window.svg")
            .size_4()
            .flex_shrink_0()
            .into_any_element(),
    }
}

#[derive(Clone)]
struct LabeledItem {
    id: String,
    label: SharedString,
    group: SharedString,
}

impl SelectItem for LabeledItem {
    type Value = String;

    fn title(&self) -> SharedString {
        self.label.clone()
    }

    fn value(&self) -> &String {
        &self.id
    }

    fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.label.to_lowercase().contains(&query) || self.group.to_lowercase().contains(&query)
    }

    fn render(&self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .gap_2()
            .justify_between()
            .child(self.label.clone())
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.group.clone()),
            )
    }
}

type Picker = Entity<SelectState<SearchableVec<LabeledItem>>>;

fn step_count(flow: &Flow) -> String {
    let n = flow.enabled_steps();
    format!("{n} step{}", if n == 1 { "" } else { "s" })
}

fn same_url(a: &str, b: &str) -> bool {
    a.trim().trim_end_matches('/') == b.trim().trim_end_matches('/')
}

fn workspace_targets() -> Vec<WorkspaceTarget> {
    let mut targets: Vec<WorkspaceTarget> = (1..=10).map(WorkspaceTarget::Number).collect();
    targets.extend([
        WorkspaceTarget::Next,
        WorkspaceTarget::Previous,
        WorkspaceTarget::Former,
    ]);
    targets
}

fn workspace_id(target: WorkspaceTarget) -> String {
    match target {
        WorkspaceTarget::Number(n) => n.to_string(),
        WorkspaceTarget::Next => "next".into(),
        WorkspaceTarget::Previous => "previous".into(),
        WorkspaceTarget::Former => "former".into(),
    }
}

// MARK: Builder

pub struct ActionBuilder {
    kind: ActionKind,
    kind_focus: FocusHandle,
    /// Whether the builder draws its own kind selector; a host that has one
    /// of its own (the flow step builder) hides it and calls `set_kind`.
    kind_strip: bool,
    direction_focus: FocusHandle,

    apps: Vec<DesktopApp>,
    app_select: Entity<SelectState<SearchableVec<AppItem>>>,
    /// The app chosen, or carried over from a bind whose entry is not installed.
    app: Option<(AppLaunch, String)>,
    /// Exec of an edited bind, matched against the entries once they load.
    pending_app_exec: Option<String>,
    app_focus: bool,

    webapp_select: Entity<SelectState<SearchableVec<AppItem>>>,
    webapp_url: Entity<InputState>,
    webapp_name: String,
    webapp_focus: bool,

    terminal_command: Entity<InputState>,
    terminal_focus: bool,

    omarchy_select: Picker,
    omarchy_id: Option<&'static str>,

    window_select: Picker,
    window: Option<WindowAction>,
    workspace_select: Picker,
    resize_x: Entity<InputState>,
    resize_y: Entity<InputState>,
    /// A dispatcher the builder cannot express, kept until replaced.
    kept_lua: Option<String>,

    flows: Vec<Flow>,
    flow_select: Picker,
    flow_id: Option<String>,

    command: Entity<InputState>,

    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ActionBuilderEvent> for ActionBuilder {}

impl ActionBuilder {
    pub fn new(initial: Option<&Dispatcher>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let initial_action = initial.and_then(Action::from_dispatcher);
        let kept_lua = match (initial, &initial_action) {
            (Some(Dispatcher::Lua(expr)), None) => Some(expr.clone()),
            _ => None,
        };
        let kind = initial_action
            .as_ref()
            .map(Action::kind)
            .unwrap_or(if kept_lua.is_some() {
                ActionKind::Window
            } else {
                ActionKind::App
            });

        let text = |window: &mut Window, cx: &mut Context<Self>, placeholder: &str, value: &str| {
            cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(placeholder.to_string())
                    .default_value(value.to_string())
            })
        };
        let picker = |window: &mut Window,
                      cx: &mut Context<Self>,
                      items: Vec<LabeledItem>,
                      selected: Option<&str>| {
            let index = selected
                .and_then(|id| items.iter().position(|i| i.id == id))
                .map(|ix| IndexPath::default().row(ix));
            cx.new(|cx| {
                SelectState::new(SearchableVec::new(items), index, window, cx).searchable(true)
            })
        };

        let (app, app_focus, pending_app_exec) = match &initial_action {
            Some(Action::App { app, focus }) => (
                Some((app.clone(), app.exec.clone())),
                *focus,
                Some(app.exec.clone()),
            ),
            _ => (None, false, None),
        };
        let (webapp_url_value, webapp_name, webapp_focus) = match &initial_action {
            Some(Action::WebApp { url, name, focus }) => (url.clone(), name.clone(), *focus),
            _ => (String::new(), String::new(), false),
        };
        let (terminal_value, terminal_focus) = match &initial_action {
            Some(Action::Terminal { command, focus }) => (command.clone(), *focus),
            _ => (String::new(), false),
        };
        let omarchy_id = match &initial_action {
            Some(Action::Omarchy(entry)) => Some(entry.id),
            _ => None,
        };
        let window_action = match &initial_action {
            Some(Action::Window(action)) => Some(action.clone()),
            _ => None,
        };
        let flow_id = match &initial_action {
            Some(Action::Flow(id)) => Some(id.clone()),
            _ => None,
        };
        let command_value = match &initial_action {
            Some(Action::Command(command)) => command.clone(),
            _ => String::new(),
        };

        let app_select = cx.new(|cx| {
            SelectState::new(SearchableVec::new(Vec::<AppItem>::new()), None, window, cx)
                .searchable(true)
        });
        let webapp_select = cx.new(|cx| {
            SelectState::new(SearchableVec::new(Vec::<AppItem>::new()), None, window, cx)
                .searchable(true)
        });
        let webapp_url = text(window, cx, "https://example.com", &webapp_url_value);
        let terminal_command = text(window, cx, "Command to run, e.g. btop", &terminal_value);
        let omarchy_items = OMARCHY_ACTIONS
            .iter()
            .map(|e| LabeledItem {
                id: e.id.to_string(),
                label: e.label.into(),
                group: e.group.into(),
            })
            .collect();
        let omarchy_select = picker(window, cx, omarchy_items, omarchy_id);
        let window_items = WindowActionKind::ALL
            .iter()
            .map(|k| LabeledItem {
                id: k.id().to_string(),
                label: k.label().into(),
                group: k.group().into(),
            })
            .collect();
        let window_select = picker(
            window,
            cx,
            window_items,
            window_action.as_ref().map(|a| a.kind.id()),
        );
        let workspace_items = workspace_targets()
            .into_iter()
            .map(|t| LabeledItem {
                id: workspace_id(t),
                label: t.label().into(),
                group: "".into(),
            })
            .collect();
        let workspace_select = picker(
            window,
            cx,
            workspace_items,
            Some(&workspace_id(
                window_action
                    .as_ref()
                    .map(|a| a.workspace)
                    .unwrap_or(WorkspaceTarget::Number(1)),
            )),
        );
        let (dx, dy) = window_action
            .as_ref()
            .map(|a| (a.dx, a.dy))
            .unwrap_or((100, 0));
        let resize_x = text(window, cx, "0", &dx.to_string());
        let resize_y = text(window, cx, "0", &dy.to_string());
        let flows = load_flows().unwrap_or_default();
        let flow_items = flows
            .iter()
            .map(|f| LabeledItem {
                id: f.id.clone(),
                label: f.name.clone().into(),
                group: step_count(f).into(),
            })
            .collect();
        let flow_select = picker(window, cx, flow_items, flow_id.as_deref());
        let command = text(
            window,
            cx,
            "Command to run, e.g. omarchy-launch-terminal",
            &command_value,
        );

        let mut subscriptions = Vec::new();
        for input in [
            &webapp_url,
            &terminal_command,
            &resize_x,
            &resize_y,
            &command,
        ] {
            subscriptions.push(cx.subscribe_in(
                input,
                window,
                |this, _, event: &InputEvent, _window, cx| {
                    if matches!(event, InputEvent::Change) {
                        this.sync_resize(cx);
                        this.changed(cx);
                    }
                },
            ));
        }
        subscriptions.push(cx.subscribe_in(
            &app_select,
            window,
            |this, _, event: &SelectEvent<SearchableVec<AppItem>>, _window, cx| {
                if let SelectEvent::Confirm(Some(id)) = event {
                    this.select_app(id, cx);
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &webapp_select,
            window,
            |this, _, event: &SelectEvent<SearchableVec<AppItem>>, window, cx| {
                if let SelectEvent::Confirm(Some(id)) = event {
                    this.select_webapp(id, window, cx);
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &omarchy_select,
            window,
            |this, _, event: &SelectEvent<SearchableVec<LabeledItem>>, _window, cx| {
                if let SelectEvent::Confirm(Some(id)) = event {
                    this.omarchy_id = omarchy_entry(id).map(|e| e.id);
                    this.changed(cx);
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &window_select,
            window,
            |this, _, event: &SelectEvent<SearchableVec<LabeledItem>>, _window, cx| {
                if let SelectEvent::Confirm(Some(id)) = event
                    && let Some(kind) = WindowActionKind::from_id(id)
                {
                    let mut action = this
                        .window
                        .clone()
                        .unwrap_or_else(|| WindowAction::new(kind));
                    action.kind = kind;
                    this.window = Some(action);
                    this.kept_lua = None;
                    this.changed(cx);
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &workspace_select,
            window,
            |this, _, event: &SelectEvent<SearchableVec<LabeledItem>>, _window, cx| {
                if let SelectEvent::Confirm(Some(id)) = event {
                    let target = workspace_targets()
                        .into_iter()
                        .find(|t| workspace_id(*t) == *id);
                    if let (Some(target), Some(action)) = (target, this.window.as_mut()) {
                        action.workspace = target;
                        this.changed(cx);
                    }
                }
            },
        ));

        subscriptions.push(cx.subscribe_in(
            &flow_select,
            window,
            |this, _, event: &SelectEvent<SearchableVec<LabeledItem>>, _window, cx| {
                if let SelectEvent::Confirm(Some(id)) = event {
                    this.flow_id = Some(id.clone());
                    this.changed(cx);
                }
            },
        ));

        let builder = Self {
            kind,
            kind_focus: focus::tab_stop(cx),
            kind_strip: true,
            direction_focus: focus::tab_stop(cx),
            apps: Vec::new(),
            app_select,
            app,
            pending_app_exec,
            app_focus,
            webapp_select,
            webapp_url,
            webapp_name,
            webapp_focus,
            terminal_command,
            terminal_focus,
            omarchy_select,
            omarchy_id,
            window_select,
            window: window_action,
            workspace_select,
            resize_x,
            resize_y,
            kept_lua,
            flows,
            flow_select,
            flow_id,
            command,
            _subscriptions: subscriptions,
        };
        builder.load_apps(window, cx);
        builder
    }

    fn load_apps(&self, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            let apps = cx.background_spawn(async { installed_apps() }).await;
            this.update_in(cx, |this, window, cx| this.set_apps(apps, window, cx))
                .ok();
        })
        .detach();
    }

    fn set_apps(&mut self, apps: Vec<DesktopApp>, window: &mut Window, cx: &mut Context<Self>) {
        let items: Vec<AppItem> = apps
            .iter()
            .filter(|a| !a.is_webapp())
            .map(|a| AppItem {
                id: a.id.clone(),
                name: a.name.clone().into(),
                exec: a.exec.clone().into(),
                icon: a.icon.clone(),
            })
            .collect();
        let webapps: Vec<AppItem> = apps
            .iter()
            .filter(|a| a.is_webapp())
            .map(|a| AppItem {
                id: a.id.clone(),
                name: a.name.clone().into(),
                exec: a.webapp_url.clone().unwrap_or_default().into(),
                icon: a.icon.clone(),
            })
            .collect();
        self.apps = apps;

        let matched = self.pending_app_exec.take().and_then(|exec| {
            let apps = self.apps.iter().filter(|a| !a.is_webapp());
            apps.clone()
                .find(|a| a.exec == exec)
                .or_else(|| {
                    let program = program_name(&exec);
                    apps.clone().find(|a| program_name(&a.exec) == program)
                })
                .map(|a| a.id.clone())
        });
        // A matched entry is shown as selected, but the bind keeps its own
        // launch command until the user picks an app.
        self.app_select.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(items), window, cx);
            if let Some(id) = &matched {
                select.set_selected_value(id, window, cx);
            }
        });

        let current_url = self.webapp_url.read(cx).value().to_string();
        let current_webapp = self
            .apps
            .iter()
            .find(|a| {
                a.webapp_url
                    .as_deref()
                    .is_some_and(|url| same_url(url, &current_url))
            })
            .map(|a| a.id.clone());
        self.webapp_select.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(webapps), window, cx);
            if let Some(id) = &current_webapp {
                select.set_selected_value(id, window, cx);
            }
        });
        cx.notify();
    }

    fn select_app(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(app) = self.apps.iter().find(|a| a.id == id) {
            self.app = Some((
                AppLaunch {
                    exec: app.exec.clone(),
                    wm_class: app.wm_class.clone(),
                    terminal: app.terminal,
                },
                app.name.clone(),
            ));
            self.changed(cx);
        }
    }

    fn select_webapp(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(app) = self.apps.iter().find(|a| a.id == id) else {
            return;
        };
        let url = app.webapp_url.clone().unwrap_or_default();
        self.webapp_name = app.name.clone();
        self.webapp_url
            .update(cx, |input, cx| input.set_value(url, window, cx));
        self.changed(cx);
    }

    fn sync_resize(&mut self, cx: &mut Context<Self>) {
        if let Some(action) = self.window.as_mut() {
            let parse = |input: &Entity<InputState>| {
                input.read(cx).value().trim().parse::<i32>().unwrap_or(0)
            };
            action.dx = parse(&self.resize_x);
            action.dy = parse(&self.resize_y);
        }
    }

    pub fn set_kind(&mut self, kind: ActionKind, cx: &mut Context<Self>) {
        if self.kind != kind {
            self.kind = kind;
            self.changed(cx);
        }
    }

    pub fn kind(&self) -> ActionKind {
        self.kind
    }

    pub fn set_kind_strip(&mut self, shown: bool, cx: &mut Context<Self>) {
        self.kind_strip = shown;
        cx.notify();
    }

    /// Drops a flow from the Flow picker, so a flow cannot pick itself.
    pub fn exclude_flow(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.flows.retain(|f| f.id != id);
        if self.flow_id.as_deref() == Some(id) {
            self.flow_id = None;
        }
        let items: Vec<LabeledItem> = self
            .flows
            .iter()
            .map(|f| LabeledItem {
                id: f.id.clone(),
                label: f.name.clone().into(),
                group: step_count(f).into(),
            })
            .collect();
        let selected = self.flow_id.clone();
        self.flow_select.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(items), window, cx);
            if let Some(id) = &selected {
                select.set_selected_value(id, window, cx);
            }
        });
    }

    fn cycle_kind(&mut self, delta: isize, cx: &mut Context<Self>) {
        let all = ActionKind::ALL;
        let ix = all.iter().position(|k| *k == self.kind).unwrap_or(0) as isize;
        let next = (ix + delta).rem_euclid(all.len() as isize) as usize;
        self.set_kind(all[next], cx);
    }

    fn set_direction(&mut self, direction: Direction, cx: &mut Context<Self>) {
        if let Some(action) = self.window.as_mut() {
            action.direction = direction;
            self.changed(cx);
        }
    }

    fn cycle_direction(&mut self, delta: isize, cx: &mut Context<Self>) {
        let all = Direction::ALL;
        let current = self
            .window
            .as_ref()
            .map(|a| a.direction)
            .unwrap_or(Direction::Left);
        let ix = all.iter().position(|d| *d == current).unwrap_or(0) as isize;
        let next = (ix + delta).rem_euclid(all.len() as isize) as usize;
        self.set_direction(all[next], cx);
    }

    fn changed(&mut self, cx: &mut Context<Self>) {
        cx.emit(ActionBuilderEvent::Changed);
        cx.notify();
    }

    /// The action the controls currently describe, or why they don't.
    pub fn action(&self, cx: &App) -> Result<Action, String> {
        match self.kind {
            ActionKind::App => self
                .app
                .as_ref()
                .map(|(app, _)| Action::App {
                    app: app.clone(),
                    focus: self.app_focus,
                })
                .ok_or_else(|| "Choose an app to launch".to_string()),
            ActionKind::WebApp => {
                let url = self.webapp_url.read(cx).value().trim().to_string();
                if url.is_empty() {
                    return Err("Enter the web app's address".into());
                }
                if !(url.starts_with("https://") || url.starts_with("http://")) {
                    return Err("The address must start with https:// or http://".into());
                }
                Ok(Action::WebApp {
                    url,
                    name: self.webapp_name.clone(),
                    focus: self.webapp_focus,
                })
            }
            ActionKind::Terminal => {
                let command = self.terminal_command.read(cx).value().trim().to_string();
                if command.is_empty() {
                    return Err("Enter the command to run in a terminal".into());
                }
                Ok(Action::Terminal {
                    command,
                    focus: self.terminal_focus,
                })
            }
            ActionKind::Omarchy => self
                .omarchy_id
                .and_then(omarchy_entry)
                .map(Action::Omarchy)
                .ok_or_else(|| "Choose an Omarchy action".to_string()),
            ActionKind::Window => self
                .window
                .clone()
                .map(Action::Window)
                .ok_or_else(|| "Choose a window action".to_string()),
            ActionKind::Flow => self
                .flow_id
                .clone()
                .map(Action::Flow)
                .ok_or_else(|| "Choose a flow to run".to_string()),
            ActionKind::Command => {
                let command = self.command.read(cx).value().trim().to_string();
                if command.is_empty() {
                    return Err("Enter the command to run".into());
                }
                Ok(Action::Command(command))
            }
        }
    }

    /// The dispatcher to save. A kept Lua expression wins while nothing
    /// replaces it.
    pub fn dispatcher(&self, cx: &App) -> Result<Dispatcher, String> {
        if self.kind == ActionKind::Window
            && self.window.is_none()
            && let Some(expr) = &self.kept_lua
        {
            return Ok(Dispatcher::Lua(expr.clone()));
        }
        self.action(cx).map(|a| a.dispatcher())
    }

    /// A description for a new bind, derived from the chosen action.
    pub fn suggested_description(&self, cx: &App) -> Option<String> {
        match self.action(cx).ok()? {
            Action::App { .. } => self.app.as_ref().map(|(_, name)| name.clone()),
            Action::Flow(id) => self
                .flows
                .iter()
                .find(|f| f.id == id)
                .map(|f| f.name.clone()),
            action => action.summary(),
        }
    }

    // MARK: Render

    fn render_kinds(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.kind_focus.is_focused(window);
        let ring = focus::focus_border(focused, cx.theme().transparent, cx);
        h_flex()
            .id("action-kinds")
            .key_context(FILTERS_CONTEXT)
            .track_focus(&self.kind_focus)
            .on_action(
                cx.listener(|this, _: &keybinds_nav::FilterPrev, _, cx| this.cycle_kind(-1, cx)),
            )
            .on_action(
                cx.listener(|this, _: &keybinds_nav::FilterNext, _, cx| this.cycle_kind(1, cx)),
            )
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(ring)
            .p_0p5()
            .gap_1()
            .flex_wrap()
            .children(ActionKind::ALL.iter().enumerate().map(|(ix, &kind)| {
                let icon = Icon::new(Icon::empty()).path(kind.icon_path());
                let button = Button::new(("action-kind", ix))
                    .icon(icon)
                    .label(kind.label())
                    .small()
                    .tab_stop(false)
                    .cursor_pointer();
                let button = if self.kind == kind {
                    button.primary()
                } else {
                    button.ghost()
                };
                button.on_click(cx.listener(move |this, _, _window, cx| this.set_kind(kind, cx)))
            }))
    }

    fn render_directions(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.direction_focus.is_focused(window);
        let ring = focus::focus_border(focused, cx.theme().transparent, cx);
        let current = self.window.as_ref().map(|a| a.direction);
        h_flex()
            .id("action-directions")
            .key_context(FILTERS_CONTEXT)
            .track_focus(&self.direction_focus)
            .on_action(
                cx.listener(|this, _: &keybinds_nav::FilterPrev, _, cx| {
                    this.cycle_direction(-1, cx)
                }),
            )
            .on_action(
                cx.listener(|this, _: &keybinds_nav::FilterNext, _, cx| {
                    this.cycle_direction(1, cx)
                }),
            )
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(ring)
            .p_0p5()
            .gap_1()
            .children(Direction::ALL.iter().enumerate().map(|(ix, &direction)| {
                let icon = match direction {
                    Direction::Left => IconName::ArrowLeft,
                    Direction::Right => IconName::ArrowRight,
                    Direction::Up => IconName::ArrowUp,
                    Direction::Down => IconName::ArrowDown,
                };
                let button = Button::new(("action-direction", ix))
                    .icon(icon)
                    .label(direction.label())
                    .small()
                    .tab_stop(false)
                    .cursor_pointer();
                let button = if current == Some(direction) {
                    button.primary()
                } else {
                    button.ghost()
                };
                button.on_click(
                    cx.listener(move |this, _, _window, cx| this.set_direction(direction, cx)),
                )
            }))
    }

    fn focus_switch(
        &self,
        id: &'static str,
        checked: bool,
        on_change: impl Fn(&mut Self, bool, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div().text_sm().child(
            FocusableSwitch::new(id)
                .label("Focus the window if it is already open")
                .checked(checked)
                .on_change(cx.listener(move |this, checked, _window, cx| {
                    on_change(this, *checked, cx);
                    this.changed(cx);
                })),
        )
    }

    fn label(text: &'static str, cx: &App) -> Div {
        div()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(cx.theme().muted_foreground)
            .child(text)
    }

    fn hint(text: impl Into<SharedString>, cx: &App) -> Div {
        div()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(text.into())
    }

    fn render_body(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        match self.kind {
            ActionKind::App => {
                let carried = self.app.as_ref().filter(|(app, _)| {
                    !self.apps.is_empty()
                        && !self
                            .apps
                            .iter()
                            .any(|a| program_name(&a.exec) == program_name(&app.exec))
                });
                v_flex()
                    .gap_2()
                    .child(
                        Select::new(&self.app_select)
                            .placeholder("Choose an installed app")
                            .search_placeholder("Search apps")
                            .menu_max_h(px(320.))
                            .small(),
                    )
                    .when_some(carried, |this, (app, _)| {
                        this.child(Self::hint(
                            format!("Currently launches `{}`, which has no menu entry.", app.exec),
                            cx,
                        ))
                    })
                    .child(self.focus_switch(
                        "action-app-focus",
                        self.app_focus,
                        |this, checked, _| this.app_focus = checked,
                        cx,
                    ))
                    .into_any_element()
            }
            ActionKind::WebApp => v_flex()
                .gap_2()
                .child(Input::new(&self.webapp_url).small())
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(Self::hint("or pick one you have installed", cx))
                        .child(
                            div().flex_1().min_w_0().child(
                                Select::new(&self.webapp_select)
                                    .placeholder("Installed web apps")
                                    .search_placeholder("Search web apps")
                                    .menu_max_h(px(320.))
                                    .small(),
                            ),
                        ),
                )
                .child(self.focus_switch(
                    "action-webapp-focus",
                    self.webapp_focus,
                    |this, checked, _| this.webapp_focus = checked,
                    cx,
                ))
                .into_any_element(),
            ActionKind::Terminal => v_flex()
                .gap_2()
                .child(Input::new(&self.terminal_command).small())
                .child(Self::hint(
                    "Opens a terminal window running the command, the way Omarchy launches btop or lazydocker.",
                    cx,
                ))
                .child(self.focus_switch(
                    "action-terminal-focus",
                    self.terminal_focus,
                    |this, checked, _| this.terminal_focus = checked,
                    cx,
                ))
                .into_any_element(),
            ActionKind::Omarchy => v_flex()
                .gap_2()
                .child(
                    Select::new(&self.omarchy_select)
                        .placeholder("Choose an Omarchy action")
                        .search_placeholder("Search menus, panels, media, capture…")
                        .menu_max_h(px(320.))
                        .small(),
                )
                .into_any_element(),
            ActionKind::Window => {
                let param = self.window.as_ref().map(|a| a.kind.param());
                v_flex()
                    .gap_2()
                    .child(
                        Select::new(&self.window_select)
                            .placeholder("Choose a window or workspace action")
                            .search_placeholder("Search window actions")
                            .menu_max_h(px(320.))
                            .small(),
                    )
                    .when_some(self.kept_lua.as_ref().filter(|_| self.window.is_none()), |this, expr| {
                        this.child(
                            v_flex().gap_1().child(Self::hint("Currently runs this Hyprland dispatcher, which the builder cannot edit. Choosing an action replaces it.", cx)).child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded(theme.radius)
                                    .bg(theme.secondary)
                                    .font_family("monospace")
                                    .text_xs()
                                    .child(expr.clone()),
                            ),
                        )
                    })
                    .when(param == Some(WindowParam::Direction), |this| {
                        this.child(
                            v_flex()
                                .gap_1()
                                .child(Self::label("Direction", cx))
                                .child(self.render_directions(window, cx)),
                        )
                    })
                    .when(
                        matches!(param, Some(WindowParam::Workspace | WindowParam::WorkspaceMove)),
                        |this| {
                            this.child(
                                v_flex().gap_1().child(Self::label("Workspace", cx)).child(
                                    Select::new(&self.workspace_select)
                                        .placeholder("Workspace")
                                        .menu_max_h(px(320.))
                                        .small(),
                                ),
                            )
                        },
                    )
                    .when(param == Some(WindowParam::WorkspaceMove), |this| {
                        let follow = self.window.as_ref().is_some_and(|a| a.follow);
                        this.child(
                            div().text_sm().child(
                                FocusableSwitch::new("action-window-follow")
                                    .label("Follow the window to its workspace")
                                    .checked(follow)
                                    .on_change(cx.listener(|this, checked, _window, cx| {
                                        if let Some(action) = this.window.as_mut() {
                                            action.follow = *checked;
                                        }
                                        this.changed(cx);
                                    })),
                            ),
                        )
                    })
                    .when(param == Some(WindowParam::Resize), |this| {
                        let field = |label: &'static str, input: &Entity<InputState>| {
                            v_flex()
                                .gap_1()
                                .w_32()
                                .child(Self::label(label, cx))
                                .child(Input::new(input).small())
                        };
                        this.child(
                            v_flex()
                                .gap_1()
                                .child(
                                    h_flex()
                                        .gap_3()
                                        .child(field("Width change (px)", &self.resize_x))
                                        .child(field("Height change (px)", &self.resize_y)),
                                )
                                .child(Self::hint(
                                    "Negative values shrink the window.",
                                    cx,
                                )),
                        )
                    })
                    .into_any_element()
            }
            ActionKind::Flow => v_flex()
                .gap_2()
                .child(
                    Select::new(&self.flow_select)
                        .placeholder(if self.flows.is_empty() {
                            "No flows yet"
                        } else {
                            "Choose a flow"
                        })
                        .search_placeholder("Search flows")
                        .menu_max_h(px(320.))
                        .small(),
                )
                .child(Self::hint(
                    if self.flows.is_empty() {
                        "Create a flow on the Flows page first: a sequence of actions that runs from one keybind."
                    } else {
                        "Runs every step of the flow, the same as the Run button on the Flows page."
                    },
                    cx,
                ))
                .into_any_element(),
            ActionKind::Command => v_flex()
                .gap_2()
                .child(Input::new(&self.command).small())
                .child(Self::hint(
                    "Runs through Hyprland's exec dispatcher, so shell syntax such as || works.",
                    cx,
                ))
                .into_any_element(),
        }
    }

    fn render_preview(&self, cx: &App) -> impl IntoElement {
        let theme = cx.theme();
        let row = h_flex().gap_2().items_start().text_xs();
        match self.dispatcher(cx) {
            Ok(dispatcher) => row
                .child(
                    div()
                        .flex_shrink_0()
                        .text_color(theme.muted_foreground)
                        .child(match dispatcher {
                            Dispatcher::Lua(_) => "Dispatches",
                            _ => "Runs",
                        }),
                )
                .child(
                    div()
                        .min_w_0()
                        .px_2()
                        .py_0p5()
                        .rounded(theme.radius)
                        .bg(theme.secondary)
                        .font_family("monospace")
                        .child(dispatcher.text().to_string()),
                ),
            Err(message) => row.child(div().text_color(theme.muted_foreground).child(message)),
        }
    }
}

impl Render for ActionBuilder {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .when(self.kind_strip, |this| {
                this.child(self.render_kinds(window, cx))
            })
            .child(self.render_body(window, cx))
            .child(self.render_preview(cx))
    }
}
