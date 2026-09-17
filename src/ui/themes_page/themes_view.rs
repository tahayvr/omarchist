use crate::system::themes::custom_themes::get_user_themes;
use crate::system::themes::system_themes::get_system_themes;
use crate::types::themes::ThemeOrigin;
use crate::ui::focus::{self, tab_strip_container};
use crate::ui::themes_page::theme_grid::{self, ThemeFilter, ThemeGrid};
use gpui::*;
use gpui_component::{
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

        cx.spawn(async move |this, cx| {
            let themes = smol::unblock(Self::load_all_themes).await;
            this.update(cx, |this, cx| {
                this.theme_grid.update(cx, |grid, cx| {
                    grid.update_themes(themes, cx);
                });
                cx.notify();
            })
            .ok();
        })
        .detach();

        Self {
            active_tab: 0,
            theme_grid,
            tabs_focus: focus::tab_stop(cx),
            focus_handle: cx.focus_handle(),
        }
    }

    /// Focuses the tab strip, the page's first control.
    pub fn focus_entry(&self, window: &mut Window, _cx: &mut Context<Self>) {
        self.tabs_focus.focus(window);
    }

    fn set_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        let index = index.min(TAB_COUNT - 1);
        if self.active_tab != index {
            self.active_tab = index;
            cx.notify();
        }
    }

    fn focus_grid(&self, window: &mut Window, cx: &Context<Self>) {
        self.theme_grid.read(cx).focus.focus(window);
    }

    pub fn refresh_themes(&mut self, cx: &mut Context<Self>) {
        let themes = Self::load_all_themes();
        self.theme_grid.update(cx, |grid, cx| {
            grid.update_themes(themes, cx);
        });
        cx.notify();
    }

    pub fn set_sidebar_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        self.theme_grid.update(cx, |grid, _| {
            grid.set_sidebar_collapsed(collapsed);
        });
    }

    fn load_all_themes() -> Vec<crate::types::themes::ThemeEntry> {
        let mut themes = Vec::new();
        if let Ok(system) = get_system_themes() {
            themes.extend(system);
        }
        if let Ok(user) = get_user_themes() {
            themes.extend(user);
        }
        themes
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
            .on_action(cx.listener(|this, _: &theme_grid::LeaveGrid, window, _cx| {
                this.tabs_focus.focus(window);
            }))
            .child(
                tab_strip_container("theme-tabs-strip", &self.tabs_focus, window, cx)
                    .on_action(cx.listener(|this, _: &focus::tab_strip::Prev, _, cx| {
                        this.set_tab(this.active_tab.saturating_sub(1), cx);
                    }))
                    .on_action(cx.listener(|this, _: &focus::tab_strip::Next, _, cx| {
                        this.set_tab(this.active_tab + 1, cx);
                    }))
                    .on_action(cx.listener(|this, _: &focus::tab_strip::First, _, cx| {
                        this.set_tab(0, cx);
                    }))
                    .on_action(cx.listener(|this, _: &focus::tab_strip::Last, _, cx| {
                        this.set_tab(TAB_COUNT - 1, cx);
                    }))
                    .on_action(
                        cx.listener(|this, _: &focus::tab_strip::Activate, window, cx| {
                            this.focus_grid(window, cx);
                        }),
                    )
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
