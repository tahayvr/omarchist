// Text search over keybinds. Every whitespace-separated token must match
// the description, the command, or the chord (spelled both `super shift k`
// and `super+shift+k`); the score prefers prefix and word-start hits so
// "close" ranks "Close window" above "Disclose...".
use super::Keybind;

pub fn score(query: &str, bind: &Keybind) -> Option<u32> {
    let tokens: Vec<String> = query.split_whitespace().map(|t| t.to_lowercase()).collect();
    if tokens.is_empty() {
        return Some(0);
    }

    let chord = bind.chord.to_omarchy_string().to_lowercase();
    let haystacks = [
        bind.description.to_lowercase(),
        bind.dispatcher.text().to_lowercase(),
        chord.replace(" + ", " "),
        chord.replace(" + ", "+"),
    ];

    let mut total = 0;
    for token in &tokens {
        let best = haystacks
            .iter()
            .filter_map(|hay| token_score(token, hay))
            .max()?;
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
