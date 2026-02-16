//! Browser tab for theme editing
//!
//! Provides UI for editing Chromium browser theme color:
//! - Theme color (RGB/Hex)

use crate::shell::theme_sh_commands::execute_bash_command;
use crate::system::themes::theme_management::{save_theme_data, update_chromium_config};
use crate::types::themes::{BrowserConfig, EditingTheme};
use crate::ui::theme_edit_page::shared::{
    color_picker_with_clipboard, form_section, help_text, tab_container,
};
use gpui::*;
use gpui_component::{
    Colorize,
    button::Button,
    color_picker::{ColorPickerEvent, ColorPickerState},
    h_flex,
};

/// Browser tab content for editing Chromium theme color
pub struct BrowserTab {
    theme_name: String,
    theme_data: EditingTheme,
    theme_color_picker: Entity<ColorPickerState>,
    is_saving: bool,
    error_message: Option<String>,
}

impl BrowserTab {
    /// Create a new BrowserTab instance
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Get current browser config or use defaults
        let browser_config = theme_data
            .apps
            .chromium
            .as_ref()
            .cloned()
            .unwrap_or_default();

        // Create color picker state with current theme color
        let theme_color =
            Self::hex_to_hsla(&browser_config.theme_color).unwrap_or(gpui::rgb(0x0F0F19).into());

        let theme_color_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(theme_color));

        let tab = Self {
            theme_name,
            theme_data,
            theme_color_picker,
            is_saving: false,
            error_message: None,
        };

        // Subscribe to theme color picker changes
        cx.subscribe_in(
            &tab.theme_color_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_browser_config(|config| {
                        config.theme_color = hex;
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

    /// Update the browser config within theme_data
    fn update_browser_config<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut BrowserConfig),
    {
        let mut config = self.theme_data.apps.chromium.clone().unwrap_or_default();
        updater(&mut config);
        self.theme_data.apps.chromium = Some(config);
    }

    /// Save the theme data and update chromium config
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
                // Also update the chromium config
                if let Some(ref browser_config) = self.theme_data.apps.chromium
                    && let Err(e) = update_chromium_config(&self.theme_name, browser_config)
                {
                    self.error_message = Some(format!("Failed to update chromium config: {}", e));
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

    /// Launch Chromium browser
    fn launch_browser(&self) {
        let command = "uwsm app -- chromium".to_string();
        if let Err(e) = execute_bash_command(command) {
            eprintln!("Failed to launch chromium: {}", e);
        }
    }
}

impl Render for BrowserTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        tab_container()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(help_text("Color for the Chromium browser."))
                    .child(
                        Button::new("launch-chromium")
                            .label("Chromium")
                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                this.launch_browser();
                            })),
                    ),
            )
            .child(form_section().child(color_picker_with_clipboard(
                "browser-theme",
                "Theme Color",
                &self.theme_color_picker,
            )))
    }
}
