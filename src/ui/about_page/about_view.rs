use gpui::*;
use gpui_component::{Icon, IconName, button::*, h_flex, v_flex};

pub struct AboutView;

impl Render for AboutView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .size_full()
            .items_center()
            .justify_center()
            .text_center()
            .child(
                img("logo/omarchist.png")
                    .w(px(128.))
                    .h(px(128.))
                    .object_fit(ObjectFit::Contain),
            )
            .child("v0.1.0")
            .child(
                h_flex()
                    .gap_4()
                    .child(
                        Button::new("btn")
                            .icon(Icon::new(Icon::empty()).path("icons/x.svg"))
                            .cursor_pointer()
                            .on_click(|_, _, cx| cx.open_url("https://x.com/tahayvr/")),
                    )
                    .child(
                        Button::new("github")
                            .icon(IconName::GitHub)
                            .cursor_pointer()
                            .on_click(|_, _, cx| {
                                cx.open_url("https://github.com/tahayvr/omarchist-rs")
                            }),
                    ),
            )
    }
}
