// Keyboard-focus helpers shared by every page.
//
// Omarchist uses GPUI's own focus system: every interactive element is a
// tab stop (`FocusHandle::tab_stop(true)`), Tab/Shift-Tab walk them with
// `Window::focus_next/focus_prev`, and composites (the sidebar, tab strips,
// the theme grid, the keybinds table) are a single tab stop whose arrow
// keys move an index inside them. Focus rings are always derived from
// `FocusHandle::is_focused`, never from shadow state.
use std::cell::Cell;
use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, Disableable, h_flex, switch::Switch};

actions!(
    focus,
    [
        FocusNext,
        FocusPrev,
        EscapeToSidebar,
        ReloadPage,
        ShowShortcuts
    ]
);

/// A focus handle that Tab/Shift-Tab can reach.
pub fn tab_stop(cx: &mut App) -> FocusHandle {
    cx.focus_handle().tab_stop(true)
}

/// Focuses `container`, then the first tab stop inside it once it has
/// rendered. Used when a page becomes active so the user lands on its first
/// control instead of on an invisible page root.
pub fn focus_first_in(container: &FocusHandle, window: &mut Window) {
    container.focus(window);
    let container = container.clone();
    window.on_next_frame(move |window, cx| {
        if !container.is_focused(window) {
            return;
        }
        window.focus_next();
        if !container.contains_focused(window, cx) {
            container.focus(window);
        }
    });
}

/// Moves focus to the next (or previous) tab stop, skipping every stop
/// outside `container`. This is the focus trap for dialogs: Tab wraps
/// inside the dialog instead of escaping to the page behind it.
pub fn focus_next_within(container: &FocusHandle, forward: bool, window: &mut Window, cx: &App) {
    // Bounded by the number of tab stops in the window; the loop only runs
    // long when the container has no stops at all.
    for _ in 0..256 {
        if forward {
            window.focus_next();
        } else {
            window.focus_prev();
        }
        if container.contains_focused(window, cx) {
            return;
        }
    }
}

/// Border color for a focusable container: the theme ring when focused,
/// otherwise `base`.
pub fn focus_border(focused: bool, base: Hsla, cx: &App) -> Hsla {
    if focused { cx.theme().ring } else { base }
}

type ChangeHandler = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// A `Switch` inside a focusable row. gpui-component's `Switch` only reacts
/// to the mouse; the row is a tab stop that toggles on Enter/Space (GPUI
/// synthesizes a click for focused elements) and draws a focus ring.
#[derive(IntoElement)]
pub struct FocusableSwitch {
    id: ElementId,
    checked: bool,
    disabled: bool,
    label: Option<SharedString>,
    on_change: Option<ChangeHandler>,
}

impl FocusableSwitch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            disabled: false,
            label: None,
            on_change: None,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: Fn(&bool, &mut Window, &mut App) + 'static,
    {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for FocusableSwitch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| tab_stop(cx))
            .read(cx)
            .clone();
        let focused = focus_handle.is_focused(window);
        let checked = self.checked;
        let disabled = self.disabled;
        let on_change = self.on_change.clone();
        let radius = cx.theme().radius;
        let ring = focus_border(focused, cx.theme().transparent, cx);

        h_flex()
            .id(self.id)
            .track_focus(&focus_handle)
            .gap_3()
            .items_center()
            .px_1()
            .py_0p5()
            .rounded(radius)
            .border_1()
            .border_color(ring)
            .when(!disabled, |this| this.cursor_pointer())
            .when_some(self.label.clone(), |this, label| this.child(label))
            .child(
                Switch::new("switch")
                    .checked(checked)
                    .disabled(disabled)
                    .when_some(on_change.clone(), |this, handler| {
                        this.on_click(move |value, window, cx| handler(value, window, cx))
                    }),
            )
            .when_some(on_change.filter(|_| !disabled), |this, handler| {
                // Keyboard activation (and clicks on the label) toggle the
                // switch; clicks on the switch itself are consumed by it.
                this.on_click(move |_, window, cx| handler(&!checked, window, cx))
            })
    }
}

pub mod tab_strip {
    gpui::actions!(tab_strip, [Prev, Next, First, Last, Activate]);
}

pub mod dialog {
    gpui::actions!(dialog, [Submit]);
}

pub const TAB_STRIP_CONTEXT: &str = "TabStrip";
pub const DIALOG_BODY_CONTEXT: &str = "DialogBody";

/// Container for a `TabBar` that makes the whole strip one tab stop.
/// The owner handles the `tab_strip` actions (left/right/home/end/enter).
pub fn tab_strip_container(
    id: impl Into<ElementId>,
    focus: &FocusHandle,
    window: &Window,
    cx: &App,
) -> Stateful<Div> {
    let focused = focus.is_focused(window);
    div()
        .id(id)
        .key_context(TAB_STRIP_CONTEXT)
        .track_focus(focus)
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(focus_border(focused, cx.theme().transparent, cx))
}

/// The body of a dialog: traps Tab/Shift-Tab inside `focus` and runs
/// `on_submit` for the `dialog::Submit` action (Ctrl+Enter). Enter itself is
/// left to the focused control so a focused Cancel button cancels.
pub fn dialog_body(
    id: impl Into<ElementId>,
    focus: &FocusHandle,
    on_submit: impl Fn(&mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let next = focus.clone();
    let prev = focus.clone();
    div()
        .id(id)
        .key_context(DIALOG_BODY_CONTEXT)
        .track_focus(focus)
        .on_action(move |_: &FocusNext, window, cx| {
            focus_next_within(&next, true, window, cx);
        })
        .on_action(move |_: &FocusPrev, window, cx| {
            focus_next_within(&prev, false, window, cx);
        })
        .on_action(move |_: &dialog::Submit, window, cx| on_submit(window, cx))
}

struct SectionState {
    focus: FocusHandle,
    bounds: Rc<Cell<Bounds<Pixels>>>,
    was_focused: bool,
}

/// A form section inside a scrolling container. When keyboard focus enters
/// the section and it is not fully visible, the container scrolls just
/// enough to show it. Wrap each `form_section()` of a Designer tab in one.
#[derive(IntoElement)]
pub struct FocusSection {
    id: ElementId,
    scroll: ScrollHandle,
    children: Vec<AnyElement>,
}

impl FocusSection {
    pub fn new(id: impl Into<ElementId>, scroll: &ScrollHandle) -> Self {
        Self {
            id: id.into(),
            scroll: scroll.clone(),
            children: Vec::new(),
        }
    }
}

impl ParentElement for FocusSection {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for FocusSection {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id.clone(), cx, |_, cx| SectionState {
            focus: cx.focus_handle(),
            bounds: Rc::new(Cell::new(Bounds::default())),
            was_focused: false,
        });
        let (focus, bounds, was_focused) = {
            let state = state.read(cx);
            (state.focus.clone(), state.bounds.clone(), state.was_focused)
        };
        let focused = focus.contains_focused(window, cx);
        if focused != was_focused {
            state.update(cx, |state, _| state.was_focused = focused);
        }
        if focused && !was_focused {
            scroll_into_view(&self.scroll, bounds.get());
        }

        let record = bounds.clone();
        div()
            .id(self.id)
            .relative()
            .track_focus(&focus)
            .child(
                canvas(move |bounds, _, _| record.set(bounds), |_, _, _, _| {})
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full(),
            )
            .children(self.children)
    }
}

/// Scrolls `scroll` the minimum distance that brings `target` (window
/// coordinates from the last frame) into its viewport.
fn scroll_into_view(scroll: &ScrollHandle, target: Bounds<Pixels>) {
    let viewport = scroll.bounds();
    if viewport.size.height <= px(0.) || target.size.height <= px(0.) {
        return;
    }
    let mut offset = scroll.offset();
    if target.top() < viewport.top() {
        offset.y += viewport.top() - target.top();
    } else if target.bottom() > viewport.bottom() {
        let overflow = target.bottom() - viewport.bottom();
        let slack = target.top() - viewport.top();
        offset.y -= overflow.min(slack);
    } else {
        return;
    }
    scroll.set_offset(offset);
}
