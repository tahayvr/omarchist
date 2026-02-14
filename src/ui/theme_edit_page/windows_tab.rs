//! Windows tab for theme editing (Hyprland window settings)
//!
//! Provides UI for editing Hyprland window settings:
//! - Active border color
//! - Inactive border color
//! - Border size (NumberInput)
//! - Gaps in (NumberInput)
//! - Gaps out (NumberInput)
//! - Rounding (NumberInput)

use crate::system::theme_management::{save_theme_data, update_hyprland_conf};
use crate::types::themes::{EditingTheme, HyprlandConfig};
use crate::ui::theme_edit_page::shared::{form_section, help_text, tab_container};
use gpui::*;
use gpui_component::{
    Colorize,
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
    h_flex,
    input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    label::Label,
};

/// Windows tab content for editing Hyprland window settings
pub struct WindowsTab {
    theme_name: String,
    theme_data: EditingTheme,
    active_border_picker: Entity<ColorPickerState>,
    inactive_border_picker: Entity<ColorPickerState>,
    border_size_input: Entity<InputState>,
    gaps_in_input: Entity<InputState>,
    gaps_out_input: Entity<InputState>,
    rounding_input: Entity<InputState>,
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

        // Create number input states
        let border_size_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("0")
                .default_value(hyprland_config.border_size.to_string())
        });

        let gaps_in_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("0")
                .default_value(hyprland_config.gaps_in.to_string())
        });

        let gaps_out_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("0")
                .default_value(hyprland_config.gaps_out.to_string())
        });

        let rounding_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("0")
                .default_value(hyprland_config.rounding.to_string())
        });

        let tab = Self {
            theme_name,
            theme_data,
            active_border_picker,
            inactive_border_picker,
            border_size_input,
            gaps_in_input,
            gaps_out_input,
            rounding_input,
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

        // Subscribe to border size input changes (text input)
        cx.subscribe_in(
            &tab.border_size_input,
            window,
            |this, _input, event: &InputEvent, _window, cx| {
                if let InputEvent::Change = event {
                    let value = this.border_size_input.read(cx).value().to_string();
                    if let Ok(num) = value.parse::<i32>() {
                        this.update_hyprland_config(|config| {
                            config.border_size = num;
                        });
                        cx.notify();
                    }
                }
            },
        )
        .detach();

        // Subscribe to border size step buttons (increment/decrement)
        cx.subscribe_in(
            &tab.border_size_input,
            window,
            |this, state, event: &NumberInputEvent, window, cx| {
                let NumberInputEvent::Step(action) = event;
                let current = this
                    .theme_data
                    .apps
                    .hyprland
                    .as_ref()
                    .map(|h| h.border_size)
                    .unwrap_or(1);
                let new_val = match action {
                    StepAction::Increment => current + 1,
                    StepAction::Decrement => current.saturating_sub(1),
                };
                state.update(cx, |input, cx| {
                    input.set_value(new_val.to_string(), window, cx);
                });
                this.update_hyprland_config(|config| {
                    config.border_size = new_val;
                });
                this.save(window, cx);
            },
        )
        .detach();

        // Subscribe to gaps in input changes (text input)
        cx.subscribe_in(
            &tab.gaps_in_input,
            window,
            |this, _input, event: &InputEvent, _window, cx| {
                if let InputEvent::Change = event {
                    let value = this.gaps_in_input.read(cx).value().to_string();
                    if let Ok(num) = value.parse::<i32>() {
                        this.update_hyprland_config(|config| {
                            config.gaps_in = num;
                        });
                        cx.notify();
                    }
                }
            },
        )
        .detach();

        // Subscribe to gaps in step buttons
        cx.subscribe_in(
            &tab.gaps_in_input,
            window,
            |this, state, event: &NumberInputEvent, window, cx| {
                let NumberInputEvent::Step(action) = event;
                let current = this
                    .theme_data
                    .apps
                    .hyprland
                    .as_ref()
                    .map(|h| h.gaps_in)
                    .unwrap_or(5);
                let new_val = match action {
                    StepAction::Increment => current + 1,
                    StepAction::Decrement => current.saturating_sub(1),
                };
                state.update(cx, |input, cx| {
                    input.set_value(new_val.to_string(), window, cx);
                });
                this.update_hyprland_config(|config| {
                    config.gaps_in = new_val;
                });
                this.save(window, cx);
            },
        )
        .detach();

        // Subscribe to gaps out input changes (text input)
        cx.subscribe_in(
            &tab.gaps_out_input,
            window,
            |this, _input, event: &InputEvent, _window, cx| {
                if let InputEvent::Change = event {
                    let value = this.gaps_out_input.read(cx).value().to_string();
                    if let Ok(num) = value.parse::<i32>() {
                        this.update_hyprland_config(|config| {
                            config.gaps_out = num;
                        });
                        cx.notify();
                    }
                }
            },
        )
        .detach();

        // Subscribe to gaps out step buttons
        cx.subscribe_in(
            &tab.gaps_out_input,
            window,
            |this, state, event: &NumberInputEvent, window, cx| {
                let NumberInputEvent::Step(action) = event;
                let current = this
                    .theme_data
                    .apps
                    .hyprland
                    .as_ref()
                    .map(|h| h.gaps_out)
                    .unwrap_or(10);
                let new_val = match action {
                    StepAction::Increment => current + 1,
                    StepAction::Decrement => current.saturating_sub(1),
                };
                state.update(cx, |input, cx| {
                    input.set_value(new_val.to_string(), window, cx);
                });
                this.update_hyprland_config(|config| {
                    config.gaps_out = new_val;
                });
                this.save(window, cx);
            },
        )
        .detach();

        // Subscribe to rounding input changes (text input)
        cx.subscribe_in(
            &tab.rounding_input,
            window,
            |this, _input, event: &InputEvent, _window, cx| {
                if let InputEvent::Change = event {
                    let value = this.rounding_input.read(cx).value().to_string();
                    if let Ok(num) = value.parse::<i32>() {
                        this.update_hyprland_config(|config| {
                            config.rounding = num;
                        });
                        cx.notify();
                    }
                }
            },
        )
        .detach();

        // Subscribe to rounding step buttons
        cx.subscribe_in(
            &tab.rounding_input,
            window,
            |this, state, event: &NumberInputEvent, window, cx| {
                let NumberInputEvent::Step(action) = event;
                let current = this
                    .theme_data
                    .apps
                    .hyprland
                    .as_ref()
                    .map(|h| h.rounding)
                    .unwrap_or(0);
                let new_val = match action {
                    StepAction::Increment => current + 1,
                    StepAction::Decrement => current.saturating_sub(1),
                };
                state.update(cx, |input, cx| {
                    input.set_value(new_val.to_string(), window, cx);
                });
                this.update_hyprland_config(|config| {
                    config.rounding = new_val;
                });
                this.save(window, cx);
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
                if let Some(ref hyprland_config) = self.theme_data.apps.hyprland {
                    if let Err(e) = update_hyprland_conf(&self.theme_name, hyprland_config) {
                        self.error_message = Some(format!("Failed to update hyprland.conf: {}", e));
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

impl Render for WindowsTab {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        tab_container()
            .child(help_text("Changes auto-save."))
            .child(
                // Border colors row
                h_flex()
                    .gap_24()
                    .flex_wrap()
                    .child(
                        form_section().child(
                            ColorPicker::new(&self.active_border_picker).label("Active Border"),
                        ),
                    )
                    .child(form_section().child(
                        ColorPicker::new(&self.inactive_border_picker).label("Inactive Border"),
                    )),
            )
            .child(
                // Numeric inputs row
                h_flex()
                    .gap_12()
                    .flex_wrap()
                    .child(
                        form_section()
                            .child(Label::new("Border Size").text_sm())
                            .child(
                                div()
                                    .w(px(120.))
                                    .child(NumberInput::new(&self.border_size_input)),
                            ),
                    )
                    .child(
                        form_section().child(Label::new("Gaps In").text_sm()).child(
                            div()
                                .w(px(120.))
                                .child(NumberInput::new(&self.gaps_in_input)),
                        ),
                    )
                    .child(
                        form_section()
                            .child(Label::new("Gaps Out").text_sm())
                            .child(
                                div()
                                    .w(px(120.))
                                    .child(NumberInput::new(&self.gaps_out_input)),
                            ),
                    )
                    .child(
                        form_section()
                            .child(Label::new("Rounding").text_sm())
                            .child(
                                div()
                                    .w(px(120.))
                                    .child(NumberInput::new(&self.rounding_input)),
                            ),
                    ),
            )
    }
}
