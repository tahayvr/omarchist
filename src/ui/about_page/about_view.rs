use gpui::FontWeight;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, Sizable, button::*, h_flex, v_flex};

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
                v_flex()
                    .mt_6()
                    .gap_1()
                    .items_center()
                    .child(
                        div()
                            .text_size(px(42.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(cx.theme().foreground)
                            .line_height(relative(1.0))
                            .child("OMARCHIST"),
                    )
                    .child(
                        div()
                            .id("omarchist-version")
                            .text_color(cx.theme().muted_foreground)
                            .child("v0.2.1"),
                    ),
            )
            .child(
                h_flex()
                    .mt_12()
                    .gap_4()
                    .child(
                        Button::new("x-com")
                            .icon(Icon::new(Icon::empty()).path("icons/x.svg").size_8())
                            .ghost()
                            .cursor_pointer()
                            .large()
                            .on_click(|_, _, cx| cx.open_url("https://x.com/tahayvr/")),
                    )
                    .child(
                        Button::new("github")
                            .icon(Icon::new(IconName::GitHub).size_8())
                            .ghost()
                            .cursor_pointer()
                            .large()
                            .on_click(|_, _, cx| {
                                cx.open_url("https://github.com/tahayvr/omarchist-rs")
                            }),
                    ),
            )
    }
}
