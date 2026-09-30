//! The Home page: what the machine looks like right now and the shortest
//! way to everything Omarchist does. A greeting, the running theme as a
//! palette ribbon, four tiles that count what has been changed and open
//! the page behind them, the themes made here as small cards, and the
//! flows with a Run button. Everything is read again on every visit.
use chrono::{Local, Timelike};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex, v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::flows::Flow;
use crate::system::flows::runner::run_in_thread;
use crate::system::flows::store::load_flows;
use crate::system::hyprland_config::manager::saved_overrides;
use crate::system::keybinds::store::load_overrides;
use crate::system::omarchy_paths::current_theme_dir;
use crate::system::themes::custom_themes::get_user_themes;
use crate::system::themes::parse_colors::parse_colors_toml;
use crate::system::themes::utils::dir_to_title;
use crate::system::ui_theme_watcher::get_active_omarchy_theme_name;
use crate::types::themes::{ThemeColors, ThemeEntry};
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::color_utils::hex_to_hsla;
use crate::ui::flows_page::flow_card::icon_tile;
use crate::ui::focus;
use crate::ui::omarchy_page::updates::{OmarchyUpdates, UpdateState};
use crate::ui::text::selectable;

const KEY_CONTEXT: &str = "HomePage";
/// Wraps the cards and tiles: arrows move, Enter opens, Ctrl+Enter acts.
pub const GRID_CONTEXT: &str = "HomeGrid";

pub mod home_nav {
    gpui::actions!(
        home,
        [Prev, Next, PrevSection, NextSection, First, Last, Open, Act]
    );
}
use home_nav::*;

/// One focusable thing on the page, in reading order.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Item {
    Tile(Tile),
    Theme(usize),
    Flow(usize),
}

/// The four counters under the ribbon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Themes,
    Configuration,
    Keybinds,
    Flows,
}

impl Tile {
    const ALL: [Tile; 4] = [
        Tile::Themes,
        Tile::Configuration,
        Tile::Keybinds,
        Tile::Flows,
    ];

    fn page(self) -> ActivePage {
        match self {
            Tile::Themes => ActivePage::Themes,
            Tile::Configuration => ActivePage::Configuration,
            Tile::Keybinds => ActivePage::Keybinds,
            Tile::Flows => ActivePage::Flows,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Tile::Themes => "Themes",
            Tile::Configuration => "Configuration",
            Tile::Keybinds => "Keybinds",
            Tile::Flows => "Flows",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Tile::Themes => "icons/palette.svg",
            Tile::Configuration => "icons/wrench.svg",
            Tile::Keybinds => "icons/keyboard.svg",
            Tile::Flows => "icons/workflow.svg",
        }
    }
}

/// Everything the page shows, read off the UI thread.
#[derive(Debug, Clone, Default)]
struct Snapshot {
    /// The theme Omarchy runs: folder, title, and its colors when readable.
    current: Option<(String, String, Option<ThemeColors>)>,
    themes: Vec<ThemeEntry>,
    flows: Vec<Flow>,
    keybinds_changed: usize,
    hyprland_changed: usize,
}

impl Snapshot {
    fn read() -> Self {
        let current = get_active_omarchy_theme_name().map(|dir| {
            let colors = current_theme_dir()
                .map(|d| d.join("colors.toml"))
                .and_then(|p| parse_colors_toml(&p));
            (dir.clone(), dir_to_title(&dir), colors)
        });
        let mut themes = get_user_themes().unwrap_or_default();
        themes.sort_by_key(|t| t.title.to_lowercase());
        let applied = current.as_ref().map(|(dir, _, _)| dir.clone());
        for theme in &mut themes {
            theme.applied = applied.as_deref() == Some(theme.dir.as_str());
        }
        Self {
            current,
            themes,
            flows: load_flows().unwrap_or_default(),
            keybinds_changed: load_overrides().map(|o| o.overrides.len()).unwrap_or(0),
            hyprland_changed: count_leaves(&saved_overrides()),
        }
    }
}

/// The number of settings in a sparse overrides object.
fn count_leaves(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(map) => map.values().map(count_leaves).sum(),
        serde_json::Value::Null => 0,
        _ => 1,
    }
}

/// The greeting for the hour.
pub fn greeting(hour: u32) -> &'static str {
    match hour {
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        18..=22 => "Good evening",
        _ => "Good night",
    }
}

pub struct HomeView {
    pub focus_handle: FocusHandle,
    grid_focus: FocusHandle,
    scroll: ScrollHandle,
    snapshot: Snapshot,
    loaded: bool,
    focused: usize,
    running: Option<String>,
    updates: Entity<OmarchyUpdates>,
    _updates_observer: Subscription,
}

impl HomeView {
    pub fn new(updates: Entity<OmarchyUpdates>, cx: &mut Context<Self>) -> Self {
        let observer = cx.observe(&updates, |_, _, cx| cx.notify());
        let mut view = Self {
            focus_handle: cx.focus_handle(),
            grid_focus: focus::tab_stop(cx),
            scroll: ScrollHandle::new(),
            snapshot: Snapshot::default(),
            loaded: false,
            focused: 0,
            running: None,
            updates,
            _updates_observer: observer,
        };
        view.refresh(cx);
        view
    }

    /// Re-reads everything off the UI thread; called on every visit.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let snapshot = cx.background_spawn(async { Snapshot::read() }).await;
            this.update(cx, |this, cx| {
                this.snapshot = snapshot;
                this.loaded = true;
                let count = this.items().len();
                if this.focused >= count {
                    this.focused = count.saturating_sub(1);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Focuses the tiles, the page's first control.
    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.grid_focus.focus(window, cx);
    }

    fn items(&self) -> Vec<Item> {
        let mut items: Vec<Item> = Tile::ALL.into_iter().map(Item::Tile).collect();
        items.extend((0..self.snapshot.themes.len()).map(Item::Theme));
        items.extend((0..self.snapshot.flows.len()).map(Item::Flow));
        items
    }

    fn section_of(item: &Item) -> usize {
        match item {
            Item::Tile(_) => 0,
            Item::Theme(_) => 1,
            Item::Flow(_) => 2,
        }
    }

    fn move_focus(&mut self, delta: isize, cx: &mut Context<Self>) {
        let count = self.items().len();
        if count == 0 {
            return;
        }
        let next = (self.focused as isize + delta).clamp(0, count as isize - 1);
        self.focused = next as usize;
        self.reveal_focused();
        cx.notify();
    }

    /// Scrolls so the focused item's section is on screen: the tiles at
    /// the top, the flows at the bottom; the themes sit between and are
    /// left where the user scrolled them.
    fn reveal_focused(&self) {
        let Some(item) = self.items().get(self.focused).cloned() else {
            return;
        };
        let mut offset = self.scroll.offset();
        match Self::section_of(&item) {
            0 => offset.y = px(0.),
            2 => offset.y = -self.scroll.max_offset().y,
            _ => return,
        }
        self.scroll.set_offset(offset);
    }

    fn move_section(&mut self, forward: bool, cx: &mut Context<Self>) {
        let items = self.items();
        let Some(current) = items.get(self.focused) else {
            return;
        };
        let section = Self::section_of(current);
        let target = if forward {
            items
                .iter()
                .position(|item| Self::section_of(item) > section)
        } else {
            items
                .iter()
                .position(|item| Self::section_of(item) == section)
                .and_then(|start| {
                    let before = items.get(start.checked_sub(1)?)?;
                    let prev = Self::section_of(before);
                    items.iter().position(|item| Self::section_of(item) == prev)
                })
        };
        if let Some(target) = target {
            self.focused = target;
            self.reveal_focused();
            cx.notify();
        }
    }

    fn open_focused(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.items().get(self.focused).cloned() {
            Some(Item::Tile(tile)) => emit(cx, AppEvent::Navigate(tile.page())),
            Some(Item::Theme(ix)) => self.open_theme(ix, cx),
            Some(Item::Flow(ix)) => self.run_flow(ix, window, cx),
            None => {}
        }
    }

    fn act_focused(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.items().get(self.focused).cloned() {
            Some(Item::Tile(tile)) => emit(cx, AppEvent::Navigate(tile.page())),
            Some(Item::Theme(ix)) => self.apply_theme(ix, window, cx),
            Some(Item::Flow(ix)) => {
                if let Some(flow) = self.snapshot.flows.get(ix) {
                    emit(
                        cx,
                        AppEvent::Navigate(ActivePage::FlowEdit(flow.id.clone())),
                    );
                }
            }
            None => {}
        }
    }

    fn open_theme(&self, ix: usize, cx: &mut Context<Self>) {
        if let Some(theme) = self.snapshot.themes.get(ix) {
            emit(
                cx,
                AppEvent::Navigate(ActivePage::ThemeEdit(theme.dir.clone())),
            );
        }
    }

    fn apply_theme(&self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(theme) = self.snapshot.themes.get(ix) {
            crate::ui::theme_apply::apply_theme(theme.dir.clone(), window, cx);
        }
    }

    fn run_flow(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(flow) = self.snapshot.flows.get(ix).cloned() else {
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
            let outcome = run_in_thread(flow.clone()).await;
            this.update_in(cx, |this, window, cx| {
                this.running = None;
                match outcome {
                    Ok(outcome) => window.push_notification(outcome.summary(&flow), cx),
                    Err(e) => window.push_notification(format!("Could not run the flow: {e}"), cx),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    // ── rendering ─────────────────────────────────────────────────────

    fn is_focused(&self, item: &Item, window: &Window) -> bool {
        self.grid_focus.is_focused(window) && self.items().get(self.focused) == Some(item)
    }

    /// The running theme's colors as one wide band: the background twice
    /// as wide, the six hues, then the foreground (black and white are
    /// left out: they read as the background and foreground again).
    fn render_ribbon(&self, cx: &Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let colors: Vec<(Hsla, f32)> =
            match self.snapshot.current.as_ref().and_then(|c| c.2.as_ref()) {
                Some(c) => {
                    let t = &c.terminal;
                    [
                        (&c.primary.background, 2.),
                        (&t.red, 1.),
                        (&t.green, 1.),
                        (&t.yellow, 1.),
                        (&t.blue, 1.),
                        (&t.magenta, 1.),
                        (&t.cyan, 1.),
                        (&c.primary.foreground, 1.),
                    ]
                    .into_iter()
                    .filter_map(|(hex, weight)| hex_to_hsla(hex).map(|c| (c, weight)))
                    .collect()
                }
                None => Vec::new(),
            };
        if colors.is_empty() {
            return div()
                .h(px(56.))
                .w_full()
                .rounded(theme.radius)
                .bg(theme.secondary)
                .into_any_element();
        }
        h_flex()
            .w_full()
            .h(px(56.))
            .rounded(theme.radius)
            .overflow_hidden()
            .border_1()
            .border_color(theme.border)
            .children(colors.into_iter().map(|(color, weight)| {
                div()
                    .flex_grow(weight)
                    .flex_basis(px(0.))
                    .h_full()
                    .bg(color)
            }))
            .into_any_element()
    }

    fn render_hero(&self, cx: &Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let updates = self.updates.read(cx);
        let version = updates
            .version()
            .map(|v| format!("Omarchy {v}"))
            .unwrap_or_else(|| "Omarchy".to_string());
        let (status, status_color) = match updates.state() {
            UpdateState::Checking => ("checking for updates…", theme.muted_foreground),
            UpdateState::UpToDate => ("up to date", theme.muted_foreground),
            UpdateState::Available(_) => ("update available", theme.warning),
            UpdateState::Failed(_) => ("update check failed", theme.muted_foreground),
            UpdateState::Updating => ("updating…", theme.warning),
        };
        let (current_title, mode) = match &self.snapshot.current {
            Some((_, title, colors)) => (
                title.clone(),
                colors
                    .as_ref()
                    .and_then(|c| hex_to_hsla(&c.primary.background))
                    .map(|bg| if bg.l > 0.5 { "light" } else { "dark" }),
            ),
            None => ("No theme applied".to_string(), None),
        };
        let editable = self
            .snapshot
            .current
            .as_ref()
            .and_then(|(dir, _, _)| self.snapshot.themes.iter().find(|t| &t.dir == dir))
            .map(|t| t.dir.clone());

        v_flex()
            .gap_4()
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(34.))
                            .line_height(relative(1.1))
                            .font_weight(FontWeight::BOLD)
                            .child(greeting(Local::now().hour())),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .flex_wrap()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(selectable("home-version", version))
                            .child(div().child("·"))
                            .child(
                                div()
                                    .id("home-update-status")
                                    .text_color(status_color)
                                    .cursor_pointer()
                                    .child(status)
                                    .on_click(|_, _, cx| {
                                        emit(cx, AppEvent::Navigate(ActivePage::Omarchy))
                                    }),
                            ),
                    ),
            )
            .child(self.render_ribbon(cx))
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .flex_wrap()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("RUNNING"),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(selectable("home-current-theme", current_title)),
                    )
                    .when_some(mode, |row, mode| {
                        row.child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded(theme.radius)
                                .border_1()
                                .border_color(theme.border)
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(mode),
                        )
                    })
                    .child(div().flex_1())
                    .when_some(editable, |row, dir| {
                        row.child(
                            Button::new("home-edit-current")
                                .label("Open in Designer")
                                .small()
                                .outline()
                                .cursor_pointer()
                                .on_click(move |_, _, cx| {
                                    emit(cx, AppEvent::Navigate(ActivePage::ThemeEdit(dir.clone())))
                                }),
                        )
                    })
                    .child(
                        Button::new("home-reapply")
                            .label("Re-apply")
                            .small()
                            .ghost()
                            .cursor_pointer()
                            .on_click(|_, window, cx| {
                                crate::ui::theme_apply::refresh_theme(window, cx)
                            }),
                    ),
            )
            .into_any_element()
    }

    fn render_tiles(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let columns = if window.viewport_size().width < px(960.) {
            2
        } else {
            4
        };
        let tiles: Vec<AnyElement> = Tile::ALL
            .into_iter()
            .map(|tile| {
                let (count, note): (usize, &str) = match tile {
                    Tile::Themes => (self.snapshot.themes.len(), "made here"),
                    Tile::Configuration => (self.snapshot.hyprland_changed, "settings changed"),
                    Tile::Keybinds => (self.snapshot.keybinds_changed, "keybinds changed"),
                    Tile::Flows => (self.snapshot.flows.len(), "flows"),
                };
                let focused = self.is_focused(&Item::Tile(tile), window);
                let ring = focus::focus_border(focused, theme.border, cx);
                let page = tile.page();
                v_flex()
                    .id(SharedString::from(format!("home-tile-{}", tile.title())))
                    .gap_2()
                    .p_4()
                    .rounded(theme.radius)
                    .border_1()
                    .border_color(ring)
                    .bg(theme.secondary.opacity(0.5))
                    .cursor_pointer()
                    .hover(|this| this.bg(theme.secondary))
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .text_color(theme.primary)
                            .child(Icon::new(Icon::empty()).path(tile.icon()).size_4())
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(tile.title()),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_baseline()
                            .child(
                                div()
                                    .text_size(px(28.))
                                    .line_height(relative(1.))
                                    .font_weight(FontWeight::BOLD)
                                    .child(selectable(
                                        ("home-tile-count", tile as usize),
                                        count.to_string(),
                                    )),
                            )
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(note),
                            ),
                    )
                    .on_click(
                        cx.listener(move |_, _, _, cx| emit(cx, AppEvent::Navigate(page.clone()))),
                    )
                    .into_any_element()
            })
            .collect();
        grid(columns, tiles).into_any_element()
    }

    fn section_header(
        &self,
        title: &'static str,
        page: ActivePage,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        h_flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.muted_foreground)
                    .child(title.to_uppercase()),
            )
            .child(
                Button::new(SharedString::from(format!("home-all-{title}")))
                    .label("See all")
                    .xsmall()
                    .ghost()
                    .cursor_pointer()
                    .on_click(
                        cx.listener(move |_, _, _, cx| emit(cx, AppEvent::Navigate(page.clone()))),
                    ),
            )
            .into_any_element()
    }

    fn render_themes(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let columns = card_columns(window);
        let cards: Vec<AnyElement> = self
            .snapshot
            .themes
            .iter()
            .enumerate()
            .map(|(ix, entry)| {
                let focused = self.is_focused(&Item::Theme(ix), window);
                let ring = focus::focus_border(focused, theme.border, cx);
                let swatches: Vec<Hsla> = entry
                    .colors
                    .as_ref()
                    .map(|c| {
                        [
                            &c.primary.background,
                            &c.terminal.red,
                            &c.terminal.green,
                            &c.terminal.yellow,
                            &c.terminal.blue,
                            &c.terminal.magenta,
                            &c.primary.foreground,
                        ]
                        .into_iter()
                        .filter_map(|hex| hex_to_hsla(hex))
                        .collect()
                    })
                    .unwrap_or_default();
                let dir = entry.dir.clone();
                let apply_dir = entry.dir.clone();
                v_flex()
                    .id(("home-theme", ix))
                    .gap_2()
                    .p_3()
                    .rounded(theme.radius)
                    .border_1()
                    .border_color(ring)
                    .bg(theme.secondary.opacity(0.5))
                    .cursor_pointer()
                    .hover(|this| this.bg(theme.secondary))
                    .child(
                        h_flex()
                            .h(px(10.))
                            .w_full()
                            .rounded(px(3.))
                            .overflow_hidden()
                            .when(swatches.is_empty(), |row| row.bg(theme.muted))
                            .children(
                                swatches
                                    .into_iter()
                                    .map(|color| div().flex_1().h_full().bg(color)),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_sm()
                                    .child(selectable(
                                        ("home-theme-name", ix),
                                        entry.title.clone(),
                                    )),
                            )
                            .child(if entry.applied {
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded(theme.radius)
                                    .bg(theme.primary.opacity(0.15))
                                    .text_color(theme.primary)
                                    .text_xs()
                                    .child("Applied")
                                    .into_any_element()
                            } else {
                                Button::new(("home-theme-apply", ix))
                                    .label("Apply")
                                    .xsmall()
                                    .ghost()
                                    .tab_stop(false)
                                    .cursor_pointer()
                                    .on_click(move |_, window, cx| {
                                        cx.stop_propagation();
                                        crate::ui::theme_apply::apply_theme(
                                            apply_dir.clone(),
                                            window,
                                            cx,
                                        );
                                    })
                                    .into_any_element()
                            }),
                    )
                    .on_click(move |_, _, cx| {
                        emit(cx, AppEvent::Navigate(ActivePage::ThemeEdit(dir.clone())))
                    })
                    .into_any_element()
            })
            .collect();

        v_flex()
            .gap_3()
            .child(self.section_header("Your themes", ActivePage::Themes, cx))
            .child(if cards.is_empty() {
                empty_row(
                    "home-no-themes",
                    "No themes made here yet",
                    "New theme",
                    ActivePage::Themes,
                    cx,
                )
            } else {
                grid(columns, cards).into_any_element()
            })
            .into_any_element()
    }

    fn render_flows(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let columns = card_columns(window);
        let cards: Vec<AnyElement> = self
            .snapshot
            .flows
            .iter()
            .enumerate()
            .map(|(ix, flow)| {
                let focused = self.is_focused(&Item::Flow(ix), window);
                let ring = focus::focus_border(focused, theme.border, cx);
                let running = self.running.as_deref() == Some(flow.id.as_str());
                let id = flow.id.clone();
                let steps = flow.enabled_steps();
                h_flex()
                    .id(("home-flow", ix))
                    .gap_3()
                    .p_3()
                    .items_center()
                    .rounded(theme.radius)
                    .border_1()
                    .border_color(ring)
                    .bg(theme.secondary.opacity(0.5))
                    .cursor_pointer()
                    .hover(|this| this.bg(theme.secondary))
                    .child(icon_tile(&flow.icon, px(32.), cx))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .truncate()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_sm()
                                    .child(selectable(("home-flow-name", ix), flow.name.clone())),
                            )
                            .child(div().text_xs().text_color(theme.muted_foreground).child(
                                format!("{steps} step{}", if steps == 1 { "" } else { "s" }),
                            )),
                    )
                    .child(
                        Button::new(("home-flow-run", ix))
                            .icon(Icon::new(Icon::empty()).path("icons/play.svg"))
                            .xsmall()
                            .ghost()
                            .tab_stop(false)
                            .loading(running)
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.run_flow(ix, window, cx);
                            })),
                    )
                    .on_click(move |_, _, cx| {
                        emit(cx, AppEvent::Navigate(ActivePage::FlowEdit(id.clone())))
                    })
                    .into_any_element()
            })
            .collect();

        v_flex()
            .gap_3()
            .child(self.section_header("Flows", ActivePage::Flows, cx))
            .child(if cards.is_empty() {
                empty_row(
                    "home-no-flows",
                    "No flows yet",
                    "New flow",
                    ActivePage::FlowNew(None),
                    cx,
                )
            } else {
                grid(columns, cards).into_any_element()
            })
            .into_any_element()
    }
}

/// Cards per row for the theme and flow sections.
fn card_columns(window: &Window) -> usize {
    let width = window.viewport_size().width;
    if width < px(640.) {
        1
    } else if width < px(1024.) {
        2
    } else if width < px(1400.) {
        3
    } else {
        4
    }
}

/// Rows of equal-width cells, the last row padded.
fn grid(columns: usize, cells: Vec<AnyElement>) -> Div {
    let columns = columns.max(1);
    let mut cells = cells;
    let remainder = cells.len() % columns;
    if remainder != 0 {
        for _ in remainder..columns {
            cells.push(div().into_any_element());
        }
    }
    let mut rows: Vec<Vec<AnyElement>> = Vec::new();
    for cell in cells {
        match rows.last_mut() {
            Some(row) if row.len() < columns => row.push(cell),
            _ => rows.push(vec![cell]),
        }
    }
    v_flex().gap_3().children(rows.into_iter().map(|row| {
        h_flex().gap_3().items_start().children(
            row.into_iter()
                .map(|cell| div().flex_1().min_w_0().child(cell)),
        )
    }))
}

/// An empty section: one line and the button that fills it.
fn empty_row(
    id: &'static str,
    text: &'static str,
    action: &'static str,
    page: ActivePage,
    cx: &mut Context<HomeView>,
) -> AnyElement {
    let theme = cx.theme();
    h_flex()
        .gap_3()
        .items_center()
        .p_3()
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.border)
        .text_sm()
        .text_color(theme.muted_foreground)
        .child(selectable(id, text))
        .child(
            Button::new(SharedString::from(format!("{id}-action")))
                .label(action)
                .xsmall()
                .outline()
                .cursor_pointer()
                .on_click(
                    cx.listener(move |_, _, _, cx| emit(cx, AppEvent::Navigate(page.clone()))),
                ),
        )
        .into_any_element()
}

impl Render for HomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("home-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .child(
                v_flex()
                    .id("home-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .pr_4()
                    .pb_8()
                    .gap_8()
                    .child(self.render_hero(cx))
                    .child(
                        v_flex()
                            .id("home-grid")
                            .test_support()
                            .key_context(GRID_CONTEXT)
                            .track_focus(&self.grid_focus)
                            .gap_8()
                            .on_action(cx.listener(|this, _: &Prev, _, cx| this.move_focus(-1, cx)))
                            .on_action(cx.listener(|this, _: &Next, _, cx| this.move_focus(1, cx)))
                            .on_action(cx.listener(|this, _: &PrevSection, _, cx| {
                                this.move_section(false, cx)
                            }))
                            .on_action(cx.listener(|this, _: &NextSection, _, cx| {
                                this.move_section(true, cx)
                            }))
                            .on_action(cx.listener(|this, _: &First, _, cx| {
                                this.focused = 0;
                                cx.notify();
                            }))
                            .on_action(cx.listener(|this, _: &Last, _, cx| {
                                this.focused = this.items().len().saturating_sub(1);
                                cx.notify();
                            }))
                            .on_action(cx.listener(|this, _: &Open, window, cx| {
                                this.open_focused(window, cx)
                            }))
                            .on_action(
                                cx.listener(|this, _: &Act, window, cx| {
                                    this.act_focused(window, cx)
                                }),
                            )
                            .child(self.render_tiles(window, cx))
                            .child(self.render_themes(window, cx))
                            .child(self.render_flows(window, cx)),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{count_leaves, greeting};

    #[test]
    fn the_greeting_follows_the_hour() {
        assert_eq!(greeting(7), "Good morning");
        assert_eq!(greeting(13), "Good afternoon");
        assert_eq!(greeting(20), "Good evening");
        assert_eq!(greeting(2), "Good night");
    }

    #[test]
    fn leaves_are_counted_through_nested_sections() {
        let value = serde_json::json!({
            "general": { "gaps_in": 4, "gaps_out": 8 },
            "input": { "touchpad": { "tap_to_click": true } },
            "empty": {}
        });
        assert_eq!(count_leaves(&value), 3);
    }
}
