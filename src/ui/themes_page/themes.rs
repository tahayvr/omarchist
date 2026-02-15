use crate::system::themes::custom_themes::get_custom_themes;
use crate::system::themes::system_themes::get_system_themes;
use crate::types::themes::{CustomTheme, SysTheme};
use crate::ui::themes_page::theme_grid::{ThemeFilter, ThemeGrid};
use gpui::*;
use gpui_component::{
    scroll::ScrollableElement,
    tab::{Tab, TabBar},
    v_flex,
};

pub struct ThemesPage {
    active_tab: usize,
    theme_grid: Entity<ThemeGrid>,
}

impl ThemesPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut themes = Vec::new();

        // Load system themes
        if let Ok(system_themes) = get_system_themes() {
            themes.extend(system_themes);
        }

        // Load custom themes and convert to SysTheme
        if let Ok(custom_themes) = get_custom_themes() {
            for custom_theme in custom_themes {
                let sys_theme = Self::custom_theme_to_sys_theme(custom_theme);
                themes.push(sys_theme);
            }
        }

        let theme_grid = cx.new(|cx| ThemeGrid::new(cx, themes.clone(), None));

        Self {
            active_tab: 0,
            theme_grid,
        }
    }

    /// Refresh the themes list - reload from disk
    pub fn refresh_themes(&mut self, cx: &mut Context<Self>) {
        let mut themes = Vec::new();

        // Load system themes
        if let Ok(system_themes) = get_system_themes() {
            themes.extend(system_themes);
        }

        // Load custom themes and convert to SysTheme
        if let Ok(custom_themes) = get_custom_themes() {
            for custom_theme in custom_themes {
                let sys_theme = Self::custom_theme_to_sys_theme(custom_theme);
                themes.push(sys_theme);
            }
        }

        // Update the theme grid with new themes
        self.theme_grid.update(cx, |grid, cx| {
            grid.update_themes(themes, cx);
        });

        cx.notify();
    }

    fn custom_theme_to_sys_theme(custom_theme: CustomTheme) -> SysTheme {
        SysTheme {
            dir: custom_theme.name.clone(),
            title: crate::system::themes::custom_themes::dir_to_title(&custom_theme.name),
            description: format!(
                "Custom theme by {}",
                custom_theme.author.as_deref().unwrap_or("Unknown")
            ),
            image: custom_theme.image,
            is_system: false,
            is_custom: true,
            colors: custom_theme.colors,
        }
    }
}

impl ThemesPage {
    pub fn set_sidebar_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        self.theme_grid.update(cx, |grid, _| {
            grid.set_sidebar_collapsed(collapsed);
        });
    }
}

impl Render for ThemesPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let filter = match self.active_tab {
            0 => ThemeFilter::System,
            1 => ThemeFilter::Custom,
            _ => ThemeFilter::System,
        };

        // Update the theme grid's filter
        self.theme_grid.update(cx, |grid, _| {
            grid.set_filter(Some(filter));
        });

        v_flex()
            .id("themes-page")
            .size_full()
            .overflow_y_scrollbar()
            .overflow_x_hidden()
            .gap_4()
            .child(
                TabBar::new("theme-tabs")
                    // .segmented()
                    .cursor_pointer()
                    .selected_index(self.active_tab)
                    .on_click(cx.listener(|view, index, _, cx| {
                        view.active_tab = *index;
                        cx.notify();
                    }))
                    .child(Tab::new().label("System Themes"))
                    .child(Tab::new().label("Custom Themes")),
            )
            .child(self.theme_grid.clone())
    }
}
