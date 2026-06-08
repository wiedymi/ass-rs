//! UTF-8 boundary-safe text truncation utilities
//!
//! Provides truncation of text to a byte limit without breaking UTF-8
//! character sequences.

/// Truncate text at UTF-8 character boundary
///
/// Safely truncates text to the specified byte length without breaking
/// UTF-8 character sequences. Essential for handling length limits
/// while maintaining valid UTF-8 encoding.
///
/// # Arguments
///
/// * `text` - Input text to truncate
/// * `max_bytes` - Maximum byte length
///
/// # Returns
///
/// Tuple of (`truncated_text`, `was_truncated`)
///
/// # Examples
///
/// ```rust
/// # use ass_core::utils::utf8::truncate_at_char_boundary;
/// let text = "Hello World";
/// let (truncated, was_truncated) = truncate_at_char_boundary(text, 5);
/// assert_eq!(truncated, "Hello");
/// assert!(was_truncated);
///
/// let text = "Hello 世界";
/// let (truncated, was_truncated) = truncate_at_char_boundary(text, 8);
/// assert_eq!(truncated, "Hello "); // Stops before the Unicode character
/// assert!(was_truncated);
/// ```
#[must_use]
pub fn truncate_at_char_boundary(text: &str, max_bytes: usize) -> (&str, bool) {
    if text.len() <= max_bytes {
        return (text, false);
    }

    let mut boundary = max_bytes;
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }

    (&text[..boundary], true)
}
