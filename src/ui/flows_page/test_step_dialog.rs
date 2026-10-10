//! Asks for the values a step needs to run alone: one field per variable
//! it uses, filled with what the last run saved under that name.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Textarea, TextareaState},
    v_flex,
};
use gpui_kit::TestSupportExt;

use crate::ui::flows_page::step_summary::StepSummary;
use crate::ui::flows_page::var_token;
use crate::ui::focus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestStepEvent {
    /// Each variable with the value to run the step with.
    Run(Vec<(String, String)>),
    Cancel,
}

pub struct TestStepDialog {
    summary: StepSummary,
    fields: Vec<(String, Entity<TextareaState>)>,
    body_focus: FocusHandle,
}

impl EventEmitter<TestStepEvent> for TestStepDialog {}

impl TestStepDialog {
    fn new(
        summary: StepSummary,
        values: &[(String, String)],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let fields = values
            .iter()
            .map(|(name, value)| {
                let state =
                    cx.new(|cx| TextareaState::new(window, cx).default_value(value.clone()));
                (name.clone(), state)
            })
            .collect();
        Self {
            summary,
            fields,
            body_focus: cx.focus_handle(),
        }
    }

    fn run(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let values = self
            .fields
            .iter()
            .map(|(name, state)| (name.clone(), state.read(cx).value().to_string()))
            .collect();
        cx.emit(TestStepEvent::Run(values));
        window.close_dialog(cx);
    }
}

impl Render for TestStepDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let view = cx.entity();
        focus::dialog_body("test-step-dialog", &self.body_focus, move |window, cx| {
            view.update(cx, |this, cx| this.run(window, cx));
        })
        .child(
            v_flex()
                .gap_4()
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .p_2()
                        .rounded(theme.radius)
                        .bg(theme.secondary)
                        .child(self.summary.tile(px(28.), cx))
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_0p5()
                                .text_sm()
                                .child(var_token::rich_text(
                                    "test-step-title",
                                    &self.summary.title,
                                    cx,
                                ))
                                .when(!self.summary.detail.is_empty(), |this| {
                                    this.child(
                                        div().text_xs().text_color(theme.muted_foreground).child(
                                            var_token::rich_text(
                                                "test-step-detail",
                                                &self.summary.detail,
                                                cx,
                                            ),
                                        ),
                                    )
                                }),
                        ),
                )
                .children(self.fields.iter().enumerate().map(|(ix, (name, state))| {
                    v_flex()
                        .gap_1p5()
                        .child(h_flex().text_sm().child(var_token::token(name, ix, cx)))
                        .child(
                            div()
                                .id(SharedString::from(format!("test-value-{name}")))
                                .test_support()
                                .child(Textarea::new(state).h(px(56.))),
                        )
                }))
                .child(
                    h_flex()
                        .justify_end()
                        .gap_2()
                        .child(
                            Button::new("test-step-cancel")
                                .outline()
                                .small()
                                .label("Cancel")
                                .cursor_pointer()
                                .on_click(cx.listener(|_, _, window, cx| {
                                    cx.emit(TestStepEvent::Cancel);
                                    window.close_dialog(cx);
                                })),
                        )
                        .child(
                            Button::new("test-step-run")
                                .primary()
                                .small()
                                .label("Run step")
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, window, cx| this.run(window, cx))),
                        ),
                ),
        )
    }
}

/// Opens the dialog for step `number` and returns its entity so the
/// caller can subscribe.
pub fn open_test_step_dialog(
    number: usize,
    summary: StepSummary,
    values: &[(String, String)],
    window: &mut Window,
    cx: &mut App,
) -> Entity<TestStepDialog> {
    let dialog = cx.new(|cx| TestStepDialog::new(summary, values, window, cx));
    let view = dialog.clone();
    let body_focus = dialog.read(cx).body_focus.clone();
    let title: SharedString = format!("Test step {number}").into();
    window.open_dialog(cx, move |d, window, _| {
        let on_close_view = view.clone();
        d.title(title.clone())
            .w(focus::dialog_width(520., window))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(false)
            .on_close(move |_, _, cx| {
                on_close_view.update(cx, |_, cx| cx.emit(TestStepEvent::Cancel));
            })
            .child(view.clone())
    });
    focus::focus_first_in(&body_focus, window, cx);
    dialog
}
