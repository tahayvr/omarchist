//! Lock Screen (Hyprlock) tab for theme editing
//!
//! Provides UI for editing Hyprlock lock screen colors using ColorPicker components:
//! - color: Main background color
//! - inner_color: Inner color
//! - outer_color: Outer/ring color
//! - font_color: Font/text color
//! - check_color: Check mark color

use crate::system::theme_management::{save_theme_data, update_hyprlock_conf};
use crate::types::themes::{EditingTheme, HyprlockConfig};
use crate::ui::theme_edit_page::shared::{form_section, help_text, tab_container};
use gpui::*;
use gpui_component::{
    Colorize,
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
    h_flex,
};

/// Lock Screen tab content for editing hyprlock colors
pub struct LockScreenTab {
    theme_name: String,
    theme_data: EditingTheme,
    color_picker: Entity<ColorPickerState>,
    inner_color_picker: Entity<ColorPickerState>,
    outer_color_picker: Entity<ColorPickerState>,
    font_color_picker: Entity<ColorPickerState>,
    check_color_picker: Entity<ColorPickerState>,
    is_saving: bool,
    error_message: Option<String>,
}

impl LockScreenTab {
    /// Create a new LockScreenTab instance
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Get current hyprlock config or use defaults
        let hyprlock_config = theme_data
            .apps
            .hyprlock
            .as_ref()
            .cloned()
            .unwrap_or_default();

        // Create color picker states with current values
        // Note: hyprlock uses rgb format (0f0f19), not hex (#0f0f19)
        // But ColorPicker expects hex, so we add # prefix for display
        let color = Self::rgb_to_hsla(&hyprlock_config.color).unwrap_or(gpui::rgb(0x0F0F19).into());
        let inner_color =
            Self::rgb_to_hsla(&hyprlock_config.inner_color).unwrap_or(gpui::rgb(0x0F0F19).into());
        let outer_color =
            Self::rgb_to_hsla(&hyprlock_config.outer_color).unwrap_or(gpui::rgb(0x33A0FF).into());
        let font_color =
            Self::rgb_to_hsla(&hyprlock_config.font_color).unwrap_or(gpui::rgb(0xFF66F5).into());
        let check_color =
            Self::rgb_to_hsla(&hyprlock_config.check_color).unwrap_or(gpui::rgb(0xFFEA00).into());

        let color_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(color));

        let inner_color_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(inner_color));

        let outer_color_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(outer_color));

        let font_color_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(font_color));

        let check_color_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(check_color));

        let tab = Self {
            theme_name,
            theme_data,
            color_picker,
            inner_color_picker,
            outer_color_picker,
            font_color_picker,
            check_color_picker,
            is_saving: false,
            error_message: None,
        };

        // Subscribe to color picker changes
        cx.subscribe_in(
            &tab.color_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let rgb = Self::hsla_to_rgb(color);
                    this.update_hyprlock_config(|config| {
                        config.color = rgb;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &tab.inner_color_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let rgb = Self::hsla_to_rgb(color);
                    this.update_hyprlock_config(|config| {
                        config.inner_color = rgb;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &tab.outer_color_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let rgb = Self::hsla_to_rgb(color);
                    this.update_hyprlock_config(|config| {
                        config.outer_color = rgb;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &tab.font_color_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let rgb = Self::hsla_to_rgb(color);
                    this.update_hyprlock_config(|config| {
                        config.font_color = rgb;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &tab.check_color_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let rgb = Self::hsla_to_rgb(color);
                    this.update_hyprlock_config(|config| {
                        config.check_color = rgb;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        tab
    }

    /// Convert RGB format (0f0f19) to Hsla
    /// hyprlock uses rgb format without #, so we need to add it for ColorPicker
    fn rgb_to_hsla(rgb: &str) -> Option<Hsla> {
        let hex = format!("#{}", rgb.trim());
        if hex.len() != 7 {
            return None;
        }

        let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
        let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
        let b = u8::from_str_radix(&hex[5..7], 16).ok()?;

        Some(gpui::rgb(u32::from_be_bytes([0, r, g, b])).into())
    }

    /// Convert Hsla to RGB format (0f0f19) for hyprlock
    fn hsla_to_rgb(color: &Hsla) -> String {
        let hex = color.to_hex();
        // Remove the # prefix to get rgb format
        hex.trim_start_matches('#').to_string()
    }

    /// Update the hyprlock config within theme_data
    fn update_hyprlock_config<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut HyprlockConfig),
    {
        let mut config = self.theme_data.apps.hyprlock.clone().unwrap_or_default();
        updater(&mut config);
        self.theme_data.apps.hyprlock = Some(config);
    }

    /// Get the current theme data
    pub fn theme_data(&self) -> &EditingTheme {
        &self.theme_data
    }

    /// Save the theme data and update hyprlock.conf
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
                // Also update the hyprlock.conf file
                if let Some(ref hyprlock_config) = self.theme_data.apps.hyprlock {
                    if let Err(e) = update_hyprlock_conf(&self.theme_name, hyprlock_config) {
                        self.error_message = Some(format!("Failed to update hyprlock.conf: {}", e));
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

impl Render for LockScreenTab {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        tab_container()
            .child(help_text("Colors for the lock screen (Hyprlock)."))
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(
                        form_section()
                            .child(ColorPicker::new(&self.color_picker).label("Background Color")),
                    )
                    .child(
                        form_section()
                            .child(ColorPicker::new(&self.inner_color_picker).label("Inner Color")),
                    )
                    .child(
                        form_section()
                            .child(ColorPicker::new(&self.outer_color_picker).label("Outer Color")),
                    ),
            )
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(
                        form_section()
                            .child(ColorPicker::new(&self.font_color_picker).label("Font Color")),
                    )
                    .child(
                        form_section()
                            .child(ColorPicker::new(&self.check_color_picker).label("Check Color")),
                    ),
            )
    }
}
