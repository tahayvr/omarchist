use crate::system::themes::custom_themes::get_user_themes;
use crate::system::themes::system_themes::get_system_themes;
use crate::types::themes::ThemeOrigin;
use crate::ui::menu::app_menu;
use crate::ui::themes_page::theme_grid::{ThemeFilter, ThemeGrid};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme,
    scroll::ScrollableElement,
    tab::{Tab, TabBar},
    v_flex,
};

const KEY_CONTEXT: &str = "ThemesPage";

// Which part of the themes page has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemesFocus {
    Tabs,
    Grid,
}

pub struct ThemesPage {
    active_tab: usize,
    theme_grid: Entity<ThemeGrid>,
    focus: ThemesFocus,
    has_global_focus: bool,
    pub focus_handle: FocusHandle,
}

impl ThemesPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let themes = Self::load_all_themes();
        let theme_grid = cx.new(|cx| ThemeGrid::new(cx, themes));

        Self {
            active_tab: 0,
            theme_grid,
            focus: ThemesFocus::Tabs,
            has_global_focus: false,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn set_global_focus(&mut self, has_focus: bool, cx: &mut Context<Self>) {
        if self.has_global_focus == has_focus {
            return;
        }
        self.has_global_focus = has_focus;
        self.sync_grid_focus(cx);
        cx.notify();
    }

    pub fn current_focus(&self) -> ThemesFocus {
        self.focus
    }

    pub fn reset_focus(&mut self, cx: &mut Context<Self>) {
        self.focus = ThemesFocus::Tabs;
        self.theme_grid.update(cx, |grid, _cx| {
            grid.clear_focus();
        });
        self.sync_grid_focus(cx);
        cx.notify();
    }

    pub fn handle_next_item(&mut self, cx: &mut Context<Self>) {
        match self.focus {
            ThemesFocus::Tabs => {
                self.focus = ThemesFocus::Grid;
                self.sync_grid_focus(cx);
                cx.notify();
            }
            ThemesFocus::Grid => {
                self.theme_grid.update(cx, |grid, cx| {
                    grid.move_down(cx);
                });
            }
        }
    }

    pub fn handle_prev_item(&mut self, cx: &mut Context<Self>) {
        match self.focus {
            ThemesFocus::Tabs => {}
            ThemesFocus::Grid => {
                // If we can't move up (already at top row), move focus back to tabs
                let moved = self.theme_grid.update(cx, |grid, cx| grid.move_up(cx));
                if !moved {
                    self.focus = ThemesFocus::Tabs;
                    self.sync_grid_focus(cx);
                    cx.notify();
                }
            }
        }
    }

    pub fn handle_select_next(&mut self, cx: &mut Context<Self>) {
        match self.focus {
            ThemesFocus::Tabs => {
                if self.active_tab < 1 {
                    self.active_tab += 1;
                    self.sync_filter_to_grid(cx);
                    cx.notify();
                }
            }
            ThemesFocus::Grid => {
                self.theme_grid.update(cx, |grid, cx| {
                    grid.move_right(cx);
                });
            }
        }
    }

    pub fn handle_select_prev(&mut self, cx: &mut Context<Self>) {
        match self.focus {
            ThemesFocus::Tabs => {
                if self.active_tab > 0 {
                    self.active_tab -= 1;
                    self.sync_filter_to_grid(cx);
                    cx.notify();
                }
            }
            ThemesFocus::Grid => {
                self.theme_grid.update(cx, |grid, cx| {
                    grid.move_left(cx);
                });
            }
        }
    }

    pub fn handle_activate(&mut self, cx: &mut Context<Self>) {
        match self.focus {
            ThemesFocus::Tabs => {
                self.focus = ThemesFocus::Grid;
                self.sync_grid_focus(cx);
                cx.notify();
            }
            ThemesFocus::Grid => {
                self.theme_grid.update(cx, |grid, cx| {
                    grid.activate_focused(cx);
                });
            }
        }
    }

    // Push the current active_tab-derived filter into the grid.
    fn sync_filter_to_grid(&mut self, cx: &mut Context<Self>) {
        let filter = self.current_filter();
        self.theme_grid.update(cx, |grid, _| {
            grid.set_filter(filter);
        });
    }

    // Push whether the grid should show as focused based on current state.
    fn sync_grid_focus(&mut self, cx: &mut Context<Self>) {
        let grid_has_focus = self.has_global_focus && self.focus == ThemesFocus::Grid;
        self.theme_grid.update(cx, |grid, _cx| {
            grid.set_has_focus(grid_has_focus);
        });
    }

    fn current_filter(&self) -> ThemeFilter {
        match self.active_tab {
            1 => ThemeFilter::Only(ThemeOrigin::Omarchist),
            _ => ThemeFilter::All,
        }
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tabs_has_focus = self.has_global_focus && self.focus == ThemesFocus::Tabs;

        let secondary_bg = cx.theme().secondary;

        v_flex()
            .id("themes-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .overflow_y_scrollbar()
            .overflow_x_hidden()
            .gap_4()
            .on_action(cx.listener(|this, _: &app_menu::NextItem, _window, cx| {
                this.handle_next_item(cx);
            }))
            .on_action(cx.listener(|this, _: &app_menu::PrevItem, _window, cx| {
                this.handle_prev_item(cx);
            }))
            .on_action(cx.listener(|this, _: &app_menu::SelectNext, _window, cx| {
                this.handle_select_next(cx);
            }))
            .on_action(cx.listener(|this, _: &app_menu::SelectPrev, _window, cx| {
                // If at the tabs level and trying to go left, bubble to MainWindow
                // so it can move focus back to the sidebar
                if this.focus != ThemesFocus::Tabs {
                    this.handle_select_prev(cx);
                }
            }))
            .on_action(
                cx.listener(|this, _: &app_menu::ActivateItem, _window, cx| {
                    this.handle_activate(cx);
                }),
            )
            .on_action(cx.listener(|this, _: &app_menu::EscapeFocus, _window, cx| {
                this.reset_focus(cx);
            }))
            .child(
                div()
                    .when(tabs_has_focus, move |this| this.bg(secondary_bg))
                    .child(
                        TabBar::new("theme-tabs")
                            .cursor_pointer()
                            .selected_index(self.active_tab)
                            .on_click(cx.listener(|view, index, _, cx| {
                                view.active_tab = *index;
                                view.sync_filter_to_grid(cx);
                                cx.notify();
                            }))
                            .child(Tab::new().label("All Themes"))
                            .child(Tab::new().label("Omarchist Themes")),
                    ),
            )
            .child(self.theme_grid.clone())
    }
}
