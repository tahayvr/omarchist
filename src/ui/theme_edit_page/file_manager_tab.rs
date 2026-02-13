//! File Manager tab for theme editing
//!
//! Provides UI for editing the file manager icon theme:
//! - Yaru color selection via radio buttons

use crate::system::theme_management::{save_theme_data, update_icons_theme};
use crate::types::themes::EditingTheme;
use crate::ui::theme_edit_page::shared::{form_section, help_text, tab_container};
use gpui::*;
use gpui_component::{h_flex, radio::Radio, v_flex};

/// Available Yaru icon theme colors
const YARU_COLORS: &[&str] = &[
    "Yaru-red",
    "Yaru-blue",
    "Yaru-olive",
    "Yaru-yellow",
    "Yaru-purple",
    "Yaru-magenta",
    "Yaru-sage",
];

/// File Manager tab content for editing icon theme
pub struct FileManagerTab {
    theme_name: String,
    theme_data: EditingTheme,
    selected_color: String,
    is_saving: bool,
    error_message: Option<String>,
}

impl FileManagerTab {
    /// Create a new FileManagerTab instance
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Self {
        // Get current icon theme or use default
        let selected_color = Self::get_current_icon_theme(&theme_data);

        Self {
            theme_name,
            theme_data,
            selected_color,
            is_saving: false,
            error_message: None,
        }
    }

    /// Extract the current icon theme from theme_data
    fn get_current_icon_theme(theme_data: &EditingTheme) -> String {
        theme_data
            .apps
            .icons
            .as_ref()
            .and_then(|icons| icons.get("theme_name"))
            .and_then(|name| name.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Yaru-red".to_string())
    }

    /// Update the icon theme in theme_data
    fn update_icon_theme(&mut self, color: String) {
        self.selected_color = color.clone();

        // Create or update the icons config
        let icons_config = serde_json::json!({
            "theme_name": color
        });

        self.theme_data.apps.icons = Some(icons_config);
    }

    /// Save the theme data
    fn save(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_saving {
            return;
        }

        // Validate theme name
        if self.theme_name.is_empty() {
            self.error_message = Some("Theme name cannot be empty".to_string());
            cx.notify();
            return;
        }

        self.is_saving = true;
        self.error_message = None;
        cx.notify();

        // Save theme data
        match save_theme_data(&self.theme_name, &self.theme_data) {
            Ok(()) => {
                // Also update the icons.theme file
                if let Err(e) = update_icons_theme(&self.theme_name, &self.selected_color) {
                    self.error_message = Some(format!("Failed to update icons.theme: {}", e));
                }
                self.is_saving = false;
            }
            Err(e) => {
                self.is_saving = false;
                self.error_message = Some(e);
            }
        }

        cx.notify();
    }

    /// Create radio button element for a color
    fn create_color_radio(&self, color: &'static str, cx: &mut Context<Self>) -> impl IntoElement {
        let color_value: SharedString = color.into();
        let is_selected = self.selected_color == color;

        h_flex().gap_2().items_center().child(
            Radio::new(color_value)
                .label(color)
                .checked(is_selected)
                .on_click(cx.listener(move |this, _checked: &bool, window, cx| {
                    this.update_icon_theme(color.to_string());
                    this.save(window, cx);
                })),
        )
    }
}

impl Render for FileManagerTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut container = v_flex().gap_3();

        for &color in YARU_COLORS {
            container = container.child(self.create_color_radio(color, cx));
        }

        tab_container()
            .child(help_text("Changes auto-save."))
            .child(form_section().child(container))
    }
}
