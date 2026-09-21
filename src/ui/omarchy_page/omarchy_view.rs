use gpui::*;
use gpui_component::{
    ActiveTheme, Sizable, button::Button, button::ButtonVariants, h_flex, text::TextView,
    text::TextViewStyle, v_flex,
};

use crate::system::omarchy::release_notes::fetch_latest_release_notes;
use crate::ui::omarchy_page::updates::{OmarchyUpdates, UpdateState};

const KEY_CONTEXT: &str = "OmarchyView";
const RELEASE_NOTES_CONTEXT: &str = "ReleaseNotes";
const SCROLL_STEP: f32 = 48.0;

actions!(
    release_notes,
    [ScrollUp, ScrollDown, PageUp, PageDown, Top, Bottom]
);

pub struct OmarchyView {
    updates: Entity<OmarchyUpdates>,
    latest_tag: Option<String>,
    release_notes: Option<String>,
    release_notes_error: Option<String>,
    pub focus_handle: FocusHandle,
    notes_focus: FocusHandle,
    notes_scroll: ScrollHandle,
    _updates_observer: Subscription,
}

impl OmarchyView {
    pub fn new(updates: Entity<OmarchyUpdates>, cx: &mut Context<Self>) -> Self {
        let observer = cx.observe(&updates, |_, _, cx| cx.notify());

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
                    this.update(cx, |this, _cx| {
                        this.release_notes_error = Some(e.to_string());
                    })
                    .ok();
                }
            },
        )
        .detach();

        Self {
            updates,
            latest_tag: None,
            release_notes: None,
            release_notes_error: None,
            focus_handle: cx.focus_handle(),
            notes_focus: crate::ui::focus::tab_stop(cx),
            notes_scroll: ScrollHandle::new(),
            _updates_observer: observer,
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

    fn run_update(&mut self, cx: &mut Context<Self>) {
        match crate::shell::omarchy_sh_commands::launch_omarchy_update() {
            Ok(()) => self
                .updates
                .update(cx, |updates, cx| updates.watch_running_update(cx)),
            Err(e) => eprintln!("{e}"),
        }
    }

    fn check_again(&mut self, cx: &mut Context<Self>) {
        self.updates.update(cx, |updates, cx| updates.refresh(cx));
    }

    /// The version line and, under it, what the update check found.
    fn render_status(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let updates = self.updates.read(cx);
        let version = updates
            .version()
            .map(|v| format!("Version {v}"))
            .unwrap_or_else(|| "Version unknown".to_string());
        let muted = |text: String| {
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(text)
        };

        let detail: AnyElement = match updates.state() {
            UpdateState::Checking => muted("Checking for updates…".into()).into_any_element(),
            UpdateState::Updating => {
                muted("Updating… the status refreshes when omarchy-update finishes".into())
                    .into_any_element()
            }
            UpdateState::UpToDate => h_flex()
                .gap_3()
                .items_center()
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
                        .xsmall()
                        .label("Check again")
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
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, _, cx| this.run_update(cx))),
                        ),
                )
                .children(pending.iter().map(|line| {
                    div()
                        .font_family("monospace")
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(line.clone())
                }))
                .into_any_element(),
            UpdateState::Failed(error) => v_flex()
                .gap_1()
                .items_center()
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(muted("Couldn't check for updates".into()))
                        .child(
                            Button::new("check-updates")
                                .ghost()
                                .xsmall()
                                .label("Try again")
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, _, cx| this.check_again(cx))),
                        ),
                )
                .child(muted(error.clone()))
                .into_any_element(),
        };

        v_flex()
            .gap_1()
            .items_center()
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(version),
            )
            .child(detail)
            .into_any_element()
    }
}

impl Render for OmarchyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let version_status = self.render_status(cx);
        let theme = cx.theme();
        let notes_focused = self.notes_focus.is_focused(window);
        let notes_border = crate::ui::focus::focus_border(notes_focused, theme.border, cx);
        let page_height = self.notes_scroll.bounds().size.height;

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
                                .child(format!("Latest release notes  ·  {tag}")),
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
            let (label, detail) = match &self.release_notes_error {
                Some(error) => ("Release notes unavailable.", Some(error.clone())),
                None => ("Loading release notes...", None),
            };
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
                        .child(label),
                )
                .children(detail.map(|detail| {
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(detail)
                }))
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
