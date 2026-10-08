//! The row a page starts with, and the controls it holds. Every page lays
//! it out the same way: `[Back] [Title] [search] … [secondary] [primary]`,
//! every control 24 px high (the kit's `small`), 12 px apart, wrapping on
//! a narrow window. A search field is 320 px wide wherever it appears; a
//! primary action is `.primary().small()` with an icon, a secondary one
//! `.outline().small()`, a utility (reload) `.ghost().small()`.
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputState};
use gpui_component::{Icon, IconName, Sizable, h_flex};

/// How wide a search field is, on every page and in every dialog.
pub const SEARCH_WIDTH: f32 = 320.;

/// The toolbar row.
pub fn bar() -> Div {
    h_flex().w_full().gap_3().items_center().flex_wrap()
}

/// A search field: the kit's small input with the magnifier and a clear
/// button. The caller adds its id and placeholder.
pub fn search_input(state: &Entity<InputState>) -> Input {
    Input::new(state)
        .small()
        .cleanable(true)
        .prefix(Icon::new(IconName::Search).size_4())
}

/// The box a search field sits in: `SEARCH_WIDTH`, never wider than the
/// row. Wrap it in the page's search key context where one applies.
pub fn search(child: impl IntoElement) -> Div {
    div().w(px(SEARCH_WIDTH)).max_w_full().child(child)
}

/// Back to the page this one was opened from.
pub fn back(
    id: impl Into<ElementId>,
    tooltip: &'static str,
    action: &dyn Action,
    context: Option<&str>,
) -> Button {
    Button::new(id)
        .ghost()
        .small()
        .icon(IconName::ArrowLeft)
        .label("Back")
        .tooltip_with_action(tooltip, action, context)
        .cursor_pointer()
}

/// A sub-page's title, next to Back: the same size as a page title.
pub fn title(text: impl Into<SharedString>) -> Div {
    div()
        .text_lg()
        .font_weight(FontWeight::SEMIBOLD)
        .whitespace_nowrap()
        .child(text.into())
}

/// Pushes what follows to the right end of the row.
pub fn spacer() -> Div {
    div().flex_1()
}
