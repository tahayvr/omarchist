//! Selectable text. Every value the app shows the user (names, paths,
//! descriptions, messages, versions) is rendered through these helpers so
//! it can be selected with the mouse and copied, like text in a browser.
//! Labels of controls stay plain text.
use gpui::*;
use gpui_base::SelectableText;

/// A run of text the user can select and copy. It takes the text style of
/// its parent, so wrap it in a `div()` for size, weight, and color. `id`
/// must be unique among the run's siblings; runs rendered from a list take
/// their index, as in `("flow-name", ix)`.
pub fn selectable(id: impl Into<ElementId>, text: impl Into<SharedString>) -> SelectableText {
    SelectableText::new(id, text)
}

/// A `div` whose only child is a selectable run, for callers that need an
/// element to style or lay out.
pub fn selectable_div(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Div {
    div().child(selectable(id, text))
}
