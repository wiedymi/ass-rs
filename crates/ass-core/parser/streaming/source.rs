//! Source reconstruction helpers for incremental edits
//!
//! Provides [`build_modified_source`], a zero-fuss utility for producing a new
//! source string by replacing a byte range with replacement text. Used by the
//! incremental parsing path to materialize edited documents.

use alloc::string::String;
use core::ops::Range;

/// Build modified source with range replacement
///
/// Creates a new source string by replacing the specified range with new text.
///
/// # Arguments
///
/// * `original` - The original source text
/// * `range` - The byte range to replace
/// * `replacement` - The text to insert in place of the range
///
/// # Returns
///
/// A new string with the replacement applied
#[must_use]
pub fn build_modified_source(original: &str, range: Range<usize>, replacement: &str) -> String {
    let mut result =
        String::with_capacity(original.len() - (range.end - range.start) + replacement.len());

    // Add text before the range
    result.push_str(&original[..range.start]);

    // Add replacement text
    result.push_str(replacement);

    // Add text after the range
    result.push_str(&original[range.end..]);

    result
}
