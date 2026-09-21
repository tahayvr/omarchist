//! Pieces the Flows page and the editor share for moving flows around: the
//! export prompt and the warning banner.
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, WindowExt, h_flex};

use crate::system::flows::Flow;
use crate::system::flows::share::{export_file_name, export_toml};

/// Asks where to save the flow as a shared file, then writes it and reports
/// in a notification. The prompt starts in the Downloads folder.
pub fn export_flow<V: 'static>(flow: &Flow, window: &mut Window, cx: &mut Context<V>) {
    let text = match export_toml(flow) {
        Ok(text) => text,
        Err(e) => {
            window.push_notification(format!("Could not export the flow: {e}"), cx);
            return;
        }
    };
    let dir = dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_default();
    let receiver = cx.prompt_for_new_path(&dir, Some(&export_file_name(flow)));
    cx.spawn_in(window, async move |this, cx| {
        if let Ok(Ok(Some(path))) = receiver.await {
            let written = std::fs::write(&path, text);
            this.update_in(cx, |_, window, cx| match written {
                Ok(()) => window.push_notification(format!("Exported to {}", path.display()), cx),
                Err(e) => window.push_notification(format!("Could not write the file: {e}"), cx),
            })
            .ok();
        }
    })
    .detach();
}

/// A one-line notice in the warning colour with a triangle icon.
pub fn warning_banner(text: String, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    h_flex()
        .gap_2()
        .items_center()
        .px_3()
        .py_2()
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.warning.opacity(0.5))
        .bg(theme.warning.opacity(0.08))
        .text_sm()
        .child(
            Icon::new(IconName::TriangleAlert)
                .size_4()
                .text_color(theme.warning),
        )
        .child(text)
}
