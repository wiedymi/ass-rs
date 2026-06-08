//! Byte-range adjustment for incremental text changes

use alloc::string::String;
use core::ops::Range;

#[cfg(not(feature = "std"))]
extern crate alloc;
/// Represents a text change in the source
#[derive(Debug, Clone)]
pub struct TextChange {
    /// Byte range that was modified
    pub range: Range<usize>,
    /// Replacement text
    pub new_text: String,
    /// Affected line numbers (1-based)
    pub line_range: Range<u32>,
}

/// Adjust a byte range for a text change
///
/// This function calculates how a range should be adjusted after a text change.
/// It handles cases where the change is before, after, or overlapping with the range.
#[must_use]
pub fn adjust_range_for_change(original_range: Range<usize>, change: &TextChange) -> Range<usize> {
    // Case 1: Change is entirely before the range
    if change.range.end <= original_range.start {
        let new_len = change.new_text.len();
        let old_len = change.range.end - change.range.start;

        if new_len >= old_len {
            let offset = new_len - old_len;
            return (original_range.start + offset)..(original_range.end + offset);
        }
        let offset = old_len - new_len;
        return original_range.start.saturating_sub(offset)
            ..original_range.end.saturating_sub(offset);
    }

    // Case 2: Change is entirely after the range
    if change.range.start >= original_range.end {
        return original_range;
    }

    // Case 3: Change overlaps - need careful handling
    // Start stays same if change starts after range start
    let new_start = original_range.start.min(change.range.start);

    // End needs adjustment based on size difference
    let new_len = change.new_text.len();
    let old_len = change.range.end - change.range.start;
    let new_end = if change.range.end >= original_range.end {
        // Change extends past range
        change.range.start + new_len
    } else {
        // Change is within range
        if new_len >= old_len {
            original_range.end + (new_len - old_len)
        } else {
            original_range.end.saturating_sub(old_len - new_len)
        }
    };

    new_start..new_end
}
