use gpui::*;
use gpui_component::{ActiveTheme, Icon, v_flex};

pub struct OmarchyView;

impl Render for OmarchyView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .size_full()
            .items_center()
            .justify_start()
            .child(
                Icon::empty()
                    .path("logo/omarchy-logo.svg")
                    .size_128()
                    .text_color(cx.theme().foreground),
            )
    }
}
