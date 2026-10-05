//! The first screen of the Add step dialog: every kind of step, grouped,
//! with a search box. The grid is one tab stop; the arrow keys move
//! through it and Enter picks.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable, h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};
use gpui_kit::TestSupportExt;

use crate::ui::flows_page::step_types::{StepChoice, StepGroup, StepType, best_match, search};
use crate::ui::focus;
use crate::ui::text::selectable;

pub const GRID_CONTEXT: &str = "StepPicker";
/// The row of group filters: one tab stop, arrows move the filter.
pub const GROUPS_CONTEXT: &str = "StepPickerGroups";
pub const SEARCH_CONTEXT: &str = "StepPickerSearch";

pub mod step_picker_nav {
    gpui::actions!(
        step_picker,
        [
            Left, Right, Up, Down, First, Last, Pick, FocusGrid, PrevGroup, NextGroup
        ]
    );
}
use step_picker_nav::*;

pub enum StepPickerEvent {
    Picked(StepChoice),
}

pub struct StepPicker {
    search: Entity<InputState>,
    /// The group the list is narrowed to; `None` shows every group.
    group: Option<StepGroup>,
    groups_focus: FocusHandle,
    grid_focus: FocusHandle,
    /// The highlighted step type, as an index into the filtered list.
    selected: usize,
    scroll: ScrollHandle,
    _subscription: Subscription,
}

impl EventEmitter<StepPickerEvent> for StepPicker {}

/// A square holding a step's icon, tinted with its group's colour.
pub fn icon_tile(icon: &'static str, accent: Hsla, size: Pixels, cx: &App) -> Div {
    div()
        .size(size)
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(cx.theme().radius)
        .bg(accent.opacity(0.16))
        .text_color(accent)
        .child(Icon::new(Icon::empty()).path(icon).size(size * 0.58))
}

/// A stable element id for a step type's tile, from its label.
pub fn tile_id(step: &StepType) -> ElementId {
    let slug: String = step
        .label
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    ElementId::Name(format!("step-type-{slug}").into())
}

impl StepPicker {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search steps"));
        let subscription = cx.subscribe_in(
            &search,
            window,
            |this, _, event: &InputEvent, _, cx| match event {
                InputEvent::Change => {
                    // Typing searches every group.
                    if !this.search.read(cx).value().trim().is_empty() {
                        this.group = None;
                    }
                    this.selected = best_match(this.search.read(cx).value().as_ref());
                    this.scroll.set_offset(point(px(0.), px(0.)));
                    cx.notify();
                }
                InputEvent::PressEnter { .. } => this.pick(cx),
                _ => {}
            },
        );
        Self {
            search,
            group: None,
            groups_focus: focus::tab_stop(cx),
            grid_focus: focus::tab_stop(cx),
            selected: 0,
            scroll: ScrollHandle::new(),
            _subscription: subscription,
        }
    }

    pub fn focus_search(&self, window: &mut Window, cx: &mut App) {
        self.search.update(cx, |input, cx| input.focus(window, cx));
    }

    fn filtered(&self, cx: &App) -> Vec<&'static StepType> {
        search(self.search.read(cx).value().as_ref())
            .into_iter()
            .filter(|step| self.group.is_none_or(|group| step.group == group))
            .collect()
    }

    fn set_group(&mut self, group: Option<StepGroup>, cx: &mut Context<Self>) {
        self.group = group;
        self.selected = 0;
        self.scroll.set_offset(point(px(0.), px(0.)));
        cx.notify();
    }

    /// Moves the group filter along All, then each group.
    fn cycle_group(&mut self, delta: isize, cx: &mut Context<Self>) {
        let all: Vec<Option<StepGroup>> = std::iter::once(None)
            .chain(StepGroup::ALL.into_iter().map(Some))
            .collect();
        let ix = all.iter().position(|g| *g == self.group).unwrap_or(0) as isize;
        let next = (ix + delta).rem_euclid(all.len() as isize) as usize;
        self.set_group(all[next], cx);
    }

    fn render_groups(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let focused = self.groups_focus.is_focused(window);
        let ring = focus::focus_border(focused, theme.transparent, cx);
        let chip = |id: SharedString, label: &'static str, group: Option<StepGroup>| {
            let selected = self.group == group;
            h_flex()
                .id(ElementId::Name(id))
                .test_support()
                .gap_1p5()
                .items_center()
                .px_2()
                .py_0p5()
                .rounded(theme.radius)
                .text_xs()
                .when(selected, |this| {
                    this.bg(theme.primary).text_color(theme.primary_foreground)
                })
                .when(!selected, |this| {
                    this.text_color(theme.muted_foreground)
                        .hover(|this| this.bg(theme.secondary).text_color(theme.foreground))
                })
                .cursor_pointer()
                .when_some(group, |this, group| {
                    this.child(div().size_2().rounded_full().bg(group.accent(cx)))
                })
                .child(label)
                .on_click(cx.listener(move |this, _, _, cx| this.set_group(group, cx)))
        };
        h_flex()
            .id("step-picker-groups")
            .test_support()
            .key_context(GROUPS_CONTEXT)
            .track_focus(&self.groups_focus)
            .on_action(cx.listener(|this, _: &PrevGroup, _, cx| this.cycle_group(-1, cx)))
            .on_action(cx.listener(|this, _: &NextGroup, _, cx| this.cycle_group(1, cx)))
            .rounded(theme.radius)
            .border_1()
            .border_color(ring)
            .p_0p5()
            .gap_0p5()
            .flex_wrap()
            .child(chip("step-group-all".into(), "All", None))
            .children(StepGroup::ALL.into_iter().map(|group| {
                chip(
                    format!("step-group-{}", group.short_label().to_lowercase()).into(),
                    group.short_label(),
                    Some(group),
                )
            }))
    }

    fn columns(window: &Window) -> usize {
        let width = window.viewport_size().width;
        if width < px(480.) {
            1
        } else if width < px(640.) {
            2
        } else {
            3
        }
    }

    /// The filtered types laid out as the grid draws them: per group, rows
    /// of indices into the filtered list.
    fn layout(types: &[&'static StepType], columns: usize) -> Vec<(StepGroup, Vec<Vec<usize>>)> {
        let mut groups = Vec::new();
        for group in StepGroup::ALL {
            let members: Vec<usize> = types
                .iter()
                .enumerate()
                .filter(|(_, t)| t.group == group)
                .map(|(ix, _)| ix)
                .collect();
            if !members.is_empty() {
                let rows = members
                    .chunks(columns.max(1))
                    .map(<[usize]>::to_vec)
                    .collect();
                groups.push((group, rows));
            }
        }
        groups
    }

    fn pick(&mut self, cx: &mut Context<Self>) {
        if let Some(step) = self.filtered(cx).get(self.selected) {
            cx.emit(StepPickerEvent::Picked(step.choice));
        }
    }

    /// Moves the highlight by whole rows, keeping the column where the
    /// next row is long enough.
    fn move_rows(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        let types = self.filtered(cx);
        let rows: Vec<Vec<usize>> = Self::layout(&types, Self::columns(window))
            .into_iter()
            .flat_map(|(_, rows)| rows)
            .collect();
        let Some(row) = rows.iter().position(|r| r.contains(&self.selected)) else {
            return;
        };
        let column = rows[row]
            .iter()
            .position(|ix| *ix == self.selected)
            .unwrap_or(0);
        let target = row as isize + delta;
        if target < 0 {
            // Above the first row is the search box.
            self.focus_search(window, cx);
            return;
        }
        let Some(next) = rows.get(target as usize) else {
            return;
        };
        self.select(next[column.min(next.len() - 1)], cx);
    }

    fn move_by(&mut self, delta: isize, cx: &mut Context<Self>) {
        let count = self.filtered(cx).len();
        if count == 0 {
            return;
        }
        let next = (self.selected as isize + delta).clamp(0, count as isize - 1);
        self.select(next as usize, cx);
    }

    fn select(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.selected = ix;
        cx.notify();
    }

    fn render_tile(
        &self,
        ix: usize,
        step: &'static StepType,
        grid_focused: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let selected = ix == self.selected;
        let ring = focus::focus_border(selected && grid_focused, theme.transparent, cx);
        let choice = step.choice;
        h_flex()
            .id(tile_id(step))
            .test_support()
            .w_full()
            .gap_2()
            .items_center()
            .p_1p5()
            .rounded(theme.radius)
            .border_1()
            .border_color(ring)
            .when(selected, |this| this.bg(theme.secondary))
            .hover(|this| this.bg(theme.secondary))
            .cursor_pointer()
            .child(icon_tile(step.icon, step.group.accent(cx), px(28.), cx))
            .child(div().min_w_0().text_sm().truncate().child(step.label))
            .on_click(cx.listener(move |_, _, _, cx| {
                cx.emit(StepPickerEvent::Picked(choice));
            }))
    }
}

impl Render for StepPicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let types = self.filtered(cx);
        let columns = Self::columns(window);
        let layout = Self::layout(&types, columns);
        let grid_focused = self.grid_focus.is_focused(window);
        self.selected = self.selected.min(types.len().saturating_sub(1));

        // Headings and rows are the scroll container's direct children, so
        // the highlighted row can be scrolled into view by its index.
        let mut children: Vec<AnyElement> = Vec::new();
        let mut selected_child = None;
        for (group, rows) in &layout {
            children.push(
                div()
                    .pt_2()
                    .pb_1()
                    .px_1p5()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.muted_foreground)
                    .child(group.label().to_uppercase())
                    .into_any_element(),
            );
            for row in rows {
                if row.contains(&self.selected) {
                    selected_child = Some(children.len());
                }
                // Equal cells, so a short last row lines up with the rows
                // above it.
                let mut line = h_flex().gap_1().items_stretch();
                for ix in row {
                    line = line.child(div().flex_1().min_w_0().child(self.render_tile(
                        *ix,
                        types[*ix],
                        grid_focused,
                        cx,
                    )));
                }
                for _ in row.len()..columns {
                    line = line.child(div().flex_1().min_w_0());
                }
                children.push(line.into_any_element());
            }
        }
        if let Some(child) = selected_child {
            self.scroll.scroll_to_item(child);
        }

        let grid_ring = focus::focus_border(false, theme.transparent, cx);
        v_flex()
            .gap_2()
            .child(
                div()
                    .key_context(SEARCH_CONTEXT)
                    .on_action(cx.listener(|this, _: &FocusGrid, window, cx| {
                        this.grid_focus.focus(window, cx);
                        cx.notify();
                    }))
                    .child(
                        Input::new(&self.search)
                            .id("step-search")
                            .small()
                            .prefix(Icon::new(IconName::Search).small())
                            .cleanable(true),
                    ),
            )
            .child(self.render_groups(window, cx))
            .child(
                v_flex()
                    .id("step-picker-grid")
                    .test_support()
                    .key_context(GRID_CONTEXT)
                    .track_focus(&self.grid_focus)
                    .on_action(cx.listener(|this, _: &Left, _, cx| this.move_by(-1, cx)))
                    .on_action(cx.listener(|this, _: &Right, _, cx| this.move_by(1, cx)))
                    .on_action(
                        cx.listener(|this, _: &Up, window, cx| this.move_rows(-1, window, cx)),
                    )
                    .on_action(
                        cx.listener(|this, _: &Down, window, cx| this.move_rows(1, window, cx)),
                    )
                    .on_action(cx.listener(|this, _: &First, _, cx| this.select(0, cx)))
                    .on_action(cx.listener(|this, _: &Last, _, cx| {
                        let last = this.filtered(cx).len().saturating_sub(1);
                        this.select(last, cx);
                    }))
                    .on_action(cx.listener(|this, _: &Pick, _, cx| this.pick(cx)))
                    .rounded(theme.radius)
                    .border_1()
                    .border_color(grid_ring)
                    .child(
                        v_flex()
                            .id("step-picker-list")
                            .max_h(focus::dialog_height(420., window))
                            .overflow_y_scroll()
                            .track_scroll(&self.scroll)
                            .children(children),
                    )
                    .when(types.is_empty(), |this| {
                        this.child(
                            div()
                                .py_6()
                                .text_center()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child(selectable("step-picker-empty", "No step matches")),
                        )
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{StepGroup, StepPicker, StepType};
    use crate::ui::flows_page::step_types::step_types;

    #[test]
    fn the_grid_keeps_groups_together_in_rows() {
        let types: Vec<&'static StepType> = step_types().iter().collect();
        let layout = StepPicker::layout(&types, 3);
        assert_eq!(layout[0].0, StepGroup::Apps);
        let seen: Vec<usize> = layout
            .iter()
            .flat_map(|(_, rows)| rows.iter().flatten().copied())
            .collect();
        assert_eq!(seen, (0..step_types().len()).collect::<Vec<_>>());
        assert!(
            layout
                .iter()
                .all(|(_, rows)| rows.iter().all(|r| r.len() <= 3))
        );
    }
}
