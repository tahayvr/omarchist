pub fn strip_jsonc_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();
    let mut in_string = false;
    let mut in_block_comment = false;

    while let Some(ch) = chars.next() {
        if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if in_string {
            out.push(ch);
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            out.push(ch);
        } else if ch == '/' {
            match chars.peek() {
                Some('/') => {
                    for c in chars.by_ref() {
                        if c == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                }
                Some('*') => {
                    chars.next();
                    in_block_comment = true;
                }
                _ => {
                    out.push(ch);
                }
            }
        } else {
            out.push(ch);
        }
    }

    out
}

/// Find the byte position of `"key"` at depth 1 in the JSON source (i.e. a
/// top-level key of the root object), skipping nested objects/arrays and
/// string literals.
pub fn find_top_level_key(src: &str, key: &str) -> Option<usize> {
    let key_pat = format!("\"{}\"", key);
    let mut depth = 0usize;
    let mut in_str = false;
    let mut i = 0usize;
    let bytes = src.as_bytes();

    while i < bytes.len() {
        let ch = bytes[i] as char;
        if in_str {
            if ch == '\\' {
                i += 2;
                continue;
            } else if ch == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        match ch {
            '"' => {
                if depth == 1 && src[i..].starts_with(key_pat.as_str()) {
                    return Some(i);
                }
                in_str = true;
            }
            '{' | '[' => depth += 1,
            '}' | ']' if depth > 0 => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Return the byte position just past the end of the JSON value that starts at
/// `start` in `src`. Handles strings, objects/arrays, and scalars.
pub fn find_value_end(src: &str, start: usize) -> Option<usize> {
    let tail = &src[start..];
    let first_char = tail.chars().next()?;

    match first_char {
        '"' => {
            let mut i = 1usize;
            let bytes = tail.as_bytes();
            while i < bytes.len() {
                let c = bytes[i] as char;
                if c == '\\' {
                    i += 2;
                } else if c == '"' {
                    return Some(start + i + 1);
                } else {
                    i += 1;
                }
            }
            None
        }
        '{' | '[' => {
            let open = first_char;
            let close = if open == '{' { '}' } else { ']' };
            let mut depth = 0usize;
            let mut in_s = false;
            for (i, ch) in tail.char_indices() {
                if in_s {
                    if ch == '\\' {
                        continue;
                    } else if ch == '"' {
                        in_s = false;
                    }
                } else {
                    match ch {
                        '"' => in_s = true,
                        c if c == open => depth += 1,
                        c if c == close => {
                            depth -= 1;
                            if depth == 0 {
                                return Some(start + i + 1);
                            }
                        }
                        _ => {}
                    }
                }
            }
            None
        }
        _ => {
            let end = tail
                .find(|c: char| [',', '}', ']', '\n'].contains(&c))
                .unwrap_or(tail.len());
            Some(start + end)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_jsonc_comments_plain_json_is_unchanged() {
        let src = r#"{"key": "value", "num": 42}"#;
        assert_eq!(
            strip_jsonc_comments(src),
            src,
            "plain JSON without comments should pass through unchanged"
        );
    }

    #[test]
    fn strip_jsonc_comments_removes_line_comment() {
        let src = "{\n  \"key\": \"value\" // this is a comment\n}";
        let result = strip_jsonc_comments(src);
        assert!(
            !result.contains("this is a comment"),
            "line comment content should be stripped"
        );
        assert!(
            result.contains("\"key\": \"value\""),
            "key-value before comment should be preserved"
        );
    }

    #[test]
    fn strip_jsonc_comments_line_comment_preserves_newline() {
        // The newline after the comment must survive so line-counting stays valid.
        let src = "{\n  \"a\": 1 // comment\n}";
        let result = strip_jsonc_comments(src);
        assert_eq!(
            result.chars().filter(|&c| c == '\n').count(),
            2,
            "newlines should be preserved after stripping line comments"
        );
    }

    #[test]
    fn strip_jsonc_comments_removes_block_comment() {
        let src = "{ /* block comment */ \"key\": 1 }";
        let result = strip_jsonc_comments(src);
        assert!(
            !result.contains("block comment"),
            "block comment content should be stripped"
        );
        assert!(
            result.contains("\"key\": 1"),
            "content after block comment should be preserved"
        );
    }

    #[test]
    fn strip_jsonc_comments_preserves_url_inside_string() {
        let src = r#"{"url": "https://example.com"}"#;
        let result = strip_jsonc_comments(src);
        assert_eq!(result, src, "// inside a string value must not be stripped");
    }

    #[test]
    fn strip_jsonc_comments_preserves_block_comment_markers_inside_string() {
        let src = r#"{"desc": "use /* and */ for blocks"}"#;
        let result = strip_jsonc_comments(src);
        assert_eq!(
            result, src,
            "/* */ inside a string value must not be treated as a block comment"
        );
    }

    #[test]
    fn strip_jsonc_comments_handles_escaped_quote_inside_string() {
        let src = r#"{"key": "val\"ue // not a comment"}"#;
        let result = strip_jsonc_comments(src);
        assert!(
            result.contains("not a comment"),
            "// after an escaped quote inside a string should not be treated as a comment"
        );
    }

    #[test]
    fn strip_jsonc_comments_empty_input_returns_empty() {
        assert_eq!(
            strip_jsonc_comments(""),
            "",
            "empty input should return empty string"
        );
    }
}
