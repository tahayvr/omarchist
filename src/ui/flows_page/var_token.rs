//! Flow variables drawn as tokens: a small pill with an icon and a readable
//! name, in place of the `{{name}}` text a flow file stores.
use gpui::*;
use gpui_component::{ActiveTheme, Icon, h_flex};

use crate::system::flows::vars;
use crate::ui::text::selectable;

/// The readable name and icon of a variable: built-ins get their own,
/// names a step saves read as written.
pub fn describe(name: &str) -> (String, &'static str) {
    match vars::normalize(name).as_str() {
        "clipboard" => ("Clipboard".into(), "icons/copy.svg"),
        "selection" => ("Selected text".into(), "icons/pen-tool.svg"),
        "date" => ("Date".into(), "icons/calendar.svg"),
        "time" => ("Time".into(), "icons/clock.svg"),
        "window" => ("Window title".into(), "icons/app-window.svg"),
        "app" => ("App".into(), "icons/square.svg"),
        "workspace" => ("Workspace".into(), "icons/layout-grid.svg"),
        "input" => ("Input".into(), "icons/log-in.svg"),
        "item" => ("Item".into(), "icons/repeat-2.svg"),
        "index" => ("Round".into(), "icons/hash.svg"),
        other => (other.to_string(), "icons/sparkles.svg"),
    }
}

/// One variable as a pill.
pub fn token(name: &str, cx: &App) -> Div {
    let theme = cx.theme();
    let (label, icon) = describe(name);
    h_flex()
        .flex_shrink_0()
        .gap_1()
        .items_center()
        .px_1p5()
        .rounded(theme.radius)
        .bg(theme.primary.opacity(0.14))
        .text_color(theme.primary)
        .child(Icon::new(Icon::empty()).path(icon).size_3())
        .child(label)
}

/// `text` with every `{{name}}` drawn as a token and the rest as
/// selectable text, wrapping as one line of words would.
pub fn rich_text(id: &str, text: &str, cx: &App) -> AnyElement {
    let refs = vars::references(text);
    if refs.is_empty() {
        return div()
            .child(selectable(
                SharedString::from(id.to_string()),
                text.to_string(),
            ))
            .into_any_element();
    }
    let mut parts: Vec<AnyElement> = Vec::new();
    let mut last = 0;
    for (ix, r) in refs.iter().enumerate() {
        let before = &text[last..r.start];
        if !before.trim().is_empty() {
            parts.push(
                div()
                    .child(selectable(
                        SharedString::from(format!("{id}-text-{ix}")),
                        before.trim().to_string(),
                    ))
                    .into_any_element(),
            );
        }
        parts.push(token(&r.name, cx).into_any_element());
        last = r.end;
    }
    let after = &text[last..];
    if !after.trim().is_empty() {
        parts.push(
            div()
                .child(selectable(
                    SharedString::from(format!("{id}-text-end")),
                    after.trim().to_string(),
                ))
                .into_any_element(),
        );
    }
    h_flex()
        .gap_1()
        .flex_wrap()
        .items_center()
        .min_w_0()
        .children(parts)
        .into_any_element()
}
