use crate::shell::theme_sh_commands::execute_bash_command;
use crate::system::themes::theme_management::{save_theme_data, update_mako_ini};
use crate::types::themes::{EditingTheme, MakoConfig};
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

pub struct NotificationTab {
    theme_name: String,
    theme_data: EditingTheme,
    text_color_picker: Entity<ColorPickerState>,
    border_color_picker: Entity<ColorPickerState>,
    background_color_picker: Entity<ColorPickerState>,
    is_saving: bool,
    error_message: Option<String>,
}

impl NotificationTab {
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Get current mako config or use defaults
        let mako_config = theme_data.apps.mako.as_ref().cloned().unwrap_or_default();

        // Create color picker states with current values
        let text_color =
            Self::hex_to_hsla(&mako_config.text_color).unwrap_or(gpui::rgb(0xEDEDFE).into());
        let border_color =
            Self::hex_to_hsla(&mako_config.border_color).unwrap_or(gpui::rgb(0x00F59B).into());
        let background_color =
            Self::hex_to_hsla(&mako_config.background_color).unwrap_or(gpui::rgb(0x0F0F19).into());

        let text_color_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(text_color));

        let border_color_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(border_color));

        let background_color_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(background_color));

        let tab = Self {
            theme_name,
            theme_data,
            text_color_picker,
            border_color_picker,
            background_color_picker,
            is_saving: false,
            error_message: None,
        };

        // Subscribe to text color picker changes
        cx.subscribe_in(
            &tab.text_color_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_mako_config(|config| {
                        config.text_color = hex;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Subscribe to border color picker changes
        cx.subscribe_in(
            &tab.border_color_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_mako_config(|config| {
                        config.border_color = hex;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Subscribe to background color picker changes
        cx.subscribe_in(
            &tab.background_color_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_mako_config(|config| {
                        config.background_color = hex;
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        tab
    }

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

    fn update_mako_config<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut MakoConfig),
    {
        let mut config = self.theme_data.apps.mako.clone().unwrap_or_default();
        updater(&mut config);
        self.theme_data.apps.mako = Some(config);
    }

    pub fn theme_data(&self) -> &EditingTheme {
        &self.theme_data
    }

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
                // Also update the mako.ini file
                if let Some(ref mako_config) = self.theme_data.apps.mako
                    && let Err(e) = update_mako_ini(&self.theme_name, mako_config)
                {
                    self.error_message = Some(format!("Failed to update mako.ini: {}", e));
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

    fn launch_test_notification(&self) {
        let command =
            r#"notify-send "Test Notification" "This is a test notification""#.to_string();
        if let Err(e) = execute_bash_command(command) {
            eprintln!("Failed to send test notification: {}", e);
        }
    }
}

impl Render for NotificationTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        tab_container()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(help_text("Colors for Notifications (Mako)."))
                    .child(
                        Button::new("launch-test-notification")
                            .label("Test Notification")
                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                this.launch_test_notification();
                            })),
                    ),
            )
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(form_section().child(color_picker_with_clipboard(
                        "mako-text",
                        "Text Color",
                        &self.text_color_picker,
                    )))
                    .child(form_section().child(color_picker_with_clipboard(
                        "mako-border",
                        "Border Color",
                        &self.border_color_picker,
                    )))
                    .child(form_section().child(color_picker_with_clipboard(
                        "mako-bg",
                        "Background Color",
                        &self.background_color_picker,
                    ))),
            )
    }
}
