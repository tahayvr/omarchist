use crate::system::theme_management::{load_theme_for_editing, save_theme_data};
use crate::types::themes::{EditingTheme, ThemeEditTab};
use crate::ui::theme_edit_page::general_tab::GeneralTab;
use crate::ui::theme_edit_page::shared::error_message;
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
    original_theme_name: String,
    theme_data: EditingTheme,
    active_tab: usize,
    is_saving: bool,
    error_message: Option<String>,
    general_tab: Entity<GeneralTab>,
}

impl ThemeEditPage {
    pub fn new(theme_name: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Load theme data
        let theme_data = match load_theme_for_editing(&theme_name) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to load theme '{}': {}", theme_name, e);
                EditingTheme::default()
            }
        };

        // Create General tab instance
        let general_tab = cx.new(|cx| GeneralTab::new(theme_data.clone(), window, cx));

        Self {
            theme_name: theme_name.clone(),
            original_theme_name: theme_name,
            theme_data,
            active_tab: 0,
            is_saving: false,
            error_message: None,
            general_tab,
        }
    }

    pub fn theme_name(&self) -> &str {
        &self.theme_name
    }

    fn save_theme(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_saving {
            return;
        }

        self.is_saving = true;
        self.error_message = None;
        cx.notify();

        // Check if theme name has changed (rename needed)
        if self.theme_name != self.original_theme_name {
            // Handle rename first, then save
            // This will be implemented when we add the rename functionality
            // For now, just save with current name
        }

        // Save theme data
        match save_theme_data(&self.theme_name, &self.theme_data) {
            Ok(()) => {
                self.is_saving = false;
                // Show success notification
                // window.push_notification("Theme saved successfully!", cx);
            }
            Err(e) => {
                let error_clone = e.clone();
                self.is_saving = false;
                self.error_message = Some(e);
                eprintln!("Failed to save theme: {}", error_clone);
            }
        }

        cx.notify();
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
                // TODO: Implement Waybar tab
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(div().text_lg().child("Waybar Settings"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(gpui::rgb(0x888888))
                            .child("Waybar configuration will be implemented here"),
                    )
                    .into_any_element()
            }
            ThemeEditTab::Windows => {
                // TODO: Implement Windows (Hyprland) tab
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(div().text_lg().child("Window Settings (Hyprland)"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(gpui::rgb(0x888888))
                            .child("Hyprland window configuration will be implemented here"),
                    )
                    .into_any_element()
            }
            ThemeEditTab::Menu => {
                // TODO: Implement Menu (Walker) tab
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(div().text_lg().child("Menu Settings (Walker)"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(gpui::rgb(0x888888))
                            .child("Walker menu configuration will be implemented here"),
                    )
                    .into_any_element()
            }
            ThemeEditTab::Terminal => {
                // TODO: Implement Terminal (Alacritty, Ghostty, Kitty) tab
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(
                        div()
                            .text_lg()
                            .child("Terminal Settings")
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(gpui::rgb(0x888888))
                            .child("Terminal (Alacritty, Ghostty, Kitty) configuration will be implemented here")
                    )
                    .into_any_element()
            }
            ThemeEditTab::Browser => {
                // TODO: Implement Browser (Chromium) tab
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(div().text_lg().child("Browser Settings (Chromium)"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(gpui::rgb(0x888888))
                            .child("Chromium theme configuration will be implemented here"),
                    )
                    .into_any_element()
            }
            ThemeEditTab::FileManager => {
                // TODO: Implement File Manager (Icons) tab
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(div().text_lg().child("File Manager Icon Theme"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(gpui::rgb(0x888888))
                            .child("Icon theme selection will be implemented here"),
                    )
                    .into_any_element()
            }
            ThemeEditTab::LockScreen => {
                // TODO: Implement Lock Screen (Hyprlock) tab
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(div().text_lg().child("Lock Screen Settings (Hyprlock)"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(gpui::rgb(0x888888))
                            .child("Hyprlock lock screen configuration will be implemented here"),
                    )
                    .into_any_element()
            }
            ThemeEditTab::Notification => {
                // TODO: Implement Notification (Mako) tab
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(div().text_lg().child("Notification Settings (Mako)"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(gpui::rgb(0x888888))
                            .child("Mako notification configuration will be implemented here"),
                    )
                    .into_any_element()
            }
            ThemeEditTab::Editor => {
                // TODO: Implement Editor (Neovim, VSCode:) tab
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(div().text_lg().child("Editor Settings"))
                    .child(
                        div().text_sm().text_color(gpui::rgb(0x888888)).child(
                            "Neovim and VSCode: theme configuration will be implemented here",
                        ),
                    )
                    .into_any_element()
            }
            ThemeEditTab::Btop => {
                // TODO: Implement Btop tab
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(div().text_lg().child("Btop Settings"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(gpui::rgb(0x888888))
                            .child("Btop activity monitor configuration will be implemented here"),
                    )
                    .into_any_element()
            }
            ThemeEditTab::Swayosd => {
                // TODO: Implement SwayOSD tab
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(div().text_lg().child("SwayOSD Settings"))
                    .child(
                        div().text_sm().text_color(gpui::rgb(0x888888)).child(
                            "SwayOSD on-screen display configuration will be implemented here",
                        ),
                    )
                    .into_any_element()
            }
            ThemeEditTab::Backgrounds => {
                // TODO: Implement Backgrounds tab
                v_flex()
                    .p_4()
                    .gap_4()
                    .child(div().text_lg().child("Background Images"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(gpui::rgb(0x888888))
                            .child("Background image management will be implemented here"),
                    )
                    .into_any_element()
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
            .child(
                // Header
                h_flex()
                    .p_4()
                    .gap_4()
                    .items_center()
                    .justify_start()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        Button::new("back-btn")
                            .label("Back")
                            .primary()
                            .outline()
                            .compact()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.navigate_back(window, cx);
                            })),
                    ),
            )
            .children(
                self.error_message
                    .as_ref()
                    .map(|error| error_message(error.clone())),
            )
            .child(
                // Tabs
                TabBar::new("theme-edit-tabs")
                    .selected_index(self.active_tab)
                    .on_click(cx.listener(|view, index, _, cx| {
                        view.active_tab = *index;
                        cx.notify();
                    }))
                    .children(tabs.iter().map(|tab| Tab::new().label(tab.as_str()))),
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
