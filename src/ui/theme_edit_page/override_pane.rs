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
            cloned: false,
            editor: None,
            view: EditorView::for_spec(spec)[0],
            latest: String::new(),
            busy: false,
            edit_generation: 0,
            error: None,
            _editor_subscription: None,
        };
        pane.cloned = theme_is_cloned(&pane.theme_name);
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
        let editor =
            OverrideEditor::new(&self.theme_name, self.spec, self.view, content, window, cx);
        self.latest = content.to_string();
        self._editor_subscription = Some(editor.subscribe(cx, |this, event, cx| {
            this.latest = event.0.clone();
            this.schedule_save(event.0.clone(), cx);
        }));
        if let Some(seeded) = editor.seeded_content(cx) {
            self.latest = seeded.clone();
            self.schedule_save(seeded, cx);
        }
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

    /// Writes what Omarchy would generate and opens it in the editor. A file
    /// with a fixed seed is written only when the seed is valid on its own;
    /// otherwise it is opened blank and written once the fields are filled.
    fn customize(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        if let overrides::Seed::Fixed(seed) = self.spec.seed {
            self.edit_generation += 1;
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
                            .label("Reset to Generated")
                            .small()
                            .outline()
                            .cursor_pointer()
                            .on_click(
                                cx.listener(|this, _, window, cx| this.confirm_reset(window, cx)),
                            ),
                    ),
                ),
            None => pane,
        };

        pane
    }
}
