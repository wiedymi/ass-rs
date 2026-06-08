//! Text normalization utilities for ASS subtitle processing
//!
//! Provides functionality for normalizing text content including line endings,
//! whitespace handling, and other text cleanup operations commonly needed
//! when processing ASS subtitle files from various sources and platforms.
//!
//! # Examples
//!
//! ```rust
//! use ass_core::utils::utf8::{normalize_line_endings, normalize_whitespace};
//!
//! let input = "Line 1\r\nLine 2\rLine 3\n";
//! let normalized = normalize_line_endings(input);
//! assert_eq!(normalized, "Line 1\nLine 2\nLine 3\n");
//! ```

mod control;
mod line_endings;
mod whitespace;

#[cfg(test)]
mod tests;

pub use control::remove_control_chars;
pub use line_endings::normalize_line_endings;
pub use whitespace::{normalize_whitespace, trim_lines};

use alloc::string::String;

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
