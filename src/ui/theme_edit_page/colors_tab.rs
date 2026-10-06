use crate::system::themes::theme_management::update_theme;
use crate::types::themes::{ColorsConfig, EditingTheme};
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::color_utils::{
    hex_to_hsla, hex6, hypr_color, hypr_color_text, with_first_hypr_color,
};
use crate::ui::notify;
use crate::ui::theme_edit_page::shared::{
    color_picker_with_clipboard, field_grid, field_label, focus_section, section_title,
    tab_container, tab_grid_columns,
};
use gpui::*;
use gpui_component::{
    Colorize,
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

pub struct ColorsTab {
    theme_name: String,
    theme_data: EditingTheme,
    accent_picker: Entity<ColorPickerState>,
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
    // Free text because Hyprland border specs can be gradients
    // ("rgba(..ee) rgba(..ee) 45deg"), which a color picker cannot express.
    active_border_input: Entity<InputState>,
    active_border_picker: Entity<ColorPickerState>,
    inactive_border_input: Entity<InputState>,
    inactive_border_picker: Entity<ColorPickerState>,
    /// Bumped on every edit; a pending save only runs if it is still the
    /// latest, so a dragged slider writes once, not once per frame.
    edit_generation: u64,
    /// The generation the last completed (or in-flight) save carried.
    saved_generation: u64,
    /// The border field holds text that is not a color; said once, when
    /// it stops being one.
    border_invalid: bool,
    scroll: ScrollHandle,
}

const SAVE_DELAY: std::time::Duration = std::time::Duration::from_millis(300);

impl ColorsTab {
    fn create_color_picker(
        window: &mut Window,
        cx: &mut Context<Self>,
        hex: &str,
        setter: impl Fn(&mut ColorsConfig, String) + 'static + Copy,
    ) -> Entity<ColorPickerState> {
        let color = hex_to_hsla(hex).unwrap_or(gpui::rgb(0x0F0F19).into());
        let picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(color));

        cx.subscribe_in(
            &picker,
            window,
            move |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = hex6(&color.to_hex());
                    this.update_colors(|config| {
                        setter(config, hex);
                    });
                    this.schedule_save(window, cx);
                }
            },
        )
        .detach();

        picker
    }

    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        scroll: &ScrollHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let colors = theme_data.colors.clone();

        let accent_picker =
            Self::create_color_picker(window, cx, &colors.accent, |c, v| c.accent = v);
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

        let (active_border_input, active_border_picker) = Self::border_input(
            window,
            cx,
            colors.hyprland_active_border.as_deref(),
            "Default: accent color",
            |c, v| c.hyprland_active_border = v,
        );
        let (inactive_border_input, inactive_border_picker) = Self::border_input(
            window,
            cx,
            colors.hyprland_inactive_border.as_deref(),
            "Default: rgba(595959aa)",
            |c, v| c.hyprland_inactive_border = v,
        );

        Self {
            theme_name,
            theme_data,
            accent_picker,
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
            active_border_input,
            active_border_picker,
            inactive_border_input,
            inactive_border_picker,
            edit_generation: 0,
            saved_generation: 0,
            border_invalid: false,
            scroll: scroll.clone(),
        }
    }

    /// A border is Hyprland's own text (a color, or a gradient of colors
    /// and an angle), so it stays a text field; the picker beside it holds
    /// the first color and writes it back in place, keeping the rest.
    fn border_input(
        window: &mut Window,
        cx: &mut Context<Self>,
        value: Option<&str>,
        placeholder: &str,
        setter: fn(&mut ColorsConfig, Option<String>),
    ) -> (Entity<InputState>, Entity<ColorPickerState>) {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(placeholder.to_string())
                .default_value(value.unwrap_or_default().to_string())
        });
        let picker = cx.new(|cx| {
            let state = ColorPickerState::new(window, cx);
            match value.and_then(hypr_color) {
                Some(color) => state.default_value(color),
                None => state,
            }
        });

        let input_picker = picker.clone();
        cx.subscribe_in(
            &input,
            window,
            move |this, input, event: &InputEvent, window, cx| {
                if let InputEvent::Change = event {
                    let raw = input.read(cx).value().to_string();
                    let trimmed = raw.trim();
                    if let Err(e) = validate_border(trimmed) {
                        // Keep the last valid value on disk; say why.
                        if !this.border_invalid {
                            notify::error(window, e, cx);
                        }
                        this.border_invalid = true;
                        return;
                    }
                    this.border_invalid = false;
                    // The picker follows the text; neither setter emits.
                    input_picker.update(cx, |picker, cx| match hypr_color(trimmed) {
                        Some(color) => picker.set_value(color, window, cx),
                        None => picker.clear_value(window, cx),
                    });
                    let value = (!trimmed.is_empty()).then(|| trimmed.to_string());
                    setter(&mut this.theme_data.colors, value);
                    this.schedule_save(window, cx);
                }
            },
        )
        .detach();

        let picker_input = input.clone();
        cx.subscribe_in(
            &picker,
            window,
            move |this, _, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let current = picker_input.read(cx).value().to_string();
                    let text = with_first_hypr_color(current.trim(), &hypr_color_text(*color));
                    picker_input.update(cx, |input, cx| input.set_value(text.clone(), window, cx));
                    this.border_invalid = false;
                    setter(&mut this.theme_data.colors, Some(text));
                    this.schedule_save(window, cx);
                }
            },
        )
        .detach();

        (input, picker)
    }

    fn update_colors<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut ColorsConfig),
    {
        updater(&mut self.theme_data.colors);
    }

    /// Saves 300 ms after the last edit, off the UI thread.
    fn schedule_save(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.edit_generation += 1;
        cx.notify();
        let generation = self.edit_generation;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SAVE_DELAY).await;
            let current = this.update(cx, |this, _| this.edit_generation == generation);
            if !matches!(current, Ok(true)) {
                return;
            }
            let Ok(save) = this.update(cx, |this, _| this.pending_save()) else {
                return;
            };
            let result = cx.background_spawn(async move { save() }).await;
            this.update(cx, |this, cx| {
                if this.edit_generation == generation
                    && let Err(e) = result
                {
                    emit(cx, AppEvent::Error(e.to_string()));
                }
            })
            .ok();
        })
        .detach();
    }

    /// Writes any edit that has not reached disk yet, now, so an Apply
    /// right after a change stages what the user sees.
    pub fn flush(&mut self, cx: &mut Context<Self>) {
        if self.edit_generation == self.saved_generation {
            return;
        }
        let save = self.pending_save();
        if let Err(e) = save() {
            emit(cx, AppEvent::Error(e.to_string()));
        }
    }

    /// The save for the current snapshot, to run on any thread. Only the
    /// palette and borders are written; the mode belongs to the General tab.
    fn pending_save(&mut self) -> Box<dyn FnOnce() -> crate::error::Result<()> + Send> {
        self.saved_generation = self.edit_generation;
        let theme_name = self.theme_name.clone();
        let colors = self.theme_data.colors.clone();
        Box::new(move || {
            update_theme(&theme_name, |theme| {
                let mode = theme.colors.mode.clone();
                theme.colors = colors;
                theme.colors.mode = mode;
            })
        })
    }
}

/// What `omarchy-theme-color` accepts for a border value; anything else it
/// drops with a note on stderr, and a quote would break the TOML string.
fn validate_border(value: &str) -> std::result::Result<(), String> {
    const ALLOWED: &str = "#(),._+/%- ";
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ALLOWED.contains(c))
    {
        Ok(())
    } else {
        Err("A border can only use letters, digits, spaces and # ( ) , . _ + / % -".to_string())
    }
}

impl Render for ColorsTab {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // One column per ANSI color, so each bright color sits under its
        // normal one; halved on narrower windows so the pairs stay together.
        let columns = match tab_grid_columns(window) {
            8.. => 8,
            4..=7 => 4,
            _ => 2,
        };
        let cell = |id: &'static str, label: &'static str, picker: &Entity<ColorPickerState>| {
            color_picker_with_clipboard(id, label, picker).into_any_element()
        };
        let section = |title: &'static str, cells: Vec<AnyElement>| {
            v_flex()
                .gap_1()
                .child(section_title(title))
                .child(field_grid(columns, cells))
        };

        let primary = section(
            "Primary Colors",
            vec![
                cell("colors-accent", "Accent", &self.accent_picker),
                cell("colors-background", "Background", &self.background_picker),
                cell("colors-foreground", "Foreground", &self.foreground_picker),
            ],
        );
        let selection = section(
            "Selection Colors",
            vec![
                cell(
                    "colors-selection-bg",
                    "Background",
                    &self.selection_bg_picker,
                ),
                cell(
                    "colors-selection-fg",
                    "Foreground",
                    &self.selection_fg_picker,
                ),
            ],
        );
        let normal = section(
            "Normal Colors",
            vec![
                cell("colors-normal-black", "Black", &self.normal_black_picker),
                cell("colors-normal-red", "Red", &self.normal_red_picker),
                cell("colors-normal-green", "Green", &self.normal_green_picker),
                cell("colors-normal-yellow", "Yellow", &self.normal_yellow_picker),
                cell("colors-normal-blue", "Blue", &self.normal_blue_picker),
                cell(
                    "colors-normal-magenta",
                    "Magenta",
                    &self.normal_magenta_picker,
                ),
                cell("colors-normal-cyan", "Cyan", &self.normal_cyan_picker),
                cell("colors-normal-white", "White", &self.normal_white_picker),
            ],
        );
        let bright = section(
            "Bright Colors",
            vec![
                cell("colors-bright-black", "Black", &self.bright_black_picker),
                cell("colors-bright-red", "Red", &self.bright_red_picker),
                cell("colors-bright-green", "Green", &self.bright_green_picker),
                cell("colors-bright-yellow", "Yellow", &self.bright_yellow_picker),
                cell("colors-bright-blue", "Blue", &self.bright_blue_picker),
                cell(
                    "colors-bright-magenta",
                    "Magenta",
                    &self.bright_magenta_picker,
                ),
                cell("colors-bright-cyan", "Cyan", &self.bright_cyan_picker),
                cell("colors-bright-white", "White", &self.bright_white_picker),
            ],
        );

        let border_input =
            |label: &'static str, state: &Entity<InputState>, picker: &Entity<ColorPickerState>| {
                v_flex()
                    .gap_2()
                    .child(field_label(label, None))
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(ColorPicker::new(picker))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(Input::new(state).cleanable(true)),
                            ),
                    )
                    .into_any_element()
            };
        let borders = v_flex()
            .gap_1()
            .child(section_title("Window Borders"))
            .child(field_grid(
                (columns / 4).max(1),
                vec![
                    border_input(
                        "Active Border",
                        &self.active_border_input,
                        &self.active_border_picker,
                    ),
                    border_input(
                        "Inactive Border",
                        &self.inactive_border_input,
                        &self.inactive_border_picker,
                    ),
                ],
            ));

        tab_container()
            .gap_8()
            .child(focus_section("colors-primary", &self.scroll, primary))
            .child(focus_section("colors-selection", &self.scroll, selection))
            .child(focus_section("colors-normal", &self.scroll, normal))
            .child(focus_section("colors-bright", &self.scroll, bright))
            .child(focus_section("colors-borders", &self.scroll, borders))
    }
}
