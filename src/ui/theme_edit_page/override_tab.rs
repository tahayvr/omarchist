use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, h_flex, v_flex};
use gpui_kit::TestSupportExt;

use crate::system::flows::requirements::is_installed;
use crate::system::themes::overrides::{self, Category, OverrideSpec, OverrideStatus};
use crate::system::themes::theme_file_ops::get_theme_path;
use crate::ui::focus;
use crate::ui::theme_edit_page::override_pane::{OverridePane, StatusChanged};
use crate::ui::theme_edit_page::shared::{help_text, tab_container};

pub mod override_nav {
    gpui::actions!(override_nav, [Prev, Next, First, Last, Activate]);
}

pub const NAV_CONTEXT: &str = "OverrideNav";

/// One optional tab of the Theme Designer: the apps of a category on the
/// left, the selected app's pane on the right.
pub struct OverrideTab {
    theme_name: String,
    category: Category,
    specs: Vec<&'static OverrideSpec>,
    installed: Vec<bool>,
    custom: Vec<bool>,
    /// Built when an app is first selected.
    panes: Vec<Option<Entity<OverridePane>>>,
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
        let specs: Vec<_> = overrides::in_category(category).collect();
        let installed = specs
            .iter()
            .map(|spec| spec.binaries.is_empty() || spec.binaries.iter().any(|b| is_installed(b)))
            .collect();
        let custom = specs
            .iter()
            .map(|spec| overrides::status(&theme_name, spec) == OverrideStatus::Custom)
            .collect();
        let mut tab = Self {
            theme_name,
            category,
            panes: vec![None; specs.len()],
            specs,
            installed,
            custom,
            active: 0,
            nav_focus: focus::tab_stop(cx),
            content_focus: cx.focus_handle(),
            _subscriptions: Vec::new(),
        };
        tab.ensure_pane(0, window, cx);
        tab
    }

    fn ensure_pane(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.panes.get(index).is_none_or(Option::is_some) {
            return;
        }
        let spec = self.specs[index];
        let installed = self.installed[index];
        let theme_name = self.theme_name.clone();
        let pane = cx.new(|cx| OverridePane::new(theme_name, spec, installed, window, cx));
        self._subscriptions.push(cx.subscribe(
            &pane,
            move |this: &mut Self, pane, _: &StatusChanged, cx| {
                this.custom[index] = pane.read(cx).is_custom();
                cx.notify();
            },
        ));
        self.panes[index] = Some(pane);
    }

    /// Selects an app by its file.
    pub fn select(&mut self, file: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(index) = self.specs.iter().position(|spec| spec.file == file) {
            self.set_active(index, window, cx);
        }
    }

    fn set_active(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let index = index.min(self.specs.len().saturating_sub(1));
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

        let items = self.specs.iter().enumerate().map(|(ix, spec)| {
            let active = self.active == ix;
            let focused = nav_focused && active;
            let theme = cx.theme();
            h_flex()
                .id(SharedString::from(format!("override-nav-{}", spec.file)))
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
                .child(div().text_sm().child(spec.app))
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
            .children(pane);
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

        // A whole shell.toml stops Omarchy from generating one; section files
        // are still applied on top of it.
        let whole_shell = self.category == Category::Desktop
            && get_theme_path(&self.theme_name, false)
                .is_some_and(|dir| dir.join("shell.toml").is_file());

        tab_container()
            .child(help_text(
                "Everything here is optional. Your palette already themes these apps; \
                 customize one only when you want it to look different.",
                cx.theme().muted_foreground,
            ))
            .when(whole_shell, |tab| {
                tab.child(help_text(
                    "This theme ships a complete shell.toml, which Omarchy uses instead of \
                     generating one. The shell sections below still apply on top of it, but \
                     their starting values come from Omarchy's template, not from that file.",
                    cx.theme().warning,
                ))
            })
            .child(layout)
    }
}
