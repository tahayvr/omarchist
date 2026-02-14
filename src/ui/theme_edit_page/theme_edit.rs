use crate::system::theme_file_ops::is_system_theme;
use crate::system::theme_management::load_theme_for_editing;
use crate::types::themes::{EditingTheme, ThemeEditTab};
use crate::ui::theme_edit_page::backgrounds_tab::BackgroundsTab;
use crate::ui::theme_edit_page::browser_tab::BrowserTab;
use crate::ui::theme_edit_page::btop_tab::BtopTab;
use crate::ui::theme_edit_page::editor_tab::EditorTab;
use crate::ui::theme_edit_page::file_manager_tab::FileManagerTab;
use crate::ui::theme_edit_page::general_tab::GeneralTab;
use crate::ui::theme_edit_page::lockscreen_tab::LockScreenTab;
use crate::ui::theme_edit_page::menu_tab::MenuTab;
use crate::ui::theme_edit_page::notification_tab::NotificationTab;
use crate::ui::theme_edit_page::shared::error_message;
use crate::ui::theme_edit_page::swayosd_tab::SwayosdTab;
use crate::ui::theme_edit_page::terminal_tab::TerminalTab;
use crate::ui::theme_edit_page::waybar_tab::WaybarTab;
use crate::ui::theme_edit_page::windows_tab::WindowsTab;
use gpui::*;
use gpui_component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
    h_flex,
    scroll::ScrollableElement,
    tab::{Tab, TabBar},
    v_flex,
};

/// Action to navigate back to themes page
#[derive(Clone, PartialEq, Action)]
#[action(no_json)]
pub struct NavigateToThemes;

use std::cell::RefCell;

thread_local! {
    /// Flag to navigate back to themes page
    pub static PENDING_NAVIGATE_TO_THEMES: RefCell<bool> = RefCell::new(false);
}

/// Action to save the current theme
#[derive(Clone, PartialEq, Action)]
#[action(no_json)]
pub struct SaveTheme;

pub struct ThemeEditPage {
    theme_name: String,
    active_tab: usize,
    error_message: Option<String>,
    general_tab: Entity<GeneralTab>,
    waybar_tab: Entity<WaybarTab>,
    windows_tab: Entity<WindowsTab>,
    menu_tab: Entity<MenuTab>,
    terminal_tab: Entity<TerminalTab>,
    browser_tab: Entity<BrowserTab>,
    file_manager_tab: Entity<FileManagerTab>,
    lockscreen_tab: Entity<LockScreenTab>,
    notification_tab: Entity<NotificationTab>,
    editor_tab: Entity<EditorTab>,
    btop_tab: Entity<BtopTab>,
    swayosd_tab: Entity<SwayosdTab>,
    backgrounds_tab: Entity<BackgroundsTab>,
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

        // Create Waybar tab instance
        let waybar_tab =
            cx.new(|cx| WaybarTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Windows tab instance
        let windows_tab =
            cx.new(|cx| WindowsTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Menu tab instance
        let menu_tab =
            cx.new(|cx| MenuTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Terminal tab instance
        let terminal_tab =
            cx.new(|cx| TerminalTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Browser tab instance
        let browser_tab =
            cx.new(|cx| BrowserTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create File Manager tab instance
        let file_manager_tab =
            cx.new(|cx| FileManagerTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Lock Screen tab instance
        let lockscreen_tab =
            cx.new(|cx| LockScreenTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Notification tab instance
        let notification_tab =
            cx.new(|cx| NotificationTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Editor tab instance
        let editor_tab =
            cx.new(|cx| EditorTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Btop tab instance
        let btop_tab =
            cx.new(|cx| BtopTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create SwayOSD tab instance
        let swayosd_tab =
            cx.new(|cx| SwayosdTab::new(theme_name.clone(), theme_data.clone(), window, cx));

        // Create Backgrounds tab instance
        let backgrounds_tab =
            cx.new(|cx| BackgroundsTab::new(theme_name.clone(), is_system, window, cx));

        Self {
            theme_name,
            active_tab: 0,
            error_message: None,
            general_tab,
            waybar_tab,
            windows_tab,
            menu_tab,
            terminal_tab,
            browser_tab,
            file_manager_tab,
            lockscreen_tab,
            notification_tab,
            editor_tab,
            btop_tab,
            swayosd_tab,
            backgrounds_tab,
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
            ThemeEditTab::Waybar => {
                // Use the WaybarTab entity
                self.waybar_tab.clone().into_any_element()
            }
            ThemeEditTab::Windows => {
                // Use the WindowsTab entity
                self.windows_tab.clone().into_any_element()
            }
            ThemeEditTab::Menu => {
                // Use the MenuTab entity
                self.menu_tab.clone().into_any_element()
            }
            ThemeEditTab::Terminal => {
                // Use the TerminalTab entity
                self.terminal_tab.clone().into_any_element()
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
            ThemeEditTab::Notification => {
                // Use the NotificationTab entity
                self.notification_tab.clone().into_any_element()
            }
            ThemeEditTab::Editor => {
                // Use the EditorTab entity
                self.editor_tab.clone().into_any_element()
            }
            ThemeEditTab::Btop => {
                // Use the BtopTab entity
                self.btop_tab.clone().into_any_element()
            }
            ThemeEditTab::Swayosd => {
                // Use the SwayosdTab entity
                self.swayosd_tab.clone().into_any_element()
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
        // let theme_name = self.theme_name.clone();

        v_flex()
            .id("theme-edit-page")
            .size_full()
            .bg(theme.background)
            .gap_4()
            .child(
                // Back button + Tabs row
                h_flex()
                    .gap_4()
                    .items_center()
                    .child(
                        Button::new("back-btn")
                            .label("Back")
                            .primary()
                            .outline()
                            .compact()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate_back(window, cx);
                            })),
                    )
                    .child(
                        TabBar::new("theme-edit-tabs")
                            .selected_index(self.active_tab)
                            .on_click(cx.listener(|view, index, _, cx| {
                                view.active_tab = *index;
                                cx.notify();
                            }))
                            .children(tabs.iter().map(|tab| Tab::new().label(tab.as_str()))),
                    ),
            )
            .children(
                self.error_message
                    .as_ref()
                    .map(|error| error_message(error.clone())),
            )
            .child(
                // Tab content area
                div()
                    .flex_1()
                    .overflow_y_scrollbar()
                    .child(self.render_tab_content(window, cx)),
            )
    }
}
