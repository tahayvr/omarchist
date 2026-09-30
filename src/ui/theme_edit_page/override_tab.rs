use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, h_flex, v_flex};
use gpui_kit::TestSupportExt;

use crate::system::flows::requirements::is_installed;
use crate::system::themes::overrides::Category;
use crate::system::themes::overrides::entries::{self, Entry};
use crate::system::themes::theme_management::load_theme_for_editing;
use crate::ui::focus;
use crate::ui::theme_edit_page::override_pane::{OverridePane, StatusChanged};
use crate::ui::theme_edit_page::palette_pane::PalettePane;
use crate::ui::theme_edit_page::shared::tab_container;

pub mod override_nav {
    gpui::actions!(override_nav, [Prev, Next, First, Last, Activate]);
}

pub const NAV_CONTEXT: &str = "OverrideNav";

#[derive(Clone)]
enum Pane {
    File(Entity<OverridePane>),
    Bundle(Entity<PalettePane>),
}

impl Pane {
    fn element(&self) -> AnyElement {
        match self {
            Pane::File(pane) => pane.clone().into_any_element(),
            Pane::Bundle(pane) => pane.clone().into_any_element(),
        }
    }
}

/// One optional tab of the Theme Designer: the apps of a category on the
/// left, the selected app's pane on the right.
pub struct OverrideTab {
    theme_name: String,
    entries: Vec<Entry>,
    installed: Vec<bool>,
    custom: Vec<bool>,
    /// Built when an app is first selected.
    panes: Vec<Option<Pane>>,
    active: usize,
    /// The app list is one tab stop; up/down move between apps.
    nav_focus: FocusHandle,
    /// Non-tab-stop handle on the pane for `focus_first_in`.
    content_focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl OverrideTab {
    pub fn new(
        theme_name: String,
        category: Category,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let entries = entries::in_category(category);
        let installed = entries
            .iter()
            .map(|entry| {
                entry.binaries().is_empty() || entry.binaries().iter().any(|b| is_installed(b))
            })
            .collect();
        let mut tab = Self {
            theme_name,
            panes: vec![None; entries.len()],
            custom: vec![false; entries.len()],
            entries,
            installed,
            active: 0,
            nav_focus: focus::tab_stop(cx),
            content_focus: cx.focus_handle(),
            _subscriptions: Vec::new(),
        };
        tab.refresh_status();
        tab.ensure_pane(0, window, cx);
        tab
    }

    fn refresh_status(&mut self) {
        let palettes = load_theme_for_editing(&self.theme_name)
            .map(|theme| theme.palettes)
            .unwrap_or_default();
        self.custom = self
            .entries
            .iter()
            .map(|entry| entry.is_custom(&self.theme_name, &palettes))
            .collect();
    }

    /// Re-reads the theme, so palette bundles show the palette's current
    /// colors after the Colors tab changed them.
    /// Writes every pane's pending edit now.
    pub fn flush(&mut self, cx: &mut Context<Self>) {
        for pane in self.panes.iter().flatten() {
            match pane {
                Pane::File(pane) => pane.update(cx, |pane, cx| pane.flush(cx)),
                Pane::Bundle(pane) => pane.update(cx, |pane, cx| pane.flush(cx)),
            }
        }
    }

    pub fn refresh(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.refresh_status();
        for pane in self.panes.iter().flatten() {
            if let Pane::Bundle(pane) = pane {
                pane.update(cx, |pane, cx| pane.reload(window, cx));
            }
        }
        cx.notify();
    }

    fn ensure_pane(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.panes.get(index).is_none_or(Option::is_some) {
            return;
        }
        let installed = self.installed[index];
        let theme_name = self.theme_name.clone();
        let pane = match self.entries[index] {
            Entry::File(spec) => {
                let pane = cx.new(|cx| OverridePane::new(theme_name, spec, installed, window, cx));
                self._subscriptions.push(cx.subscribe(
                    &pane,
                    move |this: &mut Self, pane, _: &StatusChanged, cx| {
                        this.custom[index] = pane.read(cx).is_custom();
                        cx.notify();
                    },
                ));
                Pane::File(pane)
            }
            Entry::Bundle(bundle) => {
                let pane = cx.new(|cx| PalettePane::new(theme_name, bundle, installed, window, cx));
                self._subscriptions.push(cx.subscribe(
                    &pane,
                    move |this: &mut Self, pane, _: &StatusChanged, cx| {
                        this.custom[index] = pane.read(cx).is_custom();
                        cx.notify();
                    },
                ));
                Pane::Bundle(pane)
            }
        };
        self.panes[index] = Some(pane);
    }

    /// Focuses the app list.
    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.nav_focus.focus(window, cx);
    }

    /// Selects an app by its entry id (a file name or a bundle id).
    pub fn select(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.entries.iter().position(|entry| entry.id() == id) {
            self.set_active(index, window, cx);
        }
    }

    fn set_active(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let index = index.min(self.entries.len().saturating_sub(1));
        if self.active != index {
            self.active = index;
            self.ensure_pane(index, window, cx);
            cx.notify();
        }
    }

    fn render_nav(
        &self,
        wide: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let nav_focused = self.nav_focus.is_focused(window);
        let theme = cx.theme();
        let (radius, transparent) = (theme.radius, theme.transparent);

        let items = self.entries.iter().enumerate().map(|(ix, entry)| {
            let active = self.active == ix;
            let focused = nav_focused && active;
            let theme = cx.theme();
            h_flex()
                .id(SharedString::from(format!("override-nav-{}", entry.id())))
                .gap_2()
                .px_2()
                .py_1p5()
                .rounded(radius)
                .border_1()
                .border_color(focus::focus_border(focused, transparent, cx))
                .when(active, |row| {
                    row.bg(theme.sidebar_accent)
                        .text_color(theme.sidebar_accent_foreground)
                        .font_weight(FontWeight::SEMIBOLD)
                })
                .when(!active, |row| row.hover(|row| row.bg(theme.list_hover)))
                .when(!active && !self.installed[ix], |row| {
                    row.text_color(theme.muted_foreground)
                })
                .child(
                    div()
                        .w(px(3.))
                        .h_4()
                        .flex_none()
                        .rounded_full()
                        .bg(if active { theme.primary } else { transparent }),
                )
                .child(
                    div()
                        .size_2()
                        .flex_none()
                        .rounded_full()
                        .bg(if self.custom[ix] {
                            theme.primary
                        } else {
                            theme.border
                        }),
                )
                .child(div().text_sm().child(entry.app()))
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.nav_focus.focus(window, cx);
                    this.set_active(ix, window, cx);
                }))
        });

        let list = if wide {
            v_flex().w(px(220.)).flex_none().gap_1()
        } else {
            h_flex().w_full().flex_wrap().gap_1()
        };

        // Fixed id: the list's test target.
        div()
            .id("override-nav")
            .test_support()
            .key_context(NAV_CONTEXT)
            .track_focus(&self.nav_focus)
            .cursor_pointer()
            .on_action(cx.listener(|this, _: &override_nav::Prev, window, cx| {
                this.set_active(this.active.saturating_sub(1), window, cx);
            }))
            .on_action(cx.listener(|this, _: &override_nav::Next, window, cx| {
                this.set_active(this.active + 1, window, cx);
            }))
            .on_action(cx.listener(|this, _: &override_nav::First, window, cx| {
                this.set_active(0, window, cx);
            }))
            .on_action(cx.listener(|this, _: &override_nav::Last, window, cx| {
                this.set_active(usize::MAX, window, cx);
            }))
            .on_action(cx.listener(|this, _: &override_nav::Activate, window, cx| {
                focus::focus_first_in(&this.content_focus, window, cx);
            }))
            .child(list.children(items))
    }
}

impl Render for OverrideTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let wide = window.viewport_size().width >= px(768.);
        let pane = self.panes.get(self.active).cloned().flatten();
        let content = div()
            .id("override-content")
            .track_focus(&self.content_focus)
            .flex_1()
            .min_w_0()
            .children(pane.map(|pane| pane.element()));
        let nav = self.render_nav(wide, window, cx);

        let layout = if wide {
            div()
                .flex()
                .flex_row()
                .items_start()
                .gap_6()
                .child(nav)
                .child(content)
        } else {
            div().flex().flex_col().gap_4().child(nav).child(content)
        };

        tab_container().child(layout)
    }
}
