//! A flow's last runs: how each ended, when, what started it, and, opened
//! up, what every step did.
use crate::ui::notify;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex, v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::flows::history::{self, Run, RunResult, StepResult};
use crate::ui::flows_page::var_token;
use crate::ui::focus;
use crate::ui::text::selectable;

pub const HISTORY_CONTEXT: &str = "RunHistory";

pub mod history_nav {
    gpui::actions!(run_history, [Up, Down, First, Last, Toggle]);
}
use history_nav::*;

pub struct HistoryDialog {
    id: String,
    runs: Vec<Run>,
    /// The run the keyboard is on.
    selected: usize,
    /// The run whose steps are shown.
    open: Option<usize>,
    list_focus: FocusHandle,
    body_focus: FocusHandle,
    scroll: ScrollHandle,
}

impl HistoryDialog {
    pub fn new(id: &str, cx: &mut Context<Self>) -> Self {
        let runs = history::load(id);
        Self {
            id: id.to_string(),
            // The newest run starts opened: it is the one just looked for.
            open: (!runs.is_empty()).then_some(0),
            runs,
            selected: 0,
            list_focus: focus::tab_stop(cx),
            body_focus: cx.focus_handle(),
            scroll: ScrollHandle::new(),
        }
    }

    pub fn runs(&self) -> &[Run] {
        &self.runs
    }

    fn select(&mut self, ix: usize, cx: &mut Context<Self>) {
        if self.runs.is_empty() {
            return;
        }
        self.selected = ix.min(self.runs.len() - 1);
        self.scroll.scroll_to_item(self.selected);
        cx.notify();
    }

    fn toggle(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.open = if self.open == Some(ix) {
            None
        } else {
            Some(ix)
        };
        self.selected = ix;
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match history::clear(&self.id) {
            Ok(()) => {
                self.runs.clear();
                self.open = None;
                self.selected = 0;
            }
            Err(e) => notify::error(window, e.to_string(), cx),
        }
        cx.notify();
    }

    fn result_icon(result: RunResult, cx: &App) -> Icon {
        let theme = cx.theme();
        let (path, color) = match result {
            RunResult::Finished => ("icons/circle-check.svg", theme.success),
            RunResult::Failed => ("icons/circle-x.svg", theme.danger),
            RunResult::Cancelled | RunResult::Stopped => ("icons/ban.svg", theme.muted_foreground),
        };
        Icon::new(Icon::empty())
            .path(path)
            .size_4()
            .flex_shrink_0()
            .text_color(color)
    }

    fn render_steps(&self, ix: usize, run: &Run, cx: &App) -> impl IntoElement {
        let theme = cx.theme();
        v_flex()
            .id(("history-steps", ix))
            .test_support()
            .gap_1()
            .pl_8()
            .pr_2()
            .pb_2()
            .children(run.steps.iter().enumerate().map(|(step_ix, step)| {
                let (icon, color) = match step.result {
                    StepResult::Done => ("icons/check.svg", theme.success),
                    StepResult::Failed => ("icons/x.svg", theme.danger),
                    StepResult::Cancelled => ("icons/ban.svg", theme.muted_foreground),
                };
                h_flex()
                    .gap_2()
                    .items_start()
                    .pl(px(16. * step.depth as f32))
                    .text_xs()
                    .child(
                        Icon::new(Icon::empty())
                            .path(icon)
                            .size_3()
                            .mt_0p5()
                            .flex_shrink_0()
                            .text_color(color),
                    )
                    .child(
                        div()
                            .w_5()
                            .flex_shrink_0()
                            .text_color(theme.muted_foreground)
                            .child(step.number.to_string()),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(var_token::rich_text(
                                &format!("history-step-{ix}-{step_ix}"),
                                &step.title,
                                cx,
                            ))
                            .when(!step.detail.is_empty(), |this| {
                                this.child(
                                    div()
                                        .text_color(if step.result == StepResult::Failed {
                                            theme.danger
                                        } else {
                                            theme.muted_foreground
                                        })
                                        .child(selectable(
                                            SharedString::from(format!(
                                                "history-detail-{ix}-{step_ix}"
                                            )),
                                            step.detail.clone(),
                                        )),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_color(theme.muted_foreground)
                            .child(history::took(step.ms)),
                    )
            }))
            .when(run.steps.is_empty(), |this| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(selectable(
                            SharedString::from(format!("history-summary-{ix}")),
                            run.summary.clone(),
                        )),
                )
            })
    }
}

impl Render for HistoryDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let now = chrono::Local::now().timestamp();
        let list_focused = self.list_focus.is_focused(window);
        let ring = focus::focus_border(list_focused, theme.transparent, cx);
        let count = self.runs.len();

        let mut rows: Vec<AnyElement> = Vec::new();
        for (ix, run) in self.runs.iter().enumerate() {
            let opened = self.open == Some(ix);
            let selected = list_focused && self.selected == ix;
            rows.push(
                v_flex()
                    .rounded(theme.radius)
                    .border_1()
                    .border_color(if selected { theme.ring } else { theme.border })
                    .child(
                        h_flex()
                            .id(("history-run", ix))
                            .test_support()
                            .gap_2()
                            .items_center()
                            .p_2()
                            .cursor_pointer()
                            .hover(|this| this.bg(theme.secondary))
                            .child(Self::result_icon(run.result, cx))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .text_sm()
                                    .truncate()
                                    .child(selectable(
                                        ("history-trigger", ix),
                                        run.trigger.clone(),
                                    )),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(format!(
                                        "{} · {}",
                                        history::ago(run.started, now),
                                        history::took(run.ms)
                                    )),
                            )
                            .child(
                                Icon::new(Icon::empty())
                                    .path(if opened {
                                        "icons/chevron-down.svg"
                                    } else {
                                        "icons/chevron-right.svg"
                                    })
                                    .size_3()
                                    .text_color(theme.muted_foreground),
                            )
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.list_focus.focus(window, cx);
                                this.toggle(ix, cx);
                            })),
                    )
                    .when(opened, |this| this.child(self.render_steps(ix, run, cx)))
                    .into_any_element(),
            );
        }

        focus::dialog_body("history-dialog", &self.body_focus, |_, _| {}).child(
            v_flex()
                .gap_4()
                .child(
                    v_flex()
                        .id("history-runs")
                        .test_support()
                        .key_context(HISTORY_CONTEXT)
                        .track_focus(&self.list_focus)
                        .on_action(cx.listener(|this, _: &Up, _, cx| {
                            this.select(this.selected.saturating_sub(1), cx)
                        }))
                        .on_action(
                            cx.listener(|this, _: &Down, _, cx| this.select(this.selected + 1, cx)),
                        )
                        .on_action(cx.listener(|this, _: &First, _, cx| this.select(0, cx)))
                        .on_action(cx.listener(|this, _: &Last, _, cx| this.select(usize::MAX, cx)))
                        .on_action(cx.listener(|this, _: &Toggle, _, cx| {
                            if !this.runs.is_empty() {
                                this.toggle(this.selected, cx)
                            }
                        }))
                        .rounded(theme.radius)
                        .border_1()
                        .border_color(ring)
                        .p_0p5()
                        .child(
                            v_flex()
                                .id("history-list")
                                .gap_1()
                                .max_h(focus::dialog_height(460., window))
                                .overflow_y_scroll()
                                .track_scroll(&self.scroll)
                                .children(rows),
                        )
                        .when(count == 0, |this| {
                            this.child(
                                div()
                                    .py_6()
                                    .text_center()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child(selectable("history-empty", "No runs yet")),
                            )
                        }),
                )
                .child(
                    h_flex()
                        .justify_between()
                        .gap_2()
                        .child(
                            Button::new("history-clear")
                                .ghost()
                                .small()
                                .label("Clear history")
                                .when(count == 0, |this| this.invisible())
                                .cursor_pointer()
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.clear(window, cx)),
                                ),
                        )
                        .child(
                            Button::new("history-close")
                                .outline()
                                .small()
                                .label("Close")
                                .cursor_pointer()
                                .on_click(|_, window, cx| window.close_dialog(cx)),
                        ),
                ),
        )
    }
}

/// Opens the run history of the flow `id`.
pub fn open_history_dialog(
    id: &str,
    name: &str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<HistoryDialog> {
    let dialog = cx.new(|cx| HistoryDialog::new(id, cx));
    let view = dialog.clone();
    let list_focus = dialog.read(cx).list_focus.clone();
    let title: SharedString = format!("Runs of {name}").into();
    window.open_dialog(cx, move |d, window, _| {
        d.title(title.clone())
            .w(focus::dialog_width(640., window))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .child(view.clone())
    });
    list_focus.focus(window, cx);
    dialog
}
