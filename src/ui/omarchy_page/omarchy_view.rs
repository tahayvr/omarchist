use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, button::Button, h_flex, text::TextView, text::TextViewStyle, v_flex,
};

use crate::system::omarchy::{
    omarchy_version::check_omarchy_update, release_notes::fetch_latest_release_notes,
};
use crate::ui::menu::app_menu;

const KEY_CONTEXT: &str = "OmarchyView";

pub struct OmarchyView {
    local_version: String,
    update_available: Option<bool>,
    latest_tag: Option<String>,
    release_notes: Option<String>,
    focus_handle: FocusHandle,
    /// Whether the Update button is keyboard-focused (only relevant when update_available == Some(true))
    update_btn_focused: bool,
}

impl OmarchyView {
    pub fn new(version: Option<String>, cx: &mut Context<Self>) -> Self {
        let local_version = version.unwrap_or_else(|| "unknown".to_string()); // Clone for the async block to avoid redundant git calls
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
            focus_handle: cx.focus_handle(),
            update_btn_focused: false,
        }
    }
}

impl Render for OmarchyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let update_available = self.update_available == Some(true);
        let update_btn_focused = self.update_btn_focused && update_available;
        let focused_border = theme.ring;

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
                            .child(
                                div()
                                    .rounded_md()
                                    .when(update_btn_focused, move |this: gpui::Div| {
                                        this.border_2().border_color(focused_border)
                                    })
                                    .child(
                                        Button::new("update-omarchy")
                                            .label("Update Omarchy")
                                            .on_click(|_, _, _| {
                                                if let Err(e) = crate::shell::omarchy_sh_commands::launch_omarchy_update() {
                                                    eprintln!("{e}");
                                                }
                                            }),
                                    ),
                            ),
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
                        // for testing only TODO: remove for prod
                        h_flex().gap_4().items_center().child(
                            div()
                                .text_xs()
                                .text_color(theme.green)
                                .font_weight(FontWeight::BOLD)
                                .child("Up to date"),
                        ), // .child(
                           //     Button::new("update-omarchy-test")
                           //         .label("Update Omarchy")
                           //         .on_click(|_, _, _| {
                           //             if let Err(e) = crate::shell::omarchy_sh_commands::launch_omarchy_update() {
                           //                 eprintln!("{e}");
                           //             }
                           //         }),
                           // ),
                    )
            }
        };

        let release_notes_section = if let Some(notes) = &self.release_notes {
            let tag = self
                .latest_tag
                .clone()
                .unwrap_or_else(|| "Latest".to_string());
            let style = TextViewStyle::default()
                .paragraph_gap(rems(2.0))
                .heading_font_size(|level, base_size| {
                    // Scale headings: H1 = 1.8x, H2 = 1.5x, H3 = 1.3x, H4+ = 1.1x
                    let scale = match level {
                        1 => 1.7,
                        2 => 1.5,
                        3 => 1.3,
                        _ => 1.1,
                    };
                    base_size * scale
                });

            let markdown_view = TextView::markdown("release-notes", notes.clone(), window, cx)
                .style(style)
                .line_height(rems(2.0));

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
                        .child(div().w_full().h_full().child(markdown_view)),
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
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .gap_4()
            .size_full()
            .items_center()
            .justify_start()
            .pt_8()
            .px_4()
            .on_action(cx.listener(|this, _: &app_menu::NextFocus, _window, cx| {
                if this.update_available == Some(true) {
                    this.update_btn_focused = true;
                    cx.notify();
                }
            }))
            .on_action(cx.listener(|this, _: &app_menu::PrevFocus, _window, cx| {
                if this.update_available == Some(true) {
                    this.update_btn_focused = true;
                    cx.notify();
                }
            }))
            .on_action(
                cx.listener(|this, _: &app_menu::ActivateItem, _window, _cx| {
                    if this.update_btn_focused {
                        if let Err(e) = crate::shell::omarchy_sh_commands::launch_omarchy_update() {
                            eprintln!("{e}");
                        }
                    }
                }),
            )
            .on_action(cx.listener(|this, _: &app_menu::EscapeFocus, _window, cx| {
                this.update_btn_focused = false;
                cx.notify();
            }))
            .child(
                div()
                    // set to specific dimensions of Omarchy logo
                    .w(px(400.))
                    .h(px(94.))
                    .child(img("logo/omarchy-logo.svg").h(relative(1.)).max_w(px(400.))),
            )
            .child(version_status)
            .child(release_notes_section)
    }
}
