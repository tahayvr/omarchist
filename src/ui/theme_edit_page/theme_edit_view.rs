use crate::shell::theme_sh_commands::apply_theme;
use crate::system::themes::theme_file_ops::is_system_theme;
use crate::system::themes::theme_management::load_theme_for_editing;
use crate::types::themes::EditingTheme;
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::focus::{self, tab_strip_container};
use crate::ui::menu::app_menu;
use crate::ui::theme_edit_page::backgrounds_tab::BackgroundsTab;
use crate::ui::theme_edit_page::colors_tab::ColorsTab;
use crate::ui::theme_edit_page::editor_tab::EditorTab;
use crate::ui::theme_edit_page::file_manager_tab::FileManagerTab;
use crate::ui::theme_edit_page::general_tab::GeneralTab;
use crate::ui::theme_edit_page::overrides_tab::OverridesTab;
use crate::ui::theme_edit_page::shared::error_message;
use gpui::*;
use gpui_component::{
    ActiveTheme,
    button::Button,
    h_flex,
    tab::{Tab, TabBar},
    v_flex,
};

const KEY_CONTEXT: &str = "ThemeEditPage";

// Tab order of the Theme Designer. UI-only, so it lives with the page
// rather than in the shared theme data types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeEditTab {
    General,
    Colors,
    FileManager,
    Editor,
    Overrides,
    Backgrounds,
}

impl ThemeEditTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeEditTab::General => "General",
            ThemeEditTab::Colors => "Colors",
            ThemeEditTab::FileManager => "File Manager",
            ThemeEditTab::Editor => "Editor",
            ThemeEditTab::Overrides => "Overrides",
            ThemeEditTab::Backgrounds => "Backgrounds",
        }
    }

    pub fn all() -> Vec<ThemeEditTab> {
        vec![
            ThemeEditTab::General,
            ThemeEditTab::Colors,
            ThemeEditTab::FileManager,
            ThemeEditTab::Editor,
            ThemeEditTab::Overrides,
            ThemeEditTab::Backgrounds,
        ]
    }
}

actions!(theme_edit, [ApplyTheme]);

pub struct ThemeEditPage {
    theme_name: String,
    active_tab: usize,
    tab_count: usize,
    error_message: Option<String>,
    general_tab: Entity<GeneralTab>,
    colors_tab: Entity<ColorsTab>,
    file_manager_tab: Entity<FileManagerTab>,
    editor_tab: Entity<EditorTab>,
    overrides_tab: Entity<OverridesTab>,
    backgrounds_tab: Entity<BackgroundsTab>,
    pub focus_handle: FocusHandle,
    /// The tab strip is one tab stop; left/right switch tabs.
    tabs_focus: FocusHandle,
    /// Non-tab-stop handle on the scrolling content, so `focus_first_in`
    /// can land on the active tab's first field.
    content_focus: FocusHandle,
    scroll: ScrollHandle,
}

impl ThemeEditPage {
    pub fn new(theme_name: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Determine if this is a system theme or custom theme
        let is_system = is_system_theme(&theme_name);

        // Load theme data
        let theme_data = match load_theme_for_editing(&theme_name) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to load theme '{}': {}", theme_name, e);
                EditingTheme::default()
            }
        };

        // Shared by every tab so focused sections can scroll into view.
        let scroll = ScrollHandle::new();

        let general_tab = cx
            .new(|cx| GeneralTab::new(theme_name.clone(), theme_data.clone(), &scroll, window, cx));
        let colors_tab = cx
            .new(|cx| ColorsTab::new(theme_name.clone(), theme_data.clone(), &scroll, window, cx));
        let file_manager_tab = cx.new(|cx| {
            FileManagerTab::new(theme_name.clone(), theme_data.clone(), &scroll, window, cx)
        });
        let editor_tab = cx
            .new(|cx| EditorTab::new(theme_name.clone(), theme_data.clone(), &scroll, window, cx));
        // btop / Chromium / lock screen
        let overrides_tab = cx.new(|cx| {
            OverridesTab::new(theme_name.clone(), theme_data.clone(), &scroll, window, cx)
        });
        let backgrounds_tab =
            cx.new(|cx| BackgroundsTab::new(theme_name.clone(), is_system, &scroll, window, cx));

        let tab_count = ThemeEditTab::all().len();

        let focus_handle = cx.focus_handle();
        let tabs_focus = focus::tab_stop(cx);
        tabs_focus.focus(window);

        Self {
            theme_name,
            active_tab: 0,
            tab_count,
            error_message: None,
            general_tab,
            colors_tab,
            file_manager_tab,
            editor_tab,
            overrides_tab,
            backgrounds_tab,
            focus_handle,
            tabs_focus,
            content_focus: cx.focus_handle(),
            scroll,
        }
    }

    /// Focuses the tab strip, the page's first control after Back/Apply.
    pub fn focus_entry(&self, window: &mut Window, _cx: &mut Context<Self>) {
        self.tabs_focus.focus(window);
    }

    fn apply_theme(&self) {
        let dir = self.theme_name.clone();
        smol::spawn(async move {
            if let Err(e) = apply_theme(dir).await {
                eprintln!("Failed to apply theme: {}", e);
            }
        })
        .detach();
    }

    fn set_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        let index = index.min(self.tab_count.saturating_sub(1));
        if self.active_tab != index {
            self.active_tab = index;
            self.scroll.set_offset(Point::default());
            cx.notify();
        }
    }

    pub fn theme_name(&self) -> &str {
        &self.theme_name
    }

    fn navigate_back(&self, _window: &mut Window, cx: &mut Context<Self>) {
        // Refresh first so a newly created theme is in the grid on arrival.
        emit(cx, AppEvent::RefreshThemes);
        emit(cx, AppEvent::Navigate(ActivePage::Themes));
    }

    fn next_tab(&mut self, cx: &mut Context<Self>) {
        self.set_tab(self.active_tab + 1, cx);
    }

    fn prev_tab(&mut self, cx: &mut Context<Self>) {
        self.set_tab(self.active_tab.saturating_sub(1), cx);
    }

    fn render_tab_content(&self, _window: &mut Window, _cx: &mut Context<Self>) -> AnyElement {
        let tabs = ThemeEditTab::all();
        let active_tab = tabs
            .get(self.active_tab)
            .copied()
            .unwrap_or(ThemeEditTab::General);

        match active_tab {
            ThemeEditTab::General => {
                // Use the GeneralTab entity
                self.general_tab.clone().into_any_element()
            }
            ThemeEditTab::Colors => {
                // Use the ColorsTab entity
                self.colors_tab.clone().into_any_element()
            }
            ThemeEditTab::FileManager => {
                // Use the FileManagerTab entity
                self.file_manager_tab.clone().into_any_element()
            }
            ThemeEditTab::Editor => {
                // Use the EditorTab entity
                self.editor_tab.clone().into_any_element()
            }
            ThemeEditTab::Overrides => {
                // Use the OverridesTab entity
                self.overrides_tab.clone().into_any_element()
            }
            ThemeEditTab::Backgrounds => {
                // Use the BackgroundsTab entity
                self.backgrounds_tab.clone().into_any_element()
            }
        }
    }
}

impl Render for ThemeEditPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let tabs = ThemeEditTab::all();
        let _viewport_width = window.viewport_size().width;

        v_flex()
            .id("theme-edit-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(theme.background)
            .gap_4()
            .overflow_x_hidden()
            .on_action(
                cx.listener(|this, _: &app_menu::ThemeEditNextTab, _window, cx| {
                    this.next_tab(cx);
                }),
            )
            .on_action(
                cx.listener(|this, _: &app_menu::ThemeEditPrevTab, _window, cx| {
                    this.prev_tab(cx);
                }),
            )
            .on_action(cx.listener(|this, _: &app_menu::NavigateBack, window, cx| {
                this.navigate_back(window, cx);
            }))
            .on_action(cx.listener(|this, _: &ApplyTheme, _window, _cx| {
                this.apply_theme();
            }))
            .child(
                // Back button + Tabs row - wraps on narrow screens
                h_flex()
                    .gap_4()
                    .items_start()
                    .flex_wrap()
                    .child(
                        Button::new("back-btn")
                            .label("Back")
                            .compact()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate_back(window, cx);
                            })),
                    )
                    .child(
                        Button::new("apply-theme-btn")
                            .label("Apply Theme")
                            .compact()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _window, _cx| this.apply_theme())),
                    )
                    .child(
                        tab_strip_container("theme-edit-tabs-strip", &self.tabs_focus, window, cx)
                            .flex_1()
                            .min_w_0()
                            .on_action(cx.listener(|this, _: &focus::tab_strip::Prev, _, cx| {
                                this.prev_tab(cx);
                            }))
                            .on_action(cx.listener(|this, _: &focus::tab_strip::Next, _, cx| {
                                this.next_tab(cx);
                            }))
                            .on_action(cx.listener(|this, _: &focus::tab_strip::First, _, cx| {
                                this.set_tab(0, cx);
                            }))
                            .on_action(cx.listener(|this, _: &focus::tab_strip::Last, _, cx| {
                                this.set_tab(usize::MAX, cx);
                            }))
                            .on_action(cx.listener(
                                |this, _: &focus::tab_strip::Activate, window, _cx| {
                                    focus::focus_first_in(&this.content_focus, window);
                                },
                            ))
                            .child(
                                TabBar::new("theme-edit-tabs")
                                    .cursor_pointer()
                                    .selected_index(self.active_tab)
                                    .on_click(cx.listener(|view, index, _, cx| {
                                        view.set_tab(*index, cx);
                                    }))
                                    .children(
                                        tabs.iter().map(|tab| Tab::new().label(tab.as_str())),
                                    ),
                            ),
                    ),
            )
            .children(
                self.error_message
                    .as_ref()
                    .map(|error| error_message(error.clone(), cx)),
            )
            .child(
                // Tab content area with scrolling
                div()
                    .id("tab-content")
                    .track_focus(&self.content_focus)
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .pt_4()
                    .pb_8()
                    .child(self.render_tab_content(window, cx)),
            )
    }
}
