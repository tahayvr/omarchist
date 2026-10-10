//! The flow editor's step list. Steps that hold other steps (If, Repeat,
//! Choose from a menu) show them indented under a rail, with a heading for
//! each branch that has a name and a line that adds a step at the end of
//! each branch. The whole list is one tab stop; the arrow keys walk its
//! lines.
use std::collections::HashSet;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    spinner::Spinner,
    switch::Switch,
    v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::flows::requirements::program_of;
use crate::system::flows::{self, Step, StepKind, StepPath, vars};
use crate::ui::flows_page::flow_edit_view::{
    FlowEditPage, STEPS_CONTEXT, StepState, flow_edit_nav::*,
};
use crate::ui::flows_page::step_dialog::StepDialogMode;
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::flows_page::step_types::StepGroup;
use crate::ui::flows_page::var_token;
use crate::ui::focus;
use crate::ui::text::selectable;

/// One line of the step list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowKey {
    Step(StepPath),
    /// The heading of a branch that has a name: an If's "Otherwise", a
    /// menu's choice. Holds the branch's list path.
    Branch(StepPath),
    /// The line that adds a step at the end of a branch.
    Add(StepPath),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub key: RowKey,
    /// How many blocks the line sits inside.
    pub depth: usize,
}

/// The heading of branch `branch` of a block, when it has one. An If's
/// first branch and a loop's steps follow their step directly.
fn branch_label(kind: &StepKind, branch: usize) -> Option<String> {
    match kind {
        StepKind::If { .. } if branch == 1 => Some("Otherwise".to_string()),
        StepKind::Menu { choices, .. } => choices.get(branch).map(|c| c.label.trim().to_string()),
        _ => None,
    }
}

/// The lines of the list for `steps`, leaving out what collapsed blocks
/// hold.
pub fn rows(steps: &[Step], collapsed: &HashSet<u64>) -> Vec<Row> {
    fn visit(
        steps: &[Step],
        list: &[usize],
        depth: usize,
        collapsed: &HashSet<u64>,
        out: &mut Vec<Row>,
    ) {
        for (index, step) in steps.iter().enumerate() {
            let mut path = list.to_vec();
            path.push(index);
            out.push(Row {
                key: RowKey::Step(path.clone()),
                depth,
            });
            if !step.kind.is_block() || collapsed.contains(&step.uid) {
                continue;
            }
            for (branch, inner) in step.kind.branches().into_iter().enumerate() {
                let mut inner_list = path.clone();
                inner_list.push(branch);
                if branch_label(&step.kind, branch).is_some() {
                    out.push(Row {
                        key: RowKey::Branch(inner_list.clone()),
                        depth: depth + 1,
                    });
                }
                visit(inner, &inner_list, depth + 1, collapsed, out);
                out.push(Row {
                    key: RowKey::Add(inner_list),
                    depth: depth + 1,
                });
            }
        }
    }
    let mut out = Vec::new();
    visit(steps, &[], 0, collapsed, &mut out);
    out
}

/// A stable id for the lines that belong to a branch rather than a step.
fn branch_id(prefix: &str, flow: &flows::Flow, list: &[usize]) -> ElementId {
    let (branch, step) = list.split_last().unwrap_or((&0, &[]));
    ElementId::Name(format!("{prefix}-{}-{branch}", flow.step_number(step)).into())
}

impl FlowEditPage {
    pub(super) fn rows(&self) -> Vec<Row> {
        rows(&self.flow.steps, &self.collapsed)
    }

    /// The step the keyboard is on, when it is on a step.
    fn selected_step(&self) -> Option<StepPath> {
        match &self.selected {
            Some(RowKey::Step(path)) => Some(path.clone()),
            _ => None,
        }
    }

    /// After the steps changed: what is missing is worked out again, the
    /// last run's marks are dropped, and a selection that no longer names
    /// a line is let go.
    pub(super) fn touch_steps(&mut self, cx: &mut Context<Self>) {
        self.missing = flows::requirements::missing_programs(&self.flow);
        self.step_states.clear();
        self.rounds.clear();
        let rows = self.rows();
        if let Some(selected) = &self.selected
            && !rows.iter().any(|row| &row.key == selected)
        {
            self.selected = None;
        }
        cx.notify();
    }

    fn select(&mut self, key: RowKey, window: &mut Window, cx: &mut Context<Self>) {
        self.selected = Some(key);
        // The line reports where it is when it is drawn selected; bring it
        // into view once that frame is in.
        let bounds = self.selected_bounds.clone();
        let scroll = self.scroll.clone();
        cx.on_next_frame(window, move |_, _, cx| {
            focus::scroll_into_view(&scroll, bounds.get());
            cx.notify();
        });
        cx.notify();
    }

    /// Moves the selection `delta` lines; from nothing, onto the first or
    /// last line.
    fn select_by(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        let rows = self.rows();
        if rows.is_empty() {
            return;
        }
        let current = self
            .selected
            .as_ref()
            .and_then(|key| rows.iter().position(|row| &row.key == key));
        let next = match current {
            Some(ix) => (ix as isize + delta).clamp(0, rows.len() as isize - 1) as usize,
            None if delta < 0 => rows.len() - 1,
            None => 0,
        };
        self.select(rows[next].key.clone(), window, cx);
    }

    fn select_edge(&mut self, last: bool, window: &mut Window, cx: &mut Context<Self>) {
        let rows = self.rows();
        let row = if last { rows.last() } else { rows.first() };
        if let Some(row) = row {
            self.select(row.key.clone(), window, cx);
        }
    }

    /// Where a step added from the keyboard goes: after the selected
    /// step, at the end of the selected branch, or at the end of the flow.
    fn add_target(&self, window: &Window) -> (StepPath, usize) {
        let end = (Vec::new(), self.flow.steps.len());
        if !self.steps_focus.is_focused(window) {
            return end;
        }
        match &self.selected {
            Some(RowKey::Step(path)) => match path.split_last() {
                Some((index, list)) => (list.to_vec(), index + 1),
                None => end,
            },
            Some(RowKey::Branch(list)) | Some(RowKey::Add(list)) => {
                let len = flows::list_at(&self.flow.steps, list).map_or(0, Vec::len);
                (list.clone(), len)
            }
            None => end,
        }
    }

    pub(super) fn add_step(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (list, index) = self.add_target(window);
        self.open_step_dialog(StepDialogMode::Add { list, index }, window, cx);
    }

    /// Adds at the end of the flow, whatever is selected.
    fn add_step_at_end(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mode = StepDialogMode::Add {
            list: Vec::new(),
            index: self.flow.steps.len(),
        };
        self.open_step_dialog(mode, window, cx);
    }

    fn add_step_to(&mut self, list: &[usize], window: &mut Window, cx: &mut Context<Self>) {
        let index = flows::list_at(&self.flow.steps, list).map_or(0, Vec::len);
        let mode = StepDialogMode::Add {
            list: list.to_vec(),
            index,
        };
        self.open_step_dialog(mode, window, cx);
    }

    /// Enter on a line: edits a step, adds to a branch.
    fn activate_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.selected.clone() {
            Some(RowKey::Step(path)) => {
                self.open_step_dialog(StepDialogMode::Edit(path), window, cx)
            }
            Some(RowKey::Branch(list)) | Some(RowKey::Add(list)) => {
                self.add_step_to(&list, window, cx)
            }
            None => {}
        }
    }

    /// Puts a step from the dialog into the flow and selects it.
    pub(super) fn insert_step(
        &mut self,
        list: &[usize],
        index: usize,
        step: Step,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(steps) = flows::list_at_mut(&mut self.flow.steps, list) else {
            return;
        };
        let index = index.min(steps.len());
        steps.insert(index, step);
        let mut path = list.to_vec();
        path.push(index);
        self.touch_steps(cx);
        self.select(RowKey::Step(path), window, cx);
    }

    /// Replaces what a step does, keeping the steps inside it.
    pub(super) fn replace_step(
        &mut self,
        path: &[usize],
        mut kind: StepKind,
        output: Option<String>,
        cx: &mut Context<Self>,
    ) {
        if let Some(step) = flows::step_at_mut(&mut self.flow.steps, path) {
            kind.adopt_branches(&step.kind);
            step.kind = kind;
            step.output = output;
        }
        self.touch_steps(cx);
    }

    fn remove_step(&mut self, path: &[usize], window: &mut Window, cx: &mut Context<Self>) {
        if self.refuse_while_running(window, cx) {
            return;
        }
        let Some((index, list)) = path.split_last() else {
            return;
        };
        // The line that takes the removed one's place keeps the keyboard.
        let rows = self.rows();
        let position = rows
            .iter()
            .position(|row| row.key == RowKey::Step(path.to_vec()));
        if let Some(steps) = flows::list_at_mut(&mut self.flow.steps, list)
            && *index < steps.len()
        {
            steps.remove(*index);
        }
        let rows = self.rows();
        self.selected = position
            .map(|ix| ix.min(rows.len().saturating_sub(1)))
            .and_then(|ix| rows.get(ix))
            .map(|row| row.key.clone());
        self.touch_steps(cx);
    }

    fn move_step(
        &mut self,
        path: &[usize],
        down: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.refuse_while_running(window, cx) {
            return;
        }
        let Some(moved) = flows::move_step(&mut self.flow.steps, path, down) else {
            return;
        };
        // A step moved into a collapsed block opens it, or it would vanish.
        for depth in (1..moved.len()).step_by(2) {
            if let Some(parent) = self.flow.step_at(&moved[..depth]) {
                self.collapsed.remove(&parent.uid);
            }
        }
        self.touch_steps(cx);
        self.select(RowKey::Step(moved), window, cx);
    }

    fn toggle_step(&mut self, path: &[usize], window: &mut Window, cx: &mut Context<Self>) {
        if self.refuse_while_running(window, cx) {
            return;
        }
        if let Some(step) = flows::step_at_mut(&mut self.flow.steps, path) {
            step.enabled = !step.enabled;
            self.touch_steps(cx);
        }
    }

    fn duplicate_step(&mut self, path: &[usize], window: &mut Window, cx: &mut Context<Self>) {
        if self.refuse_while_running(window, cx) {
            return;
        }
        let Some((index, list)) = path.split_last() else {
            return;
        };
        let Some(copy) = self.flow.step_at(path).map(Step::duplicate) else {
            return;
        };
        self.insert_step(list, index + 1, copy, window, cx);
    }

    /// Shows or hides what a block holds.
    fn set_collapsed(&mut self, path: &[usize], collapsed: bool, cx: &mut Context<Self>) {
        let Some(step) = self.flow.step_at(path) else {
            return;
        };
        if !step.kind.is_block() {
            return;
        }
        if collapsed {
            self.collapsed.insert(step.uid);
        } else {
            self.collapsed.remove(&step.uid);
        }
        cx.notify();
    }

    /// Left on a line: closes an open block, or goes to the block the
    /// line is in.
    fn collapse_or_parent(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (path, is_open_block) = match &self.selected {
            Some(RowKey::Step(path)) => {
                let open = self
                    .flow
                    .step_at(path)
                    .is_some_and(|s| s.kind.is_block() && !self.collapsed.contains(&s.uid));
                (path.clone(), open)
            }
            Some(RowKey::Branch(list)) | Some(RowKey::Add(list)) => {
                let mut parent = list.clone();
                parent.pop();
                self.select(RowKey::Step(parent), window, cx);
                return;
            }
            None => return,
        };
        if is_open_block {
            self.set_collapsed(&path, true, cx);
        } else if path.len() > 2 {
            self.select(RowKey::Step(path[..path.len() - 2].to_vec()), window, cx);
        }
    }

    // MARK: Render

    /// The rails that mark how deep a line sits: one per block around it.
    fn rails(depth: usize, cx: &App) -> Vec<Div> {
        let rail = StepGroup::Logic.accent(cx).opacity(0.45);
        (0..depth)
            .map(|_| {
                div()
                    .w(px(26.))
                    .flex_shrink_0()
                    .flex()
                    .justify_center()
                    .child(div().w(px(2.)).h_full().bg(rail))
            })
            .collect()
    }

    /// A line of the list: its rails, then what it shows.
    fn line(depth: usize, content: impl IntoElement, cx: &App) -> Div {
        div()
            .flex()
            .flex_row()
            .children(Self::rails(depth, cx))
            .child(div().flex_1().min_w_0().child(content))
    }

    /// Records where the selected line is, so it can be scrolled to.
    fn record_bounds(&self) -> impl IntoElement {
        let record = self.selected_bounds.clone();
        canvas(move |bounds, _, _| record.set(bounds), |_, _, _, _| {})
            .absolute()
            .top_0()
            .left_0()
            .size_full()
    }

    fn render_branch_row(
        &self,
        list: &[usize],
        label: String,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let accent = StepGroup::Logic.accent(cx);
        let key = RowKey::Branch(list.to_vec());
        h_flex()
            .id(branch_id("flow-branch", &self.flow, list))
            .test_support()
            .relative()
            .my_0p5()
            .px_3()
            .py_1()
            .gap_2()
            .items_center()
            .rounded(theme.radius)
            .border_1()
            .border_color(if selected {
                theme.ring
            } else {
                theme.transparent
            })
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(accent)
            .cursor_pointer()
            .when(selected, |this| this.child(self.record_bounds()))
            .child(
                Icon::new(Icon::empty())
                    .path("icons/corner-down-right.svg")
                    .size_3(),
            )
            .child(label)
            .on_click(cx.listener(move |this, _, window, cx| {
                this.steps_focus.focus(window, cx);
                this.select(key.clone(), window, cx);
            }))
    }

    fn render_add_row(
        &self,
        list: &[usize],
        selected: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let target = list.to_vec();
        h_flex()
            .id(branch_id("flow-add", &self.flow, list))
            .test_support()
            .relative()
            .my_0p5()
            .px_3()
            .py_1()
            .gap_2()
            .items_center()
            .rounded(theme.radius)
            .border_1()
            .border_color(if selected {
                theme.ring
            } else {
                theme.transparent
            })
            .text_xs()
            .text_color(theme.muted_foreground)
            .hover(|this| this.text_color(theme.foreground).bg(theme.secondary))
            .cursor_pointer()
            .when(selected, |this| this.child(self.record_bounds()))
            .child(Icon::new(Icon::empty()).path("icons/plus.svg").size_3())
            .child("Add step")
            .on_click(cx.listener(move |this, _, window, cx| {
                this.steps_focus.focus(window, cx);
                this.selected = Some(RowKey::Add(target.clone()));
                this.add_step_to(&target, window, cx);
            }))
    }

    #[allow(clippy::too_many_arguments)]
    fn render_step_row(
        &self,
        path: &StepPath,
        step: &Step,
        number: usize,
        summaries: &SummaryContext,
        selected: bool,
        connect: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let summary = summaries.summarize(&step.kind);
        let missing_program = program_of(&step.kind).filter(|p| self.missing.contains(p));
        let state = self
            .step_states
            .get(path)
            .cloned()
            .unwrap_or(StepState::Idle);
        let is_block = step.kind.is_block();
        let collapsed = self.collapsed.contains(&step.uid);
        let inside = step
            .kind
            .branches()
            .into_iter()
            .map(|branch| flows::walk(branch).len())
            .sum::<usize>();
        let round = self.rounds.get(path).copied();
        let failure = match &state {
            StepState::Failed(error) => Some(error.clone()),
            _ => None,
        };
        // The first line of what the step produced, so a run shows what its
        // variables will hold.
        let produced = match &state {
            StepState::Done(Some(output)) => {
                let first = output.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
                let more = output.lines().filter(|l| !l.trim().is_empty()).count() > 1;
                Some(if first.is_empty() {
                    "Nothing printed".to_string()
                } else if more {
                    format!("{first} …")
                } else {
                    first.to_string()
                })
            }
            _ => None,
        };
        // A move or delete can leave a step using a name nothing before it
        // saves; saving the flow refuses that, so point at the step now.
        let (list, index) = match path.split_last() {
            Some((index, list)) => (list, *index),
            None => (&[][..], 0),
        };
        let known = self.flow.names_at(list, index);
        let unknown: Vec<String> = step
            .kind
            .references()
            .into_iter()
            .filter(|n| !vars::is_builtin(n) && !known.contains(n))
            .collect();
        let unknown = (!unknown.is_empty()).then(|| {
            let names: Vec<String> = unknown.iter().map(|n| var_token::describe(n).0).collect();
            format!("Nothing earlier saves {}", names.join(", "))
        });
        let state_icon: Option<AnyElement> = match state {
            StepState::Idle => None,
            StepState::Running => Some(Spinner::new().small().into_any_element()),
            StepState::Done(_) => Some(
                Icon::new(Icon::empty())
                    .path("icons/circle-check.svg")
                    .size_4()
                    .text_color(theme.success)
                    .into_any_element(),
            ),
            StepState::Failed(_) => Some(
                Icon::new(Icon::empty())
                    .path("icons/circle-x.svg")
                    .size_4()
                    .text_color(theme.danger)
                    .into_any_element(),
            ),
            StepState::Cancelled => Some(
                Icon::new(Icon::empty())
                    .path("icons/ban.svg")
                    .size_4()
                    .text_color(theme.muted_foreground)
                    .into_any_element(),
            ),
        };
        let first = path.as_slice() == [0];
        let last = path.len() == 1 && path[0] + 1 == self.flow.steps.len();
        let muted_chip = |text: String| {
            div()
                .flex_shrink_0()
                .px_1p5()
                .rounded(theme.radius)
                .bg(theme.secondary)
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(text)
        };

        let (p_click, p_up, p_down, p_edit, p_remove, p_toggle, p_fold, p_test) = (
            path.clone(),
            path.clone(),
            path.clone(),
            path.clone(),
            path.clone(),
            path.clone(),
            path.clone(),
            path.clone(),
        );
        let card = h_flex()
            .id(("flow-step", number))
            .test_support()
            .relative()
            .gap_3()
            .items_center()
            .p_3()
            .rounded(theme.radius)
            .border_1()
            .border_color(if selected { theme.ring } else { theme.border })
            .bg(if selected {
                theme.secondary
            } else {
                theme.background
            })
            .hover(|s| s.bg(theme.secondary))
            .cursor_pointer()
            .when(selected, |this| this.child(self.record_bounds()))
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                this.steps_focus.focus(window, cx);
                this.select(RowKey::Step(p_click.clone()), window, cx);
                if event.click_count() >= 2 {
                    this.open_step_dialog(StepDialogMode::Edit(p_click.clone()), window, cx);
                }
            }))
            .child(
                div()
                    .size_6()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .bg(theme.secondary)
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.muted_foreground)
                    .child(number.to_string()),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .opacity(if step.enabled { 1. } else { 0.4 })
                    .child(summary.tile(px(28.), cx)),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_0p5()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .min_w_0()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .truncate()
                                    .text_color(if step.enabled {
                                        theme.foreground
                                    } else {
                                        theme.muted_foreground
                                    })
                                    .child(var_token::rich_text(
                                        &format!("step-title-{number}"),
                                        &summary.title,
                                        cx,
                                    )),
                            )
                            .when_some(step.output.clone(), |this, name| {
                                this.child(
                                    h_flex()
                                        .flex_shrink_0()
                                        .gap_1()
                                        .items_center()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child(
                                            Icon::new(Icon::empty())
                                                .path("icons/arrow-down.svg")
                                                .size_3(),
                                        )
                                        .child(var_token::token(&name, 0, cx)),
                                )
                            })
                            .when(is_block && collapsed, |this| {
                                this.child(muted_chip(format!(
                                    "{inside} step{}",
                                    if inside == 1 { "" } else { "s" }
                                )))
                            })
                            .when_some(round, |this, (round, of)| {
                                this.child(muted_chip(format!("{round} of {of}")))
                            }),
                    )
                    .when(!summary.detail.is_empty(), |this| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                // Reviewing an import: every character counts.
                                .when(
                                    !self.reviewing()
                                        && vars::references(&summary.detail).is_empty(),
                                    |this| this.truncate(),
                                )
                                .child(var_token::rich_text(
                                    &format!("step-detail-{number}"),
                                    &summary.detail,
                                    cx,
                                )),
                        )
                    })
                    .when_some(missing_program, |this, program| {
                        this.child(
                            h_flex()
                                .gap_1()
                                .items_center()
                                .text_xs()
                                .text_color(theme.warning)
                                .child(Icon::new(IconName::TriangleAlert).size_3())
                                .child(selectable(
                                    ("step-missing", number),
                                    format!("{program} is not installed"),
                                )),
                        )
                    })
                    .when_some(unknown, |this, message| {
                        this.child(
                            h_flex()
                                .id(("step-unknown-variable", number))
                                .test_support()
                                .gap_1()
                                .items_center()
                                .text_xs()
                                .text_color(theme.warning)
                                .child(Icon::new(IconName::TriangleAlert).size_3())
                                .child(selectable(("step-unknown-var", number), message)),
                        )
                    })
                    .when_some(produced, |this, produced| {
                        this.child(
                            h_flex()
                                .id(("step-result", number))
                                .test_support()
                                .gap_2()
                                .items_center()
                                .min_w_0()
                                .text_xs()
                                .child(
                                    div()
                                        .flex_shrink_0()
                                        .px_1p5()
                                        .rounded(theme.radius)
                                        .bg(theme.success.opacity(0.14))
                                        .text_color(theme.success)
                                        .child("Result"),
                                )
                                .child(
                                    div()
                                        .min_w_0()
                                        .truncate()
                                        .text_color(theme.foreground)
                                        .child(selectable(("step-output", number), produced)),
                                ),
                        )
                    })
                    .when_some(failure, |this, error| {
                        this.child(
                            div()
                                .id(("step-failure", number))
                                .test_support()
                                .text_xs()
                                .text_color(theme.danger)
                                .child(selectable(("step-error", number), error)),
                        )
                    }),
            )
            .children(state_icon)
            .child(
                h_flex()
                    .gap_0p5()
                    .flex_shrink_0()
                    .when(is_block, |this| {
                        this.child(
                            Button::new(("step-fold", number))
                                .ghost()
                                .xsmall()
                                .tab_stop(false)
                                .icon(Icon::new(Icon::empty()).path(if collapsed {
                                    "icons/chevron-right.svg"
                                } else {
                                    "icons/chevron-down.svg"
                                }))
                                .tooltip(if collapsed {
                                    "Show steps"
                                } else {
                                    "Hide steps"
                                })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    cx.stop_propagation();
                                    this.set_collapsed(&p_fold, !collapsed, cx);
                                })),
                        )
                    })
                    .child(
                        Button::new(("step-test", number))
                            .ghost()
                            .xsmall()
                            .tab_stop(false)
                            .disabled(self.running || self.reviewing())
                            .icon(Icon::new(Icon::empty()).path("icons/play.svg"))
                            .tooltip_with_action(
                                if self.running {
                                    "Wait for the run to finish"
                                } else if self.reviewing() {
                                    "Save the flow first"
                                } else {
                                    "Run only this step"
                                },
                                &TestStep,
                                Some(STEPS_CONTEXT),
                            )
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.test_step(&p_test, window, cx);
                            })),
                    )
                    .child(
                        Button::new(("step-up", number))
                            .ghost()
                            .xsmall()
                            .tab_stop(false)
                            .disabled(first)
                            .icon(Icon::new(Icon::empty()).path("icons/arrow-up.svg"))
                            .tooltip(if first { "Already first" } else { "Move up" })
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.move_step(&p_up, false, window, cx);
                            })),
                    )
                    .child(
                        Button::new(("step-down", number))
                            .ghost()
                            .xsmall()
                            .tab_stop(false)
                            .disabled(last)
                            .icon(Icon::new(Icon::empty()).path("icons/arrow-down.svg"))
                            .tooltip(if last { "Already last" } else { "Move down" })
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.move_step(&p_down, true, window, cx);
                            })),
                    )
                    .child(
                        Button::new(("step-edit", number))
                            .ghost()
                            .xsmall()
                            .tab_stop(false)
                            .icon(Icon::new(Icon::empty()).path("icons/pencil.svg"))
                            .tooltip("Edit")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.open_step_dialog(
                                    StepDialogMode::Edit(p_edit.clone()),
                                    window,
                                    cx,
                                );
                            })),
                    )
                    .child(
                        Button::new(("step-remove", number))
                            .ghost()
                            .xsmall()
                            .tab_stop(false)
                            .icon(Icon::new(Icon::empty()).path("icons/trash.svg"))
                            .tooltip("Remove")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.remove_step(&p_remove, window, cx);
                            })),
                    )
                    .child(
                        div().ml_1().child(
                            Switch::new(("step-enabled", number))
                                .small()
                                // The list is one tab stop; Space toggles.
                                .tab_stop(false)
                                .checked(step.enabled)
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    cx.stop_propagation();
                                    this.toggle_step(&p_toggle, window, cx);
                                })),
                        ),
                    ),
            );

        v_flex()
            .child(card)
            // The stroke under the step number that leads to the next step.
            .child(
                div()
                    .ml(px(25.))
                    .w(px(2.))
                    .h(px(10.))
                    .when(connect, |this| this.bg(theme.border)),
            )
    }

    pub(super) fn render_steps(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let list_focused = self.steps_focus.is_focused(window);
        let ring = focus::focus_border(list_focused, theme.transparent, cx);
        let summaries = SummaryContext {
            apps: &self.apps,
            flows: &self.flows,
        };
        let rows = self.rows();
        let numbers: Vec<StepPath> = self.flow.walk().into_iter().map(|(p, _)| p).collect();
        let mut list = v_flex()
            .id("flow-steps")
            .test_support()
            .key_context(STEPS_CONTEXT)
            .track_focus(&self.steps_focus)
            .on_action(cx.listener(|this, _: &StepUp, window, cx| this.select_by(-1, window, cx)))
            .on_action(cx.listener(|this, _: &StepDown, window, cx| this.select_by(1, window, cx)))
            .on_action(
                cx.listener(|this, _: &StepFirst, window, cx| this.select_edge(false, window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &StepLast, window, cx| this.select_edge(true, window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &EditStep, window, cx| this.activate_selected(window, cx)),
            )
            .on_action(cx.listener(|this, _: &RemoveStep, window, cx| {
                if let Some(path) = this.selected_step() {
                    this.remove_step(&path, window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &MoveStepUp, window, cx| {
                if let Some(path) = this.selected_step() {
                    this.move_step(&path, false, window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &MoveStepDown, window, cx| {
                if let Some(path) = this.selected_step() {
                    this.move_step(&path, true, window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &ToggleStep, window, cx| {
                if let Some(path) = this.selected_step() {
                    this.toggle_step(&path, window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &TestStep, window, cx| {
                if let Some(path) = this.selected_step() {
                    this.test_step(&path, window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &DuplicateStep, window, cx| {
                if let Some(path) = this.selected_step() {
                    this.duplicate_step(&path, window, cx);
                }
            }))
            .on_action(
                cx.listener(|this, _: &CollapseStep, window, cx| {
                    this.collapse_or_parent(window, cx)
                }),
            )
            .on_action(cx.listener(|this, _: &ExpandStep, _, cx| {
                if let Some(path) = this.selected_step() {
                    this.set_collapsed(&path, false, cx);
                }
            }))
            .rounded(theme.radius)
            .border_1()
            .border_color(ring)
            .p_0p5();
        for (ix, row) in rows.iter().enumerate() {
            let selected = list_focused && self.selected.as_ref() == Some(&row.key);
            let element = match &row.key {
                RowKey::Step(path) => {
                    let Some(step) = self.flow.step_at(path) else {
                        continue;
                    };
                    let number = numbers.iter().position(|p| p == path).map_or(0, |n| n + 1);
                    // A stroke leads to the next step when that is the
                    // line right below, on the same level.
                    let connect = rows.get(ix + 1).is_some_and(|next| {
                        next.depth == row.depth && matches!(next.key, RowKey::Step(_))
                    });
                    self.render_step_row(path, step, number, &summaries, selected, connect, cx)
                        .into_any_element()
                }
                RowKey::Branch(branch) => {
                    let label = branch
                        .split_last()
                        .and_then(|(b, step)| branch_label(&self.flow.step_at(step)?.kind, *b))
                        .unwrap_or_default();
                    self.render_branch_row(branch, label, selected, cx)
                        .into_any_element()
                }
                RowKey::Add(branch) => self.render_add_row(branch, selected, cx).into_any_element(),
            };
            list = list.child(Self::line(row.depth, element, cx));
        }
        if rows.is_empty() {
            list = list.child(
                v_flex()
                    .items_center()
                    .py_6()
                    .gap_1()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(selectable("no-steps", "No steps yet")),
            );
        }

        v_flex()
            .gap_3()
            .child(Self::section_title("STEPS", cx))
            .child(list)
            .child(
                Button::new("flow-add-step")
                    .outline()
                    .small()
                    .w_full()
                    .icon(Icon::new(Icon::empty()).path("icons/plus.svg"))
                    .label("Add step")
                    .tooltip_with_action("Add step", &AddStep, Some("FlowEditPage"))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| this.add_step_at_end(window, cx))),
            )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{Row, RowKey, rows};
    use crate::system::flows::condition::Condition;
    use crate::system::flows::{MenuChoice, Step, StepKind};

    fn wait() -> Step {
        Step::new(StepKind::Wait { ms: 1 })
    }

    fn keys(rows: &[Row]) -> Vec<(RowKey, usize)> {
        rows.iter().map(|r| (r.key.clone(), r.depth)).collect()
    }

    #[test]
    fn blocks_show_their_steps_branch_headings_and_add_lines() {
        let steps = vec![
            wait(),
            Step::new(StepKind::If {
                condition: Condition::OnBattery,
                not: false,
                then: vec![wait()],
                otherwise: Vec::new(),
            }),
            Step::new(StepKind::Menu {
                prompt: "Power".into(),
                choices: vec![MenuChoice {
                    label: "Lock".into(),
                    steps: Vec::new(),
                }],
            }),
        ];
        let all = rows(&steps, &HashSet::new());
        assert_eq!(
            keys(&all),
            vec![
                (RowKey::Step(vec![0]), 0),
                (RowKey::Step(vec![1]), 0),
                (RowKey::Step(vec![1, 0, 0]), 1),
                (RowKey::Add(vec![1, 0]), 1),
                (RowKey::Branch(vec![1, 1]), 1),
                (RowKey::Add(vec![1, 1]), 1),
                (RowKey::Step(vec![2]), 0),
                (RowKey::Branch(vec![2, 0]), 1),
                (RowKey::Add(vec![2, 0]), 1),
            ]
        );

        // A collapsed block is one line.
        let collapsed: HashSet<u64> = [steps[1].uid].into_iter().collect();
        let fewer = rows(&steps, &collapsed);
        assert_eq!(fewer.len(), all.len() - 4);
        assert_eq!(fewer[1].key, RowKey::Step(vec![1]));
        assert_eq!(fewer[2].key, RowKey::Step(vec![2]));
    }
}
