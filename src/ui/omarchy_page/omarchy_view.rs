use gpui::*;
use gpui_component::{ActiveTheme, button::Button, h_flex, text::TextView, v_flex};

use crate::system::omarchy::{
    omarchy_version::check_omarchy_update, release_notes::fetch_latest_release_notes,
};

pub struct OmarchyView {
    local_version: String,
    update_available: Option<bool>,
    latest_tag: Option<String>,
    release_notes: Option<String>,
}

impl OmarchyView {
    pub fn new(version: Option<String>, cx: &mut Context<Self>) -> Self {
        let local_version = version.unwrap_or_else(|| "unknown".to_string());

        // Clone for the async block to avoid redundant git calls
        let version_for_check = local_version.clone();

        // Spawn async task to check for updates
        cx.spawn(
            async move |this, cx| match check_omarchy_update(&version_for_check).await {
                Ok(update_available) => {
                    this.update(cx, |this, _cx| {
                        this.update_available = Some(update_available);
                    })
                    .ok();
                }
                Err(e) => {
                    eprintln!("Failed to check for updates: {e}");
                    this.update(cx, |this, _cx| {
                        this.update_available = Some(false);
                    })
                    .ok();
                }
            },
        )
        .detach();

        // Spawn async task to fetch latest release notes
        cx.spawn(
            async move |this, cx| match fetch_latest_release_notes().await {
                Ok((tag, notes)) => {
                    this.update(cx, |this, _cx| {
                        this.latest_tag = Some(tag);
                        this.release_notes = Some(notes);
                    })
                    .ok();
                }
                Err(e) => {
                    eprintln!("Failed to fetch release notes: {e}");
                }
            },
        )
        .detach();

        Self {
            local_version,
            update_available: None,
            latest_tag: None,
            release_notes: None,
        }
    }
}

impl Render for OmarchyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .child(format!("Version {}", self.local_version)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Checking for updates..."),
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
                            .child(format!("Version {}", self.local_version)),
                    )
                    .child(
                        h_flex()
                            .gap_4()
                            .items_center()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.red)
                                    .child("Update available"),
                            )
                            .child(Button::new("update-omarchy")),
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
                            .child(format!("Version {}", self.local_version)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.green)
                            .font_weight(FontWeight::BOLD)
                            .child("Up to date"),
                    )
            }
        };

        let release_notes_section = if let Some(notes) = &self.release_notes {
            let tag = self
                .latest_tag
                .clone()
                .unwrap_or_else(|| "Latest".to_string());
            let markdown_view = TextView::markdown("release-notes", notes.clone(), window, cx);

            v_flex()
                .gap_2()
                .w_full()
                .flex_1()
                .min_h(px(0.))
                .child(
                    div()
                        .text_color(cx.theme().foreground)
                        .font_weight(FontWeight::BOLD)
                        .child(format!("{} Release Notes:", tag)),
                )
                .child(
                    div()
                        .id("release-notes-content")
                        .flex_1()
                        .p_4()
                        .bg(cx.theme().background)
                        .border_1()
                        .border_color(cx.theme().border)
                        .overflow_y_scroll()
                        .child(markdown_view),
                )
        } else {
            v_flex()
                .gap_2()
                .w_full()
                .flex_1()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("Loading release notes..."),
                )
        };

        v_flex()
            .id("omarchy-page")
            .gap_4()
            .size_full()
            .items_center()
            .justify_start()
            .pt_8()
            .px_4()
            .child(
                img("logo/omarchy-logo.svg")
                    .w(relative(1.)) // Full width
                    .max_w(px(400.))
                    .h(px(150.))
                    .text_color(cx.theme().foreground),
            )
            .child(version_status)
            .child(release_notes_section)
    }
}
