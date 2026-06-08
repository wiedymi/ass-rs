//! Line-ending normalization for ASS subtitle text.
//!
//! Converts Windows (`\r\n`) and classic Mac (`\r`) line endings to the
//! Unix (`\n`) convention used throughout the toolkit.

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
/// # use ass_core::utils::utf8::normalize_line_endings;
/// let input = "Line 1\r\nLine 2\rLine 3\n";
/// let normalized = normalize_line_endings(input);
/// assert_eq!(normalized, "Line 1\nLine 2\nLine 3\n");
/// ```
#[must_use]
pub fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}
