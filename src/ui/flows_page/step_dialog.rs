//! The dialog that adds or edits one step of a flow. Adding starts on the
//! list of step types; picking one shows its form.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex, v_flex,
};

use crate::system::flows::{StepKind, StepPath};
use crate::ui::flows_page::step_builder::{StepBuilder, StepBuilderEvent};
use crate::ui::flows_page::step_picker::{StepPicker, StepPickerEvent};
use crate::ui::focus;
use crate::ui::text::selectable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepDialogMode {
    /// A new step at `index` of the list of steps `list` names (the
    /// flow's own, or a branch of a step that holds steps).
    Add { list: StepPath, index: usize },
    /// The step at this path.
    Edit(StepPath),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepDialogEvent {
    /// The step and the name it saves its output under.
    Save(StepDialogMode, StepKind, Option<String>),
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    /// Choosing what kind of step to add.
    Pick,
    /// Filling in the step.
    Form,
}

pub struct StepDialog {
    mode: StepDialogMode,
    stage: Stage,
    picker: Entity<StepPicker>,
    builder: Entity<StepBuilder>,
    error: Option<String>,
    body_focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<StepDialogEvent> for StepDialog {}

impl StepDialog {
    pub fn new(
        mode: StepDialogMode,
        initial: Option<&StepKind>,
        output: Option<&str>,
        saved: &[String],
        exclude_flow: Option<&str>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let builder =
            cx.new(|cx| StepBuilder::new(initial, output, saved, exclude_flow, window, cx));
        let picker = cx.new(|cx| StepPicker::new(window, cx));
        let subscriptions = vec![
            cx.subscribe_in(
                &builder,
                window,
                |this, _, event: &StepBuilderEvent, window, cx| match event {
                    StepBuilderEvent::Changed => {
                        this.error = None;
                        cx.notify();
                    }
                    StepBuilderEvent::ChangeType => {
                        this.stage = Stage::Pick;
                        this.error = None;
                        this.picker
                            .update(cx, |picker, cx| picker.focus_search(window, cx));
                        cx.notify();
                    }
                },
            ),
            cx.subscribe_in(
                &picker,
                window,
                |this, _, event: &StepPickerEvent, window, cx| {
                    let StepPickerEvent::Picked(choice) = event;
                    this.stage = Stage::Form;
                    this.builder.update(cx, |builder, cx| {
                        builder.set_choice(*choice, window, cx);
                        builder.focus_first(window, cx);
                    });
                    // A step with nothing to fill in is added as it is.
                    if !choice.has_form() {
                        this.save(window, cx);
                    }
                    cx.notify();
                },
            ),
        ];
        Self {
            stage: match mode {
                StepDialogMode::Add { .. } => Stage::Pick,
                StepDialogMode::Edit(_) => Stage::Form,
            },
            mode,
            picker,
            builder,
            error: None,
            body_focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }

    /// Puts the keyboard where the dialog starts: the search box when
    /// adding, the step's first field when editing.
    fn focus_start(&self, window: &mut Window, cx: &mut App) {
        match self.stage {
            Stage::Pick => self
                .picker
                .update(cx, |picker, cx| picker.focus_search(window, cx)),
            Stage::Form => self
                .builder
                .update(cx, |builder, cx| builder.focus_first(window, cx)),
        }
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.stage == Stage::Pick {
            return;
        }
        let builder = self.builder.read(cx);
        match builder
            .step(cx)
            .and_then(|step| Ok((step, builder.output_name(cx)?)))
        {
            Ok((step, output)) => {
                cx.emit(StepDialogEvent::Save(self.mode.clone(), step, output));
                window.close_dialog(cx);
            }
            Err(error) => {
                self.error = Some(error);
                cx.notify();
            }
        }
    }
}

impl Render for StepDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let save_label = match self.mode {
            StepDialogMode::Add { .. } => "Add step",
            StepDialogMode::Edit(_) => "Save step",
        };
        let picking = self.stage == Stage::Pick;
        let view = cx.entity();
        focus::dialog_body("step-dialog", &self.body_focus, move |window, cx| {
            view.update(cx, |this, cx| this.save(window, cx));
        })
        .child(
            v_flex()
                .gap_4()
                .child(if picking {
                    self.picker.clone().into_any_element()
                } else {
                    self.builder.clone().into_any_element()
                })
                .when_some(self.error.clone(), |this, error| {
                    this.child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded(theme.radius)
                            .border_1()
                            .border_color(theme.danger.opacity(0.4))
                            .bg(theme.danger.opacity(0.08))
                            .text_sm()
                            .child(selectable("step-error", error)),
                    )
                })
                .child(
                    h_flex()
                        .justify_end()
                        .gap_2()
                        .child(
                            Button::new("step-cancel")
                                .outline()
                                .small()
                                .label("Cancel")
                                .cursor_pointer()
                                .on_click(cx.listener(|_, _, window, cx| {
                                    cx.emit(StepDialogEvent::Cancel);
                                    window.close_dialog(cx);
                                })),
                        )
                        .when(!picking, |this| {
                            this.child(
                                Button::new("step-save")
                                    .primary()
                                    .small()
                                    .label(save_label)
                                    .cursor_pointer()
                                    .on_click(
                                        cx.listener(|this, _, window, cx| this.save(window, cx)),
                                    ),
                            )
                        }),
                ),
        )
    }
}

/// Opens the dialog and returns its entity so the caller can subscribe.
pub fn open_step_dialog(
    mode: StepDialogMode,
    initial: Option<&StepKind>,
    output: Option<&str>,
    saved: &[String],
    exclude_flow: Option<&str>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<StepDialog> {
    let title = match mode {
        StepDialogMode::Add { .. } => "Add step",
        StepDialogMode::Edit(_) => "Edit step",
    };
    let dialog =
        cx.new(|cx| StepDialog::new(mode, initial, output, saved, exclude_flow, window, cx));
    let view = dialog.clone();
    window.open_dialog(cx, move |d, window, _| {
        let on_close_view = view.clone();
        d.title(title)
            .w(crate::ui::focus::dialog_width(720., window))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(false)
            .on_close(move |_, _, cx| {
                on_close_view.update(cx, |_, cx| cx.emit(StepDialogEvent::Cancel));
            })
            .child(view.clone())
    });
    let start = dialog.clone();
    window.on_next_frame(move |window, cx| {
        start.update(cx, |dialog, cx| dialog.focus_start(window, cx));
    });
    dialog
}
