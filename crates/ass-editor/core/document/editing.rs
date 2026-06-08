//! Undo-aware text editing operations
//!
//! Public `insert`, `delete`, and `replace` methods that run the matching
//! command, record the operation in the undo history, invalidate the
//! validation cache, and emit document events.

use super::EditorDocument;
use crate::core::errors::Result;
use crate::core::position::{Position, Range};

#[cfg(feature = "std")]
use crate::events::DocumentEvent;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

impl EditorDocument {
    /// Insert text at position with undo support
    ///
    /// Inserts text at the given position, automatically updating the underlying
    /// text representation and recording the operation in the undo history.
    ///
    /// # Examples
    ///
    /// ```
    /// use ass_editor::{EditorDocument, Position};
    ///
    /// let mut doc = EditorDocument::from_content("Hello World").unwrap();
    /// let pos = Position::new(5); // Insert after "Hello"
    /// doc.insert(pos, " there").unwrap();
    ///
    /// assert_eq!(doc.text(), "Hello there World");
    ///
    /// // Can undo the operation
    /// doc.undo().unwrap();
    /// assert_eq!(doc.text(), "Hello World");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the position is beyond the document bounds.
    pub fn insert(&mut self, pos: Position, text: &str) -> Result<()> {
        use crate::commands::{EditorCommand, InsertTextCommand};
        use crate::core::history::Operation;

        let command = InsertTextCommand::new(pos, text.to_string());
        let result = command.execute(self)?;

        // Record the operation in history
        let operation = Operation::Insert {
            position: pos,
            text: text.to_string(),
        };
        self.history
            .record_operation(operation, command.description().to_string(), &result);

        // Clear validation cache since content changed
        self.validator.clear_cache();

        // Emit event
        #[cfg(feature = "std")]
        self.emit(DocumentEvent::TextInserted {
            position: pos,
            text: text.to_string(),
            length: text.len(),
        });

        Ok(())
    }

    /// Delete text in range with undo support
    pub fn delete(&mut self, range: Range) -> Result<()> {
        use crate::commands::{DeleteTextCommand, EditorCommand};
        use crate::core::history::Operation;

        // Capture the text that will be deleted BEFORE deletion
        let deleted_text = self.text_range(range)?;

        let command = DeleteTextCommand::new(range);
        let result = command.execute(self)?;

        // Record the operation in history
        let operation = Operation::Delete {
            range,
            deleted_text: deleted_text.clone(),
        };
        self.history
            .record_operation(operation, command.description().to_string(), &result);

        // Clear validation cache since content changed
        self.validator.clear_cache();

        // Emit event
        #[cfg(feature = "std")]
        self.emit(DocumentEvent::TextDeleted {
            range,
            deleted_text,
        });

        Ok(())
    }

    /// Replace text in range with undo support
    pub fn replace(&mut self, range: Range, text: &str) -> Result<()> {
        use crate::commands::{EditorCommand, ReplaceTextCommand};
        use crate::core::history::Operation;

        // Capture the old text BEFORE replacement
        let old_text = self.text_range(range)?;

        let command = ReplaceTextCommand::new(range, text.to_string());
        let result = command.execute(self)?;

        // Record the operation in history
        let operation = Operation::Replace {
            range,
            old_text: old_text.clone(),
            new_text: text.to_string(),
        };
        self.history
            .record_operation(operation, command.description().to_string(), &result);

        // Clear validation cache since content changed
        self.validator.clear_cache();

        // Emit event
        #[cfg(feature = "std")]
        self.emit(DocumentEvent::TextReplaced {
            range,
            old_text,
            new_text: text.to_string(),
        });

        Ok(())
    }
}
