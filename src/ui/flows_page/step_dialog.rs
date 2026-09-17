//! The dialog that adds or edits one step of a flow.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex, v_flex,
};

use crate::system::flows::StepKind;
use crate::ui::flows_page::step_builder::{StepBuilder, StepBuilderEvent};
use crate::ui::focus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepDialogMode {
    Add,
    Edit(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepDialogEvent {
    Save(StepDialogMode, StepKind),
    Cancel,
}

pub struct StepDialog {
    mode: StepDialogMode,
    builder: Entity<StepBuilder>,
    error: Option<String>,
    body_focus: FocusHandle,
    _subscription: Subscription,
}

impl EventEmitter<StepDialogEvent> for StepDialog {}

impl StepDialog {
    pub fn new(
        mode: StepDialogMode,
        initial: Option<&StepKind>,
        exclude_flow: Option<&str>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let builder = cx.new(|cx| StepBuilder::new(initial, exclude_flow, window, cx));
        let subscription = cx.subscribe(&builder, |this, _, event: &StepBuilderEvent, cx| {
            let StepBuilderEvent::Changed = event;
            this.error = None;
            cx.notify();
        });
        Self {
            mode,
            builder,
            error: None,
            body_focus: cx.focus_handle(),
            _subscription: subscription,
        }
    }

    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.builder.read(cx).step(cx) {
            Ok(step) => {
                cx.emit(StepDialogEvent::Save(self.mode, step));
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
            StepDialogMode::Add => "Add step",
            StepDialogMode::Edit(_) => "Save step",
        };
        let view = cx.entity();
        focus::dialog_body("step-dialog", &self.body_focus, move |window, cx| {
            view.update(cx, |this, cx| this.save(window, cx));
        })
        .child(
            v_flex()
                .gap_4()
                .child(self.builder.clone())
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
                            .child(error),
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
                        .child(
                            Button::new("step-save")
                                .primary()
                                .small()
                                .label(save_label)
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                        ),
                ),
        )
    }
}

/// Opens the dialog and returns its entity so the caller can subscribe.
pub fn open_step_dialog(
    mode: StepDialogMode,
    initial: Option<&StepKind>,
    exclude_flow: Option<&str>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<StepDialog> {
    let title = match mode {
        StepDialogMode::Add => "Add a step",
        StepDialogMode::Edit(_) => "Edit step",
    };
    let dialog = cx.new(|cx| StepDialog::new(mode, initial, exclude_flow, window, cx));
    let view = dialog.clone();
    let body_focus = dialog.read(cx).body_focus.clone();
    window.open_dialog(cx, move |d, _, _| {
        let on_close_view = view.clone();
        d.title(title)
            .w(px(640.))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(false)
            .on_close(move |_, _, cx| {
                on_close_view.update(cx, |_, cx| cx.emit(StepDialogEvent::Cancel));
            })
            .child(view.clone())
    });
    focus::focus_first_in(&body_focus, window, cx);
    dialog
}
