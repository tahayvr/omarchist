use crate::system::themes::theme_management::save_theme_data;
use crate::types::themes::{ColorsConfig, EditingTheme};
use crate::ui::theme_edit_page::shared::{
    color_picker_with_clipboard, form_section, help_text, tab_container,
};
use gpui::*;
use gpui_component::{
    ActiveTheme, Colorize,
    color_picker::{ColorPickerEvent, ColorPickerState},
    divider::Divider,
    h_flex, v_flex,
};

pub struct ColorsTab {
    theme_name: String,
    theme_data: EditingTheme,
    background_picker: Entity<ColorPickerState>,
    foreground_picker: Entity<ColorPickerState>,
    selection_bg_picker: Entity<ColorPickerState>,
    selection_fg_picker: Entity<ColorPickerState>,
    normal_black_picker: Entity<ColorPickerState>,
    normal_red_picker: Entity<ColorPickerState>,
    normal_green_picker: Entity<ColorPickerState>,
    normal_yellow_picker: Entity<ColorPickerState>,
    normal_blue_picker: Entity<ColorPickerState>,
    normal_magenta_picker: Entity<ColorPickerState>,
    normal_cyan_picker: Entity<ColorPickerState>,
    normal_white_picker: Entity<ColorPickerState>,
    bright_black_picker: Entity<ColorPickerState>,
    bright_red_picker: Entity<ColorPickerState>,
    bright_green_picker: Entity<ColorPickerState>,
    bright_yellow_picker: Entity<ColorPickerState>,
    bright_blue_picker: Entity<ColorPickerState>,
    bright_magenta_picker: Entity<ColorPickerState>,
    bright_cyan_picker: Entity<ColorPickerState>,
    bright_white_picker: Entity<ColorPickerState>,
    is_saving: bool,
    error_message: Option<String>,
}

impl ColorsTab {
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
        setter: impl Fn(&mut ColorsConfig, String) + 'static + Copy,
    ) -> Entity<ColorPickerState> {
        let color = Self::hex_to_hsla(hex).unwrap_or(gpui::rgb(0x0F0F19).into());
        let picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(color));

        cx.subscribe_in(
            &picker,
            window,
            move |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_colors(|config| {
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
        let colors = theme_data.colors.clone();

        let background_picker =
            Self::create_color_picker(window, cx, &colors.background, |c, v| c.background = v);
        let foreground_picker =
            Self::create_color_picker(window, cx, &colors.foreground, |c, v| c.foreground = v);
        let selection_bg_picker =
            Self::create_color_picker(window, cx, &colors.selection_background, |c, v| {
                c.selection_background = v
            });
        let selection_fg_picker =
            Self::create_color_picker(window, cx, &colors.selection_foreground, |c, v| {
                c.selection_foreground = v
            });
        let normal_black_picker =
            Self::create_color_picker(window, cx, &colors.color0, |c, v| c.color0 = v);
        let normal_red_picker =
            Self::create_color_picker(window, cx, &colors.color1, |c, v| c.color1 = v);
        let normal_green_picker =
            Self::create_color_picker(window, cx, &colors.color2, |c, v| c.color2 = v);
        let normal_yellow_picker =
            Self::create_color_picker(window, cx, &colors.color3, |c, v| c.color3 = v);
        let normal_blue_picker =
            Self::create_color_picker(window, cx, &colors.color4, |c, v| c.color4 = v);
        let normal_magenta_picker =
            Self::create_color_picker(window, cx, &colors.color5, |c, v| c.color5 = v);
        let normal_cyan_picker =
            Self::create_color_picker(window, cx, &colors.color6, |c, v| c.color6 = v);
        let normal_white_picker =
            Self::create_color_picker(window, cx, &colors.color7, |c, v| c.color7 = v);
        let bright_black_picker =
            Self::create_color_picker(window, cx, &colors.color8, |c, v| c.color8 = v);
        let bright_red_picker =
            Self::create_color_picker(window, cx, &colors.color9, |c, v| c.color9 = v);
        let bright_green_picker =
            Self::create_color_picker(window, cx, &colors.color10, |c, v| c.color10 = v);
        let bright_yellow_picker =
            Self::create_color_picker(window, cx, &colors.color11, |c, v| c.color11 = v);
        let bright_blue_picker =
            Self::create_color_picker(window, cx, &colors.color12, |c, v| c.color12 = v);
        let bright_magenta_picker =
            Self::create_color_picker(window, cx, &colors.color13, |c, v| c.color13 = v);
        let bright_cyan_picker =
            Self::create_color_picker(window, cx, &colors.color14, |c, v| c.color14 = v);
        let bright_white_picker =
            Self::create_color_picker(window, cx, &colors.color15, |c, v| c.color15 = v);

        Self {
            theme_name,
            theme_data,
            background_picker,
            foreground_picker,
            selection_bg_picker,
            selection_fg_picker,
            normal_black_picker,
            normal_red_picker,
            normal_green_picker,
            normal_yellow_picker,
            normal_blue_picker,
            normal_magenta_picker,
            normal_cyan_picker,
            normal_white_picker,
            bright_black_picker,
            bright_red_picker,
            bright_green_picker,
            bright_yellow_picker,
            bright_blue_picker,
            bright_magenta_picker,
            bright_cyan_picker,
            bright_white_picker,
            is_saving: false,
            error_message: None,
        }
    }

    fn update_colors<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut ColorsConfig),
    {
        updater(&mut self.theme_data.colors);
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

impl Render for ColorsTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let wide = window.viewport_size().width >= px(1000.0);

        // Selection Colors section
        let selection_section = form_section()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Selection Colors"),
            )
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(color_picker_with_clipboard(
                        "colors-selection-bg",
                        "Background",
                        &self.selection_bg_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-selection-fg",
                        "Foreground",
                        &self.selection_fg_picker,
                    )),
            );

        // Primary Colors section
        let primary_section = form_section()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Primary Colors"),
            )
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(color_picker_with_clipboard(
                        "colors-background",
                        "Background",
                        &self.background_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-foreground",
                        "Foreground",
                        &self.foreground_picker,
                    )),
            );

        // Normal Colors section
        let normal_section = form_section()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Normal Colors"),
            )
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(color_picker_with_clipboard(
                        "colors-normal-black",
                        "Black",
                        &self.normal_black_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-normal-red",
                        "Red",
                        &self.normal_red_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-normal-green",
                        "Green",
                        &self.normal_green_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-normal-yellow",
                        "Yellow",
                        &self.normal_yellow_picker,
                    )),
            )
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(color_picker_with_clipboard(
                        "colors-normal-blue",
                        "Blue",
                        &self.normal_blue_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-normal-magenta",
                        "Magenta",
                        &self.normal_magenta_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-normal-cyan",
                        "Cyan",
                        &self.normal_cyan_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-normal-white",
                        "White",
                        &self.normal_white_picker,
                    )),
            );

        // Bright Colors section
        let bright_section = form_section()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Bright Colors"),
            )
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(color_picker_with_clipboard(
                        "colors-bright-black",
                        "Black",
                        &self.bright_black_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-bright-red",
                        "Red",
                        &self.bright_red_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-bright-green",
                        "Green",
                        &self.bright_green_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-bright-yellow",
                        "Yellow",
                        &self.bright_yellow_picker,
                    )),
            )
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(color_picker_with_clipboard(
                        "colors-bright-blue",
                        "Blue",
                        &self.bright_blue_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-bright-magenta",
                        "Magenta",
                        &self.bright_magenta_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-bright-cyan",
                        "Cyan",
                        &self.bright_cyan_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "colors-bright-white",
                        "White",
                        &self.bright_white_picker,
                    )),
            );

        tab_container()
            .child(help_text(
                "This is the theme's full palette (colors.toml) — Omarchy generates your terminal, window borders, and other app colors from these values.",
                cx.theme().muted_foreground,
            ))
            .child(
                v_flex()
                    .gap_6()
                    .child(primary_section)
                    .child(Divider::horizontal())
                    .child(selection_section)
                    .child(Divider::horizontal())
                    // Normal + Bright — 2 cols on wide, stacked on narrow
                    .child(if wide {
                        div()
                            .grid()
                            .grid_cols(2)
                            .gap_6()
                            .child(normal_section)
                            .child(bright_section)
                    } else {
                        div()
                            .flex()
                            .flex_col()
                            .gap_6()
                            .child(normal_section)
                            .child(bright_section)
                    }),
            )
            .children(
                self.error_message
                    .as_ref()
                    .map(|msg| crate::ui::theme_edit_page::shared::error_message(msg.clone(), cx)),
            )
    }
}
