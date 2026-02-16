//! Waybar tab for theme editing
//!
//! Provides UI for editing Waybar colors using ColorPicker components:
//! - Background color
//! - Foreground color

use crate::system::themes::theme_management::{save_theme_data, update_waybar_css};
use crate::types::themes::{EditingTheme, WaybarConfig};
use crate::ui::theme_edit_page::shared::{
    color_picker_with_clipboard, form_section, help_text, tab_container,
};
use gpui::*;
use gpui_component::{
    Colorize,
    color_picker::{ColorPickerEvent, ColorPickerState},
    h_flex,
};

/// Waybar tab content for editing waybar colors
pub struct WaybarTab {
    theme_name: String,
    theme_data: EditingTheme,
    background_picker: Entity<ColorPickerState>,
    foreground_picker: Entity<ColorPickerState>,
    is_saving: bool,
    error_message: Option<String>,
}

impl WaybarTab {
    /// Create a new WaybarTab instance
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Get current waybar config or use defaults
        let waybar_config = theme_data.apps.waybar.as_ref().cloned().unwrap_or_default();

        // Create color picker states with current values
        let background_color =
            Self::hex_to_hsla(&waybar_config.background).unwrap_or(gpui::rgb(0x0F0F19).into());
        let foreground_color =
            Self::hex_to_hsla(&waybar_config.foreground).unwrap_or(gpui::rgb(0xEDEDFE).into());

        let background_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(background_color));

        let foreground_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(foreground_color));

        let tab = Self {
            theme_name,
            theme_data,
            background_picker,
            foreground_picker,
            is_saving: false,
            error_message: None,
        };

        // Subscribe to background color picker changes
        cx.subscribe_in(
            &tab.background_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_waybar_config(|config| {
                        config.background = hex;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Subscribe to foreground color picker changes
        cx.subscribe_in(
            &tab.foreground_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_waybar_config(|config| {
                        config.foreground = hex;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        tab
    }

    /// Convert hex color string (#RRGGBB) to Hsla
    fn hex_to_hsla(hex: &str) -> Option<Hsla> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(gpui::rgb(u32::from_be_bytes([0, r, g, b])).into())
    }

    /// Update the waybar config within theme_data
    fn update_waybar_config<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut WaybarConfig),
    {
        let mut config = self.theme_data.apps.waybar.clone().unwrap_or_default();
        updater(&mut config);
        self.theme_data.apps.waybar = Some(config);
    }

    /// Get the current theme data
    pub fn theme_data(&self) -> &EditingTheme {
        &self.theme_data
    }

    /// Save the theme data and update waybar.css
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
                // Also update the waybar.css file
                if let Some(ref waybar_config) = self.theme_data.apps.waybar
                    && let Err(e) = update_waybar_css(&self.theme_name, waybar_config)
                {
                    self.error_message = Some(format!("Failed to update waybar.css: {}", e));
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
}

impl Render for WaybarTab {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        tab_container()
            .child(help_text("Colors for Waybar."))
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(form_section().child(color_picker_with_clipboard(
                        "waybar-bg",
                        "Background",
                        &self.background_picker,
                    )))
                    .child(form_section().child(color_picker_with_clipboard(
                        "waybar-fg",
                        "Foreground",
                        &self.foreground_picker,
                    ))),
            )
    }
}
