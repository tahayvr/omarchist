use crate::shell::theme_sh_commands::apply_theme;
use crate::system::themes::theme_file_ops::is_system_theme;
use crate::system::themes::theme_management::load_theme_for_editing;
use crate::types::themes::{EditingTheme, ThemeEditTab};
use crate::ui::theme_edit_page::backgrounds_tab::BackgroundsTab;
use crate::ui::theme_edit_page::browser_tab::BrowserTab;
use crate::ui::theme_edit_page::btop_tab::BtopTab;
use crate::ui::theme_edit_page::colors_tab::ColorsTab;
use crate::ui::theme_edit_page::editor_tab::EditorTab;
use crate::ui::theme_edit_page::file_manager_tab::FileManagerTab;
use crate::ui::theme_edit_page::general_tab::GeneralTab;
use crate::ui::theme_edit_page::lockscreen_tab::LockScreenTab;
use crate::ui::theme_edit_page::shared::error_message;
use crate::ui::theme_edit_page::windows_tab::WindowsTab;
use gpui::*;
use gpui_component::{
    ActiveTheme,
    button::Button,
    h_flex,
    tab::{Tab, TabBar},
    v_flex,
};

const KEY_CONTEXT: &str = "ThemeEditPage";

#[derive(Clone, PartialEq, Action)]
#[action(no_json)]
pub struct NavigateToThemes;

use std::cell::RefCell;

thread_local! {
    pub static PENDING_NAVIGATE_TO_THEMES: RefCell<bool> = const { RefCell::new(false) };
}

#[derive(Clone, PartialEq, Action)]
#[action(no_json)]
pub struct SaveTheme;

pub struct ThemeEditPage {
    theme_name: String,
    active_tab: usize,
    tab_count: usize,
    error_message: Option<String>,
    general_tab: Entity<GeneralTab>,
    colors_tab: Entity<ColorsTab>,
    windows_tab: Entity<WindowsTab>,
    browser_tab: Entity<BrowserTab>,
    file_manager_tab: Entity<FileManagerTab>,
    lockscreen_tab: Entity<LockScreenTab>,
    editor_tab: Entity<EditorTab>,
    btop_tab: Entity<BtopTab>,
    backgrounds_tab: Entity<BackgroundsTab>,
    pub focus_handle: FocusHandle,
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

        // Create General tab instance
        let general_tab =
            cx.new(|cx| GeneralTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Colors tab instance
        let colors_tab =
            cx.new(|cx| ColorsTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Windows tab instance
        let windows_tab =
            cx.new(|cx| WindowsTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Browser tab instance
        let browser_tab =
            cx.new(|cx| BrowserTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create File Manager tab instance
        let file_manager_tab =
            cx.new(|cx| FileManagerTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Lock Screen tab instance
        let lockscreen_tab =
            cx.new(|cx| LockScreenTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Editor tab instance
        let editor_tab =
            cx.new(|cx| EditorTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Btop tab instance
        let btop_tab =
            cx.new(|cx| BtopTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Backgrounds tab instance
        let backgrounds_tab =
            cx.new(|cx| BackgroundsTab::new(theme_name.clone(), is_system, window, cx));

        let tab_count = ThemeEditTab::all().len();

        // Create focus handle and request focus immediately
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window);

        Self {
            theme_name,
            active_tab: 0,
            tab_count,
            error_message: None,
            general_tab,
            colors_tab,
            windows_tab,
            browser_tab,
            file_manager_tab,
            lockscreen_tab,
            editor_tab,
            btop_tab,
            backgrounds_tab,
            focus_handle,
        }
    }

    pub fn theme_name(&self) -> &str {
        &self.theme_name
    }

    fn navigate_back(&self, _window: &mut Window, _cx: &mut Context<Self>) {
        PENDING_NAVIGATE_TO_THEMES.with(|flag| {
            *flag.borrow_mut() = true;
        });
        // Also trigger themes refresh so new themes appear
        crate::ui::dialogs::create_theme_dialog::PENDING_REFRESH_THEMES.with(|flag| {
            *flag.borrow_mut() = true;
        });
    }

    fn next_tab(&mut self, cx: &mut Context<Self>) {
        if self.active_tab < self.tab_count.saturating_sub(1) {
            self.active_tab += 1;
            cx.notify();
        }
    }

    fn prev_tab(&mut self, cx: &mut Context<Self>) {
        if self.active_tab > 0 {
            self.active_tab -= 1;
            cx.notify();
        }
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
            ThemeEditTab::Windows => {
                // Use the WindowsTab entity
                self.windows_tab.clone().into_any_element()
            }
            ThemeEditTab::Browser => {
                // Use the BrowserTab entity
                self.browser_tab.clone().into_any_element()
            }
            ThemeEditTab::FileManager => {
                // Use the FileManagerTab entity
                self.file_manager_tab.clone().into_any_element()
            }
            ThemeEditTab::LockScreen => {
                // Use the LockScreenTab entity
                self.lockscreen_tab.clone().into_any_element()
            }
            ThemeEditTab::Editor => {
                // Use the EditorTab entity
                self.editor_tab.clone().into_any_element()
            }
            ThemeEditTab::Btop => {
                // Use the BtopTab entity
                self.btop_tab.clone().into_any_element()
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
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::ThemeEditNextTab, _window, cx| {
                    this.next_tab(cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::ThemeEditPrevTab, _window, cx| {
                    this.prev_tab(cx);
                },
            ))
            .on_action(cx.listener(
                |this, _: &crate::ui::menu::app_menu::NavigateBack, window, cx| {
                    this.navigate_back(window, cx);
                },
            ))
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
                            .on_click(cx.listener(|this, _, _window, _cx| {
                                let dir = this.theme_name.clone();
                                smol::spawn(async move {
                                    if let Err(e) = apply_theme(dir).await {
                                        eprintln!("Failed to apply theme: {}", e);
                                    }
                                })
                                .detach();
                            })),
                    )
                    .child(
                        div().flex_1().min_w_0().child(
                            TabBar::new("theme-edit-tabs")
                                .cursor_pointer()
                                .selected_index(self.active_tab)
                                .on_click(cx.listener(|view, index, _, cx| {
                                    view.active_tab = *index;
                                    cx.notify();
                                }))
                                .children(tabs.iter().map(|tab| Tab::new().label(tab.as_str()))),
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
                    .flex_1()
                    .overflow_y_scroll()
                    .pt_4()
                    .pb_8()
                    .child(self.render_tab_content(window, cx)),
            )
    }
}
