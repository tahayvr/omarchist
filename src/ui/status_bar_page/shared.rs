use gpui::*;
use gpui_component::{
    Sizable,
    input::{Input, InputState},
    label::Label,
    v_flex,
};

// Labeled input field with a fixed narrow width (160 px).
pub fn labeled_input(
    label: &str,
    input: &Entity<InputState>,
    label_color: Hsla,
) -> impl IntoElement {
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

// Labeled input field with a wider fixed width (340 px), for longer values.
pub fn labeled_input_wide(
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
