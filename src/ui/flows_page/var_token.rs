//! Flow variables drawn as tokens: a small pill with an icon and a readable
//! name, in place of the `{{name}}` text a flow file stores. Each distinct
//! variable in a text wears its own colour from the theme's palette, in
//! the order the variables first appear, so two of them in one command are
//! told apart at a glance.
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, h_flex,
    input::{InlineToken, InlineTokenContext, Input, InputContent, InputState},
};

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

/// The colour of the `slot`th distinct variable in a text: the palette's
/// hues in turn, red left out because it means an error elsewhere.
pub fn accent(slot: usize, cx: &App) -> Hsla {
    let theme = cx.theme();
    let hues = [
        theme.blue,
        theme.magenta,
        theme.cyan,
        theme.yellow,
        theme.green,
    ];
    hues[slot % hues.len()]
}

/// The slot of each name among `names` in order of first appearance, so
/// the same variable used twice keeps one colour.
fn slots<'a>(names: impl Iterator<Item = &'a str>) -> Vec<usize> {
    let mut seen: Vec<String> = Vec::new();
    names
        .map(|name| {
            let name = vars::normalize(name);
            match seen.iter().position(|s| *s == name) {
                Some(slot) => slot,
                None => {
                    seen.push(name);
                    seen.len() - 1
                }
            }
        })
        .collect()
}

/// One variable as a pill, in the colour of its `slot` (its place among
/// the distinct variables shown next to it).
pub fn token(name: &str, slot: usize, cx: &App) -> Div {
    let theme = cx.theme();
    let (label, icon) = describe(name);
    let accent = accent(slot, cx);
    h_flex()
        .flex_shrink_0()
        .gap_1()
        .items_center()
        .px_1p5()
        .rounded(theme.radius)
        .bg(accent.opacity(0.16))
        .text_color(accent)
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
    let slots = slots(refs.iter().map(|r| r.name.as_str()));
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
        parts.push(token(&r.name, slots[ix], cx).into_any_element());
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

/// A variable as an atomic token inside a text field: the field's text
/// keeps the `{{name}}` the flow file stores, so what reads the field
/// sees no difference, while the person sees the pill.
pub fn inline(name: &str) -> InlineToken {
    InlineToken::new(name.to_string(), format!("{{{{{name}}}}}")).with_label(describe(name).0)
}

/// `text` with every `{{name}}` in it attached as a token, as a field is
/// filled when it opens on saved text.
pub fn content(text: &str) -> InputContent {
    vars::references(text)
        .into_iter()
        .fold(InputContent::new(text.to_string()), |content, r| {
            content
                .clone()
                .with_token(r.start..r.end, inline(&r.name))
                .unwrap_or(content)
        })
}

/// A single-line field holding `value`, its variables as tokens.
pub fn field(
    placeholder: &str,
    value: &str,
    window: &mut Window,
    cx: &mut App,
) -> Entity<InputState> {
    let input = cx.new(|cx| InputState::new(window, cx).placeholder(placeholder.to_string()));
    if !value.is_empty() {
        input.update(cx, |input, cx| input.set_value(content(value), window, cx));
    }
    input
}

/// Puts the variable where the cursor is, as a token, and gives the field
/// the keyboard.
pub fn insert(input: &Entity<InputState>, name: &str, window: &mut Window, cx: &mut App) {
    input.update(cx, |input, cx| {
        // Text, should the token be refused (never for a valid name).
        if input.replace_with_token(inline(name), window, cx).is_err() {
            input.insert(format!("{{{{{name}}}}}"), window, cx);
        }
        input.focus(window, cx);
    });
}

/// Draws a token in a field the way [`token`] draws one elsewhere, in the
/// colour of its place among the field's distinct variables.
pub fn render_inline(context: &InlineTokenContext, slot: usize, cx: &App) -> AnyElement {
    let theme = cx.theme();
    let (label, icon) = describe(context.token().id());
    let accent = accent(slot, cx);
    h_flex()
        .id("variable-token")
        .flex_shrink_0()
        .gap_1()
        .items_center()
        .h(context.line_height())
        .max_w(context.available_width())
        .px_1p5()
        .rounded(theme.radius)
        .bg(if context.is_selected() {
            theme.selection
        } else {
            accent.opacity(0.16)
        })
        .text_color(accent)
        .child(Icon::new(Icon::empty()).path(icon).size_3())
        .child(div().min_w_0().text_ellipsis().child(label))
        .into_any_element()
}

/// The field drawn with its variables as tokens, each distinct one in its
/// own colour. `state` is the field's state, read for the order the
/// variables appear in.
pub fn with_tokens(state: &Entity<InputState>, input: Input) -> Input {
    let state = state.clone();
    input.token(move |context, _, cx| {
        let slot = {
            let state = state.read(cx);
            let mut spans: Vec<_> = state.tokens().to_vec();
            spans.sort_by_key(|span| span.range().start);
            let slots = slots(spans.iter().map(|span| span.token().id().as_ref()));
            spans
                .iter()
                .position(|span| span.range() == context.range())
                .map(|ix| slots[ix])
                .unwrap_or(0)
        };
        render_inline(context, slot, cx)
    })
}
