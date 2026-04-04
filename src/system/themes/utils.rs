pub fn dir_to_title(dir_name: &str) -> String {
    let mut title = String::with_capacity(dir_name.len() + 10);
    let mut capitalize_next = true;

    for ch in dir_name.chars() {
        match ch {
            '-' | '_' => {
                title.push(' ');
                capitalize_next = true;
            }
            c if capitalize_next => {
                title.extend(c.to_uppercase());
                capitalize_next = false;
            }
            c => title.push(c),
        }
    }
    title
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dir_to_title_plain_word_is_capitalised() {
        assert_eq!(dir_to_title("catppuccin"), "Catppuccin");
    }

    #[test]
    fn dir_to_title_hyphen_becomes_space_and_capitalises_next_word() {
        assert_eq!(dir_to_title("rose-pine"), "Rose Pine");
    }

    #[test]
    fn dir_to_title_underscore_becomes_space_and_capitalises_next_word() {
        assert_eq!(dir_to_title("gruvbox_dark"), "Gruvbox Dark");
    }

    #[test]
    fn dir_to_title_multiple_separators() {
        assert_eq!(dir_to_title("one-two_three"), "One Two Three");
    }

    #[test]
    fn dir_to_title_empty_string_returns_empty() {
        assert_eq!(
            dir_to_title(""),
            "",
            "empty input should return empty string"
        );
    }

    #[test]
    fn dir_to_title_already_capitalised_first_char_unchanged() {
        // The function capitalises the first character; if it's already upper it stays upper.
        assert_eq!(dir_to_title("Tokyo"), "Tokyo");
    }

    #[test]
    fn dir_to_title_consecutive_separators_produce_consecutive_spaces() {
        // Two consecutive hyphens produce two spaces; second word is still capitalised.
        let result = dir_to_title("a--b");
        assert_eq!(result, "A  B");
    }
}
