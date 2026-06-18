//! Tag-block splitting and tag-name extraction

#[cfg(feature = "nostd")]
use alloc::{string::String, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::String, vec::Vec};

/// Split tags carefully, handling nested parentheses in \t tags
pub(super) fn split_tags_carefully(content: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' && depth == 0 {
            // Start of a new tag
            if !current.is_empty() {
                parts.push(current);
                current = String::new();
            }
        } else {
            current.push(ch);

            // Track parentheses depth for \t tags
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;

                // If we closed all parentheses, this tag is complete
                if depth == 0 && !current.is_empty() {
                    // Check if next char is a backslash (new tag) or continue
                    if chars.peek() == Some(&'\\') {
                        parts.push(current.clone());
                        current = String::new();
                    }
                }
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

/// Extract the tag name and its argument string from a single tag fragment.
pub(super) fn extract_tag_name_and_args(part: &str) -> (&str, &str) {
    // Extract tag name and arguments
    // Tag names can be alphabetic or start with a digit (like 1c, 3a, 4a)
    // Special handling for font name tag (fn) which can have letters immediately after
    if part.starts_with("fn") && part.len() > 2 {
        // Special case for \fn tag - everything after "fn" is the font name
        ("fn", &part[2..])
    } else if part.starts_with(|c: char| c.is_ascii_digit()) {
        // Colour/alpha tags (\1c \2c \3c \4c \1a \2a \3a \4a) are a digit plus
        // exactly one letter, with the value following immediately. Take just
        // those two chars so a malformed value like `\1cH&H2A4F5D&` (a stray
        // letter before the `&H`, very common in real scripts — 6500+ times in
        // the benchmark) still resolves to `1c` rather than an unknown `1cH`;
        // parse_color / parse_alpha tolerate the leftover prefix.
        if part.len() >= 2 {
            (&part[..2], &part[2..])
        } else {
            (part, "")
        }
    } else if let Some(idx) = part.find(|c: char| !c.is_ascii_alphabetic()) {
        (&part[..idx], &part[idx..])
    } else {
        (part, "")
    }
}
