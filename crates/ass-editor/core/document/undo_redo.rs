//! Undo/redo execution and history manager access
//!
//! Implements reversing and replaying recorded operations (including
//! stream-feature delta operations) and exposes the underlying
//! `UndoManager` for configuration and inspection.

use super::EditorDocument;
use crate::commands::CommandResult;
use crate::core::errors::{EditorError, Result};
use crate::core::history::UndoManager;
use crate::core::position::{Position, Range};

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

impl EditorDocument {
    /// Perform an undo operation
    ///
    /// Retrieves the most recent operation from the undo stack and reverses it.
    /// If the operation includes a script delta, it will be applied for efficient updates.
    pub fn undo(&mut self) -> Result<CommandResult> {
        use crate::core::history::Operation;

        // Pop from undo stack
        if let Some(entry) = self.history.pop_undo_entry() {
            let mut result = CommandResult::success();
            result.content_changed = true;

            // Execute the inverse of the operation
            match &entry.operation {
                Operation::Insert { position, text } => {
                    // Undo insert by deleting the inserted text
                    let end_pos = Position::new(position.offset + text.len());
                    let range = Range::new(*position, end_pos);
                    self.delete_raw(range)?;
                    result.modified_range = Some(Range::new(*position, *position));
                    result.new_cursor = entry.cursor_before;
                }
                Operation::Delete {
                    range,
                    deleted_text,
                } => {
                    // Undo delete by inserting the deleted text
                    self.insert_raw(range.start, deleted_text)?;
                    let end_pos = Position::new(range.start.offset + deleted_text.len());
                    result.modified_range = Some(Range::new(range.start, end_pos));
                    result.new_cursor = entry.cursor_before;
                }
                Operation::Replace {
                    range, old_text, ..
                } => {
                    // Undo replace by restoring old text
                    self.replace_raw(*range, old_text)?;
                    let end_pos = Position::new(range.start.offset + old_text.len());
                    result.modified_range = Some(Range::new(range.start, end_pos));
                    result.new_cursor = entry.cursor_before;
                }
                #[cfg(feature = "stream")]
                Operation::Delta { forward, undo_data } => {
                    // Restore removed sections
                    for (index, section_text) in undo_data.removed_sections.iter() {
                        self.insert_section_at(*index, section_text)?;
                    }

                    // Restore modified sections
                    for (index, original_text) in undo_data.modified_sections.iter() {
                        self.replace_section(*index, original_text)?;
                    }

                    // Remove added sections
                    for _ in 0..forward.added.len() {
                        self.remove_last_section()?;
                    }

                    result.message = Some("Delta operation undone".to_string());
                }
            }

            // Push to redo stack for future redo
            self.history.push_redo_entry(entry);

            // Apply script delta if available
            #[cfg(feature = "stream")]
            if let Some(delta) = result.script_delta.as_ref() {
                self.apply_script_delta(delta.clone())?;
            }

            result.message = Some("Undo successful".to_string());
            Ok(result)
        } else {
            Err(EditorError::NothingToUndo)
        }
    }

    /// Perform a redo operation
    ///
    /// Retrieves the most recent operation from the redo stack and re-executes it.
    /// If the operation includes a script delta, it will be applied for efficient updates.
    pub fn redo(&mut self) -> Result<CommandResult> {
        use crate::core::history::Operation;

        // Pop from redo stack
        if let Some(entry) = self.history.pop_redo_entry() {
            let mut result = CommandResult::success();
            result.content_changed = true;

            // Re-execute the original operation
            match &entry.operation {
                Operation::Insert { position, text } => {
                    // Redo insert
                    self.insert_raw(*position, text)?;
                    let end_pos = Position::new(position.offset + text.len());
                    result.modified_range = Some(Range::new(*position, end_pos));
                    result.new_cursor = entry.cursor_after;
                }
                Operation::Delete { range, .. } => {
                    // Redo delete
                    self.delete_raw(*range)?;
                    result.modified_range = Some(Range::new(range.start, range.start));
                    result.new_cursor = entry.cursor_after;
                }
                Operation::Replace {
                    range, new_text, ..
                } => {
                    // Redo replace
                    self.replace_raw(*range, new_text)?;
                    let end_pos = Position::new(range.start.offset + new_text.len());
                    result.modified_range = Some(Range::new(range.start, end_pos));
                    result.new_cursor = entry.cursor_after;
                }
                #[cfg(feature = "stream")]
                Operation::Delta {
                    forward,
                    undo_data: _,
                } => {
                    // Re-apply the delta
                    self.apply_script_delta(forward.clone())?;
                    result.message = Some("Delta re-applied".to_string());
                }
            }

            // Record in history without using the public methods (to avoid recursion)
            // We need to manually update the history manager's cursor
            if let Some(cursor) = result.new_cursor {
                self.history.set_cursor(Some(cursor));
            }

            // Create a new history entry for the redo operation
            let new_entry = crate::core::history::HistoryEntry::new(
                entry.operation,
                entry.description,
                &result,
                entry.cursor_before,
            );

            // Push back to undo stack
            self.history.stack_mut().push(new_entry);

            result.message = Some("Redo successful".to_string());
            Ok(result)
        } else {
            Err(EditorError::NothingToRedo)
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Get description of the next undo operation
    pub fn next_undo_description(&self) -> Option<&str> {
        self.history.next_undo_description()
    }

    /// Get description of the next redo operation
    pub fn next_redo_description(&self) -> Option<&str> {
        self.history.next_redo_description()
    }

    /// Get mutable reference to the undo manager for configuration
    pub fn undo_manager_mut(&mut self) -> &mut UndoManager {
        &mut self.history
    }

    /// Get reference to the undo manager
    pub fn undo_manager(&self) -> &UndoManager {
        &self.history
    }
}
