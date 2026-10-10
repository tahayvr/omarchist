//! Makes a flow ready for the catalog and hands it to GitHub in the
//! browser, where the author proposes the file and the reviewers take it
//! from there. Nothing is sent from the app itself.
use crate::ui::notify;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::config::config_setup::{settings, update_settings};
use crate::system::flows::Flow;
use crate::system::flows::catalog::{self, CATEGORIES, Index};
use crate::ui::focus::{self, FocusableSwitch};
use crate::ui::keybinds_page::keybinds_view::{FILTERS_CONTEXT, keybinds_nav};

pub struct PublishDialog {
    flow: Flow,
    /// The catalog as last seen, to tell a new flow from a new version.
    index: Option<Index>,
    author: Entity<InputState>,
    tags: Entity<InputState>,
    category: usize,
    /// The author releases the flow under the catalog's license.
    agreed: bool,
    body_focus: FocusHandle,
    category_focus: FocusHandle,
}

/// `Music, morning  mix` as `["music", "morning", "mix"]`.
fn parse_tags(text: &str) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();
    for word in text.split([',', ' ']) {
        let tag: String = word
            .trim()
            .trim_start_matches('#')
            .to_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        let tag = tag.trim_matches('-').to_string();
        if !tag.is_empty() && !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    tags
}

impl PublishDialog {
    fn new(flow: Flow, index: Option<Index>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // A flow published before starts from what it was published with.
        let author = match flow.meta.author.trim() {
            "" => settings().catalog_author,
            author => author.to_string(),
        };
        let category = CATEGORIES
            .iter()
            .position(|category| *category == flow.meta.category)
            .unwrap_or(0);
        let tags = flow.meta.tags.join(", ");
        Self {
            author: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("Your GitHub user name")
                    .default_value(author)
            }),
            tags: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("music, morning")
                    .default_value(tags)
            }),
            flow,
            index,
            category,
            agreed: false,
            body_focus: cx.focus_handle(),
            category_focus: focus::tab_stop(cx),
        }
    }

    fn cycle_category(&mut self, delta: isize, cx: &mut Context<Self>) {
        let count = CATEGORIES.len() as isize;
        self.category = (self.category as isize + delta).rem_euclid(count) as usize;
        cx.notify();
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let author = self.author.read(cx).value().trim().to_string();
        let tags = parse_tags(&self.tags.read(cx).value());
        if !self.agreed {
            notify::error(
                window,
                format!(
                    "A flow in the catalog is released under {}",
                    catalog::LICENSE
                ),
                cx,
            );
            return;
        }
        let submission = match catalog::prepare_submission(
            &self.flow,
            &author,
            CATEGORIES[self.category],
            &tags,
            self.index.as_ref(),
        ) {
            Ok(submission) => submission,
            Err(e) => {
                notify::error(window, e.to_string(), cx);
                return;
            }
        };
        let name = author.trim_start_matches('@').to_string();
        if let Err(e) = update_settings(|settings| settings.catalog_author = name) {
            eprintln!("{e}");
        }
        // The text travels on the clipboard too: GitHub's page for a new
        // version takes none from its address, and a long file may not fit.
        cx.write_to_clipboard(ClipboardItem::new_string(submission.toml.clone()));
        cx.open_url(&submission.url);
        window.close_dialog(cx);
        notify::success(
            window,
            if submission.update {
                format!(
                    "Copied version {}. On GitHub, replace the file's text with it and propose the change.",
                    submission.version
                )
            } else {
                "Copied the flow. On GitHub, propose the new file to send it for review."
                    .to_string()
            },
            cx,
        );
    }

    fn field(label: &'static str, control: impl IntoElement) -> Div {
        v_flex()
            .gap_1p5()
            .child(div().text_sm().child(label))
            .child(control)
    }
}

impl Render for PublishDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let view = cx.entity();
        let ring = focus::focus_border(
            self.category_focus.is_focused(window),
            theme.transparent,
            cx,
        );
        focus::dialog_body("publish-dialog", &self.body_focus, move |window, cx| {
            view.update(cx, |this, cx| this.submit(window, cx));
        })
        .child(
            v_flex()
                .gap_4()
                .child(Self::field(
                    "GitHub user name",
                    Input::new(&self.author).id("publish-author").small(),
                ))
                .child(Self::field(
                    "Category",
                    h_flex()
                        .id("publish-category")
                        .test_support()
                        .key_context(FILTERS_CONTEXT)
                        .track_focus(&self.category_focus)
                        .on_action(cx.listener(|this, _: &keybinds_nav::FilterPrev, _, cx| {
                            this.cycle_category(-1, cx)
                        }))
                        .on_action(cx.listener(|this, _: &keybinds_nav::FilterNext, _, cx| {
                            this.cycle_category(1, cx)
                        }))
                        .rounded(theme.radius)
                        .border_1()
                        .border_color(ring)
                        .p_0p5()
                        .gap_0p5()
                        .flex_wrap()
                        .children(CATEGORIES.iter().enumerate().map(|(ix, category)| {
                            let selected = self.category == ix;
                            div()
                                .id(SharedString::from(format!(
                                    "publish-category-{}",
                                    category.to_lowercase()
                                )))
                                .test_support()
                                .px_2()
                                .py_0p5()
                                .rounded(theme.radius)
                                .text_xs()
                                .when(selected, |this| {
                                    this.bg(theme.primary).text_color(theme.primary_foreground)
                                })
                                .when(!selected, |this| {
                                    this.text_color(theme.muted_foreground).hover(|this| {
                                        this.bg(theme.secondary).text_color(theme.foreground)
                                    })
                                })
                                .cursor_pointer()
                                .child(*category)
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.category_focus.focus(window, cx);
                                    this.category = ix;
                                    cx.notify();
                                }))
                        })),
                ))
                .child(Self::field(
                    "Tags",
                    Input::new(&self.tags).id("publish-tags").small(),
                ))
                .child(
                    div().text_sm().child(
                        FocusableSwitch::new("publish-license")
                            .label("Release it under CC0, free for anyone to use")
                            .checked(self.agreed)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.agreed = *checked;
                                cx.notify();
                            })),
                    ),
                )
                .child(
                    h_flex()
                        .justify_end()
                        .gap_2()
                        .child(
                            Button::new("publish-cancel")
                                .outline()
                                .small()
                                .label("Cancel")
                                .cursor_pointer()
                                .on_click(|_, window, cx| window.close_dialog(cx)),
                        )
                        .child(
                            Button::new("publish-submit")
                                .primary()
                                .small()
                                .label("Continue on GitHub")
                                .cursor_pointer()
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.submit(window, cx)),
                                ),
                        ),
                ),
        )
    }
}

/// Opens the dialog for `flow`, as the editor holds it.
pub fn open_publish_dialog(
    flow: Flow,
    index: Option<Index>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<PublishDialog> {
    let dialog = cx.new(|cx| PublishDialog::new(flow, index, window, cx));
    let view = dialog.clone();
    let body_focus = dialog.read(cx).body_focus.clone();
    window.open_dialog(cx, move |d, window, _| {
        d.title("Publish to the catalog")
            .w(focus::dialog_width(520., window))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(false)
            .child(view.clone())
    });
    focus::focus_first_in(&body_focus, window, cx);
    dialog
}

#[cfg(test)]
mod tests {
    use super::parse_tags;

    #[test]
    fn tags_are_read_from_loose_text() {
        assert_eq!(
            parse_tags("Music, morning  #mix, music,, --x-- "),
            vec!["music", "morning", "mix", "x"]
        );
        assert!(parse_tags("  , ").is_empty());
    }
}
