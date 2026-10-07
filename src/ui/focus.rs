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
use gpui_kit::TestSupportExt;

actions!(
    focus,
    [
        FocusNext,
        FocusPrev,
        EscapeToSidebar,
        ReloadPage,
        ShowCommands,
        ShowShortcuts
    ]
);

/// A focus handle that Tab/Shift-Tab can reach.
/// A dialog width that fits the window: `wanted` at most, and never wider
/// than the viewport minus a margin, so a dialog in a narrow tile keeps
/// its close button on screen.
pub fn dialog_width(wanted: f32, window: &Window) -> Pixels {
    let viewport: f32 = window.viewport_size().width.into();
    px(wanted.min((viewport - 32.).max(280.)))
}

/// A dialog list height that fits the window: `wanted` at most, and never
/// taller than the viewport minus room for the title and margins.
pub fn dialog_height(wanted: f32, window: &Window) -> Pixels {
    let viewport: f32 = window.viewport_size().height.into();
    px(wanted.min((viewport - 160.).max(160.)))
}

pub fn tab_stop(cx: &mut App) -> FocusHandle {
    cx.focus_handle().tab_stop(true)
}

/// Focuses `container`, then the first tab stop inside it once it has
/// rendered. Used when a page becomes active so the user lands on its first
/// control instead of on an invisible page root.
pub fn focus_first_in(container: &FocusHandle, window: &mut Window, cx: &mut App) {
    container.focus(window, cx);
    let container = container.clone();
    window.on_next_frame(move |window, cx| {
        if !container.is_focused(window) {
            return;
        }
        window.focus_next(cx);
        if !container.contains_focused(window, cx) {
            container.focus(window, cx);
        }
    });
}

/// Moves focus to the next (or previous) tab stop without leaving the active
/// focus trap (every dialog is one). Equivalent to `Root`'s own Tab handling;
/// used where a control's `tab` binding is overridden.
pub fn focus_next_trapped(forward: bool, window: &mut Window, cx: &mut App) {
    let step = |window: &mut Window, cx: &mut App| {
        if forward {
            window.focus_next(cx);
        } else {
            window.focus_prev(cx);
        }
    };
    let Some(trap) = gpui_base::active_focus_trap(window, cx) else {
        step(window, cx);
        return;
    };
    let start = window.focused(cx);
    step(window, cx);
    // A trap without tab stops would otherwise never terminate.
    for _ in 0..256 {
        if trap.contains_focused(window, cx) || window.focused(cx) == start {
            return;
        }
        step(window, cx);
    }
}

/// Border color for a focusable container: the theme ring when focused,
/// otherwise `base`.
pub fn focus_border(focused: bool, base: Hsla, cx: &App) -> Hsla {
    if focused { cx.theme().ring } else { base }
}

type ChangeHandler = Rc<dyn Fn(&bool, &mut Window, &mut App)>;

/// A `Switch` with a clickable label. The switch itself is the tab stop
/// and draws the focus ring (gpui-component's `Switch` handles Tab, Enter,
/// Space and the ring); the row adds nothing of its own, so a focused
/// switch is ringed once, never the whole row as well.
#[derive(IntoElement)]
pub struct FocusableSwitch {
    id: ElementId,
    checked: bool,
    disabled: bool,
    label: Option<SharedString>,
    /// The row fills its container, the label at one end and the switch
    /// at the other.
    between: bool,
    on_change: Option<ChangeHandler>,
}

impl FocusableSwitch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            disabled: false,
            label: None,
            between: false,
            on_change: None,
        }
    }

    pub fn between(mut self) -> Self {
        self.between = true;
        self
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
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let checked = self.checked;
        let disabled = self.disabled;
        let on_change = self.on_change.clone();

        h_flex()
            .id(self.id)
            .test_support()
            .gap_3()
            .items_center()
            // The row never grows past its cell: a long label wraps in
            // place of running into the next grid column.
            .max_w_full()
            .min_w_0()
            .when(self.between, |this| this.w_full().justify_between())
            .px_1()
            .py_0p5()
            .when(!disabled, |this| this.cursor_pointer())
            .when_some(self.label.clone(), |this, label| {
                this.child(div().min_w_0().flex_shrink(1.).child(label))
            })
            .child(
                Switch::new("switch")
                    .flex_none()
                    .checked(checked)
                    .disabled(disabled)
                    .when_some(on_change.clone(), |this, handler| {
                        this.on_click(move |value, window, cx| {
                            // Otherwise the row's own click handler toggles
                            // a second time.
                            cx.stop_propagation();
                            handler(value, window, cx)
                        })
                    }),
            )
            .when_some(on_change.filter(|_| !disabled), |this, handler| {
                // Clicks on the label toggle the switch; clicks and key
                // presses on the switch itself are consumed by it.
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

pub mod scroll {
    gpui::actions!(scroll, [LineUp, LineDown, PageUp, PageDown, Top, Bottom]);
}

/// The key context of [`scroll_area`]; its arrow, page and home/end keys
/// live in `shortcuts.rs`. Anything deeper that binds the same keys (an
/// input, a list, a select, a number field) keeps them.
pub const SCROLL_CONTEXT: &str = "ScrollArea";

/// How far one arrow press scrolls.
const SCROLL_LINE: f32 = 48.;

/// The wrapper for a scroll container's content: while focus is anywhere
/// inside, the arrow keys, Page Up/Down and Home/End scroll `handle`.
/// It goes inside the element that tracks the handle, around everything
/// that can take focus.
pub fn scroll_area(handle: &ScrollHandle) -> Div {
    let by = |handle: &ScrollHandle, delta: Pixels| {
        let mut offset = handle.offset();
        let max = handle.max_offset().y;
        offset.y = (offset.y - delta).clamp(-max, px(0.));
        handle.set_offset(offset);
    };
    let page = |handle: &ScrollHandle| (handle.bounds().size.height * 0.9).max(px(SCROLL_LINE));
    let (up, down, page_up, page_down, top, bottom) = (
        handle.clone(),
        handle.clone(),
        handle.clone(),
        handle.clone(),
        handle.clone(),
        handle.clone(),
    );
    div()
        .key_context(SCROLL_CONTEXT)
        .w_full()
        .on_action(move |_: &scroll::LineUp, window, _| {
            by(&up, px(-SCROLL_LINE));
            window.refresh();
        })
        .on_action(move |_: &scroll::LineDown, window, _| {
            by(&down, px(SCROLL_LINE));
            window.refresh();
        })
        .on_action(move |_: &scroll::PageUp, window, _| {
            by(&page_up, -page(&page_up));
            window.refresh();
        })
        .on_action(move |_: &scroll::PageDown, window, _| {
            by(&page_down, page(&page_down));
            window.refresh();
        })
        .on_action(move |_: &scroll::Top, window, _| {
            by(&top, px(-1_000_000.));
            window.refresh();
        })
        .on_action(move |_: &scroll::Bottom, window, _| {
            by(&bottom, px(1_000_000.));
            window.refresh();
        })
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

/// The body of a dialog: runs `on_submit` for the `dialog::Submit` action
/// (Ctrl+Enter). Enter is left to the focused control so a focused Cancel
/// button cancels; Tab is trapped by the dialog itself.
///
/// The library dialog binds Enter to `Confirm`, which closes it; that action
/// is stopped here so it never pre-empts the focused control's own click.
///
/// `test_support` wraps the element when the test feature is on, so the
/// return type is a trait bound rather than `Stateful<Div>`.
pub fn dialog_body(
    id: impl Into<ElementId>,
    focus: &FocusHandle,
    on_submit: impl Fn(&mut Window, &mut App) + 'static,
) -> impl ParentElement + StatefulInteractiveElement + Styled + IntoElement {
    div()
        .id(id)
        .test_support()
        .key_context(DIALOG_BODY_CONTEXT)
        .track_focus(focus)
        .on_action(|_: &gpui_component::dialog::Confirm, _, _| {})
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
pub fn scroll_into_view(scroll: &ScrollHandle, target: Bounds<Pixels>) {
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
