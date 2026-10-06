//! Section headings. A card's or a section's title is small, medium
//! weight, muted and in capitals (`DETAILS`, `WINDOW BORDERS`, `STEPS`)
//! wherever it appears; a page's title is the only larger heading, and a
//! control's label is sentence case.
use gpui::*;
use gpui_component::ActiveTheme;

/// A section heading, uppercased from the text given.
pub fn section(text: impl Into<SharedString>, cx: &App) -> Div {
    let text: SharedString = text.into().to_uppercase().into();
    div()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(cx.theme().muted_foreground)
        .child(text)
}
