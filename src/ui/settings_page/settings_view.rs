//! The app's own settings: how Omarchist looks, where it opens, when it
//! checks for Omarchy updates, and what it notifies about. Every value
//! lives in `~/.config/omarchist/settings.json` (`config_setup.rs`).
use std::rc::Rc;

use crate::ui::app_events::{AppEvent, emit};
use crate::ui::notify;
use gpui::*;
use gpui_component::{
    Icon, IconName, Sizable as _,
    button::Button,
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants},
    h_flex,
    menu::{DropdownMenu, PopupMenuItem},
    scroll::ScrollableElement as _,
    v_flex,
};

use crate::system::bar_widget;
use crate::system::config::config_setup::{SettingsConfig, settings, update_settings};
use crate::system::flows::service;
use crate::system::ui_theme_watcher;
use crate::ui::explain::{explain, explained_label};
use crate::ui::focus::{FocusSection, FocusableSwitch};
use crate::ui::palette::{self, Area};
const KEY_CONTEXT: &str = "SettingsPage";

const FONT_SIZES: &[(&str, &str)] = &[("small", "Small"), ("medium", "Medium"), ("large", "Large")];
const THEME_MODES: &[(&str, &str)] = &[
    ("omarchy", "Omarchy"),
    ("light", "Light"),
    ("dark", "Dark"),
];
const STARTUP_PAGES: &[(&str, &str)] = &[
    ("themes", "Themes"),
    ("config", "Configuration"),
    ("keybinds", "Keybinds"),
    ("flows", "Flows"),
    ("omarchy", "Omarchy"),
    ("settings", "Settings"),
    ("last", "Last page used"),
];
const CHECK_INTERVALS: &[(u32, &str)] = &[
    (1, "Every hour"),
    (3, "Every 3 hours"),
    (6, "Every 6 hours"),
    (12, "Every 12 hours"),
    (24, "Once a day"),
];

pub struct SettingsView {
    settings: SettingsConfig,
    /// The bar switch shows what `shell.json` says, not the saved setting:
    /// the plugin can be disabled from the bar's own layout editor.
    bar_widget_on: bool,
    /// `omarchy plugin enable/disable` is running in the background.
    bar_widget_pending: bool,
    /// Whether the service that runs automations is installed; systemd's
    /// answer, not a saved setting.
    automations_on: bool,
    automations_pending: bool,
    /// A failed write of settings.json, shown on the next render (which
    /// has the window).
    pub focus_handle: FocusHandle,
    scroll: ScrollHandle,
}

impl SettingsView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            settings: settings(),
            bar_widget_on: bar_widget::is_enabled(),
            bar_widget_pending: false,
            automations_on: service::is_enabled(),
            automations_pending: false,
            focus_handle: cx.focus_handle(),
            scroll: ScrollHandle::new(),
        }
    }

    /// Focuses the first control on the page.
    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        crate::ui::focus::focus_first_in(&self.focus_handle, window, cx);
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

    /// Re-reads the file: the gear menu and the shortcuts change these
    /// settings too.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.settings = settings();
        if !self.bar_widget_pending {
            self.bar_widget_on = bar_widget::is_enabled();
        }
        if !self.automations_pending {
            self.automations_on = service::is_enabled();
        }
        cx.notify();
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

    fn set_theme_mode(&mut self, mode: &'static str, cx: &mut Context<Self>) {
        self.change(move |s| s.theme_mode = mode.to_string(), cx);
        ui_theme_watcher::apply_ui_theme(cx);
        cx.refresh_windows();
    }

    fn render_row(
        &self,
        id: &'static str,
        label: &'static str,
        description: &'static str,
        control: AnyElement,
        cx: &App,
    ) -> impl IntoElement {
        h_flex()
            .id(id)
            .w_full()
            .gap_4()
            .items_center()
            .justify_between()
            .py_2()
            .child(div().flex_1().min_w_0().child(explain(
                SharedString::from(format!("{id}-desc")),
                explained_label(label, cx),
                description,
            )))
            .child(div().flex_none().child(control))
    }

    fn render_switch(
        &self,
        id: &'static str,
        label: &'static str,
        description: &'static str,
        checked: bool,
        change: impl Fn(&mut SettingsConfig, bool) + Clone + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let control = FocusableSwitch::new(id)
            .checked(checked)
            .on_change(cx.listener(move |this, value, _, cx| {
                let value = *value;
                let change = change.clone();
                this.change(move |s| change(s, value), cx);
            }))
            .into_any_element();
        self.render_row(id, label, description, control, cx)
    }

    /// A dropdown over string values; `pick` runs on the view with the
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

    /// A group of rows under a title, whose icon wears the colour of the
    /// area the rows concern.
    fn section(
        &self,
        id: &'static str,
        title: &'static str,
        icon: &'static str,
        area: Area,
        rows: Vec<AnyElement>,
        cx: &App,
    ) -> AnyElement {
        FocusSection::new(id, &self.scroll)
            .child(
                GroupBox::new()
                    .with_variant(GroupBoxVariant::Outline)
                    .title(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(palette::tile(
                                palette::icon(icon),
                                area.accent(cx),
                                px(22.),
                                cx,
                            ))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(title),
                            ),
                    )
                    .children(rows),
            )
            .into_any_element()
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let s = self.settings.clone();
        let font_size: &'static str = FONT_SIZES
            .iter()
            .map(|(v, _)| *v)
            .find(|v| *v == s.font_size)
            .unwrap_or("small");
        let theme_mode: &'static str = THEME_MODES
            .iter()
            .map(|(v, _)| *v)
            .find(|v| *v == s.theme_mode)
            .unwrap_or("omarchy");
        let startup_page: &'static str = STARTUP_PAGES
            .iter()
            .map(|(v, _)| *v)
            .find(|v| *v == s.startup_page)
            .unwrap_or("themes");
        let interval = CHECK_INTERVALS
            .iter()
            .map(|(v, _)| *v)
            .find(|v| *v == s.update_check_hours)
            .unwrap_or(6);

        let appearance = self.section(
            "settings-appearance",
            "Appearance",
            "icons/palette.svg",
            Area::Settings,
            vec![
                self.render_row(
                    "font-size",
                    "Font Size",
                    "Text size across the app",
                    self.dropdown(
                        "font-size-pick",
                        font_size,
                        FONT_SIZES,
                        |this, value, cx| this.set_font_size(value, cx),
                        cx,
                    ),
                    cx,
                )
                .into_any_element(),
                self.render_row(
                    "theme-mode",
                    "Look",
                    "Follow the desktop theme's light or dark mode, or force one",
                    self.dropdown(
                        "theme-mode-pick",
                        theme_mode,
                        THEME_MODES,
                        |this, value, cx| this.set_theme_mode(value, cx),
                        cx,
                    ),
                    cx,
                )
                .into_any_element(),
            ],
            cx,
        );

        let startup = self.section(
            "settings-startup",
            "Startup",
            "icons/rocket.svg",
            Area::Settings,
            vec![
                self.render_row(
                    "startup-page",
                    "Open On",
                    "The page shown when Omarchist starts without a --view",
                    self.dropdown(
                        "startup-page-pick",
                        startup_page,
                        STARTUP_PAGES,
                        |this, value, cx| {
                            this.change(move |s| s.startup_page = value.to_string(), cx)
                        },
                        cx,
                    ),
                    cx,
                )
                .into_any_element(),
            ],
            cx,
        );

        let updates = self.section(
            "settings-updates",
            "Omarchy Updates",
            "icons/cloud-download.svg",
            Area::Omarchy,
            vec![
                self.render_switch(
                    "check-updates",
                    "Check in the Background",
                    "Ask Omarchy for updates while the app is open",
                    s.check_updates,
                    |s, v| s.check_updates = v,
                    cx,
                )
                .into_any_element(),
                self.render_row(
                    "update-interval",
                    "Check Interval",
                    "How often the background check runs",
                    self.dropdown(
                        "update-interval-pick",
                        interval,
                        CHECK_INTERVALS,
                        |this, value, cx| this.change(move |s| s.update_check_hours = value, cx),
                        cx,
                    ),
                    cx,
                )
                .into_any_element(),
                self.render_switch(
                    "notify-updates",
                    "Notify When an Update Is Found",
                    "A desktop notification when a background check finds an update",
                    s.notify_updates,
                    |s, v| s.notify_updates = v,
                    cx,
                )
                .into_any_element(),
            ],
            cx,
        );

        let designer = self.section(
            "settings-designer",
            "Theme Designer",
            "icons/pen-tool.svg",
            Area::Themes,
            vec![
                self.render_switch(
                    "auto-apply-theme",
                    "Auto-apply Theme on Edit",
                    "Apply a theme to the desktop when its editor opens",
                    s.auto_apply_theme,
                    |s, v| s.auto_apply_theme = v,
                    cx,
                )
                .into_any_element(),
            ],
            cx,
        );

        let bar = self.section(
            "settings-bar",
            "Bar",
            "icons/panel-top.svg",
            Area::Omarchy,
            vec![
                self.render_row(
                    "bar-widget",
                    "Show Omarchist in the Bar",
                    "A bar widget that runs your flows and opens Omarchist on a page",
                    FocusableSwitch::new("bar-widget-switch")
                        .checked(self.bar_widget_on)
                        .disabled(self.bar_widget_pending)
                        .on_change(cx.listener(|this, value, window, cx| {
                            this.set_bar_widget(*value, window, cx);
                        }))
                        .into_any_element(),
                    cx,
                )
                .into_any_element(),
            ],
            cx,
        );

        let flows = self.section(
            "settings-flows",
            "Flows",
            "icons/workflow.svg",
            Area::Flows,
            vec![
                self.render_switch(
                    "notify-flows",
                    "Notify When a Flow Finishes",
                    "A desktop notification after a flow run from a keybind or the command line",
                    s.notify_flows,
                    |s, v| s.notify_flows = v,
                    cx,
                )
                .into_any_element(),
                self.render_switch(
                    "gallery-count-installs",
                    "Count My Installs in the Gallery",
                    "Saving a flow from the gallery sends the flow's name, and nothing about you",
                    s.gallery_count_installs,
                    |s, v| s.gallery_count_installs = v,
                    cx,
                )
                .into_any_element(),
                self.render_row(
                    "automations",
                    "Run Automations in the Background",
                    "A service that starts flows on a schedule or when something happens",
                    FocusableSwitch::new("automations-switch")
                        .checked(self.automations_on)
                        .disabled(self.automations_pending)
                        .on_change(cx.listener(|this, value, window, cx| {
                            this.set_automations(*value, window, cx);
                        }))
                        .into_any_element(),
                    cx,
                )
                .into_any_element(),
            ],
            cx,
        );

        v_flex()
            .id("settings-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .child(
                div()
                    .id("settings-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .child(
                        crate::ui::focus::scroll_area(&self.scroll).child(
                            v_flex()
                                .w_full()
                                .gap_4()
                                .pb_8()
                                .pr_4()
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .items_center()
                                        .child(palette::tile(
                                            Icon::new(IconName::Settings2),
                                            Area::Settings.accent(cx),
                                            px(28.),
                                            cx,
                                        ))
                                        .child(
                                            div()
                                                .text_lg()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .child("Settings"),
                                        ),
                                )
                                .child(appearance)
                                .child(startup)
                                .child(updates)
                                .child(designer)
                                .child(flows)
                                .child(bar),
                        ),
                    ),
            )
            .vertical_scrollbar(&self.scroll)
    }
}
