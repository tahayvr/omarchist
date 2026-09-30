// Text search over keybinds. Every whitespace-separated token must match
// the description, the command, or the chord (spelled both `super shift k`
// and `super+shift+k`); the score prefers prefix and word-start hits so
// "close" ranks "Close window" above "Disclose...".
use super::Keybind;
use super::keymap::chord_display_parts;

pub fn score(query: &str, bind: &Keybind) -> Option<u32> {
    let tokens: Vec<String> = query.split_whitespace().map(|t| t.to_lowercase()).collect();
    if tokens.is_empty() {
        return Some(0);
    }

    let chord = bind.chord.to_omarchy_string().to_lowercase();
    // The chips show "Enter", "Page Up", "1" (for code:10): search must
    // find what the table shows, not only the Lua spelling.
    let shown = chord_display_parts(&bind.chord).join(" ").to_lowercase();
    let haystacks = [
        bind.description.to_lowercase(),
        bind.dispatcher.text().to_lowercase(),
        chord.replace(" + ", " "),
        chord.replace(" + ", "+"),
        shown.clone(),
    ];
    // A token that names a key on its own must match a whole key, so "1"
    // finds workspace 1 and not every workspace (code:1x, 10..19).
    let key_parts: Vec<String> = shown.split(' ').map(str::to_string).collect();

    let mut total = 0;
    for token in &tokens {
        let best = haystacks
            .iter()
            .filter_map(|hay| token_score(token, hay))
            .max();
        let best = match best {
            Some(score) if token.chars().all(|c| c.is_ascii_digit()) => {
                // Digits: only an exact key part or a word in the text counts.
                let exact_key = key_parts.iter().any(|part| part == token);
                let in_text = haystacks[..2].iter().any(|hay| {
                    hay.split(|c: char| !c.is_alphanumeric())
                        .any(|word| word == token)
                });
                if exact_key || in_text {
                    score
                } else {
                    return None;
                }
            }
            Some(score) => score,
            None => return None,
        };
        total += best;
    }
    Some(total)
}

fn token_score(token: &str, haystack: &str) -> Option<u32> {
    let position = haystack.find(token)?;
    if position == 0 {
        return Some(3);
    }
    let at_word_start = haystack[..position]
        .chars()
        .next_back()
        .is_some_and(|c| !c.is_alphanumeric());
    Some(if at_word_start { 2 } else { 1 })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::system::keybinds::{BindOptions, BindStatus, Chord, Dispatcher, Origin};

    #[test]
    fn search_matches_the_labels_the_table_shows() {
        let terminal = bind("SUPER + RETURN", "Terminal", "omarchy-launch-terminal");
        assert!(score("enter", &terminal).is_some(), "chip label");
        assert!(score("return", &terminal).is_some(), "lua spelling");
        let ws1 = bind("SUPER + code:10", "Switch to workspace 1", "x");
        let ws10 = bind("SUPER + code:19", "Switch to workspace 10", "x");
        assert!(score("super 1", &ws1).is_some());
        assert!(score("super 1", &ws10).is_none(), "1 is not a prefix of 10");
        assert!(score("10", &ws10).is_some());
    }

    fn bind(keys: &str, description: &str, command: &str) -> Keybind {
        Keybind {
            seq: 1,
            chord: Chord::parse(keys).unwrap(),
            keys_raw: keys.into(),
            description: description.into(),
            dispatcher: Dispatcher::Exec(command.into()),
            options: BindOptions::default(),
            origin: Origin::Default,
            source: PathBuf::from("/x.lua"),
            status: BindStatus::Active,
        }
    }

    #[test]
    fn empty_query_matches_everything() {
        assert_eq!(score("  ", &bind("SUPER + K", "Keybindings", "x")), Some(0));
    }

    #[test]
    fn all_tokens_must_match_across_fields() {
        let b = bind(
            "SUPER + SHIFT + K",
            "Keybindings",
            "omarchy-menu-keybindings",
        );
        assert!(score("super k", &b).is_some());
        assert!(score("shift+k", &b).is_some());
        assert!(score("menu", &b).is_some());
        assert!(score("menu window", &b).is_none());
    }

    #[test]
    fn prefix_ranks_above_substring() {
        let close = bind("SUPER + W", "Close window", "x");
        let disclose = bind("SUPER + D", "Disclose", "x");
        let word = bind("SUPER + Q", "Force close", "x");
        assert!(score("close", &close) > score("close", &word));
        assert!(score("close", &word) > score("close", &disclose));
    }
}
