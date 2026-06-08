//! Incremental editing entry points backed by core's partial parser
//!
//! Implements `edit_incremental` (with error-recovery fallbacks) plus the
//! insert/delete convenience wrappers. The fast-path helpers live in the
//! sibling `incremental_fast` module.

use super::EditorDocument;
use crate::commands::CommandResult;
use crate::core::errors::Result;
use crate::core::position::{Position, Range};
use ass_core::parser::script::ScriptDeltaOwned;

#[cfg(feature = "std")]
use crate::events::DocumentEvent;

#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString, vec, vec::Vec};

impl EditorDocument {
    // === INCREMENTAL PARSING WITH CORE INTEGRATION ===

    /// Perform incremental edit using core's parse_partial for optimal performance
    ///
    /// Includes error recovery with fallback strategies:
    /// 1. Try incremental parsing with Script::parse_partial()
    /// 2. On failure, fall back to full reparse
    /// 3. On repeated failures, reset parser state and retry
    pub fn edit_incremental(&mut self, range: Range, new_text: &str) -> Result<ScriptDeltaOwned> {
        use crate::core::history::Operation;

        // Fast path for simple edits that don't require full parsing
        let is_simple_edit = new_text.len() <= 100 && // Small to medium edits
            !new_text.contains('[') && // No new sections
            new_text.matches('\n').count() <= 1 && // At most one line break
            range.len() <= 50; // Small replacements

        if is_simple_edit {
            return self.edit_fast_path(range, new_text);
        }

        // Get the old text for undo data
        #[cfg(feature = "std")]
        let old_text = self.text_range(range)?;
        #[cfg(not(feature = "std"))]
        let _old_text = self.text_range(range)?;

        // Apply change with incremental parsing (includes fallback to full parse)
        let current_text = self.text();
        let delta = match self
            .incremental_parser
            .apply_change(&current_text, range, new_text)
        {
            Ok(delta) => delta,
            Err(_e) => {
                // Log the error for debugging
                #[cfg(feature = "std")]
                eprintln!("Incremental parsing failed, attempting recovery: {_e}");

                // If incremental parsing fails repeatedly, reset the parser
                if self.incremental_parser.should_reparse() {
                    self.incremental_parser.clear_cache();
                }

                // Try one more time with a fresh parser state
                match self
                    .incremental_parser
                    .apply_change(&current_text, range, new_text)
                {
                    Ok(delta) => delta,
                    Err(_) => {
                        // Final fallback: return a minimal delta indicating the change
                        ScriptDeltaOwned {
                            added: Vec::new(),
                            modified: vec![(0, "Script modified".to_string())],
                            removed: Vec::new(),
                            new_issues: Vec::new(),
                        }
                    }
                }
            }
        };

        // Create undo data from the delta (must be captured BEFORE applying changes)
        let undo_data = self.capture_delta_undo_data(&delta)?;

        // Create delta operation for history
        let operation = Operation::Delta {
            forward: delta.clone(),
            undo_data,
        };

        // Create command result
        let result = CommandResult::success_with_change(
            range,
            Position::new(range.start.offset + new_text.len()),
        );

        // Record in history
        let result_with_delta = result.with_delta(delta.clone());
        self.history.record_operation(
            operation,
            format!("Incremental edit at {}", range.start.offset),
            &result_with_delta,
        );

        // Apply the text change
        self.replace_raw(range, new_text)?;

        // Mark as modified
        self.modified = true;

        // Emit event
        #[cfg(feature = "std")]
        self.emit(DocumentEvent::TextReplaced {
            range,
            old_text,
            new_text: new_text.to_string(),
        });

        Ok(delta)
    }

    /// Insert text with incremental parsing (< 1ms target)
    pub fn insert_incremental(&mut self, pos: Position, text: &str) -> Result<ScriptDeltaOwned> {
        let range = Range::new(pos, pos); // Zero-length range for insertion
        self.edit_incremental(range, text)
    }

    /// Delete text with incremental parsing
    pub fn delete_incremental(&mut self, range: Range) -> Result<ScriptDeltaOwned> {
        self.edit_incremental(range, "")
    }
}
