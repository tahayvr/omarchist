use gpui::*;
use gpui_component::v_flex;

pub struct StatusBarView;

impl Render for StatusBarView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .child("Status Bar")
    }
}
