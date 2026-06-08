//! Line-number calculations from byte offsets in source text

use core::ops::Range;

/// Calculate line range from byte range in source text
#[must_use]
pub fn calculate_line_range(source: &str, byte_range: Range<usize>) -> Range<u32> {
    let mut line = 1u32;
    let mut byte_pos = 0;
    let mut start_line = 0u32;
    let mut end_line = 0u32;

    for ch in source.chars() {
        if byte_pos >= byte_range.start && start_line == 0 {
            start_line = line;
        }
        if byte_pos >= byte_range.end {
            end_line = line;
            break;
        }
        if ch == '\n' {
            line += 1;
        }
        byte_pos += ch.len_utf8();
    }

    if end_line == 0 {
        end_line = line;
    }

    start_line..end_line
}

/// Calculate the line number for a given byte position
#[must_use]
pub fn calculate_line_number(source: &str, byte_pos: usize) -> u32 {
    let mut line = 1u32;
    let mut current_pos = 0;

    for ch in source.chars() {
        if current_pos >= byte_pos {
            break;
        }
        if ch == '\n' {
            line += 1;
        }
        current_pos += ch.len_utf8();
    }

    line
}
