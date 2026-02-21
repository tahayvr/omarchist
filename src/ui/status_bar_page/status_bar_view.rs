use gpui::*;
use gpui_component::v_flex;

use crate::ui::status_bar_page::design_area::DesignArea;
use crate::ui::status_bar_page::header::StatusBarHeader;

pub struct StatusBarView {
    header: Entity<StatusBarHeader>,
    design_area: Entity<DesignArea>,
}

impl StatusBarView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let header = cx.new(|cx| StatusBarHeader::new(window, cx));
        let design_area = cx.new(|cx| DesignArea::new("omarchy-default", cx));
        Self {
            header,
            design_area,
        }
    }
}

impl Render for StatusBarView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("status-bar-page")
            .size_full()
            .gap_4()
            .child(self.header.clone())
            .child(self.design_area.clone())
    }
}
