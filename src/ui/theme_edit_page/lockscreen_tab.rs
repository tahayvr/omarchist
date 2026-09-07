use crate::system::themes::theme_management::{save_theme_data, update_lock_toml};
use crate::types::themes::{EditingTheme, LockScreenConfig};
use crate::ui::theme_edit_page::shared::{
    color_picker_with_clipboard, form_section, help_text, tab_container,
};
use gpui::*;
use gpui_component::{
    ActiveTheme, Colorize,
    color_picker::{ColorPickerEvent, ColorPickerState},
    h_flex,
};

pub struct LockScreenTab {
    theme_name: String,
    theme_data: EditingTheme,
    text_picker: Entity<ColorPickerState>,
    placeholder_picker: Entity<ColorPickerState>,
    text_error_picker: Entity<ColorPickerState>,
    border_picker: Entity<ColorPickerState>,
    border_active_picker: Entity<ColorPickerState>,
    border_error_picker: Entity<ColorPickerState>,
    is_saving: bool,
    error_message: Option<String>,
}

impl LockScreenTab {
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

    fn create_color_picker(
        window: &mut Window,
        cx: &mut Context<Self>,
        hex: &str,
        setter: impl Fn(&mut LockScreenConfig, String) + 'static + Copy,
    ) -> Entity<ColorPickerState> {
        let color = Self::hex_to_hsla(hex).unwrap_or(gpui::rgb(0x33A1FF).into());
        let picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(color));

        cx.subscribe_in(
            &picker,
            window,
            move |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_lock_config(|config| {
                        setter(config, hex);
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        picker
    }

    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let config = theme_data.apps.lock.as_ref().cloned().unwrap_or_default();

        let text_picker = Self::create_color_picker(window, cx, &config.text, |c, v| c.text = v);
        let placeholder_picker =
            Self::create_color_picker(window, cx, &config.placeholder, |c, v| c.placeholder = v);
        let text_error_picker =
            Self::create_color_picker(window, cx, &config.text_error, |c, v| c.text_error = v);
        let border_picker =
            Self::create_color_picker(window, cx, &config.border, |c, v| c.border = v);
        let border_active_picker =
            Self::create_color_picker(window, cx, &config.border_active, |c, v| {
                c.border_active = v
            });
        let border_error_picker =
            Self::create_color_picker(window, cx, &config.border_error, |c, v| c.border_error = v);

        Self {
            theme_name,
            theme_data,
            text_picker,
            placeholder_picker,
            text_error_picker,
            border_picker,
            border_active_picker,
            border_error_picker,
            is_saving: false,
            error_message: None,
        }
    }

    fn update_lock_config<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut LockScreenConfig),
    {
        let mut config = self.theme_data.apps.lock.clone().unwrap_or_default();
        updater(&mut config);
        self.theme_data.apps.lock = Some(config);
    }

    pub fn theme_data(&self) -> &EditingTheme {
        &self.theme_data
    }

    fn save(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_saving {
            return;
        }

        if self.theme_name.is_empty() {
            self.error_message = Some("Theme name cannot be empty".to_string());
            cx.notify();
            return;
        }

        self.is_saving = true;
        self.error_message = None;
        cx.notify();

        match save_theme_data(&self.theme_name, &self.theme_data) {
            Ok(()) => {
                if let Some(ref lock_config) = self.theme_data.apps.lock
                    && let Err(e) = update_lock_toml(&self.theme_name, lock_config)
                {
                    self.error_message = Some(format!("Failed to update shell.lock.toml: {}", e));
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        tab_container()
            .child(help_text(
                "Colors for the lock screen.",
                cx.theme().muted_foreground,
            ))
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(form_section().child(color_picker_with_clipboard(
                        "lock-text",
                        "Text",
                        &self.text_picker,
                    )))
                    .child(form_section().child(color_picker_with_clipboard(
                        "lock-placeholder",
                        "Placeholder",
                        &self.placeholder_picker,
                    )))
                    .child(form_section().child(color_picker_with_clipboard(
                        "lock-text-error",
                        "Text Error",
                        &self.text_error_picker,
                    ))),
            )
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(form_section().child(color_picker_with_clipboard(
                        "lock-border",
                        "Border",
                        &self.border_picker,
                    )))
                    .child(form_section().child(color_picker_with_clipboard(
                        "lock-border-active",
                        "Border Active",
                        &self.border_active_picker,
                    )))
                    .child(form_section().child(color_picker_with_clipboard(
                        "lock-border-error",
                        "Border Error",
                        &self.border_error_picker,
                    ))),
            )
    }
}
