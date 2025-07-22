//! Text normalization utilities for ASS subtitle processing
//!
//! Provides functionality for normalizing text content including line endings,
//! whitespace handling, and other text cleanup operations commonly needed
//! when processing ASS subtitle files from various sources and platforms.
//!
//! # Examples
//!
//! ```rust
//! use ass_core::utils::utf8::normalization::{normalize_line_endings, normalize_whitespace};
//!
//! let input = "Line 1\r\nLine 2\rLine 3\n";
//! let normalized = normalize_line_endings(input);
//! assert_eq!(normalized, "Line 1\nLine 2\nLine 3\n");
//! ```

use alloc::string::String;

/// Normalize line endings to Unix style (\n)
///
/// Converts Windows (\r\n) and classic Mac (\r) line endings to Unix (\n).
/// This ensures consistent line ending handling across different platforms
/// and source files.
///
/// # Arguments
///
/// * `text` - Input text with potentially mixed line endings
///
/// # Returns
///
/// String with normalized Unix line endings
///
/// # Examples
///
/// ```rust
/// # use ass_core::utils::utf8::normalization::normalize_line_endings;
/// let input = "Line 1\r\nLine 2\rLine 3\n";
/// let normalized = normalize_line_endings(input);
/// assert_eq!(normalized, "Line 1\nLine 2\nLine 3\n");
/// ```
pub fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// Normalize whitespace characters for consistent processing
///
/// Converts various Unicode whitespace characters to standard spaces
/// and optionally collapses multiple consecutive whitespace characters.
///
/// # Arguments
///
/// * `text` - Input text with potentially mixed whitespace
/// * `collapse_multiple` - Whether to collapse multiple spaces into one
///
/// # Returns
///
/// String with normalized whitespace
pub fn normalize_whitespace(text: &str, collapse_multiple: bool) -> String {
    let mut result = text
        .chars()
        .map(|c| {
            if c.is_whitespace() && c != '\n' && c != '\t' {
                ' ' // Convert all whitespace except newlines and tabs to space
            } else {
                c
            }
        })
        .collect::<String>();

    if collapse_multiple {
        result = collapse_consecutive_spaces(&result);
    }

    result
}

/// Remove or normalize control characters for safe text processing
///
/// Removes potentially problematic control characters while preserving
/// essential ones like newlines and tabs. Helps ensure text is safe
/// for processing and display.
///
/// # Arguments
///
/// * `text` - Input text that may contain control characters
///
/// # Returns
///
/// String with control characters removed or normalized
pub fn remove_control_chars(text: &str) -> String {
    text.chars()
        .filter(|&c| {
            // Keep printable characters, newlines, tabs, and carriage returns
            !c.is_control() || c == '\n' || c == '\t' || c == '\r'
        })
        .collect()
}

/// Trim whitespace from start and end of each line
///
/// Removes leading and trailing whitespace from each line while
/// preserving the line structure. Useful for cleaning up formatted
/// text that may have inconsistent indentation.
///
/// # Arguments
///
/// * `text` - Input text with potentially inconsistent line formatting
///
/// # Returns
///
/// String with trimmed lines
pub fn trim_lines(text: &str) -> String {
    text.lines()
        .map(|line| line.trim())
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Collapse consecutive whitespace characters into single spaces
///
/// Internal helper function that reduces multiple consecutive space
/// characters to single spaces while preserving newlines and tabs.
fn collapse_consecutive_spaces(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut prev_was_space = false;

    for c in text.chars() {
        if c == ' ' {
            if !prev_was_space {
                result.push(c);
                prev_was_space = true;
            }
        } else {
            result.push(c);
            prev_was_space = false;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_line_endings_windows() {
        let input = "Line 1\r\nLine 2\r\nLine 3";
        let normalized = normalize_line_endings(input);
        assert_eq!(normalized, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn normalize_line_endings_mac() {
        let input = "Line 1\rLine 2\rLine 3";
        let normalized = normalize_line_endings(input);
        assert_eq!(normalized, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn normalize_line_endings_mixed() {
        let input = "Line 1\r\nLine 2\rLine 3\n";
        let normalized = normalize_line_endings(input);
        assert_eq!(normalized, "Line 1\nLine 2\nLine 3\n");
    }

    #[test]
    fn normalize_line_endings_unix() {
        let input = "Line 1\nLine 2\nLine 3\n";
        let normalized = normalize_line_endings(input);
        assert_eq!(normalized, "Line 1\nLine 2\nLine 3\n");
    }

    #[test]
    fn normalize_whitespace_basic() {
        let input = "Hello\u{00A0}World\u{2000}Test"; // Non-breaking space and en quad
        let normalized = normalize_whitespace(input, false);
        assert_eq!(normalized, "Hello World Test");
    }

    #[test]
    fn normalize_whitespace_preserve_structure() {
        let input = "Hello\tWorld\nNext Line";
        let normalized = normalize_whitespace(input, false);
        assert_eq!(normalized, "Hello\tWorld\nNext Line");
    }

    #[test]
    fn normalize_whitespace_collapse() {
        let input = "Hello    World   Test";
        let normalized = normalize_whitespace(input, true);
        assert_eq!(normalized, "Hello World Test");
    }

    #[test]
    fn normalize_whitespace_no_collapse() {
        let input = "Hello    World   Test";
        let normalized = normalize_whitespace(input, false);
        assert_eq!(normalized, "Hello    World   Test");
    }

    #[test]
    fn remove_control_chars_basic() {
        let input = "Hello\x00World\x1FTest";
        let cleaned = remove_control_chars(input);
        assert_eq!(cleaned, "HelloWorldTest");
    }

    #[test]
    fn remove_control_chars_preserve_essential() {
        let input = "Hello\tWorld\nNext\rLine";
        let cleaned = remove_control_chars(input);
        assert_eq!(cleaned, "Hello\tWorld\nNext\rLine");
    }

    #[test]
    fn trim_lines_basic() {
        let input = "  Line 1  \n\t Line 2 \t\n   Line 3   ";
        let trimmed = trim_lines(input);
        assert_eq!(trimmed, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn trim_lines_empty_lines() {
        let input = "Line 1\n   \nLine 3";
        let trimmed = trim_lines(input);
        assert_eq!(trimmed, "Line 1\n\nLine 3");
    }

    #[test]
    fn collapse_consecutive_spaces_basic() {
        let input = "Hello    World   Test";
        let collapsed = collapse_consecutive_spaces(input);
        assert_eq!(collapsed, "Hello World Test");
    }

    #[test]
    fn collapse_consecutive_spaces_preserve_other() {
        let input = "Hello\t\tWorld\n\nTest";
        let collapsed = collapse_consecutive_spaces(input);
        assert_eq!(collapsed, "Hello\t\tWorld\n\nTest");
    }

    #[test]
    fn normalization_chain() {
        let input = "  Line 1  \r\n\t Line 2 \t\r   Line 3   ";
        let normalized = normalize_line_endings(input);
        let trimmed = trim_lines(&normalized);
        let final_result = normalize_whitespace(&trimmed, true);
        assert_eq!(final_result, "Line 1\nLine 2\nLine 3");
    }
}
