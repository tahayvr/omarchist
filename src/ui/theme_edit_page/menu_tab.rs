//! Menu tab for theme editing (Walker menu settings)
//!
//! Provides UI for editing Walker menu colors:
//! - Background color
//! - Base color
//! - Border color
//! - Foreground color
//! - Text color
//! - Selected text color

use crate::system::theme_management::{save_theme_data, update_walker_css};
use crate::types::themes::{EditingTheme, WalkerConfig};
use crate::ui::theme_edit_page::shared::{form_section, help_text, tab_container};
use gpui::*;
use gpui_component::{
    Colorize,
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
    h_flex,
};

/// Menu tab content for editing Walker menu colors
pub struct MenuTab {
    theme_name: String,
    theme_data: EditingTheme,
    background_picker: Entity<ColorPickerState>,
    base_picker: Entity<ColorPickerState>,
    border_picker: Entity<ColorPickerState>,
    foreground_picker: Entity<ColorPickerState>,
    text_picker: Entity<ColorPickerState>,
    selected_text_picker: Entity<ColorPickerState>,
    is_saving: bool,
    error_message: Option<String>,
}

impl MenuTab {
    /// Create a new MenuTab instance
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Get current walker config or use defaults
        let walker_config = theme_data.apps.walker.as_ref().cloned().unwrap_or_default();

        // Create color picker states with current values
        let background_color =
            Self::hex_to_hsla(&walker_config.background).unwrap_or(gpui::rgb(0x0F0F19).into());
        let base_color =
            Self::hex_to_hsla(&walker_config.base).unwrap_or(gpui::rgb(0x0F0F19).into());
        let border_color =
            Self::hex_to_hsla(&walker_config.border).unwrap_or(gpui::rgb(0x33A1FF).into());
        let foreground_color =
            Self::hex_to_hsla(&walker_config.foreground).unwrap_or(gpui::rgb(0xEDEDFE).into());
        let text_color =
            Self::hex_to_hsla(&walker_config.text).unwrap_or(gpui::rgb(0xEDEDFE).into());
        let selected_text_color =
            Self::hex_to_hsla(&walker_config.selected_text).unwrap_or(gpui::rgb(0xFF66F6).into());

        let background_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(background_color));
        let base_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(base_color));
        let border_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(border_color));
        let foreground_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(foreground_color));
        let text_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(text_color));
        let selected_text_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(selected_text_color));

        let tab = Self {
            theme_name,
            theme_data,
            background_picker,
            base_picker,
            border_picker,
            foreground_picker,
            text_picker,
            selected_text_picker,
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
                    this.update_walker_config(|config| {
                        config.background = hex;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Subscribe to base color picker changes
        cx.subscribe_in(
            &tab.base_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_walker_config(|config| {
                        config.base = hex;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Subscribe to border color picker changes
        cx.subscribe_in(
            &tab.border_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_walker_config(|config| {
                        config.border = hex;
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
                    this.update_walker_config(|config| {
                        config.foreground = hex;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Subscribe to text color picker changes
        cx.subscribe_in(
            &tab.text_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_walker_config(|config| {
                        config.text = hex;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Subscribe to selected text color picker changes
        cx.subscribe_in(
            &tab.selected_text_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_walker_config(|config| {
                        config.selected_text = hex;
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

    /// Update the walker config within theme_data
    fn update_walker_config<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut WalkerConfig),
    {
        let mut config = self.theme_data.apps.walker.clone().unwrap_or_default();
        updater(&mut config);
        self.theme_data.apps.walker = Some(config);
    }

    /// Get the current theme data
    pub fn theme_data(&self) -> &EditingTheme {
        &self.theme_data
    }

    /// Save the theme data and update walker.css
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
                // Also update the walker.css file
                if let Some(ref walker_config) = self.theme_data.apps.walker {
                    if let Err(e) = update_walker_css(&self.theme_name, walker_config) {
                        self.error_message = Some(format!("Failed to update walker.css: {}", e));
                    }
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

impl Render for MenuTab {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        tab_container()
            .child(help_text("Changes auto-save."))
            .child(
                // First row of color pickers - wraps on narrow screens
                h_flex()
                    .gap_8()
                    .flex_wrap()
                    .child(
                        form_section()
                            .child(ColorPicker::new(&self.background_picker).label("Background")),
                    )
                    .child(form_section().child(ColorPicker::new(&self.base_picker).label("Base")))
                    .child(
                        form_section().child(ColorPicker::new(&self.border_picker).label("Border")),
                    ),
            )
            .child(
                // Second row of color pickers - wraps on narrow screens
                h_flex()
                    .gap_8()
                    .flex_wrap()
                    .child(
                        form_section()
                            .child(ColorPicker::new(&self.foreground_picker).label("Foreground")),
                    )
                    .child(form_section().child(ColorPicker::new(&self.text_picker).label("Text")))
                    .child(form_section().child(
                        ColorPicker::new(&self.selected_text_picker).label("Selected Text"),
                    )),
            )
    }
}
