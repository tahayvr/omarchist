// Keyboard-focus helpers shared by every page.
//
// Omarchist uses GPUI's own focus system: every interactive element is a
// tab stop (`FocusHandle::tab_stop(true)`), Tab/Shift-Tab walk them with
// `Window::focus_next/focus_prev`, and composites (the sidebar, tab strips,
// the theme grid, the keybinds table) are a single tab stop whose arrow
// keys move an index inside them. Focus rings are always derived from
// `FocusHandle::is_focused`, never from shadow state.
use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, Disableable, h_flex, switch::Switch};

actions!(focus, [FocusNext, FocusPrev, EscapeToSidebar, ReloadPage]);

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
