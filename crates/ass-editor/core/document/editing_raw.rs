//! Low-level text mutation primitives without undo recording
//!
//! These `pub(crate)` helpers mutate the underlying rope/string directly and
//! are the building blocks used by the higher-level editing and history APIs.

use super::EditorDocument;
use crate::core::errors::{EditorError, Result};
use crate::core::position::{Position, Range};

impl EditorDocument {
    /// Insert text at position (low-level operation without undo)
    pub(crate) fn insert_raw(&mut self, pos: Position, text: &str) -> Result<()> {
        if pos.offset > self.len_bytes() {
            return Err(EditorError::PositionOutOfBounds {
                position: pos.offset,
                length: self.len_bytes(),
            });
        }

        #[cfg(feature = "rope")]
        {
            // Convert byte offset to char index for rope operations
            let char_idx = self.text_rope.byte_to_char(pos.offset);
            self.text_rope.insert(char_idx, text);
        }
        #[cfg(not(feature = "rope"))]
        {
            self.text_content.insert_str(pos.offset, text);
        }

        self.modified = true;
        Ok(())
    }

    /// Delete text in range (low-level operation without undo)
    pub(crate) fn delete_raw(&mut self, range: Range) -> Result<()> {
        if range.end.offset > self.len_bytes() {
            return Err(EditorError::InvalidRange {
                start: range.start.offset,
                end: range.end.offset,
                length: self.len_bytes(),
            });
        }

        #[cfg(feature = "rope")]
        {
            // Convert byte offsets to char indices for rope operations
            let start_char = self.text_rope.byte_to_char(range.start.offset);
            let end_char = self.text_rope.byte_to_char(range.end.offset);
            self.text_rope.remove(start_char..end_char);
        }
        #[cfg(not(feature = "rope"))]
        {
            self.text_content
                .drain(range.start.offset..range.end.offset);
        }

        self.modified = true;
        Ok(())
    }

    /// Replace text in range (low-level operation without undo)
    pub(crate) fn replace_raw(&mut self, range: Range, text: &str) -> Result<()> {
        self.delete_raw(range)?;
        self.insert_raw(range.start, text)?;
        Ok(())
    }
}
