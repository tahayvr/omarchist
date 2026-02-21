use gpui::*;
use gpui_component::{v_flex, ActiveTheme};

#[derive(IntoElement)]
pub struct DesignArea;

impl RenderOnce for DesignArea {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .w_full()
            .flex_1()
            .p_4()
            .border_1()
            .border_color(theme.border)
            .rounded_md()
    }
}
