use gpui::*;
use gpui_component::{h_flex, ActiveTheme};

#[derive(IntoElement)]
pub struct StatusBarHeader;

impl RenderOnce for StatusBarHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        h_flex()
            .w_full()
            .p_4()
            .border_1()
            .border_color(theme.border)
            .rounded_md()
            .child(
                div()
                    .text_color(theme.foreground)
                    .font_weight(FontWeight::BOLD)
                    .child("Status Bar"),
            )
    }
}
