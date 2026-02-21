//! Terminal tab for theme editing
//!
//! Provides UI for editing unified terminal colors that apply to all terminal emulators:
//! - Alacritty
//! - Kitty
//! - Ghostty

use crate::shell::theme_sh_commands::execute_bash_command;
use crate::system::themes::theme_management::{save_theme_data, update_terminal_configs};
use crate::types::themes::{EditingTheme, TerminalConfig};
use crate::ui::theme_edit_page::shared::{
    color_picker_with_clipboard, form_section, help_text, tab_container,
};
use gpui::*;
use gpui_component::{
    Colorize,
    button::Button,
    color_picker::{ColorPickerEvent, ColorPickerState},
    divider::Divider,
    h_flex, v_flex,
};

/// Terminal tab content for editing unified terminal colors
pub struct TerminalTab {
    theme_name: String,
    theme_data: EditingTheme,
    primary_bg_picker: Entity<ColorPickerState>,
    primary_fg_picker: Entity<ColorPickerState>,
    cursor_cursor_picker: Entity<ColorPickerState>,
    cursor_text_picker: Entity<ColorPickerState>,
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

impl TerminalTab {
    /// Helper to convert hex to Hsla
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

    /// Helper to create a color picker with subscription
    fn create_color_picker(
        window: &mut Window,
        cx: &mut Context<Self>,
        hex: &str,
        setter: impl Fn(&mut TerminalConfig, String) + 'static + Copy,
    ) -> Entity<ColorPickerState> {
        let color = Self::hex_to_hsla(hex).unwrap_or(gpui::rgb(0x0F0F19).into());
        let picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(color));

        cx.subscribe_in(
            &picker,
            window,
            move |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_terminal_config(|config| {
                        setter(config, hex);
                    });
                    this.save(window, cx);
                }
            },
        )
        .detach();

        picker
    }

    /// Create a new TerminalTab instance
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let config = theme_data
            .apps
            .terminal
            .as_ref()
            .cloned()
            .unwrap_or_default();

        let primary_bg_picker =
            Self::create_color_picker(window, cx, &config.primary.background, |c, v| {
                c.primary.background = v
            });
        let primary_fg_picker =
            Self::create_color_picker(window, cx, &config.primary.foreground, |c, v| {
                c.primary.foreground = v
            });
        let cursor_cursor_picker =
            Self::create_color_picker(window, cx, &config.cursor.cursor, |c, v| {
                c.cursor.cursor = v
            });
        let cursor_text_picker =
            Self::create_color_picker(window, cx, &config.cursor.text, |c, v| c.cursor.text = v);
        let selection_bg_picker =
            Self::create_color_picker(window, cx, &config.selection.background, |c, v| {
                c.selection.background = v
            });
        let selection_fg_picker =
            Self::create_color_picker(window, cx, &config.selection.foreground, |c, v| {
                c.selection.foreground = v
            });
        let normal_black_picker =
            Self::create_color_picker(window, cx, &config.normal.black, |c, v| c.normal.black = v);
        let normal_red_picker =
            Self::create_color_picker(window, cx, &config.normal.red, |c, v| c.normal.red = v);
        let normal_green_picker =
            Self::create_color_picker(window, cx, &config.normal.green, |c, v| c.normal.green = v);
        let normal_yellow_picker =
            Self::create_color_picker(window, cx, &config.normal.yellow, |c, v| {
                c.normal.yellow = v
            });
        let normal_blue_picker =
            Self::create_color_picker(window, cx, &config.normal.blue, |c, v| c.normal.blue = v);
        let normal_magenta_picker =
            Self::create_color_picker(window, cx, &config.normal.magenta, |c, v| {
                c.normal.magenta = v
            });
        let normal_cyan_picker =
            Self::create_color_picker(window, cx, &config.normal.cyan, |c, v| c.normal.cyan = v);
        let normal_white_picker =
            Self::create_color_picker(window, cx, &config.normal.white, |c, v| c.normal.white = v);
        let bright_black_picker =
            Self::create_color_picker(window, cx, &config.bright.black, |c, v| c.bright.black = v);
        let bright_red_picker =
            Self::create_color_picker(window, cx, &config.bright.red, |c, v| c.bright.red = v);
        let bright_green_picker =
            Self::create_color_picker(window, cx, &config.bright.green, |c, v| c.bright.green = v);
        let bright_yellow_picker =
            Self::create_color_picker(window, cx, &config.bright.yellow, |c, v| {
                c.bright.yellow = v
            });
        let bright_blue_picker =
            Self::create_color_picker(window, cx, &config.bright.blue, |c, v| c.bright.blue = v);
        let bright_magenta_picker =
            Self::create_color_picker(window, cx, &config.bright.magenta, |c, v| {
                c.bright.magenta = v
            });
        let bright_cyan_picker =
            Self::create_color_picker(window, cx, &config.bright.cyan, |c, v| c.bright.cyan = v);
        let bright_white_picker =
            Self::create_color_picker(window, cx, &config.bright.white, |c, v| c.bright.white = v);

        Self {
            theme_name,
            theme_data,
            primary_bg_picker,
            primary_fg_picker,
            cursor_cursor_picker,
            cursor_text_picker,
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

    /// Update the terminal config within theme_data
    fn update_terminal_config<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut TerminalConfig),
    {
        let mut config = self.theme_data.apps.terminal.clone().unwrap_or_default();
        updater(&mut config);
        self.theme_data.apps.terminal = Some(config);
    }

    /// Save the theme data and update all terminal config files
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
                if let Some(ref terminal_config) = self.theme_data.apps.terminal
                    && let Err(e) = update_terminal_configs(&self.theme_name, terminal_config)
                {
                    self.error_message = Some(format!("Failed to update terminal configs: {}", e));
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

    /// Launch a terminal emulator
    fn launch_terminal(&self, app_name: &str) {
        let command = format!("uwsm app -- {}", app_name);
        if let Err(e) = execute_bash_command(command) {
            eprintln!("Failed to launch {}: {}", app_name, e);
        }
    }
}

impl Render for TerminalTab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let wide = window.viewport_size().width >= px(1000.0);

        // Cursor Colors section
        let cursor_section = form_section()
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Cursor Colors"),
            )
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(color_picker_with_clipboard(
                        "term-cursor",
                        "Cursor",
                        &self.cursor_cursor_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-cursor-text",
                        "Cursor Text",
                        &self.cursor_text_picker,
                    )),
            );

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
                        "term-selection-bg",
                        "Background",
                        &self.selection_bg_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-selection-fg",
                        "Foreground",
                        &self.selection_fg_picker,
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
                        "term-normal-black",
                        "Black",
                        &self.normal_black_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-normal-red",
                        "Red",
                        &self.normal_red_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-normal-green",
                        "Green",
                        &self.normal_green_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-normal-yellow",
                        "Yellow",
                        &self.normal_yellow_picker,
                    )),
            )
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(color_picker_with_clipboard(
                        "term-normal-blue",
                        "Blue",
                        &self.normal_blue_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-normal-magenta",
                        "Magenta",
                        &self.normal_magenta_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-normal-cyan",
                        "Cyan",
                        &self.normal_cyan_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-normal-white",
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
                        "term-bright-black",
                        "Black",
                        &self.bright_black_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-bright-red",
                        "Red",
                        &self.bright_red_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-bright-green",
                        "Green",
                        &self.bright_green_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-bright-yellow",
                        "Yellow",
                        &self.bright_yellow_picker,
                    )),
            )
            .child(
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(color_picker_with_clipboard(
                        "term-bright-blue",
                        "Blue",
                        &self.bright_blue_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-bright-magenta",
                        "Magenta",
                        &self.bright_magenta_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-bright-cyan",
                        "Cyan",
                        &self.bright_cyan_picker,
                    ))
                    .child(color_picker_with_clipboard(
                        "term-bright-white",
                        "White",
                        &self.bright_white_picker,
                    )),
            );

        tab_container()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(help_text(
                        "Color changes apply to Alacritty, Kitty, and Ghostty.",
                    ))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Button::new("launch-alacritty").label("Alacritty").on_click(
                                cx.listener(|this, _event, _window, _cx| {
                                    this.launch_terminal("alacritty");
                                }),
                            ))
                            .child(Button::new("launch-kitty").label("Kitty").on_click(
                                cx.listener(|this, _event, _window, _cx| {
                                    this.launch_terminal("kitty");
                                }),
                            ))
                            .child(Button::new("launch-ghostty").label("Ghostty").on_click(
                                cx.listener(|this, _event, _window, _cx| {
                                    this.launch_terminal("ghostty");
                                }),
                            )),
                    ),
            )
            .child(
                v_flex()
                    .gap_6()
                    // Primary Colors — full width
                    .child(
                        form_section()
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
                                        "term-primary-bg",
                                        "Background",
                                        &self.primary_bg_picker,
                                    ))
                                    .child(color_picker_with_clipboard(
                                        "term-primary-fg",
                                        "Foreground",
                                        &self.primary_fg_picker,
                                    )),
                            ),
                    )
                    .child(Divider::horizontal())
                    // Cursor + Selection — 2 cols on wide, stacked on narrow
                    .child(if wide {
                        div()
                            .grid()
                            .grid_cols(2)
                            .gap_6()
                            .child(cursor_section)
                            .child(selection_section)
                    } else {
                        div()
                            .flex()
                            .flex_col()
                            .gap_6()
                            .child(cursor_section)
                            .child(selection_section)
                    })
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
    }
}
