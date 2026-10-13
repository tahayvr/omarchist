//! The Omarchy dialog: the installed version, what the update check found,
//! and the latest release notes, over whatever page is shown.

use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    scroll::ScrollableElement,
    text::{TextView, TextViewStyle},
    v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::omarchy::release_notes::fetch_latest_release_notes;
use crate::ui::focus;
use crate::ui::heading;
use crate::ui::notify;
use crate::ui::omarchy_page::updates::{OmarchyUpdates, UpdateState};
use crate::ui::text::selectable;

/// Opens the dialog and checks for updates again if the last check is old.
pub fn open_omarchy_dialog(updates: Entity<OmarchyUpdates>, window: &mut Window, cx: &mut App) {
    updates.update(cx, |updates, cx| updates.refresh_if_stale(cx));
    let panel = cx.new(|cx| OmarchyPanel::new(updates, cx));
    let trap_focus = cx.focus_handle();
    let notes = panel.read(cx).notes_focus.clone();
    window.open_dialog(cx, move |dialog, window, _| {
        dialog
            .title("Omarchy")
            .w(focus::dialog_width(760., window))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(true)
            .child(
                focus::dialog_body("omarchy-dialog", &trap_focus, |window, cx| {
                    window.close_dialog(cx);
                })
                .child(panel.clone()),
            )
    });
    // The notes are the first tab stop, so the arrow and page keys scroll
    // them at once, as in the shortcuts dialog.
    notes.focus(window, cx);
    window.on_next_frame(move |window, cx| notes.focus(window, cx));
}

/// The notes' height: what the window leaves under the title, the logo and
/// the update status, 420 px at most.
fn notes_height(window: &Window) -> Pixels {
    let viewport: f32 = window.viewport_size().height.into();
    px(420f32.min(viewport - 380.).max(120.))
}

pub struct OmarchyPanel {
    updates: Entity<OmarchyUpdates>,
    latest_tag: Option<String>,
    release_notes: Option<String>,
    release_notes_error: Option<String>,
    notes_focus: FocusHandle,
    notes_scroll: ScrollHandle,
    _updates_observer: Subscription,
}

impl OmarchyPanel {
    fn new(updates: Entity<OmarchyUpdates>, cx: &mut Context<Self>) -> Self {
        let observer = cx.observe(&updates, |_, _, cx| cx.notify());
        let mut panel = Self {
            updates,
            latest_tag: None,
            release_notes: None,
            release_notes_error: None,
            notes_focus: focus::tab_stop(cx),
            notes_scroll: ScrollHandle::new(),
            _updates_observer: observer,
        };
        panel.load_notes(cx);
        panel
    }

    /// Fetches the release notes; run when the dialog opens and again from
    /// "Check again" and Ctrl+R, so a dialog opened offline recovers.
    fn load_notes(&mut self, cx: &mut Context<Self>) {
        self.release_notes_error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let fetched = fetch_latest_release_notes().await;
            this.update(cx, |this, cx| {
                match fetched {
                    Ok((tag, notes)) => {
                        this.latest_tag = Some(tag);
                        this.release_notes = Some(notes);
                    }
                    Err(e) => this.release_notes_error = Some(e.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn run_update(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match crate::shell::omarchy_sh_commands::launch_omarchy_update() {
            Ok(()) => self
                .updates
                .update(cx, |updates, cx| updates.watch_running_update(cx)),
            Err(e) => notify::error(window, format!("Could not start the update: {e}"), cx),
        }
    }

    fn check_again(&mut self, cx: &mut Context<Self>) {
        self.updates.update(cx, |updates, cx| updates.refresh(cx));
        if self.release_notes.is_none() {
            self.load_notes(cx);
        }
    }

    /// The version line and, under it, what the update check found.
    fn render_status(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let updates = self.updates.read(cx);
        let version = updates
            .version()
            .map(|v| format!("Version {v}"))
            .unwrap_or_else(|| "Version unknown".to_string());
        let muted = |id: &'static str, text: String| {
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(selectable(id, text))
        };

        let detail: AnyElement = match updates.state() {
            UpdateState::Checking => {
                muted("checking", "Checking for updates…".into()).into_any_element()
            }
            UpdateState::Updating => muted(
                "updating",
                "Updating… the status refreshes when omarchy-update finishes".into(),
            )
            .into_any_element(),
            UpdateState::UpToDate => h_flex()
                .gap_3()
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.green)
                        .font_weight(FontWeight::BOLD)
                        .child("Up to date"),
                )
                .child(
                    Button::new("check-updates")
                        .ghost()
                        .small()
                        .icon(Icon::new(Icon::empty()).path("icons/refresh-cw.svg"))
                        .tooltip("Check again")
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _, _, cx| this.check_again(cx))),
                )
                .into_any_element(),
            UpdateState::Available(pending) => v_flex()
                .gap_2()
                .items_center()
                .child(
                    h_flex()
                        .gap_4()
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.red)
                                .child("Update available"),
                        )
                        .child(
                            Button::new("update-omarchy")
                                .primary()
                                .small()
                                .label("Update Omarchy")
                                .cursor_pointer()
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.run_update(window, cx)),
                                ),
                        ),
                )
                .children(pending.iter().enumerate().map(|(ix, line)| {
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(selectable(("pending", ix), line.clone()))
                }))
                .into_any_element(),
            UpdateState::Failed(error) => v_flex()
                .gap_1()
                .items_center()
                .child(
                    h_flex()
                        .gap_3()
                        .child(muted("check-failed", "Couldn't check for updates".into()))
                        .child(
                            Button::new("check-updates")
                                .ghost()
                                .small()
                                .label("Try again")
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, _, cx| this.check_again(cx))),
                        ),
                )
                .child(muted("check-error", error.clone()))
                .into_any_element(),
        };

        v_flex()
            .gap_1()
            .items_center()
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(selectable("version", version)),
            )
            .child(detail)
            .into_any_element()
    }

    fn render_notes(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let Some(notes) = &self.release_notes else {
            let (label, detail) = match &self.release_notes_error {
                Some(error) => ("Release notes unavailable.", Some(error.clone())),
                None => ("Loading release notes…", None),
            };
            // Still the tab stop the dialog focuses, so focus never falls
            // through to the page behind.
            return v_flex()
                .id("omarchy-notes")
                .track_focus(&self.notes_focus)
                .h(notes_height(window))
                .gap_2()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child(selectable("notes-status", label)),
                )
                .children(detail.map(|detail| {
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(selectable("notes-error", detail))
                }))
                .into_any_element();
        };

        let is_dark = theme.mode.is_dark();
        let style = TextViewStyle {
            paragraph_gap: rems(0.75),
            heading_base_font_size: px(15.),
            highlight_theme: if is_dark {
                gpui_component::highlighter::HighlightTheme::default_dark()
            } else {
                gpui_component::highlighter::HighlightTheme::default_light()
            },
            is_dark,
            heading_font_size: Some(std::sync::Arc::new(|level, base_size| match level {
                1 => base_size * 1.45,
                2 => base_size * 1.25,
                3 => base_size * 1.1,
                _ => base_size,
            })),
            ..Default::default()
        };
        let tag = self.latest_tag.clone().unwrap_or_else(|| "Latest".into());
        let border = focus::focus_border(self.notes_focus.is_focused(window), theme.border, cx);

        v_flex()
            .gap_2()
            .w_full()
            .child(
                h_flex()
                    .gap_2()
                    .child(heading::section("Latest release notes", cx))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(selectable("release-notes-title", tag)),
                    ),
            )
            .child(
                div()
                    .id("omarchy-notes")
                    .h(notes_height(window))
                    .bg(theme.muted)
                    .border_1()
                    .border_color(border)
                    .rounded_lg()
                    .overflow_y_scroll()
                    .track_scroll(&self.notes_scroll)
                    .child(
                        focus::scroll_area(&self.notes_scroll)
                            .id("omarchy-notes-content")
                            .test_support()
                            .track_focus(&self.notes_focus)
                            .px_5()
                            .py_4()
                            .child(
                                TextView::markdown("release-notes", notes.clone())
                                    .style(style)
                                    .line_height(rems(1.6))
                                    .selectable(true),
                            ),
                    )
                    .vertical_scrollbar(&self.notes_scroll),
            )
            .into_any_element()
    }
}

impl Render for OmarchyPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.render_status(cx);
        let notes = self.render_notes(window, cx);
        v_flex()
            .id("omarchy-panel")
            .gap_4()
            .items_center()
            // Ctrl+R checks again here, not on the page behind the dialog.
            .on_action(cx.listener(|this, _: &focus::ReloadPage, _, cx| {
                cx.stop_propagation();
                this.updates.update(cx, |updates, cx| updates.refresh(cx));
                this.load_notes(cx);
            }))
            .child(
                // The Omarchy logo's own proportions.
                div()
                    .w(px(280.))
                    .h(px(66.))
                    .child(img("logo/omarchy-logo.svg").h(relative(1.)).max_w(px(280.))),
            )
            .child(status)
            .child(notes)
    }
}
