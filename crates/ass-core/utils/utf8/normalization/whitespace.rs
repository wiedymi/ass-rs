//! Whitespace normalization helpers for ASS subtitle text.
//!
//! Provides conversion of Unicode whitespace to plain spaces, optional
//! collapsing of consecutive spaces, and per-line trimming.

use alloc::{string::String, vec::Vec};

use super::collapse_consecutive_spaces;

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
#[must_use]
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
#[must_use]
pub fn trim_lines(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .collect::<Vec<&str>>()
        .join("\n")
}
