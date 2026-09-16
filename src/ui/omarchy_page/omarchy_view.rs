use gpui::*;
use gpui_component::{
    ActiveTheme, button::Button, h_flex, text::TextView, text::TextViewStyle, v_flex,
};

use crate::system::omarchy::{
    omarchy_version::{check_omarchy_update, get_local_omarchy_version},
    release_notes::fetch_latest_release_notes,
};
use crate::ui::menu::title_bar::MainTitleBar;

const KEY_CONTEXT: &str = "OmarchyView";
const RELEASE_NOTES_CONTEXT: &str = "ReleaseNotes";
const SCROLL_STEP: f32 = 48.0;

actions!(
    release_notes,
    [ScrollUp, ScrollDown, PageUp, PageDown, Top, Bottom]
);

const POST_UPDATE_RECHECK_SECS: u64 = 60;

pub struct OmarchyView {
    local_version: String,
    update_available: Option<bool>,
    latest_tag: Option<String>,
    release_notes: Option<String>,
    pub focus_handle: FocusHandle,
    notes_focus: FocusHandle,
    notes_scroll: ScrollHandle,
}

impl OmarchyView {
    pub fn new(
        version: Option<String>,
        title_bar: Entity<MainTitleBar>,
        cx: &mut Context<Self>,
    ) -> Self {
        let local_version = version.unwrap_or_else(|| "unknown".to_string());

        // Spawn async task to check for updates and update both the in-page
        // display and the title bar badge.
        Self::spawn_version_check(local_version.clone(), title_bar.clone(), cx);

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

        // Note: the 30-minute periodic version-check loop lives in
        // MainWindowView::new() (via the background spawn) so the
        // title-bar badge stays fresh even if this page is never opened.

        Self {
            local_version,
            update_available: None,
            latest_tag: None,
            release_notes: None,
            focus_handle: cx.focus_handle(),
            notes_focus: crate::ui::focus::tab_stop(cx),
            notes_scroll: ScrollHandle::new(),
        }
    }

    /// Focuses the first control on the page.
    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        crate::ui::focus::focus_first_in(&self.focus_handle, window, cx);
    }

    fn scroll_notes_by(&self, delta: f32, cx: &mut Context<Self>) {
        let mut offset = self.notes_scroll.offset();
        let max = self.notes_scroll.max_offset().y;
        offset.y = (offset.y - px(delta)).clamp(-max, px(0.));
        self.notes_scroll.set_offset(offset);
        cx.notify();
    }

    fn scroll_notes_to(&self, top: bool, cx: &mut Context<Self>) {
        let mut offset = self.notes_scroll.offset();
        offset.y = if top {
            px(0.)
        } else {
            -self.notes_scroll.max_offset().y
        };
        self.notes_scroll.set_offset(offset);
        cx.notify();
    }

    fn spawn_version_check(
        version: String,
        title_bar: Entity<MainTitleBar>,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(
            async move |this, cx| match check_omarchy_update(&version).await {
                Ok(update_available) => {
                    this.update(cx, |this, _cx| {
                        this.update_available = Some(update_available);
                    })
                    .ok();
                    title_bar.update(cx, |tb, _| {
                        tb.set_omarchy_update_available(update_available);
                    });
                }
                Err(e) => {
                    eprintln!("Failed to check for omarchy updates: {e}");
                    this.update(cx, |this, _cx| {
                        this.update_available = Some(false);
                    })
                    .ok();
                }
            },
        )
        .detach();
    }

    fn spawn_delayed_recheck(cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            smol::Timer::after(std::time::Duration::from_secs(POST_UPDATE_RECHECK_SECS)).await;

            // Re-read local version (the update script will have changed git tags)
            let current_version =
                get_local_omarchy_version().unwrap_or_else(|_| "unknown".to_string());

            // Update local_version and set to checking state
            this.update(cx, |this, _cx| {
                this.local_version = current_version.clone();
                this.update_available = None;
            })
            .ok();

            match check_omarchy_update(&current_version).await {
                Ok(update_available) => {
                    this.update(cx, |this, _cx| {
                        this.update_available = Some(update_available);
                    })
                    .ok();
                    crate::ui::app_events::emit_async(
                        cx,
                        crate::ui::app_events::AppEvent::OmarchyUpdateStatus(update_available),
                    );
                }
                Err(e) => {
                    eprintln!("Post-update omarchy version check failed: {e}");
                    this.update(cx, |this, _cx| {
                        this.update_available = Some(false);
                    })
                    .ok();
                }
            }
        })
        .detach();
    }
}

impl Render for OmarchyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let notes_focused = self.notes_focus.is_focused(window);
        let notes_border = crate::ui::focus::focus_border(notes_focused, theme.border, cx);
        let page_height = self.notes_scroll.bounds().size.height;

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
                                Button::new("update-omarchy")
                                    .label("Update Omarchy")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                    if let Err(e) =
                                        crate::shell::omarchy_sh_commands::launch_omarchy_update()
                                    {
                                        eprintln!("{e}");
                                    } else {
                                        // Show "Checking..." immediately while the update runs
                                        this.update_available = None;
                                        cx.notify();
                                        // Schedule a re-check after the update has had time to complete
                                        Self::spawn_delayed_recheck(cx);
                                    }
                                })),
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
                        h_flex().gap_4().items_center().child(
                            div()
                                .text_xs()
                                .text_color(theme.green)
                                .font_weight(FontWeight::BOLD)
                                .child("Up to date"),
                        ),
                    )
            }
        };

        let release_notes_section = if let Some(notes) = &self.release_notes {
            let tag = self
                .latest_tag
                .clone()
                .unwrap_or_else(|| "Latest".to_string());

            let is_dark = cx.theme().mode.is_dark();
            let highlight_theme = if is_dark {
                gpui_component::highlighter::HighlightTheme::default_dark()
            } else {
                gpui_component::highlighter::HighlightTheme::default_light()
            };

            let style = TextViewStyle {
                paragraph_gap: rems(0.75),
                heading_base_font_size: px(15.),
                highlight_theme,
                heading_font_size: Some(std::sync::Arc::new(|level, base_size| match level {
                    1 => base_size * 1.45,
                    2 => base_size * 1.25,
                    3 => base_size * 1.1,
                    _ => base_size,
                })),
                ..Default::default()
            };

            let markdown_view = TextView::markdown("release-notes", notes.clone())
                .style(style)
                .line_height(rems(1.6))
                .selectable(true);

            v_flex()
                .gap_2()
                .w_full()
                .flex_1()
                .min_h(px(0.))
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .w(px(3.))
                                .h(px(16.))
                                .rounded_full()
                                .bg(cx.theme().accent_foreground),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(cx.theme().foreground)
                                .child(format!("Release Notes  ·  {}", tag)),
                        ),
                )
                .child(
                    div()
                        .id("release-notes-content")
                        .key_context(RELEASE_NOTES_CONTEXT)
                        .track_focus(&self.notes_focus)
                        .on_action(cx.listener(|this, _: &ScrollUp, _, cx| {
                            this.scroll_notes_by(-SCROLL_STEP, cx)
                        }))
                        .on_action(cx.listener(|this, _: &ScrollDown, _, cx| {
                            this.scroll_notes_by(SCROLL_STEP, cx)
                        }))
                        .on_action(cx.listener(move |this, _: &PageUp, _, cx| {
                            this.scroll_notes_by(-f32::from(page_height) * 0.9, cx)
                        }))
                        .on_action(cx.listener(move |this, _: &PageDown, _, cx| {
                            this.scroll_notes_by(f32::from(page_height) * 0.9, cx)
                        }))
                        .on_action(
                            cx.listener(|this, _: &Top, _, cx| this.scroll_notes_to(true, cx)),
                        )
                        .on_action(
                            cx.listener(|this, _: &Bottom, _, cx| this.scroll_notes_to(false, cx)),
                        )
                        .flex_1()
                        .min_h(px(0.))
                        .px_5()
                        .py_4()
                        .bg(cx.theme().muted)
                        .border_1()
                        .border_color(notes_border)
                        .rounded_lg()
                        .overflow_y_scroll()
                        .track_scroll(&self.notes_scroll)
                        .child(div().w_full().pb_2().child(markdown_view)),
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
