//! Control-character removal for safe ASS subtitle text processing.
//!
//! Strips problematic control characters while preserving the essential
//! `\n`, `\t`, and `\r` characters needed for layout.

use alloc::string::String;

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
#[must_use]
pub fn remove_control_chars(text: &str) -> String {
    text.chars()
        .filter(|&c| {
            // Keep printable characters, newlines, tabs, and carriage returns
            !c.is_control() || c == '\n' || c == '\t' || c == '\r'
        })
        .collect()
}
