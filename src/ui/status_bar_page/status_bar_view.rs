use gpui::*;
use gpui_component::v_flex;

use crate::ui::status_bar_page::design_area::DesignArea;
use crate::ui::status_bar_page::header::StatusBarHeader;

pub struct StatusBarView;

impl Render for StatusBarView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("status-bar-page")
            .size_full()
            .gap_4()
            .child(StatusBarHeader)
            .child(DesignArea)
    }
}
