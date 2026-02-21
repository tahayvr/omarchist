use gpui::*;
use gpui_component::{v_flex, ActiveTheme};

use crate::ui::status_bar_page::waybar_preview::WaybarPreview;

pub struct DesignArea {
    preview: Entity<WaybarPreview>,
}

impl DesignArea {
    pub fn new(profile_name: &str, cx: &mut Context<Self>) -> Self {
        let name = profile_name.to_string();
        let preview = cx.new(|_| WaybarPreview::new(&name));
        Self { preview }
    }

    pub fn switch_profile(&mut self, profile_name: &str, cx: &mut Context<Self>) {
        self.preview.update(cx, |preview, _| {
            preview.reload(profile_name);
        });
    }
}

impl Render for DesignArea {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .w_full()
            .flex_1()
            .p_4()
            .gap_4()
            .border_1()
            .border_color(theme.border)
            .rounded_md()
            .child(self.preview.clone())
    }
}
