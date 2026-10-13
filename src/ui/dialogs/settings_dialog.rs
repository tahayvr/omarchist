//! The app's own settings: how Omarchist looks, where it opens, when it
//! checks for Omarchy updates, and what it notifies about. A dialog that
//! opens over any page (Ctrl+,, the Omarchist menu, the palette, `--view
//! settings`), laid out like the Configuration page: a search box, a list
//! of pages, and the chosen page's groups of rows. Every value lives in
//! `~/.config/omarchist/settings.json` (`config_setup.rs`).
use std::rc::Rc;

use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, Sizable as _, WindowExt,
    button::Button,
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants},
    h_flex,
    input::{InputEvent, InputState},
    menu::{DropdownMenu, PopupMenuItem},
    scroll::ScrollableElement as _,
    sidebar::{SidebarItem, SidebarMenuItem},
    v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::bar_widget;
use crate::system::config::config_setup::{SettingsConfig, settings, update_settings};
use crate::system::flows::service;
use crate::system::logging;
use crate::system::ui_theme_watcher;
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::explain::{explain, explained_label};
use crate::ui::focus::{self, FocusSection, FocusableSwitch};
use crate::ui::heading;
use crate::ui::notify;
use crate::ui::palette::{self, Area};
use crate::ui::text::folds;
use crate::ui::toolbar;

pub const NAV_CONTEXT: &str = "SettingsNav";

pub mod settings_nav {
    gpui::actions!(settings_nav, [Prev, Next, First, Last, Activate]);
}

const FONT_SIZES: &[(&str, &str)] = &[("small", "Small"), ("medium", "Medium"), ("large", "Large")];
const THEME_MODES: &[(&str, &str)] =
    &[("omarchy", "Omarchy"), ("light", "Light"), ("dark", "Dark")];
const STARTUP_PAGES: &[(&str, &str)] = &[
    ("dashboard", "Dashboard"),
    ("themes", "Themes"),
    ("config", "Configuration"),
    ("keybinds", "Keybinds"),
    ("flows", "Flows"),
    ("last", "Last page used"),
];
const LOG_KEEP_DAYS: &[(u32, &str)] = &[(3, "3 days"), (7, "7 days"), (30, "30 days")];
const CHECK_INTERVALS: &[(u32, &str)] = &[
    (1, "Every hour"),
    (3, "Every 3 hours"),
    (6, "Every 6 hours"),
    (12, "Every 12 hours"),
    (24, "Once a day"),
];

/// One setting: a label on the left, its control on the right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    FontSize,
    Look,
    StartupPage,
    AutoApplyTheme,
    NotifyFlows,
    Automations,
    CountInstalls,
    CheckUpdates,
    UpdateInterval,
    NotifyUpdates,
    BarWidget,
    LogKeepDays,
    LogFolder,
}

impl Row {
    fn id(self) -> &'static str {
        match self {
            Row::FontSize => "font-size",
            Row::Look => "theme-mode",
            Row::StartupPage => "startup-page",
            Row::AutoApplyTheme => "auto-apply-theme",
            Row::NotifyFlows => "notify-flows",
            Row::Automations => "automations",
            Row::CountInstalls => "catalog-count-installs",
            Row::CheckUpdates => "check-updates",
            Row::UpdateInterval => "update-interval",
            Row::NotifyUpdates => "notify-updates",
            Row::BarWidget => "bar-widget",
            Row::LogKeepDays => "log-keep-days",
            Row::LogFolder => "log-folder",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Row::FontSize => "Font Size",
            Row::Look => "Look",
            Row::StartupPage => "Open On",
            Row::AutoApplyTheme => "Auto-apply Theme on Edit",
            Row::NotifyFlows => "Notify When a Flow Finishes",
            Row::Automations => "Run Automations in the Background",
            Row::CountInstalls => "Count My Installs in the Catalog",
            Row::CheckUpdates => "Check in the Background",
            Row::UpdateInterval => "Check Interval",
            Row::NotifyUpdates => "Notify When an Update Is Found",
            Row::BarWidget => "Show Omarchist in the Bar",
            Row::LogKeepDays => "Keep Logs For",
            Row::LogFolder => "Log Files",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Row::FontSize => "Text size across the app",
            Row::Look => "Follow the desktop theme's light or dark mode, or force one",
            Row::StartupPage => "The page shown when Omarchist starts without a --view",
            Row::AutoApplyTheme => "Apply a theme to the desktop when its editor opens",
            Row::NotifyFlows => {
                "A desktop notification after a flow run from a keybind or the command line"
            }
            Row::Automations => {
                "A service that starts flows on a schedule or when something happens"
            }
            Row::CountInstalls => {
                "Saving a flow from the catalog sends the flow's name, and nothing about you"
            }
            Row::CheckUpdates => "Ask Omarchy for updates while the app is open",
            Row::UpdateInterval => "How often the background check runs",
            Row::NotifyUpdates => "A desktop notification when a background check finds an update",
            Row::BarWidget => "A bar widget that runs your flows and opens Omarchist on a page",
            Row::LogKeepDays => "Older logs are deleted; the folder never grows past 50 MB",
            Row::LogFolder => "One file a day in ~/.local/state/omarchist/logs",
        }
    }
}

struct Group {
    title: &'static str,
    rows: &'static [Row],
}

/// One entry of the list on the left.
struct Page {
    title: &'static str,
    icon: &'static str,
    area: Area,
    groups: &'static [Group],
}

const PAGES: &[Page] = &[
    Page {
        title: "General",
        icon: "icons/settings-2.svg",
        area: Area::Settings,
        groups: &[
            Group {
                title: "Appearance",
                rows: &[Row::FontSize, Row::Look],
            },
            Group {
                title: "Startup",
                rows: &[Row::StartupPage],
            },
            Group {
                title: "Logs",
                rows: &[Row::LogKeepDays, Row::LogFolder],
            },
        ],
    },
    Page {
        title: "Themes",
        icon: "icons/palette.svg",
        area: Area::Themes,
        groups: &[Group {
            title: "Theme designer",
            rows: &[Row::AutoApplyTheme],
        }],
    },
    Page {
        title: "Flows",
        icon: "icons/workflow.svg",
        area: Area::Flows,
        groups: &[
            Group {
                title: "Runs",
                rows: &[Row::NotifyFlows, Row::Automations],
            },
            Group {
                title: "Catalog",
                rows: &[Row::CountInstalls],
            },
        ],
    },
    Page {
        title: "Omarchy",
        icon: "icons/cloud-download.svg",
        area: Area::Omarchy,
        groups: &[
            Group {
                title: "Updates",
                rows: &[Row::CheckUpdates, Row::UpdateInterval, Row::NotifyUpdates],
            },
            Group {
                title: "Bar",
                rows: &[Row::BarWidget],
            },
        ],
    },
];

/// Opens the log folder in the file manager, making it first if no log
/// has been written yet.
fn open_log_folder(window: &mut Window, cx: &mut App) {
    match logging::log_dir().and_then(|dir| {
        std::fs::create_dir_all(&dir)
            .map_err(|e| crate::error::Error::io("Failed to create the log folder", e))?;
        Ok(dir)
    }) {
        Ok(dir) => cx.open_url(&format!("file://{}", dir.display())),
        Err(e) => notify::error(window, format!("Could not open the log folder: {e}"), cx),
    }
}

fn row_matches(row: Row, query: &str) -> bool {
    query.is_empty()
        || folds(row.label()).contains(query)
        || folds(row.description()).contains(query)
}

/// Opens the Settings dialog over whatever page is shown, on its list of
/// pages.
pub fn open_settings_dialog(window: &mut Window, cx: &mut App) {
    let panel = cx.new(|cx| SettingsPanel::new(window, cx));
    let trap_focus = cx.focus_handle();
    let nav = panel.read(cx).nav_focus.clone();
    window.open_dialog(cx, move |dialog, window, _| {
        dialog
            .title("Settings")
            .w(focus::dialog_width(900., window))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(true)
            .child(
                focus::dialog_body("settings-dialog", &trap_focus, |window, cx| {
                    window.close_dialog(cx);
                })
                .child(panel.clone()),
            )
    });
    // The list exists from the dialog's first frame, as the shortcuts
    // dialog's does.
    nav.focus(window, cx);
    window.on_next_frame(move |window, cx| nav.focus(window, cx));
}

pub struct SettingsPanel {
    settings: SettingsConfig,
    /// The bar switch shows what `shell.json` says, not the saved setting:
    /// the plugin can be disabled from the bar's own layout editor.
    bar_widget_on: bool,
    /// `omarchy plugin enable/disable` is running in the background.
    bar_widget_pending: bool,
    /// Whether the service that runs automations is installed; systemd's
    /// answer, read in the background when the dialog opens.
    automations_on: bool,
    automations_pending: bool,
    search: Entity<InputState>,
    page: usize,
    /// The list of pages is one tab stop; Up and Down move between pages.
    nav_focus: FocusHandle,
    /// Not a tab stop: Enter on the list focuses the first control in it.
    content_focus: FocusHandle,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl SettingsPanel {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let settings = settings();
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search settings"));
        let subscriptions = vec![cx.subscribe(&search, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.scroll.set_offset(Point::default());
                cx.notify();
            }
        })];
        // systemd and `shell.json` answer off the UI thread, so the dialog
        // opens at once; until then the switches show the saved setting.
        cx.spawn(async move |this, cx| {
            let (bar, automations) = cx
                .background_spawn(async { (bar_widget::is_enabled(), service::is_enabled()) })
                .await;
            this.update(cx, |this, cx| {
                if !this.bar_widget_pending {
                    this.bar_widget_on = bar;
                }
                if !this.automations_pending {
                    this.automations_on = automations;
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        Self {
            bar_widget_on: settings.bar_widget,
            settings,
            bar_widget_pending: false,
            automations_on: false,
            automations_pending: false,
            search,
            page: 0,
            nav_focus: focus::tab_stop(cx),
            content_focus: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    fn query(&self, cx: &App) -> String {
        folds(self.search.read(cx).value().trim())
    }

    fn set_page(&mut self, page: usize, window: &mut Window, cx: &mut Context<Self>) {
        let page = page.min(PAGES.len() - 1);
        // Picking a page leaves a search for that page.
        if !self.search.read(cx).value().is_empty() {
            self.search
                .update(cx, |input, cx| input.set_value("", window, cx));
        }
        if self.page != page {
            self.page = page;
            self.scroll.set_offset(Point::default());
        }
        cx.notify();
    }

    /// Applies a change to the settings in memory and on disk.
    fn change(&mut self, change: impl Fn(&mut SettingsConfig), cx: &mut Context<Self>) {
        change(&mut self.settings);
        if let Err(e) = update_settings(change) {
            // The control already shows the new value; say that it will
            // not survive a restart.
            emit(
                cx,
                AppEvent::Error(format!("Settings could not be saved: {e}")),
            );
        }
        cx.notify();
    }

    fn set_font_size(&mut self, size: &'static str, cx: &mut Context<Self>) {
        self.change(move |s| s.font_size = size.to_string(), cx);
        ui_theme_watcher::apply_font_size(cx);
        cx.refresh_windows();
    }

    fn set_theme_mode(&mut self, mode: &'static str, cx: &mut Context<Self>) {
        self.change(move |s| s.theme_mode = mode.to_string(), cx);
        ui_theme_watcher::apply_ui_theme(cx);
        cx.refresh_windows();
    }

    /// Installs and starts the automations service, or stops and removes
    /// it, in the background.
    fn set_automations(&mut self, on: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.automations_pending {
            return;
        }
        self.automations_pending = true;
        self.automations_on = on;
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
                this.automations_pending = false;
                this.automations_on = now_on;
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

    /// Installs and enables the bar widget, or takes it off the bar, in the
    /// background (the shell's plugin commands take a moment), and keeps the
    /// setting in step with what the shell ended up with.
    fn set_bar_widget(&mut self, on: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.bar_widget_pending {
            return;
        }
        self.bar_widget_pending = true;
        self.bar_widget_on = on;
        cx.notify();
        let task = cx.background_spawn(async move {
            if on {
                bar_widget::enable()
            } else {
                bar_widget::disable()
            }
        });
        cx.spawn_in(window, async move |this, cx| {
            let result = task.await;
            this.update_in(cx, |this, window, cx| {
                this.bar_widget_pending = false;
                this.bar_widget_on = bar_widget::is_enabled();
                let on = this.bar_widget_on;
                this.change(move |s| s.bar_widget = on, cx);
                let message = match result {
                    Ok(()) if on => {
                        "Omarchist is on the bar. Move it with the bar's Edit Layout.".to_string()
                    }
                    Ok(()) => "Omarchist was removed from the bar.".to_string(),
                    Err(e) => {
                        notify::error(window, e.to_string(), cx);
                        return;
                    }
                };
                notify::success(window, message, cx);
            })
            .ok();
        })
        .detach();
    }

    /// A dropdown over a list of values; `pick` runs on the panel with the
    /// chosen value.
    fn dropdown<V: Copy + PartialEq + 'static>(
        &self,
        id: &'static str,
        current: V,
        options: &'static [(V, &'static str)],
        pick: impl Fn(&mut Self, V, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let label = options
            .iter()
            .find(|(value, _)| *value == current)
            .map(|(_, label)| *label)
            .unwrap_or("");
        let view = cx.entity();
        let pick = Rc::new(pick);
        Button::new(id)
            .label(label)
            .dropdown_caret(true)
            .outline()
            .small()
            .cursor_pointer()
            .dropdown_menu(move |menu, _, _| {
                options.iter().fold(menu, |menu, (value, label)| {
                    let view = view.clone();
                    let pick = pick.clone();
                    let value = *value;
                    menu.item(
                        PopupMenuItem::new(*label)
                            .checked(value == current)
                            .on_click(move |_, _, cx| {
                                view.update(cx, |this, cx| pick(this, value, cx));
                            }),
                    )
                })
            })
            .into_any_element()
    }

    fn switch(
        &self,
        id: &'static str,
        checked: bool,
        change: impl Fn(&mut SettingsConfig, bool) + Clone + 'static,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        FocusableSwitch::new(id)
            .checked(checked)
            .on_change(cx.listener(move |this, value, _, cx| {
                let value = *value;
                let change = change.clone();
                this.change(move |s| change(s, value), cx);
            }))
            .into_any_element()
    }

    fn control(&self, row: Row, cx: &mut Context<Self>) -> AnyElement {
        let s = &self.settings;
        let pick_of = |options: &'static [(&'static str, &'static str)], value: &str, or| {
            options
                .iter()
                .map(|(v, _)| *v)
                .find(|v| *v == value)
                .unwrap_or(or)
        };
        match row {
            Row::FontSize => self.dropdown(
                "font-size-pick",
                pick_of(FONT_SIZES, &s.font_size, "small"),
                FONT_SIZES,
                |this, value, cx| this.set_font_size(value, cx),
                cx,
            ),
            Row::Look => self.dropdown(
                "theme-mode-pick",
                pick_of(THEME_MODES, &s.theme_mode, "omarchy"),
                THEME_MODES,
                |this, value, cx| this.set_theme_mode(value, cx),
                cx,
            ),
            Row::StartupPage => self.dropdown(
                "startup-page-pick",
                pick_of(STARTUP_PAGES, &s.startup_page, "dashboard"),
                STARTUP_PAGES,
                |this, value, cx| this.change(move |s| s.startup_page = value.to_string(), cx),
                cx,
            ),
            Row::LogKeepDays => self.dropdown(
                "log-keep-days-pick",
                LOG_KEEP_DAYS
                    .iter()
                    .map(|(v, _)| *v)
                    .find(|v| *v == s.log_keep_days)
                    .unwrap_or(logging::DEFAULT_KEEP_DAYS),
                LOG_KEEP_DAYS,
                |this, value, cx| {
                    this.change(move |s| s.log_keep_days = value, cx);
                    cx.background_spawn(async move { logging::purge_now(value) })
                        .detach();
                },
                cx,
            ),
            Row::LogFolder => Button::new("log-folder-open")
                .icon(Icon::new(Icon::empty()).path("icons/folder-open.svg"))
                .label("Open folder")
                .outline()
                .small()
                .cursor_pointer()
                .on_click(cx.listener(|_, _, window, cx| open_log_folder(window, cx)))
                .into_any_element(),
            Row::UpdateInterval => self.dropdown(
                "update-interval-pick",
                CHECK_INTERVALS
                    .iter()
                    .map(|(v, _)| *v)
                    .find(|v| *v == s.update_check_hours)
                    .unwrap_or(6),
                CHECK_INTERVALS,
                |this, value, cx| this.change(move |s| s.update_check_hours = value, cx),
                cx,
            ),
            Row::AutoApplyTheme => self.switch(
                "auto-apply-theme-switch",
                s.auto_apply_theme,
                |s, v| s.auto_apply_theme = v,
                cx,
            ),
            Row::NotifyFlows => self.switch(
                "notify-flows-switch",
                s.notify_flows,
                |s, v| s.notify_flows = v,
                cx,
            ),
            Row::CountInstalls => self.switch(
                "catalog-count-installs-switch",
                s.catalog_count_installs,
                |s, v| s.catalog_count_installs = v,
                cx,
            ),
            Row::CheckUpdates => self.switch(
                "check-updates-switch",
                s.check_updates,
                |s, v| s.check_updates = v,
                cx,
            ),
            Row::NotifyUpdates => self.switch(
                "notify-updates-switch",
                s.notify_updates,
                |s, v| s.notify_updates = v,
                cx,
            ),
            Row::Automations => FocusableSwitch::new("automations-switch")
                .checked(self.automations_on)
                .disabled(self.automations_pending)
                .on_change(cx.listener(|this, value, window, cx| {
                    this.set_automations(*value, window, cx);
                }))
                .into_any_element(),
            Row::BarWidget => FocusableSwitch::new("bar-widget-switch")
                .checked(self.bar_widget_on)
                .disabled(self.bar_widget_pending)
                .on_change(cx.listener(|this, value, window, cx| {
                    this.set_bar_widget(*value, window, cx);
                }))
                .into_any_element(),
        }
    }

    fn render_row(&self, row: Row, cx: &mut Context<Self>) -> AnyElement {
        let control = self.control(row, cx);
        h_flex()
            .id(row.id())
            .w_full()
            .gap_4()
            .items_center()
            .justify_between()
            .py_2()
            .child(div().flex_1().min_w_0().child(explain(
                SharedString::from(format!("{}-desc", row.id())),
                explained_label(row.label(), cx),
                row.description(),
            )))
            .child(div().flex_none().child(control))
            .test_support()
            .into_any_element()
    }

    /// A page's title and the groups with a row that matches `query`;
    /// `None` when nothing on it matches.
    fn render_page(&self, ix: usize, query: &str, cx: &mut Context<Self>) -> Option<AnyElement> {
        let page = &PAGES[ix];
        let groups: Vec<AnyElement> = page
            .groups
            .iter()
            .filter_map(|group| {
                let rows: Vec<AnyElement> = group
                    .rows
                    .iter()
                    .filter(|row| row_matches(**row, query))
                    .map(|row| self.render_row(*row, cx))
                    .collect();
                (!rows.is_empty()).then(|| {
                    FocusSection::new(
                        ElementId::Name(format!("settings-group-{ix}-{}", group.title).into()),
                        &self.scroll,
                    )
                    .child(
                        GroupBox::new()
                            .with_variant(GroupBoxVariant::Outline)
                            .title(heading::section(group.title, cx))
                            .children(rows),
                    )
                    .into_any_element()
                })
            })
            .collect();
        if groups.is_empty() {
            return None;
        }
        Some(
            v_flex()
                .gap_4()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(palette::tile(
                            palette::icon(page.icon),
                            page.area.accent(cx),
                            px(28.),
                            cx,
                        ))
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(page.title),
                        ),
                )
                .children(groups)
                .into_any_element(),
        )
    }

    fn render_nav(&self, query: &str, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let nav_focused = self.nav_focus.is_focused(window);
        let radius = cx.theme().radius;
        let transparent = cx.theme().transparent;
        let muted = cx.theme().muted_foreground;
        let searching = !query.is_empty();
        let items = PAGES
            .iter()
            .enumerate()
            .map(|(ix, page)| {
                let hits = page
                    .groups
                    .iter()
                    .flat_map(|g| g.rows)
                    .filter(|row| row_matches(**row, query))
                    .count();
                let current = !searching && self.page == ix;
                let icon = palette::icon(page.icon).text_color(page.area.accent(cx));
                let mut item = SidebarMenuItem::new(page.title)
                    .icon(icon)
                    .active(current)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.nav_focus.focus(window, cx);
                        this.set_page(ix, window, cx);
                    }));
                if searching {
                    item = item.suffix(move |_, _| {
                        div().text_xs().text_color(muted).child(hits.to_string())
                    });
                }
                div()
                    .id(("settings-page", ix))
                    .rounded(radius)
                    .border_1()
                    .border_color(focus::focus_border(
                        nav_focused && self.page == ix,
                        transparent,
                        cx,
                    ))
                    .when(searching && hits == 0, |this| this.opacity(0.5))
                    .child(item.render(("settings-page-item", ix), window, cx))
            })
            .collect::<Vec<_>>();
        v_flex()
            .id("settings-nav")
            .test_support()
            .key_context(NAV_CONTEXT)
            .track_focus(&self.nav_focus)
            .on_action(cx.listener(|this, _: &settings_nav::Prev, window, cx| {
                this.set_page(this.page.saturating_sub(1), window, cx);
            }))
            .on_action(cx.listener(|this, _: &settings_nav::Next, window, cx| {
                this.set_page(this.page + 1, window, cx);
            }))
            .on_action(cx.listener(|this, _: &settings_nav::First, window, cx| {
                this.set_page(0, window, cx);
            }))
            .on_action(cx.listener(|this, _: &settings_nav::Last, window, cx| {
                this.set_page(usize::MAX, window, cx);
            }))
            .on_action(cx.listener(|this, _: &settings_nav::Activate, window, cx| {
                focus::focus_first_in(&this.content_focus, window, cx);
            }))
            .w(px(200.))
            .flex_none()
            .gap_1()
            .pr_4()
            .cursor_pointer()
            .children(items)
            .into_any_element()
    }
}

impl Render for SettingsPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.query(cx);
        let pages: Vec<AnyElement> = if query.is_empty() {
            self.render_page(self.page, "", cx).into_iter().collect()
        } else {
            (0..PAGES.len())
                .filter_map(|ix| self.render_page(ix, &query, cx))
                .collect()
        };
        let empty = pages.is_empty();
        let muted = cx.theme().muted_foreground;
        let nav = self.render_nav(&query, window, cx);
        v_flex()
            .id("settings-panel")
            .h(focus::dialog_height(540., window))
            .gap_4()
            .child(toolbar::search(
                toolbar::search_input(&self.search).id("settings-search"),
            ))
            .child(
                // Not `h_flex`: its `items_center` would stop the content
                // from filling the row's height, which it needs to scroll.
                div().flex().flex_row().flex_1().min_h_0().child(nav).child(
                    div()
                        .id("settings-content")
                        .flex_1()
                        .min_w_0()
                        .overflow_y_scroll()
                        .track_scroll(&self.scroll)
                        .child(
                            focus::scroll_area(&self.scroll)
                                .track_focus(&self.content_focus)
                                .child(
                                    v_flex()
                                        .w_full()
                                        .gap_6()
                                        .pb_4()
                                        .pr_3()
                                        .when(empty, |this| {
                                            this.child(
                                                div()
                                                    .text_sm()
                                                    .text_color(muted)
                                                    .child("No settings match your search."),
                                            )
                                        })
                                        .children(pages),
                                ),
                        )
                        .vertical_scrollbar(&self.scroll),
                ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{PAGES, Row, row_matches};
    use crate::ui::text::folds;

    #[test]
    fn every_setting_is_on_one_page() {
        let all = [
            Row::FontSize,
            Row::Look,
            Row::StartupPage,
            Row::AutoApplyTheme,
            Row::NotifyFlows,
            Row::Automations,
            Row::CountInstalls,
            Row::CheckUpdates,
            Row::UpdateInterval,
            Row::NotifyUpdates,
            Row::BarWidget,
        ];
        for row in all {
            let places = PAGES
                .iter()
                .flat_map(|p| p.groups)
                .flat_map(|g| g.rows)
                .filter(|r| **r == row)
                .count();
            assert_eq!(places, 1, "{row:?}");
        }
    }

    #[test]
    fn search_reads_labels_and_descriptions_loosely() {
        assert!(row_matches(Row::AutoApplyTheme, &folds("autoapply")));
        assert!(row_matches(Row::BarWidget, &folds("widget")));
        assert!(!row_matches(Row::FontSize, &folds("widget")));
        assert!(row_matches(Row::FontSize, ""));
    }
}
