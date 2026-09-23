use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Editor, EditorState},
    v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::themes::overrides::{self, OverrideSpec};
use crate::ui::dialogs::confirm_dialog::{ConfirmDialog, open_confirm_dialog};
use crate::ui::focus::FocusableSwitch;
use crate::ui::theme_edit_page::override_editors::{OverrideEditor, preview_editor};
use crate::ui::theme_edit_page::shared::{error_message, help_text};

const SAVE_DELAY: Duration = Duration::from_millis(300);

/// Emitted when the theme starts or stops shipping the file.
pub struct StatusChanged;

/// One optional file of a theme: whether the theme ships it, and its editor.
pub struct OverridePane {
    theme_name: String,
    spec: &'static OverrideSpec,
    installed: bool,
    /// `Some` while the theme ships its own file.
    editor: Option<OverrideEditor>,
    preview: Option<Entity<EditorState>>,
    busy: bool,
    /// Bumped on every edit so only the last one in a burst is written.
    edit_generation: u64,
    error: Option<String>,
    _editor_subscription: Option<Subscription>,
}

impl EventEmitter<StatusChanged> for OverridePane {}

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
            editor: None,
            preview: None,
            busy: false,
            edit_generation: 0,
            error: None,
            _editor_subscription: None,
        };
        match overrides::read(&pane.theme_name, spec) {
            Ok(Some(content)) => pane.show_editor(&content, window, cx),
            Ok(None) => {}
            Err(e) => pane.error = Some(e.to_string()),
        }
        pane
    }

    pub fn is_custom(&self) -> bool {
        self.editor.is_some()
    }

    fn show_editor(&mut self, content: &str, window: &mut Window, cx: &mut Context<Self>) {
        let editor = OverrideEditor::new(self.spec, content, window, cx);
        self._editor_subscription = Some(editor.subscribe(cx, |this, event, cx| {
            this.schedule_save(event.0.clone(), cx);
        }));
        self.editor = Some(editor);
        self.preview = None;
    }

    fn schedule_save(&mut self, content: String, cx: &mut Context<Self>) {
        self.edit_generation += 1;
        let generation = self.edit_generation;
        let theme = self.theme_name.clone();
        let spec = self.spec;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SAVE_DELAY).await;
            let current = this.update(cx, |this, _| this.edit_generation == generation);
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

    /// Writes what Omarchy would generate and opens it in the editor.
    fn customize(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.edit_generation += 1;
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
        self.edit_generation += 1;
        match overrides::remove(&self.theme_name, self.spec) {
            Ok(()) => {
                self.editor = None;
                self._editor_subscription = None;
                self.error = None;
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

    fn toggle_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.preview.take().is_some() {
            cx.notify();
            return;
        }
        let theme = self.theme_name.clone();
        let spec = self.spec;
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move { overrides::generated(&theme, spec) })
                .await;
            this.update_in(cx, |this, window, cx| {
                match result {
                    Ok(content) => this.preview = Some(preview_editor(spec, &content, window, cx)),
                    Err(e) => this.error = Some(e.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
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
                v_flex()
                    .gap_1()
                    .min_w_0()
                    .flex_1()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .flex_wrap()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.spec.app),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_family(theme.mono_font_family.clone())
                                    .text_color(theme.muted_foreground)
                                    .child(self.spec.file),
                            )
                            .child(if custom {
                                badge("Custom", theme.primary)
                            } else {
                                badge("Generated", theme.muted_foreground)
                            })
                            .when(!self.installed, |row| {
                                row.child(badge("Not installed", theme.muted_foreground))
                            }),
                    )
                    .child(help_text(self.spec.description, theme.muted_foreground)),
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
        let theme = cx.theme();
        let muted = theme.muted_foreground;
        let warning = theme.warning;
        let file = self.spec.file;

        let mut pane = v_flex()
            .id(SharedString::from(format!("override-pane-{file}")))
            .test_support()
            .gap_4()
            .min_w_0()
            .child(self.render_header(cx))
            .when(self.spec.git_restricted(), |pane| {
                pane.child(help_text(
                    "Ignored when this theme is installed from a git repository: Omarchy does \
                     not load Lua, terminal configs, or vscode.json from a theme it cloned.",
                    warning,
                ))
            })
            .children(self.error.clone().map(|error| error_message(error, cx)));

        pane = match &self.editor {
            Some(editor) => pane.child(editor.element()).child(
                h_flex().child(
                    Button::new(SharedString::from(format!("reset-{file}")))
                        .label("Reset to Generated")
                        .small()
                        .outline()
                        .cursor_pointer()
                        .on_click(
                            cx.listener(|this, _, window, cx| this.confirm_reset(window, cx)),
                        ),
                ),
            ),
            None => pane
                .child(help_text(
                    "Omarchy generates this from your palette. Nothing to do here unless you \
                     want it to look different.",
                    muted,
                ))
                .when(
                    !matches!(self.spec.seed, overrides::Seed::Fixed(_)),
                    |pane| {
                        pane.child(
                            h_flex().child(
                                Button::new(SharedString::from(format!("preview-{file}")))
                                    .label(if self.preview.is_some() {
                                        "Hide Generated File"
                                    } else {
                                        "Show Generated File"
                                    })
                                    .small()
                                    .ghost()
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.toggle_preview(window, cx)
                                    })),
                            ),
                        )
                    },
                )
                .children(self.preview.as_ref().map(|preview| {
                    div().h(px(360.)).child(
                        Editor::new(preview)
                            .readonly(true)
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .h_full()
                            .appearance(false),
                    )
                })),
        };

        pane
    }
}
