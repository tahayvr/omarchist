use gpui::*;
use gpui_component::v_flex;

const KEY_CONTEXT: &str = "DashboardView";

/// The page the app opens on. Empty for now; its content is designed next.
pub struct DashboardView {
    pub focus_handle: FocusHandle,
}

impl DashboardView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }

    /// Nothing on the page takes focus yet, so the page itself does, which
    /// keeps Escape toggling between the sidebar and the page.
    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
    }
}

impl Render for DashboardView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("dashboard-view")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
    }
}
