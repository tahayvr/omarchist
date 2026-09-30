use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Sizable,
    button::{Button, ButtonVariants},
    h_flex, v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::themes::overrides::{self, OverrideSpec};
use crate::ui::dialogs::confirm_dialog::{ConfirmDialog, open_confirm_dialog};
use crate::ui::focus::FocusableSwitch;
use crate::ui::text::selectable;
use crate::ui::theme_edit_page::color_map_form::is_user_plugin_spec;
use crate::ui::theme_edit_page::override_editors::{EditorView, OverrideEditor};
use crate::ui::theme_edit_page::shared::{error_message, git_ignored_note, theme_is_cloned};

const SAVE_DELAY: Duration = Duration::from_millis(300);

/// Emitted when the theme starts or stops shipping the file.
pub struct StatusChanged;

/// One optional file of a theme: whether the theme ships it, and its editor.
pub struct OverridePane {
    theme_name: String,
    spec: &'static OverrideSpec,
    installed: bool,
    /// The theme was installed with `omarchy theme install`, a git clone.
    cloned: bool,
    /// `Some` while the theme ships its own file.
    editor: Option<OverrideEditor>,
    view: EditorView,
    /// The file as the editor last left it, saved or not; a view switch
    /// starts from it.
    latest: String,
    busy: bool,
    /// Bumped on every edit so only the last one in a burst is written.
    edit_generation: u64,
    /// The generation the last started save carried; behind
    /// `edit_generation` while a debounced save is pending.
    saved_generation: u64,
    error: Option<String>,
    /// The colors Omarchy's own template would put in the file, shown
    /// while the theme does not ship it so Customize is an informed choice.
    preview: Option<Vec<String>>,
    _editor_subscription: Option<Subscription>,
}

impl EventEmitter<StatusChanged> for OverridePane {}

/// The distinct `#rrggbb` colors in `content`, in order of first use.
pub fn preview_colors(content: &str) -> Vec<String> {
    let bytes = content.as_bytes();
    let mut colors: Vec<String> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'#' {
            // Six digits, or eight with alpha; anything else is not a color.
            let run = bytes[i + 1..]
                .iter()
                .take_while(|b| b.is_ascii_hexdigit())
                .count();
            if run == 6 || run == 8 {
                let hex = content[i..i + 7].to_ascii_lowercase();
                if !colors.contains(&hex) {
                    colors.push(hex);
                }
            }
            i += 1 + run;
            continue;
        }
        i += 1;
    }
    colors
}

impl OverridePane {
    pub fn new(
        theme_name: String,
        spec: &'static OverrideSpec,
        installed: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut pane = Self {
            theme_name,
            spec,
            installed,
            cloned: false,
            editor: None,
            view: EditorView::for_spec(spec)[0],
            latest: String::new(),
            busy: false,
            edit_generation: 0,
            saved_generation: 0,
            error: None,
            preview: None,
            _editor_subscription: None,
        };
        pane.cloned = theme_is_cloned(&pane.theme_name);
        match overrides::read(&pane.theme_name, spec) {
            Ok(Some(content)) => {
                // Open on the view that matches the file.
                if EditorView::for_spec(spec).contains(&EditorView::Plugin)
                    && is_user_plugin_spec(&content)
                {
                    pane.view = EditorView::Plugin;
                }
                pane.show_editor(&content, window, cx)
            }
            Ok(None) => pane.load_preview(cx),
            Err(e) => pane.error = Some(e.to_string()),
        }
        pane
    }

    /// Renders Omarchy's template off the UI thread for the swatch strip.
    fn load_preview(&mut self, cx: &mut Context<Self>) {
        let theme = self.theme_name.clone();
        let spec = self.spec;
        cx.spawn(async move |this, cx| {
            let colors = cx
                .background_spawn(async move {
                    overrides::generated(&theme, spec).map(|content| preview_colors(&content))
                })
                .await;
            this.update(cx, |this, cx| {
                if this.editor.is_none() {
                    this.preview = Some(colors.unwrap_or_default());
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    fn render_preview(&self, cx: &App) -> Option<AnyElement> {
        let colors = self.preview.as_ref().filter(|c| !c.is_empty())?;
        let theme = cx.theme();
        Some(
            h_flex()
                .gap_2()
                .flex_wrap()
                .children(colors.iter().enumerate().map(|(ix, hex)| {
                    let color = crate::ui::color_utils::hex_to_hsla(hex).unwrap_or(theme.muted);
                    v_flex()
                        .gap_1()
                        .items_center()
                        .child(
                            div()
                                .size_8()
                                .rounded(theme.radius)
                                .border_1()
                                .border_color(theme.border)
                                .bg(color),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(selectable(("preview-hex", ix), hex.clone())),
                        )
                }))
                .into_any_element(),
        )
    }

    pub fn is_custom(&self) -> bool {
        self.editor.is_some()
    }

    fn show_editor(&mut self, content: &str, window: &mut Window, cx: &mut Context<Self>) {
        let editor =
            OverrideEditor::new(&self.theme_name, self.spec, self.view, content, window, cx);
        self.latest = content.to_string();
        self._editor_subscription = Some(editor.subscribe(cx, |this, event, cx| {
            this.latest = event.0.clone();
            this.schedule_save(event.0.clone(), cx);
        }));
        self.editor = Some(editor);
    }

    fn set_view(&mut self, view: EditorView, window: &mut Window, cx: &mut Context<Self>) {
        if self.view == view || self.editor.is_none() {
            return;
        }
        self.view = view;
        let content = self.latest.clone();
        self.show_editor(&content, window, cx);
        cx.notify();
    }

    fn render_views(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let views = EditorView::for_spec(self.spec);
        (views.len() > 1 && self.editor.is_some()).then(|| {
            h_flex().gap_1().children(views.iter().map(|&view| {
                Button::new(SharedString::from(format!(
                    "view-{}-{}",
                    self.spec.file,
                    view.label()
                )))
                .label(view.label())
                .small()
                .map(|b| {
                    if self.view == view {
                        b.primary()
                    } else {
                        b.ghost()
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, window, cx| this.set_view(view, window, cx)))
            }))
        })
    }

    /// Writes a pending edit now (before an Apply or leaving the page).
    pub fn flush(&mut self, cx: &mut Context<Self>) {
        if self.edit_generation == self.saved_generation || self.editor.is_none() {
            return;
        }
        self.saved_generation = self.edit_generation;
        let result = overrides::write(&self.theme_name, self.spec, &self.latest);
        self.error = result.err().map(|e| e.to_string());
        cx.notify();
    }

    /// Drops a pending save: the file is being replaced or removed.
    fn cancel_pending_save(&mut self) {
        self.edit_generation += 1;
        self.saved_generation = self.edit_generation;
    }

    fn schedule_save(&mut self, content: String, cx: &mut Context<Self>) {
        self.edit_generation += 1;
        let generation = self.edit_generation;
        let theme = self.theme_name.clone();
        let spec = self.spec;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SAVE_DELAY).await;
            let current = this.update(cx, |this, _| {
                let pending = this.edit_generation == generation;
                if pending {
                    this.saved_generation = generation;
                }
                pending
            });
            if !matches!(current, Ok(true)) {
                return;
            }
            let result = cx
                .background_spawn(async move { overrides::write(&theme, spec, &content) })
                .await;
            this.update(cx, |this, cx| {
                if this.edit_generation == generation {
                    this.error = result.err().map(|e| e.to_string());
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    /// Writes what Omarchy would generate and opens it in the editor. A file
    /// with a fixed seed is written only when the seed is valid on its own;
    /// otherwise it is opened blank and written once the fields are filled.
    fn customize(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        if let overrides::Seed::Fixed(seed) = self.spec.seed {
            self.cancel_pending_save();
            self.error = None;
            if overrides::validate(self.spec, seed).is_ok()
                && let Err(e) = overrides::write(&self.theme_name, self.spec, seed)
            {
                self.error = Some(e.to_string());
            }
            self.show_editor(seed, window, cx);
            cx.emit(StatusChanged);
            cx.notify();
            return;
        }
        self.busy = true;
        self.cancel_pending_save();
        let theme = self.theme_name.clone();
        let spec = self.spec;
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let content = overrides::generated(&theme, spec)?;
                    overrides::write(&theme, spec, &content)?;
                    Ok::<_, crate::error::Error>(content)
                })
                .await;
            this.update_in(cx, |this, window, cx| {
                this.busy = false;
                match result {
                    Ok(content) => {
                        this.error = None;
                        this.show_editor(&content, window, cx);
                        cx.emit(StatusChanged);
                    }
                    Err(e) => this.error = Some(e.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn remove(&mut self, cx: &mut Context<Self>) {
        self.cancel_pending_save();
        match overrides::remove(&self.theme_name, self.spec) {
            Ok(()) => {
                self.editor = None;
                self._editor_subscription = None;
                self.error = None;
                self.load_preview(cx);
                cx.emit(StatusChanged);
            }
            Err(e) => self.error = Some(e.to_string()),
        }
        cx.notify();
    }

    fn confirm_remove(&self, window: &mut Window, cx: &mut Context<Self>) {
        let pane = cx.entity().downgrade();
        open_confirm_dialog(
            ConfirmDialog {
                title: "Stop customizing?",
                message: format!(
                    "This deletes the theme's {} and Omarchy generates it from the palette again.",
                    self.spec.file
                ),
                confirm_label: "Delete",
                danger: true,
            },
            move |_, cx| {
                pane.update(cx, |pane, cx| pane.remove(cx)).ok();
            },
            window,
            cx,
        );
    }

    fn confirm_reset(&self, window: &mut Window, cx: &mut Context<Self>) {
        let pane = cx.entity().downgrade();
        open_confirm_dialog(
            ConfirmDialog {
                title: "Reset to generated?",
                message: format!(
                    "This replaces your {} with what Omarchy generates from the current palette.",
                    self.spec.file
                ),
                confirm_label: "Reset",
                danger: true,
            },
            move |window, cx| {
                pane.update(cx, |pane, cx| pane.customize(window, cx)).ok();
            },
            window,
            cx,
        );
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let custom = self.is_custom();
        let badge = |text: &'static str, color: Hsla| {
            div()
                .px_2()
                .py_0p5()
                .rounded(theme.radius)
                .border_1()
                .border_color(color.opacity(0.4))
                .text_xs()
                .text_color(color)
                .child(text)
        };

        h_flex()
            .gap_4()
            .items_start()
            .justify_between()
            .flex_wrap()
            .child(
                v_flex().gap_1().min_w_0().flex_1().child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .flex_wrap()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(selectable("app", self.spec.app)),
                        )
                        .child(if custom {
                            badge("Custom", theme.primary)
                        } else {
                            badge("Generated", theme.muted_foreground)
                        })
                        .when(!self.installed, |row| {
                            row.child(badge("Not installed", theme.muted_foreground))
                        }),
                ),
            )
            .child(
                FocusableSwitch::new(SharedString::from(format!("customize-{}", self.spec.file)))
                    .label("Customize")
                    .checked(custom)
                    .disabled(self.busy)
                    .on_change(cx.listener(|this, checked: &bool, window, cx| {
                        if *checked {
                            this.customize(window, cx);
                        } else {
                            this.confirm_remove(window, cx);
                        }
                    })),
            )
    }
}

impl Render for OverridePane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let file = self.spec.file;

        let mut pane = v_flex()
            .id(SharedString::from(format!("override-pane-{file}")))
            .test_support()
            .gap_4()
            .min_w_0()
            .child(self.render_header(cx))
            .when(self.spec.git_restricted() && self.cloned, |pane| {
                pane.child(git_ignored_note(cx))
            })
            .children(self.error.clone().map(|error| error_message(error, cx)));

        pane = match &self.editor {
            Some(editor) => pane
                .children(self.render_views(cx))
                .child(editor.element())
                .child(
                    h_flex().child(
                        Button::new(SharedString::from(format!("reset-{file}")))
                            .label("Reset to generated")
                            .small()
                            .outline()
                            .cursor_pointer()
                            .on_click(
                                cx.listener(|this, _, window, cx| this.confirm_reset(window, cx)),
                            ),
                    ),
                ),
            None => pane.children(self.render_preview(cx)),
        };

        pane
    }
}

#[cfg(test)]
mod tests {
    use super::preview_colors;

    #[test]
    fn preview_lists_each_color_once_and_skips_hashes() {
        let text =
            "bg = \"#1A2B3C\"\nfg = '#1a2b3c'\nalpha = #ff000080\nhash #deadbeefcafe\nshort #abc\n";
        assert_eq!(preview_colors(text), vec!["#1a2b3c", "#ff0000"]);
    }
}
