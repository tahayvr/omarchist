use crate::system::themes::theme_management::{save_theme_data, update_swayosd_css};
use crate::types::themes::{EditingTheme, SwayosdConfig};
use crate::ui::theme_edit_page::shared::{
    color_picker_with_clipboard, form_section, help_text, tab_container,
};
use gpui::*;
use gpui_component::{
    ActiveTheme, Colorize,
    color_picker::{ColorPickerEvent, ColorPickerState},
    h_flex,
};

pub struct SwayosdTab {
    theme_name: String,
    theme_data: EditingTheme,
    background_color_picker: Entity<ColorPickerState>,
    border_color_picker: Entity<ColorPickerState>,
    label_picker: Entity<ColorPickerState>,
    image_picker: Entity<ColorPickerState>,
    progress_picker: Entity<ColorPickerState>,
    is_saving: bool,
    error_message: Option<String>,
}

impl SwayosdTab {
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Get current swayosd config or use defaults
        let swayosd_config = theme_data
            .apps
            .swayosd
            .as_ref()
            .cloned()
            .unwrap_or_default();

        // Create color picker states with current values
        let background_color = Self::hex_to_hsla(&swayosd_config.background_color)
            .unwrap_or(gpui::rgb(0x0F0F19).into());
        let border_color =
            Self::hex_to_hsla(&swayosd_config.border_color).unwrap_or(gpui::rgb(0x33A1FF).into());
        let label = Self::hex_to_hsla(&swayosd_config.label).unwrap_or(gpui::rgb(0x8A8A8D).into());
        let image = Self::hex_to_hsla(&swayosd_config.image).unwrap_or(gpui::rgb(0x8A8A8D).into());
        let progress =
            Self::hex_to_hsla(&swayosd_config.progress).unwrap_or(gpui::rgb(0x8A8A8D).into());

        let background_color_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(background_color));
        let border_color_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(border_color));
        let label_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(label));
        let image_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(image));
        let progress_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(progress));

        let mut tab = Self {
            theme_name,
            theme_data,
            background_color_picker,
            border_color_picker,
            label_picker,
            image_picker,
            progress_picker,
            is_saving: false,
            error_message: None,
        };

        // Subscribe to all color picker changes
        tab.subscribe_to_pickers(window, cx);

        tab
    }

    fn subscribe_to_pickers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        cx.subscribe_in(
            &self.background_color_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_swayosd_config(|config| config.background_color = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.border_color_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_swayosd_config(|config| config.border_color = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.label_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_swayosd_config(|config| config.label = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.image_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_swayosd_config(|config| config.image = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.progress_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_swayosd_config(|config| config.progress = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();
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

    fn update_swayosd_config<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut SwayosdConfig),
    {
        let mut config = self.theme_data.apps.swayosd.clone().unwrap_or_default();
        updater(&mut config);
        self.theme_data.apps.swayosd = Some(config);
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
                // Also update the swayosd.css file
                if let Some(ref swayosd_config) = self.theme_data.apps.swayosd
                    && let Err(e) = update_swayosd_css(&self.theme_name, swayosd_config)
                {
                    self.error_message = Some(format!("Failed to update swayosd.css: {}", e));
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

impl Render for SwayosdTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        tab_container()
            .child(help_text(
                "Colors for On-Screen Display (Volume Change, Display Brightness, etc).",
                cx.theme().muted_foreground,
            ))
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(form_section().child(color_picker_with_clipboard(
                        "swayosd-bg",
                        "Background",
                        &self.background_color_picker,
                    )))
                    .child(form_section().child(color_picker_with_clipboard(
                        "swayosd-border",
                        "Border",
                        &self.border_color_picker,
                    )))
                    .child(form_section().child(color_picker_with_clipboard(
                        "swayosd-label",
                        "Label",
                        &self.label_picker,
                    ))),
            )
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(form_section().child(color_picker_with_clipboard(
                        "swayosd-image",
                        "Image",
                        &self.image_picker,
                    )))
                    .child(form_section().child(color_picker_with_clipboard(
                        "swayosd-progress",
                        "Progress",
                        &self.progress_picker,
                    ))),
            )
    }
}
