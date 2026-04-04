use gpui::*;
use gpui_component::{
    ActiveTheme, Colorize,
    clipboard::Clipboard,
    color_picker::{ColorPicker, ColorPickerState},
    h_flex,
    input::{Input, InputState},
    label::Label,
    switch::Switch,
    v_flex,
};

pub struct FormField {
    label: String,
    input: Entity<InputState>,
}

impl FormField {
    pub fn new(
        label: &str,
        initial_value: impl Into<String>,
        placeholder: impl Into<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(initial_value.into())
                .placeholder(placeholder.into())
        });

        Self {
            label: label.to_string(),
            input,
        }
    }

    pub fn input(&self) -> &Entity<InputState> {
        &self.input
    }

    pub fn value(&self, cx: &App) -> String {
        self.input.read(cx).value().to_string()
    }
}

impl RenderOnce for FormField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(
                Label::new(&self.label)
                    .text_sm()
                    .text_color(cx.theme().muted_foreground),
            )
            .child(Input::new(&self.input).cleanable(true))
    }
}

type ToggleChangeCallback = Box<dyn Fn(bool, &mut Window, &mut App)>;

pub struct ToggleField {
    id: String,
    label: String,
    is_checked: bool,
    on_change: Option<ToggleChangeCallback>,
}

impl ToggleField {
    pub fn new(id: &str, label: &str, is_checked: bool) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            is_checked,
            on_change: None,
        }
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: Fn(bool, &mut Window, &mut App) + 'static,
    {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ToggleField {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let is_checked = self.is_checked;
        let on_change = self.on_change;
        let id: gpui::SharedString = self.id.into();

        h_flex()
            .gap_4()
            .items_center()
            .child(Label::new(&self.label))
            .child(
                Switch::new(id)
                    .checked(is_checked)
                    .cursor_pointer()
                    .on_click(move |checked, window, cx| {
                        if let Some(ref handler) = on_change {
                            handler(*checked, window, cx);
                        }
                    }),
            )
    }
}

pub fn form_section() -> Div {
    v_flex().gap_2()
}

pub fn help_text(text: impl Into<SharedString>) -> Div {
    div()
        .text_sm()
        .text_color(gpui::rgb(0x888888))
        .child(text.into())
}

pub trait TabInputHandler: Sized {
    fn on_input_change(
        &mut self,
        field_id: &str,
        value: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    );

    fn trigger_save(&mut self, window: &mut Window, cx: &mut Context<Self>);
}

pub fn tab_container() -> Div {
    v_flex().gap_6().pt_4().pb_4()
}

pub fn error_message(text: impl Into<SharedString>) -> Div {
    div()
        .p_2()
        .bg(gpui::rgb(0xffcccc))
        .border_1()
        .border_color(gpui::rgb(0xff0000))
        .child(
            div()
                .text_sm()
                .text_color(gpui::rgb(0xff0000))
                .child(text.into()),
        )
}

pub fn color_picker_with_clipboard(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    picker_state: &Entity<ColorPickerState>,
) -> impl IntoElement {
    let picker_state_clone = picker_state.clone();
    let label_text: SharedString = label.into();
    let id: SharedString = id.into();
    let clipboard_id: SharedString = format!("{}-clipboard", id).into();

    v_flex()
        .gap_2()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(Label::new(label_text).text_sm())
                .child(Clipboard::new(clipboard_id).value_fn(move |_, cx| {
                    picker_state_clone
                        .read(cx)
                        .value()
                        .map(|c| c.to_hex())
                        .unwrap_or_default()
                        .into()
                })),
        )
        .child(ColorPicker::new(picker_state))
}
