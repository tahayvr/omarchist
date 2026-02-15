use gpui::*;
use gpui_component::{ActiveTheme, Icon, v_flex};

use crate::system::omarchy_version::check_omarchy_update;

pub struct OmarchyView {
    local_version: String,
    update_available: Option<bool>,
}

impl OmarchyView {
    pub fn new(version: Option<String>, cx: &mut Context<Self>) -> Self {
        let local_version = version.unwrap_or_else(|| "unknown".to_string());
        
        // Clone for the async block to avoid redundant git calls
        let version_for_check = local_version.clone();
        
        // Spawn async task to check for updates
        cx.spawn(async move |this, cx| {
            match check_omarchy_update(&version_for_check).await {
                Ok(update_available) => {
                    this.update(cx, |this, _cx| {
                        this.update_available = Some(update_available);
                    }).ok();
                }
                Err(e) => {
                    eprintln!("Failed to check for updates: {e}");
                    this.update(cx, |this, _cx| {
                        this.update_available = Some(false);
                    }).ok();
                }
            }
        }).detach();
        
        Self {
            local_version,
            update_available: None,
        }
    }
}

impl Render for OmarchyView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        
        let version_status = match self.update_available {
            None => {
                // Still checking
                v_flex()
                    .gap_1()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(format!("Version {}", self.local_version))
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Checking for updates...")
                    )
            }
            Some(true) => {
                // Update available
                v_flex()
                    .gap_1()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(format!("Version {}", self.local_version))
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.red)
                            .child("Update available")
                    )
            }
            Some(false) => {
                // Up to date
                v_flex()
                    .gap_1()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(format!("Version {}", self.local_version))
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.green)
                            .child("Up to date")
                    )
            }
        };
        
        v_flex()
            .gap_4()
            .size_full()
            .items_center()
            .justify_start()
            .pt_8()
            .child(
                Icon::empty()
                    .path("logo/omarchy-logo.svg")
                    .size_128()
                    .text_color(theme.foreground),
            )
            .child(version_status)
    }
}
