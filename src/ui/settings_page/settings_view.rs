use gpui::*;
use gpui_component::v_flex;

pub struct SettingsView;

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .child("Settings")
    }
}
