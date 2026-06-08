//! UTF-8 recovery and replacement-character utilities
//!
//! Provides recovery from invalid UTF-8 sequences via the Unicode replacement
//! character and counting of replacement characters for diagnostics.

use alloc::string::{String, ToString};
use core::str;

/// Attempt to recover from UTF-8 errors by replacing invalid sequences
///
/// Returns valid UTF-8 text with invalid sequences replaced by the Unicode
/// replacement character (�). Also returns the number of replacements made
/// for diagnostic purposes.
///
/// # Arguments
///
/// * `bytes` - Byte sequence that may contain invalid UTF-8
///
/// # Returns
///
/// Tuple of (`recovered_text`, `replacement_count`)
///
/// # Examples
///
/// ```rust
/// # use ass_core::utils::utf8::recover_utf8;
/// let valid_text = "Hello, World!";
/// let (recovered, replacements) = recover_utf8(valid_text.as_bytes());
/// assert_eq!(recovered, "Hello, World!");
/// assert_eq!(replacements, 0);
///
/// let invalid_bytes = &[b'H', b'i', 0xFF, b'!'];
/// let (recovered, replacements) = recover_utf8(invalid_bytes);
/// assert_eq!(recovered, "Hi�!");
/// assert_eq!(replacements, 1);
/// ```
#[must_use]
pub fn recover_utf8(bytes: &[u8]) -> (String, usize) {
    str::from_utf8(bytes).map_or_else(
        |_| {
            let recovered = String::from_utf8_lossy(bytes);
            let replacements = recovered.matches('\u{FFFD}').count();
            (recovered.into_owned(), replacements)
        },
        |s| (s.to_string(), 0),
    )
}

/// Count replacement characters in text
///
/// Counts the number of Unicode replacement characters (�) in text,
/// which typically indicate encoding errors or data corruption.
/// Useful for assessing text quality and encoding issues.
///
/// # Arguments
///
/// * `text` - Text to analyze
///
/// # Returns
///
/// Number of replacement characters found
#[must_use]
pub fn count_replacement_chars(text: &str) -> usize {
    text.matches('\u{FFFD}').count()
}
