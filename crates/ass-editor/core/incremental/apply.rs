//! Incremental change application for [`IncrementalParser`].
//!
//! Implements [`IncrementalParser::apply_change`], which applies a single edit
//! via `Script::parse_partial`, tracks the resulting delta, and falls back to a
//! full reparse when incremental parsing is unavailable or fails.

use super::{DocumentChange, IncrementalParser};
use crate::core::errors::EditorError;
use crate::core::{Range, Result};
use ass_core::parser::{script::ScriptDeltaOwned, Script};

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::{borrow::Cow, string::ToString};

impl IncrementalParser {
    /// Apply a change incrementally, returning the delta
    pub fn apply_change(
        &mut self,
        document_text: &str,
        range: Range,
        new_text: &str,
    ) -> Result<ScriptDeltaOwned> {
        // If we don't have a cached script or too many changes accumulated, do full parse
        if self.cached_script.is_none() || self.bytes_changed >= self.reparse_threshold {
            return self.full_reparse(document_text);
        }

        // Validate range
        if range.end.offset > document_text.len() || range.start.offset > range.end.offset {
            return Err(EditorError::InvalidRange {
                start: range.start.offset,
                end: range.end.offset,
                length: document_text.len(),
            });
        }

        // Check if we're already on valid UTF-8 boundaries
        let start_is_valid = range.start.offset == 0
            || range.start.offset == document_text.len()
            || document_text.is_char_boundary(range.start.offset);
        let end_is_valid = range.end.offset == 0
            || range.end.offset == document_text.len()
            || document_text.is_char_boundary(range.end.offset);

        if !start_is_valid || !end_is_valid {
            // The range is not on valid UTF-8 boundaries - this is an error
            // We should not silently adjust the range as it will cause undo/redo issues
            return Err(EditorError::command_failed(
                "Edit range is not on valid UTF-8 character boundaries",
            ));
        }

        // Get the old text being replaced
        let old_text = &document_text[range.start.offset..range.end.offset];
        let (start_byte, end_byte) = (range.start.offset, range.end.offset);

        // Track the change (convert to owned for storage)
        let change = DocumentChange {
            range,
            new_text: Cow::Owned(new_text.to_string()),
            old_text: Cow::Owned(old_text.to_string()),
            #[cfg(feature = "std")]
            timestamp: std::time::Instant::now(),
            change_id: self.next_change_id,
        };
        self.next_change_id += 1;

        // Update bytes changed counter
        let change_size = new_text.len().abs_diff(old_text.len());
        self.bytes_changed += change_size;

        // Store the change for potential rollback
        self.pending_changes.push(change);

        // Convert editor Range to std::ops::Range for parse_partial
        // Use the corrected boundaries from old_text extraction
        let byte_range = start_byte..end_byte;

        // Parse the cached script first to get a Script instance
        let cached = self.cached_script.as_ref().ok_or_else(|| {
            EditorError::command_failed("Cached script unavailable for incremental parsing")
        })?;
        let script = Script::parse(cached).map_err(EditorError::from)?;

        // Apply incremental parsing
        match script.parse_partial(byte_range, new_text) {
            Ok(delta) => {
                // Update cached script with the change
                self.update_cached_script(range, new_text)?;
                Ok(delta)
            }
            Err(_e) => {
                // Fall back to full reparse on error
                self.pending_changes.pop(); // Remove failed change
                self.bytes_changed -= change_size;

                // Log the error for debugging
                #[cfg(feature = "std")]
                eprintln!("Incremental parse failed, falling back to full parse: {_e}");

                self.full_reparse(document_text)
            }
        }
    }
}
