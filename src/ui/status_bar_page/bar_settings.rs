use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, IndexPath, Sizable, StyledExt, h_flex,
    input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    label::Label,
    select::{Select, SelectEvent, SelectState},
    v_flex,
};

use crate::system::waybar::{BarSettings, get_bar_settings, set_bar_setting};
use crate::ui::status_bar_page::shared::labeled_input;

pub struct BarSettingsPanel {
    profile_name: String,
    is_read_only: bool,
    settings: BarSettings,
    expanded: bool,

    // Input states for text / number fields
    height_input: Entity<InputState>,
    spacing_input: Entity<InputState>,
    output_input: Entity<InputState>,
    margin_top_input: Entity<InputState>,
    margin_right_input: Entity<InputState>,
    margin_bottom_input: Entity<InputState>,
    margin_left_input: Entity<InputState>,

    // Select states for enum fields
    position_select: Entity<SelectState<Vec<SharedString>>>,
    layer_select: Entity<SelectState<Vec<SharedString>>>,

    // Keep subscriptions alive for the lifetime of this entity
    _subscriptions: Vec<Subscription>,
}

const POSITIONS: &[&str] = &["top", "bottom", "left", "right"];
const LAYERS: &[&str] = &["top", "bottom", "overlay", "background"];

impl BarSettingsPanel {
    pub fn new(
        profile_name: &str,
        is_read_only: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let settings = load_settings(profile_name, is_read_only).unwrap_or_default();

        let mk_input = |val: String, placeholder: &str, window: &mut Window, cx: &mut App| {
            cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(val)
                    .placeholder(placeholder.to_string())
            })
        };

        let height_input = mk_input(
            settings.height.map(|v| v.to_string()).unwrap_or_default(),
            "e.g. 26",
            window,
            cx,
        );
        let spacing_input = mk_input(
            settings.spacing.map(|v| v.to_string()).unwrap_or_default(),
            "e.g. 4",
            window,
            cx,
        );
        let output_input = mk_input(
            settings.output.clone().unwrap_or_default(),
            "e.g. DP-1",
            window,
            cx,
        );
        let margin_top_input = mk_input(
            settings
                .margin_top
                .map(|v| v.to_string())
                .unwrap_or_default(),
            "0",
            window,
            cx,
        );
        let margin_right_input = mk_input(
            settings
                .margin_right
                .map(|v| v.to_string())
                .unwrap_or_default(),
            "0",
            window,
            cx,
        );
        let margin_bottom_input = mk_input(
            settings
                .margin_bottom
                .map(|v| v.to_string())
                .unwrap_or_default(),
            "0",
            window,
            cx,
        );
        let margin_left_input = mk_input(
            settings
                .margin_left
                .map(|v| v.to_string())
                .unwrap_or_default(),
            "0",
            window,
            cx,
        );

        let pos_items: Vec<SharedString> =
            POSITIONS.iter().map(|s| SharedString::from(*s)).collect();
        let pos_idx = settings
            .position
            .as_deref()
            .and_then(|p| POSITIONS.iter().position(|&x| x == p))
            .map(IndexPath::new);
        let position_select = cx.new(|cx| SelectState::new(pos_items, pos_idx, window, cx));

        let layer_items: Vec<SharedString> =
            LAYERS.iter().map(|s| SharedString::from(*s)).collect();
        let layer_idx = settings
            .layer
            .as_deref()
            .and_then(|l| LAYERS.iter().position(|&x| x == l))
            .map(IndexPath::new);
        let layer_select = cx.new(|cx| SelectState::new(layer_items, layer_idx, window, cx));

        let subscriptions = Self::build_subscriptions(
            profile_name,
            &height_input,
            &spacing_input,
            &output_input,
            &margin_top_input,
            &margin_right_input,
            &margin_bottom_input,
            &margin_left_input,
            &position_select,
            &layer_select,
            window,
            cx,
        );

        Self {
            profile_name: profile_name.to_string(),
            is_read_only,
            settings,
            expanded: false,
            height_input,
            spacing_input,
            output_input,
            margin_top_input,
            margin_right_input,
            margin_bottom_input,
            margin_left_input,
            position_select,
            layer_select,
            _subscriptions: subscriptions,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_subscriptions(
        profile_name: &str,
        height_input: &Entity<InputState>,
        spacing_input: &Entity<InputState>,
        output_input: &Entity<InputState>,
        margin_top_input: &Entity<InputState>,
        margin_right_input: &Entity<InputState>,
        margin_bottom_input: &Entity<InputState>,
        margin_left_input: &Entity<InputState>,
        position_select: &Entity<SelectState<Vec<SharedString>>>,
        layer_select: &Entity<SelectState<Vec<SharedString>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Subscription> {
        let profile = profile_name.to_string();

        // Macro: subscribe an input, read its value on Change, parse and save
        macro_rules! sub {
            ($input:expr, $key:literal, $parse:expr) => {{
                let p = profile.clone();
                let input_ref = $input.clone();
                cx.subscribe_in(
                    $input,
                    window,
                    move |_this, _input, event: &InputEvent, _window, cx| {
                        if matches!(event, InputEvent::Change) {
                            let val = input_ref.read(cx).value().to_string();
                            if let Some(v) = $parse(&val) {
                                if let Err(e) = set_bar_setting(&p, $key, &v) {
                                    eprintln!("Bar settings save error: {}", e);
                                }
                            }
                        }
                    },
                )
            }};
        }

        // Macro: subscribe a select, save selected value on Confirm
        macro_rules! sub_select {
            ($select:expr, $key:literal) => {{
                let p = profile.clone();
                cx.subscribe_in(
                    $select,
                    window,
                    move |_this, _select, event: &SelectEvent<Vec<SharedString>>, _window, _cx| {
                        if let SelectEvent::Confirm(Some(val)) = event {
                            let v = serde_json::Value::from(val.to_string());
                            if let Err(e) = set_bar_setting(&p, $key, &v) {
                                eprintln!("Bar settings save error: {}", e);
                            }
                        }
                    },
                )
            }};
        }

        // Macro: subscribe a number input to handle +/- button step events
        macro_rules! sub_step {
            ($input:expr, $signed:literal) => {{
                let input_ref = $input.clone();
                cx.subscribe_in(
                    $input,
                    window,
                    move |_this, _input, event: &NumberInputEvent, window, cx| {
                        let NumberInputEvent::Step(action) = event;
                        input_ref.update(cx, |state, cx| {
                            let val = state.value().to_string();
                            if $signed {
                                let n = val.trim().parse::<i64>().unwrap_or(0);
                                let new_val = if *action == StepAction::Increment {
                                    n + 1
                                } else {
                                    n - 1
                                };
                                state.set_value(
                                    SharedString::from(new_val.to_string()),
                                    window,
                                    cx,
                                );
                            } else {
                                let n = val.trim().parse::<u64>().unwrap_or(0);
                                let new_val = if *action == StepAction::Increment {
                                    n + 1
                                } else {
                                    n.saturating_sub(1)
                                };
                                state.set_value(
                                    SharedString::from(new_val.to_string()),
                                    window,
                                    cx,
                                );
                            }
                        });
                    },
                )
            }};
        }

        vec![
            sub!(height_input, "height", |s: &str| {
                s.trim().parse::<u64>().ok().map(serde_json::Value::from)
            }),
            sub_step!(height_input, false),
            sub!(spacing_input, "spacing", |s: &str| {
                s.trim().parse::<u64>().ok().map(serde_json::Value::from)
            }),
            sub_step!(spacing_input, false),
            sub!(output_input, "output", |s: &str| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(serde_json::Value::from(t.to_string()))
                }
            }),
            sub!(margin_top_input, "margin-top", |s: &str| {
                s.trim().parse::<i64>().ok().map(serde_json::Value::from)
            }),
            sub_step!(margin_top_input, true),
            sub!(margin_right_input, "margin-right", |s: &str| {
                s.trim().parse::<i64>().ok().map(serde_json::Value::from)
            }),
            sub_step!(margin_right_input, true),
            sub!(margin_bottom_input, "margin-bottom", |s: &str| {
                s.trim().parse::<i64>().ok().map(serde_json::Value::from)
            }),
            sub_step!(margin_bottom_input, true),
            sub!(margin_left_input, "margin-left", |s: &str| {
                s.trim().parse::<i64>().ok().map(serde_json::Value::from)
            }),
            sub_step!(margin_left_input, true),
            sub_select!(position_select, "position"),
            sub_select!(layer_select, "layer"),
        ]
    }

    pub fn reload(
        &mut self,
        profile_name: &str,
        is_read_only: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.profile_name = profile_name.to_string();
        self.is_read_only = is_read_only;
        let settings = load_settings(profile_name, is_read_only).unwrap_or_default();

        macro_rules! update_input {
            ($field:expr, $val:expr) => {
                $field.update(cx, |state, cx| {
                    state.set_value(&$val, window, cx);
                });
            };
        }

        update_input!(
            self.height_input,
            settings.height.map(|v| v.to_string()).unwrap_or_default()
        );
        update_input!(
            self.spacing_input,
            settings.spacing.map(|v| v.to_string()).unwrap_or_default()
        );
        update_input!(
            self.output_input,
            settings.output.clone().unwrap_or_default()
        );
        update_input!(
            self.margin_top_input,
            settings
                .margin_top
                .map(|v| v.to_string())
                .unwrap_or_default()
        );
        update_input!(
            self.margin_right_input,
            settings
                .margin_right
                .map(|v| v.to_string())
                .unwrap_or_default()
        );
        update_input!(
            self.margin_bottom_input,
            settings
                .margin_bottom
                .map(|v| v.to_string())
                .unwrap_or_default()
        );
        update_input!(
            self.margin_left_input,
            settings
                .margin_left
                .map(|v| v.to_string())
                .unwrap_or_default()
        );

        let pos_idx = settings
            .position
            .as_deref()
            .and_then(|p| POSITIONS.iter().position(|&x| x == p))
            .map(IndexPath::new);
        self.position_select.update(cx, |state, cx| {
            state.set_selected_index(pos_idx, window, cx);
        });

        let layer_idx = settings
            .layer
            .as_deref()
            .and_then(|l| LAYERS.iter().position(|&x| x == l))
            .map(IndexPath::new);
        self.layer_select.update(cx, |state, cx| {
            state.set_selected_index(layer_idx, window, cx);
        });

        self.settings = settings;

        // Rebuild subscriptions so they point at the new profile name
        self._subscriptions = Self::build_subscriptions(
            profile_name,
            &self.height_input,
            &self.spacing_input,
            &self.output_input,
            &self.margin_top_input,
            &self.margin_right_input,
            &self.margin_bottom_input,
            &self.margin_left_input,
            &self.position_select,
            &self.layer_select,
            window,
            cx,
        );
    }
}

impl Render for BarSettingsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let expanded = self.expanded;

        let header = h_flex()
            .id("bar-settings-header")
            .w_full()
            .gap_2()
            .items_center()
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.expanded = !this.expanded;
                cx.notify();
            }))
            .child(
                Icon::new(if expanded {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                })
                .text_color(theme.muted_foreground),
            )
            .child(
                div()
                    .text_sm()
                    .font_semibold()
                    .text_color(theme.foreground)
                    .child("Bar Settings"),
            );

        let body: AnyElement = if expanded {
            let pos_entity = self.position_select.clone();
            let layer_entity = self.layer_select.clone();
            let read_only = self.is_read_only;

            // Position + Layer row
            let selects_row = h_flex()
                .gap_4()
                .flex_wrap()
                .child(
                    v_flex()
                        .gap_1()
                        .w(px(160.))
                        .child(
                            Label::new("Position")
                                .text_sm()
                                .text_color(theme.muted_foreground),
                        )
                        .child(Select::new(&pos_entity).small().disabled(read_only)),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .w(px(160.))
                        .child(
                            Label::new("Layer")
                                .text_sm()
                                .text_color(theme.muted_foreground),
                        )
                        .child(Select::new(&layer_entity).small().disabled(read_only)),
                );

            // Height + Spacing row
            let sizes_row = h_flex()
                .gap_4()
                .flex_wrap()
                .child(
                    v_flex()
                        .gap_1()
                        .w(px(160.))
                        .child(
                            Label::new("Height (px)")
                                .text_sm()
                                .text_color(theme.muted_foreground),
                        )
                        .child(
                            NumberInput::new(&self.height_input)
                                .small()
                                .disabled(read_only),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .w(px(160.))
                        .child(
                            Label::new("Spacing (px)")
                                .text_sm()
                                .text_color(theme.muted_foreground),
                        )
                        .child(
                            NumberInput::new(&self.spacing_input)
                                .small()
                                .disabled(read_only),
                        ),
                );

            // Output field
            let output_row = labeled_input(
                "Output",
                &self.output_input,
                theme.muted_foreground,
                read_only,
            );

            // Margins row
            let margins_row = h_flex()
                .gap_4()
                .flex_wrap()
                .child(
                    v_flex()
                        .gap_1()
                        .w(px(160.))
                        .child(
                            Label::new("Margin Top")
                                .text_sm()
                                .text_color(theme.muted_foreground),
                        )
                        .child(
                            NumberInput::new(&self.margin_top_input)
                                .small()
                                .disabled(read_only),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .w(px(160.))
                        .child(
                            Label::new("Margin Right")
                                .text_sm()
                                .text_color(theme.muted_foreground),
                        )
                        .child(
                            NumberInput::new(&self.margin_right_input)
                                .small()
                                .disabled(read_only),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .w(px(160.))
                        .child(
                            Label::new("Margin Bottom")
                                .text_sm()
                                .text_color(theme.muted_foreground),
                        )
                        .child(
                            NumberInput::new(&self.margin_bottom_input)
                                .small()
                                .disabled(read_only),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .w(px(160.))
                        .child(
                            Label::new("Margin Left")
                                .text_sm()
                                .text_color(theme.muted_foreground),
                        )
                        .child(
                            NumberInput::new(&self.margin_left_input)
                                .small()
                                .disabled(read_only),
                        ),
                );

            v_flex()
                .pt_3()
                .gap_4()
                .when(read_only, |this: gpui::Div| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Bar settings are shown for reference until you import or manage this config."),
                    )
                })
                .child(selects_row)
                .child(sizes_row)
                .child(output_row)
                .child(margins_row)
                .into_any()
        } else {
            div().into_any()
        };

        v_flex()
            .w_full()
            .p_3()
            .gap_1()
            .border_1()
            .border_color(theme.border)
            .rounded_md()
            .child(header)
            .child(body)
    }
}

fn load_settings(profile_name: &str, is_read_only: bool) -> Option<BarSettings> {
    let _ = is_read_only;
    get_bar_settings(profile_name)
}
