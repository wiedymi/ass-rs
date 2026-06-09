//! Editing-scenario change generators for the incremental parsing benchmarks.
//!
//! Builds sequences of [`TextChange`] values that emulate typing, backspacing,
//! and copy/paste interactions in an editor.

#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};
use ass_core::parser::incremental::TextChange;

/// Create typing simulation - many small insertions
pub fn create_typing_simulation(script_text: &str, count: usize) -> Vec<TextChange> {
    let mut changes = Vec::new();
    let start_pos = script_text.len() / 2;

    for i in 0..count {
        changes.push(TextChange {
            range: start_pos + i..start_pos + i,
            new_text: ((b'a' + u8::try_from(i % 26).unwrap_or(0)) as char).to_string(),
            line_range: 10..10,
        });
    }

    changes
}

/// Create backspace simulation - many small deletions
pub fn create_backspace_simulation(script_text: &str, count: usize) -> Vec<TextChange> {
    let mut changes = Vec::new();
    let start_pos = script_text.len() / 2;

    for i in 0..count {
        let pos = start_pos.saturating_sub(i);
        changes.push(TextChange {
            range: pos..pos,
            new_text: String::new(),
            line_range: 10..10,
        });
    }

    changes
}

/// Create paste simulation - larger insertions
pub fn create_paste_simulation(script_text: &str, count: usize) -> Vec<TextChange> {
    let mut changes = Vec::new();
    let start_pos = script_text.len() / 2;

    for i in 0..count {
        changes.push(TextChange {
            range: start_pos + i * 100..start_pos + i * 100,
            new_text: format!("Pasted content block {i} with some text\n"),
            line_range: 10 + u32::try_from(i).unwrap_or(0)..11 + u32::try_from(i).unwrap_or(0),
        });
    }

    changes
}
