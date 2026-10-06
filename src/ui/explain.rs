//! A longer explanation lives off the page: it opens as a hover card over
//! its trigger (a label, an icon) and reads as one short paragraph. A
//! tooltip holds a few words; anything that is a sentence goes here.
use std::time::Duration;

use gpui::*;
use gpui_component::{ActiveTheme, hover_card::HoverCard};

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
