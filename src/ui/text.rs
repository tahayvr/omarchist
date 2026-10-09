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

/// Words a title keeps in lower case, unless first or last.
const CONNECTORS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "nor", "of", "on", "in", "at", "to", "by", "for", "from",
    "with", "as", "up", "off", "into", "onto", "per", "via", "vs",
];

/// A flow's or template's name the way a title is shown: every word
/// capitalised except the connectors ("Four Sets of Pomodoros"), the
/// first and last word always. A word that already carries a capital
/// past its first letter (iPhone, GitHub, DNS) is left as it is; what is
/// stored never changes, only what is drawn.
pub fn title_case(name: &str) -> String {
    let words: Vec<&str> = name.split(' ').collect();
    let last = words.len().saturating_sub(1);
    let mut out = Vec::with_capacity(words.len());
    for (ix, word) in words.iter().enumerate() {
        let is_connector =
            ix != 0 && ix != last && CONNECTORS.contains(&word.to_lowercase().as_str());
        let has_inner_capital = word.chars().skip(1).any(char::is_uppercase);
        out.push(if is_connector {
            word.to_lowercase()
        } else if has_inner_capital {
            word.to_string()
        } else {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        });
    }
    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::title_case;

    #[test]
    fn titles_capitalise_every_word_but_the_connectors() {
        assert_eq!(title_case("morning start"), "Morning Start");
        assert_eq!(
            title_case("Four sets of pomodoros"),
            "Four Sets of Pomodoros"
        );
        assert_eq!(title_case("the end of the line"), "The End of the Line");
        assert_eq!(title_case("look up"), "Look Up");
        assert_eq!(
            title_case("sync my iPhone to GitHub"),
            "Sync My iPhone to GitHub"
        );
        assert_eq!(title_case("set DNS"), "Set DNS");
        assert_eq!(title_case(""), "");
        assert_eq!(title_case("a"), "A");
    }
}
