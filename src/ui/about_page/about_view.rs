use gpui::FontWeight;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, button::*, h_flex, v_flex};

pub struct AboutView;

impl Render for AboutView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_0()
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
            .child(
                div()
                    .mt_6()
                    .text_3xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(cx.theme().foreground)
                    .child("OMARCHIST"),
            )
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child("v0.1.0"),
            )
            .child(
                h_flex()
                    .mt_12()
                    .gap_4()
                    .child(
                        Button::new("x-com")
                            .icon(Icon::new(Icon::empty()).path("icons/x.svg"))
                            .outline()
                            .cursor_pointer()
                            .on_click(|_, _, cx| cx.open_url("https://x.com/tahayvr/")),
                    )
                    .child(
                        Button::new("github")
                            .icon(IconName::GitHub)
                            .outline()
                            .cursor_pointer()
                            .on_click(|_, _, cx| {
                                cx.open_url("https://github.com/tahayvr/omarchist-rs")
                            }),
                    ),
            )
    }
}
