//! Fluent API builder for document operations at a specific position.

use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "rope"))]
use crate::core::errors::EditorError;

#[cfg(all(not(feature = "std"), not(feature = "rope")))]
use alloc::string::ToString;

/// Fluent API builder for document operations at a specific position
pub struct AtPosition<'a> {
    document: &'a mut EditorDocument,
    position: Position,
}

impl<'a> AtPosition<'a> {
    /// Create a new fluent API at position
    pub(crate) fn new(document: &'a mut EditorDocument, position: Position) -> Self {
        Self { document, position }
    }

    /// Insert text at the current position
    pub fn insert_text(self, text: &str) -> Result<&'a mut EditorDocument> {
        let range = Range::empty(self.position);
        self.document.replace(range, text)?;
        Ok(self.document)
    }

    /// Insert a line break at the current position
    pub fn insert_line(self) -> Result<&'a mut EditorDocument> {
        self.insert_text("\n")
    }

    /// Delete a number of characters forward from position
    pub fn delete(self, count: usize) -> Result<&'a mut EditorDocument> {
        let end = self.position.advance(count);
        let range = Range::new(self.position, end);
        self.document.delete(range)?;
        Ok(self.document)
    }

    /// Delete characters backward from position (backspace)
    pub fn backspace(self, count: usize) -> Result<&'a mut EditorDocument> {
        let start = self.position.retreat(count);
        let range = Range::new(start, self.position);
        self.document.delete(range)?;
        Ok(self.document)
    }

    /// Replace text from position to end of line
    pub fn replace_to_line_end(self, text: &str) -> Result<&'a mut EditorDocument> {
        #[cfg(feature = "rope")]
        {
            let rope = self.document.rope();
            let line_idx = rope.byte_to_line(self.position.offset);
            let line_end_byte = if line_idx + 1 < rope.len_lines() {
                rope.line_to_byte(line_idx + 1).saturating_sub(1)
            } else {
                rope.len_bytes()
            };
            let range = Range::new(self.position, Position::new(line_end_byte));
            self.document.replace(range, text)?;
            Ok(self.document)
        }

        #[cfg(not(feature = "rope"))]
        {
            Err(EditorError::FeatureNotEnabled {
                feature: "line-based operations".to_string(),
                required_feature: "rope".to_string(),
            })
        }
    }

    /// Get the current position
    pub const fn position(&self) -> Position {
        self.position
    }

    /// Convert position to line/column
    #[cfg(feature = "rope")]
    pub fn to_line_column(&self) -> Result<(usize, usize)> {
        let rope = self.document.rope();
        let line_idx = rope.byte_to_line(self.position.offset);
        let line_start = rope.line_to_byte(line_idx);
        let col_offset = self.position.offset - line_start;

        // Convert byte offset to character offset
        let line = rope.line(line_idx);
        let mut char_col = 0;
        let mut byte_count = 0;

        for ch in line.chars() {
            if byte_count >= col_offset {
                break;
            }
            byte_count += ch.len_utf8();
            char_col += 1;
        }

        Ok((line_idx + 1, char_col + 1)) // Convert to 1-indexed
    }
}
