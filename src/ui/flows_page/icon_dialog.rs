//! The dialog that picks a flow's icon: a search box over a grid of every
//! icon. The grid is one tab stop; the arrow keys move through it and
//! Enter picks, as does Enter in the search box.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, WindowExt, h_flex,
    input::{InputEvent, InputState},
    tooltip::Tooltip,
    v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::flows::ICONS;
use crate::ui::focus;
use crate::ui::text::selectable;
use crate::ui::toolbar;

pub const GRID_CONTEXT: &str = "IconPicker";
pub const SEARCH_CONTEXT: &str = "IconPickerSearch";

pub mod icon_picker_nav {
    gpui::actions!(
        icon_picker,
        [Left, Right, Up, Down, First, Last, Pick, FocusGrid]
    );
}
use icon_picker_nav::*;

const TILE: Pixels = px(44.);

pub enum IconDialogEvent {
    Picked(&'static str),
    Cancel,
}

pub struct IconDialog {
    search: Entity<InputState>,
    /// The flow's icon as it is now.
    current: String,
    grid_focus: FocusHandle,
    /// The highlighted icon, as an index into the filtered list.
    selected: usize,
    body_focus: FocusHandle,
    _subscription: Subscription,
}

impl EventEmitter<IconDialogEvent> for IconDialog {}

impl IconDialog {
    fn new(current: &str, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search icons"));
        let subscription = cx.subscribe_in(
            &search,
            window,
            |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::Change => this.select(0, cx),
                InputEvent::PressEnter { .. } => this.pick(window, cx),
                _ => {}
            },
        );
        let selected = ICONS.iter().position(|icon| *icon == current).unwrap_or(0);
        Self {
            search,
            current: current.to_string(),
            grid_focus: focus::tab_stop(cx),
            selected,
            body_focus: cx.focus_handle(),
            _subscription: subscription,
        }
    }

    /// The icons whose name holds the search text, every one without it.
    fn filtered(&self, cx: &App) -> Vec<&'static str> {
        let query = self.search.read(cx).value().trim().to_lowercase();
        ICONS
            .iter()
            .copied()
            .filter(|icon| query.is_empty() || icon.replace('-', " ").contains(&query))
            .collect()
    }

    fn columns(window: &Window) -> usize {
        if window.viewport_size().width < px(480.) {
            5
        } else {
            8
        }
    }

    fn select(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.selected = ix;
        cx.notify();
    }

    fn move_by(&mut self, delta: isize, cx: &mut Context<Self>) {
        let count = self.filtered(cx).len();
        if count == 0 {
            return;
        }
        let next = (self.selected as isize + delta).clamp(0, count as isize - 1);
        self.select(next as usize, cx);
    }

    /// Moves the highlight by whole rows, keeping the column; above the
    /// first row is the search box.
    fn move_rows(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        let count = self.filtered(cx).len();
        let columns = Self::columns(window) as isize;
        let target = self.selected as isize + delta * columns;
        if target < 0 {
            self.search.update(cx, |input, cx| input.focus(window, cx));
            return;
        }
        if target < count as isize {
            self.select(target as usize, cx);
        }
    }

    fn pick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(icon) = self.filtered(cx).get(self.selected).copied() {
            cx.emit(IconDialogEvent::Picked(icon));
            window.close_dialog(cx);
        }
    }

    fn render_tile(
        &self,
        ix: usize,
        icon: &'static str,
        grid_focused: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let selected = ix == self.selected;
        let current = self.current == icon;
        let ring = focus::focus_border(selected && grid_focused, theme.transparent, cx);
        div()
            .id(SharedString::from(format!("icon-{icon}")))
            .test_support()
            .size(TILE)
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .rounded(theme.radius)
            .border_1()
            .border_color(ring)
            .text_color(theme.foreground)
            .when(selected, |this| this.bg(theme.secondary))
            .when(current, |this| {
                this.bg(theme.primary.opacity(0.12))
                    .text_color(theme.primary)
            })
            .hover(|this| this.bg(theme.secondary))
            .cursor_pointer()
            .tooltip(move |window, cx| Tooltip::new(icon.replace('-', " ")).build(window, cx))
            .child(
                Icon::new(Icon::empty())
                    .path(format!("icons/{icon}.svg"))
                    .size(px(22.)),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                this.selected = ix;
                this.pick(window, cx);
            }))
    }
}

impl Render for IconDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let view = cx.entity();
        let icons = self.filtered(cx);
        let columns = Self::columns(window);
        let grid_focused = self.grid_focus.is_focused(window);
        self.selected = self.selected.min(icons.len().saturating_sub(1));

        let mut rows: Vec<AnyElement> = Vec::new();
        for (row, chunk) in icons.chunks(columns).enumerate() {
            let mut line = h_flex().gap_1();
            for (column, icon) in chunk.iter().enumerate() {
                line = line.child(self.render_tile(row * columns + column, icon, grid_focused, cx));
            }
            rows.push(line.into_any_element());
        }

        focus::dialog_body("icon-dialog", &self.body_focus, move |window, cx| {
            view.update(cx, |this, cx| this.pick(window, cx));
        })
        .child(
            v_flex()
                .gap_3()
                .child(
                    div()
                        .key_context(SEARCH_CONTEXT)
                        .on_action(cx.listener(|this, _: &FocusGrid, window, cx| {
                            this.grid_focus.focus(window, cx);
                            cx.notify();
                        }))
                        .child(toolbar::search_input(&self.search).id("icon-search")),
                )
                .child(
                    v_flex()
                        .id("icon-grid")
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
                        .on_action(cx.listener(|this, _: &Pick, window, cx| this.pick(window, cx)))
                        .gap_1()
                        .children(rows)
                        .when(icons.is_empty(), |this| {
                            this.child(
                                div()
                                    .py_6()
                                    .text_center()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child(selectable("icon-empty", "No icon matches")),
                            )
                        }),
                ),
        )
    }
}

/// Opens the dialog with `current` marked and returns its entity so the
/// caller can subscribe.
pub fn open_icon_dialog(current: &str, window: &mut Window, cx: &mut App) -> Entity<IconDialog> {
    let dialog = cx.new(|cx| IconDialog::new(current, window, cx));
    let view = dialog.clone();
    let body_focus = dialog.read(cx).body_focus.clone();
    window.open_dialog(cx, move |d, window, _| {
        let on_close_view = view.clone();
        d.title("Choose an icon")
            .w(focus::dialog_width(440., window))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .on_close(move |_, _, cx| {
                on_close_view.update(cx, |_, cx| cx.emit(IconDialogEvent::Cancel));
            })
            .child(view.clone())
    });
    focus::focus_first_in(&body_focus, window, cx);
    dialog
}
