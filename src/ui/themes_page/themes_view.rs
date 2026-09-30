use crate::system::themes::custom_themes::get_user_themes;
use crate::system::themes::system_themes::get_system_themes;
use crate::types::themes::ThemeOrigin;
use crate::ui::dialogs::create_theme_dialog::open_create_theme_dialog;
use crate::ui::focus::{self, tab_strip_container};
use crate::ui::menu::app_menu::NewTheme;
use crate::ui::themes_page::theme_grid::{self, ThemeFilter, ThemeGrid};
use gpui::*;
use gpui_component::{
    Icon, Sizable as _, WindowExt,
    button::{Button, ButtonVariants as _},
    h_flex,
    tab::{Tab, TabBar},
    v_flex,
};

const KEY_CONTEXT: &str = "ThemesPage";
const TAB_COUNT: usize = 2;

pub struct ThemesPage {
    active_tab: usize,
    theme_grid: Entity<ThemeGrid>,
    /// The "All / Omarchist" strip is one tab stop; left/right switch tabs.
    tabs_focus: FocusHandle,
    pub focus_handle: FocusHandle,
}

impl ThemesPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Start with an empty grid so the main thread is not blocked at startup.
        // Themes are loaded on a background thread and pushed into the grid once ready.
        let theme_grid = cx.new(|cx| ThemeGrid::new(vec![], window, cx));

        // Not `smol::unblock`: the test scheduler only drives gpui's own executor.
        Self::spawn_load(window, cx);

        Self {
            active_tab: 0,
            theme_grid,
            tabs_focus: focus::tab_stop(cx),
            focus_handle: cx.focus_handle(),
        }
    }

    /// Focuses the tab strip, the page's first control.
    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.tabs_focus.focus(window, cx);
    }

    fn set_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        let index = index.min(TAB_COUNT - 1);
        if self.active_tab != index {
            self.active_tab = index;
            cx.notify();
        }
    }

    fn focus_grid(&self, window: &mut Window, cx: &mut Context<Self>) {
        let focus = self.theme_grid.read(cx).focus.clone();
        focus.focus(window, cx);
    }

    /// Rescans both theme folders in the background and replaces the grid.
    pub fn refresh_themes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        Self::spawn_load(window, cx);
    }

    /// Reads both theme folders off the UI thread and fills the grid; a
    /// folder that could not be read is reported, since an empty grid
    /// would otherwise look like "no themes".
    fn spawn_load(window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            let (themes, errors) = cx.background_spawn(async { Self::load_all_themes() }).await;
            this.update_in(cx, |this, window, cx| {
                this.theme_grid.update(cx, |grid, cx| {
                    grid.update_themes(themes, cx);
                });
                for error in errors {
                    window.push_notification(error, cx);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn set_sidebar_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        self.theme_grid.update(cx, |grid, _| {
            grid.set_sidebar_collapsed(collapsed);
        });
    }

    fn load_all_themes() -> (Vec<crate::types::themes::ThemeEntry>, Vec<String>) {
        let mut themes = Vec::new();
        let mut errors = Vec::new();
        match get_system_themes() {
            Ok(system) => themes.extend(system),
            Err(e) => errors.push(format!("Could not read Omarchy's themes: {e}")),
        }
        match get_user_themes() {
            Ok(user) => themes.extend(user),
            Err(e) => errors.push(format!("Could not read your themes: {e}")),
        }
        // One list, A to Z, with the running theme marked.
        themes.sort_by_key(|theme| theme.title.to_lowercase());
        let applied = crate::system::ui_theme_watcher::get_active_omarchy_theme_name();
        for theme in &mut themes {
            theme.applied = applied.as_deref() == Some(theme.dir.as_str());
        }
        (themes, errors)
    }
}

impl Render for ThemesPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let filter = match self.active_tab {
            1 => ThemeFilter::Only(ThemeOrigin::Omarchist),
            _ => ThemeFilter::All,
        };

        self.theme_grid.update(cx, |grid, _| {
            grid.set_filter(filter);
        });

        v_flex()
            .id("themes-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_hidden()
            .gap_4()
            .on_action(cx.listener(|this, _: &theme_grid::LeaveGrid, window, cx| {
                this.tabs_focus.focus(window, cx);
            }))
            .child(
                h_flex()
                    .gap_3()
                    .child(
                        div().flex_1().min_w_0().child(
                            tab_strip_container("theme-tabs-strip", &self.tabs_focus, window, cx)
                                .on_action(cx.listener(
                                    |this, _: &focus::tab_strip::Prev, _, cx| {
                                        this.set_tab(this.active_tab.saturating_sub(1), cx);
                                    },
                                ))
                                .on_action(cx.listener(
                                    |this, _: &focus::tab_strip::Next, _, cx| {
                                        this.set_tab(this.active_tab + 1, cx);
                                    },
                                ))
                                .on_action(cx.listener(
                                    |this, _: &focus::tab_strip::First, _, cx| {
                                        this.set_tab(0, cx);
                                    },
                                ))
                                .on_action(cx.listener(
                                    |this, _: &focus::tab_strip::Last, _, cx| {
                                        this.set_tab(TAB_COUNT - 1, cx);
                                    },
                                ))
                                .on_action(cx.listener(
                                    |this, _: &focus::tab_strip::Activate, window, cx| {
                                        this.focus_grid(window, cx);
                                    },
                                ))
                                .child(
                                    TabBar::new("theme-tabs")
                                        .cursor_pointer()
                                        .selected_index(self.active_tab)
                                        .on_click(cx.listener(|view, index, _, cx| {
                                            view.set_tab(*index, cx);
                                        }))
                                        .child(Tab::new().label("All Themes"))
                                        .child(Tab::new().label("Omarchist Themes")),
                                ),
                        ),
                    )
                    .child(
                        Button::new("new-theme")
                            .primary()
                            .small()
                            .icon(Icon::new(Icon::empty()).path("icons/plus.svg"))
                            .label("New theme")
                            .tooltip_with_action("Create a theme", &NewTheme, None)
                            .cursor_pointer()
                            .on_click(|_, window, cx| open_create_theme_dialog(window, cx)),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(self.theme_grid.clone()),
            )
    }
}
