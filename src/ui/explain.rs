//! A longer explanation lives off the page: it opens as a hover card over
//! its trigger (a label, an icon) and reads as one short paragraph. A
//! tooltip holds a few words; anything that is a sentence goes here.
use std::time::Duration;

use gpui::*;
use gpui_component::{ActiveTheme, hover_card::HoverCard};

/// A label that has an explanation: a dashed underline says so, since
/// gpui has no help cursor to switch to. The row around it stays as it
/// is; the underline fits the words.
pub fn explained_label(text: impl Into<SharedString>, cx: &App) -> Div {
    div().flex().child(
        div()
            .text_sm()
            .border_b_1()
            .border_dashed()
            .border_color(cx.theme().muted_foreground.opacity(0.5))
            .child(text.into()),
    )
}

pub fn explain(
    id: impl Into<ElementId>,
    trigger: impl IntoElement + 'static,
    text: impl Into<SharedString>,
) -> HoverCard {
    let text = text.into();
    HoverCard::new(id)
        .anchor(Anchor::BottomLeft)
        .open_delay(Duration::from_millis(400))
        .close_delay(Duration::from_millis(150))
        .w(px(320.))
        .trigger(trigger)
        .content(move |_, _, cx| {
            div()
                .text_sm()
                .text_color(cx.theme().foreground)
                .child(text.clone())
        })
}
