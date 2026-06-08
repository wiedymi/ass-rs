//! Fast-path helpers for trivial incremental edits
//!
//! `edit_fast_path` skips full parsing for small, boundary-safe edits and is
//! invoked by `edit_incremental`; it falls back to the slower path when a
//! range lands on an invalid UTF-8 boundary.

use super::EditorDocument;
use crate::commands::CommandResult;
use crate::core::errors::Result;
use crate::core::position::{Position, Range};
use ass_core::parser::script::ScriptDeltaOwned;

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

impl EditorDocument {
    /// Fast path for simple edits that avoids heavy parsing
    pub(super) fn edit_fast_path(
        &mut self,
        range: Range,
        new_text: &str,
    ) -> Result<ScriptDeltaOwned> {
        use crate::core::history::Operation;

        // Validate that range boundaries are on valid UTF-8 char boundaries
        let text = self.text();
        if !text.is_char_boundary(range.start.offset) || !text.is_char_boundary(range.end.offset) {
            // Fall back to regular incremental parsing for invalid boundaries
            return self.edit_incremental_fallback(range, new_text);
        }

        // Get old text for undo
        let old_text = if range.is_empty() {
            String::new()
        } else {
            self.text_range(range)?
        };

        // Simple text replacement without parsing
        self.replace_raw(range, new_text)?;

        // Create minimal undo operation
        let operation = Operation::Replace {
            range,
            new_text: new_text.to_string(),
            old_text,
        };

        // Create result
        let result = CommandResult::success_with_change(
            range,
            Position::new(range.start.offset + new_text.len()),
        );

        // Record in history (without delta)
        self.history
            .record_operation(operation, "Fast character insert".to_string(), &result);

        // Mark as modified
        self.modified = true;

        // Return minimal delta
        Ok(ScriptDeltaOwned {
            added: Vec::new(),
            modified: Vec::new(),
            removed: Vec::new(),
            new_issues: Vec::new(),
        })
    }

    /// Fallback for edit_incremental that avoids infinite recursion
    fn edit_incremental_fallback(
        &mut self,
        range: Range,
        new_text: &str,
    ) -> Result<ScriptDeltaOwned> {
        // Just do a simple replace without the fast path
        self.replace(range, new_text)?;

        // Return minimal delta
        Ok(ScriptDeltaOwned {
            added: Vec::new(),
            modified: Vec::new(),
            removed: Vec::new(),
            new_issues: Vec::new(),
        })
    }
}
