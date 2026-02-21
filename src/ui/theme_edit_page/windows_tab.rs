//! Windows tab for theme editing (Hyprland window settings)
//!
//! Provides UI for editing Hyprland window border colors:
//! - Active border color
//! - Inactive border color

use crate::system::themes::theme_management::{save_theme_data, update_hyprland_conf};
use crate::types::themes::{EditingTheme, HyprlandConfig};
use crate::ui::theme_edit_page::shared::{
    color_picker_with_clipboard, form_section, help_text, tab_container,
};
use gpui::*;
use gpui_component::{
    Colorize,
    color_picker::{ColorPickerEvent, ColorPickerState},
    h_flex,
};

/// Windows tab content for editing Hyprland window settings
pub struct WindowsTab {
    theme_name: String,
    theme_data: EditingTheme,
    active_border_picker: Entity<ColorPickerState>,
    inactive_border_picker: Entity<ColorPickerState>,
    is_saving: bool,
    error_message: Option<String>,
}

impl WindowsTab {
    /// Create a new WindowsTab instance
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Get current hyprland config or use defaults
        let hyprland_config = theme_data
            .apps
            .hyprland
            .as_ref()
            .cloned()
            .unwrap_or_default();

        // Create color picker states with current values
        // Hyprland colors are without # prefix in the config
        let active_border_hex = format!("#{}", hyprland_config.active_border);
        let inactive_border_hex = format!("#{}", hyprland_config.inactive_border);

        let active_border_color =
            Self::hex_to_hsla(&active_border_hex).unwrap_or(gpui::rgb(0x6e6e92).into());
        let inactive_border_color =
            Self::hex_to_hsla(&inactive_border_hex).unwrap_or(gpui::rgb(0x5C5C5E).into());

        let active_border_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(active_border_color));

        let inactive_border_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(inactive_border_color));

        let tab = Self {
            theme_name,
            theme_data,
            active_border_picker,
            inactive_border_picker,
            is_saving: false,
            error_message: None,
        };

        // Subscribe to active border color picker changes
        cx.subscribe_in(
            &tab.active_border_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    // Remove # prefix for Hyprland format
                    let hex = color.to_hex().trim_start_matches('#').to_string();
                    this.update_hyprland_config(|config| {
                        config.active_border = hex;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Subscribe to inactive border color picker changes
        cx.subscribe_in(
            &tab.inactive_border_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    // Remove # prefix for Hyprland format
                    let hex = color.to_hex().trim_start_matches('#').to_string();
                    this.update_hyprland_config(|config| {
                        config.inactive_border = hex;
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

    /// Update the hyprland config within theme_data
    fn update_hyprland_config<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut HyprlandConfig),
    {
        let mut config = self.theme_data.apps.hyprland.clone().unwrap_or_default();
        updater(&mut config);
        self.theme_data.apps.hyprland = Some(config);
    }

    /// Get the current theme data
    pub fn theme_data(&self) -> &EditingTheme {
        &self.theme_data
    }

    /// Save the theme data and update hyprland.conf
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
                // Also update the hyprland.conf file
                if let Some(ref hyprland_config) = self.theme_data.apps.hyprland
                    && let Err(e) = update_hyprland_conf(&self.theme_name, hyprland_config)
                {
                    self.error_message = Some(format!("Failed to update hyprland.conf: {}", e));
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

impl Render for WindowsTab {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        tab_container()
            .child(help_text("Colors for Hyprland windows and tiles."))
            .child(
                // Border colors row
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(form_section().child(color_picker_with_clipboard(
                        "active-border",
                        "Active Border",
                        &self.active_border_picker,
                    )))
                    .child(form_section().child(color_picker_with_clipboard(
                        "inactive-border",
                        "Inactive Border",
                        &self.inactive_border_picker,
                    ))),
            )
    }
}
