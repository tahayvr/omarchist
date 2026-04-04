use std::cell::RefCell;

use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable, StyledExt, h_flex,
    input::{Input, InputEvent, InputState},
    label::Label,
    v_flex,
};

use crate::system::waybar::{get_module_config, set_module_config_field};

thread_local! {
    pub static PENDING_MODULE_EDIT: RefCell<Option<(String, String)>> = const { RefCell::new(None) };
}

pub fn request_module_edit(profile_name: String, module_key: String) {
    PENDING_MODULE_EDIT.with(|cell| {
        *cell.borrow_mut() = Some((profile_name, module_key));
    });
}

pub fn take_pending_module_edit() -> Option<(String, String)> {
    PENDING_MODULE_EDIT.with(|cell| cell.borrow_mut().take())
}

// ---------------------------------------------------------------------------
// ModuleEditorPanel
// ---------------------------------------------------------------------------

pub struct ModuleEditorPanel {
    profile_name: String,
    module_key: String,
    is_open: bool,

    format_input: Entity<InputState>,
    interval_input: Entity<InputState>,
    tooltip_format_input: Entity<InputState>,
    on_click_input: Entity<InputState>,
    max_length_input: Entity<InputState>,

    _subscriptions: Vec<Subscription>,
}

impl ModuleEditorPanel {
    pub fn new(profile_name: &str, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let format_input = cx.new(|cx| InputState::new(window, cx).placeholder("{icon}"));
        let interval_input = cx.new(|cx| InputState::new(window, cx).placeholder("seconds"));
        let tooltip_format_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("tooltip text / format"));
        let on_click_input = cx.new(|cx| InputState::new(window, cx).placeholder("shell command"));
        let max_length_input = cx.new(|cx| InputState::new(window, cx).placeholder("e.g. 50"));

        let subs = Self::build_subs(
            profile_name,
            "",
            &format_input,
            &interval_input,
            &tooltip_format_input,
            &on_click_input,
            &max_length_input,
            window,
            cx,
        );

        Self {
            profile_name: profile_name.to_string(),
            module_key: String::new(),
            is_open: false,
            format_input,
            interval_input,
            tooltip_format_input,
            on_click_input,
            max_length_input,
            _subscriptions: subs,
        }
    }

    pub fn open(&mut self, module_key: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.module_key = module_key.to_string();
        self.is_open = true;

        let cfg = get_module_config(&self.profile_name, module_key);

        let str_field = |key: &str| -> String {
            cfg.get(key)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string()
        };
        let num_field = |key: &str| -> String {
            cfg.get(key)
                .and_then(|v| v.as_i64())
                .map(|n| n.to_string())
                .unwrap_or_default()
        };

        self.format_input.update(cx, |s, cx| {
            s.set_value(str_field("format"), window, cx);
        });
        self.interval_input.update(cx, |s, cx| {
            s.set_value(num_field("interval"), window, cx);
        });
        self.tooltip_format_input.update(cx, |s, cx| {
            s.set_value(str_field("tooltip-format"), window, cx);
        });
        self.on_click_input.update(cx, |s, cx| {
            s.set_value(str_field("on-click"), window, cx);
        });
        self.max_length_input.update(cx, |s, cx| {
            s.set_value(num_field("max-length"), window, cx);
        });

        // Rebuild subscriptions for the new module key
        self._subscriptions = Self::build_subs(
            &self.profile_name,
            module_key,
            &self.format_input,
            &self.interval_input,
            &self.tooltip_format_input,
            &self.on_click_input,
            &self.max_length_input,
            window,
            cx,
        );

        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.is_open = false;
        cx.notify();
    }

    pub fn switch_profile(&mut self, profile_name: &str) {
        self.profile_name = profile_name.to_string();
        self.is_open = false;
    }

    #[allow(clippy::too_many_arguments)]
    fn build_subs(
        profile_name: &str,
        module_key: &str,
        format_input: &Entity<InputState>,
        interval_input: &Entity<InputState>,
        tooltip_format_input: &Entity<InputState>,
        on_click_input: &Entity<InputState>,
        max_length_input: &Entity<InputState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Subscription> {
        let profile = profile_name.to_string();
        let mkey = module_key.to_string();

        macro_rules! sub {
            ($input:expr, $field:literal, $parse:expr) => {{
                let p = profile.clone();
                let mk = mkey.clone();
                let input_ref = $input.clone();
                cx.subscribe_in(
                    $input,
                    window,
                    move |_this, _input, event: &InputEvent, _window, cx| {
                        if matches!(event, InputEvent::Change) && !mk.is_empty() {
                            let val = input_ref.read(cx).value().to_string();
                            if let Some(v) = $parse(&val) {
                                if let Err(e) = set_module_config_field(&p, &mk, $field, &v) {
                                    eprintln!("Module editor save error: {}", e);
                                }
                            }
                        }
                    },
                )
            }};
        }

        vec![
            sub!(format_input, "format", |s: &str| {
                Some(serde_json::Value::from(s.to_string()))
            }),
            sub!(interval_input, "interval", |s: &str| {
                // Allow clearing the field (empty = do not save)
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    t.parse::<i64>().ok().map(serde_json::Value::from)
                }
            }),
            sub!(tooltip_format_input, "tooltip-format", |s: &str| {
                Some(serde_json::Value::from(s.to_string()))
            }),
            sub!(on_click_input, "on-click", |s: &str| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(serde_json::Value::from(t.to_string()))
                }
            }),
            sub!(max_length_input, "max-length", |s: &str| {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    t.parse::<i64>().ok().map(serde_json::Value::from)
                }
            }),
        ]
    }
}

impl Render for ModuleEditorPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.is_open {
            return div().into_any();
        }

        let theme = cx.theme();
        let module_key = self.module_key.clone();

        let title_row = h_flex()
            .w_full()
            .gap_2()
            .items_center()
            .justify_between()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Icon::new(IconName::Settings).text_color(theme.muted_foreground))
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child(format!("Edit: {}", module_key)),
                    ),
            )
            .child(
                div()
                    .id("module-editor-close")
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .cursor_pointer()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .hover(|s| s.text_color(theme.foreground))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.close(cx);
                    }))
                    .child("✕"),
            );

        let help = div().text_xs().text_color(theme.muted_foreground).child(
            "Changes are saved immediately. Leave a field empty to inherit Waybar defaults.",
        );

        let fields_row = h_flex()
            .gap_4()
            .flex_wrap()
            .child(labeled_input(
                "Format",
                &self.format_input,
                theme.muted_foreground,
            ))
            .child(labeled_input(
                "Interval (s)",
                &self.interval_input,
                theme.muted_foreground,
            ))
            .child(labeled_input(
                "Max Length",
                &self.max_length_input,
                theme.muted_foreground,
            ));

        let fields_row2 = h_flex()
            .gap_4()
            .flex_wrap()
            .child(labeled_input_wide(
                "Tooltip Format",
                &self.tooltip_format_input,
                theme.muted_foreground,
            ))
            .child(labeled_input_wide(
                "On Click",
                &self.on_click_input,
                theme.muted_foreground,
            ));

        v_flex()
            .w_full()
            .p_3()
            .gap_3()
            .border_1()
            .border_color(theme.accent.opacity(0.6))
            .rounded_md()
            .bg(theme.secondary.opacity(0.3))
            .child(title_row)
            .child(help)
            .child(fields_row)
            .child(fields_row2)
            .into_any()
    }
}

fn labeled_input(label: &str, input: &Entity<InputState>, label_color: Hsla) -> impl IntoElement {
    v_flex()
        .gap_1()
        .w(px(160.))
        .child(
            Label::new(label.to_string())
                .text_sm()
                .text_color(label_color),
        )
        .child(Input::new(input).small())
}

fn labeled_input_wide(
    label: &str,
    input: &Entity<InputState>,
    label_color: Hsla,
) -> impl IntoElement {
    v_flex()
        .gap_1()
        .w(px(340.))
        .child(
            Label::new(label.to_string())
                .text_sm()
                .text_color(label_color),
        )
        .child(Input::new(input).small())
}
