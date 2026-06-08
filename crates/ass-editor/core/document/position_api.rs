//! Fluent position-anchored editing API
//!
//! Defines `DocumentPosition`, a short-lived handle returned by
//! `EditorDocument::at` that offers ergonomic insert/delete/replace
//! operations relative to a fixed position.

use super::EditorDocument;
use crate::core::errors::Result;
use crate::core::position::{Position, Range};

/// Fluent position API for editor operations
pub struct DocumentPosition<'a> {
    document: &'a mut EditorDocument,
    position: Position,
}

impl<'a> DocumentPosition<'a> {
    /// Insert text at this position
    pub fn insert_text(self, text: &str) -> Result<()> {
        self.document.insert(self.position, text)
    }

    /// Delete text range starting from this position
    pub fn delete_range(self, len: usize) -> Result<()> {
        let end_pos = Position::new(self.position.offset + len);
        let range = Range::new(self.position, end_pos);
        self.document.delete(range)
    }

    /// Replace text at this position
    pub fn replace_text(self, len: usize, new_text: &str) -> Result<()> {
        let end_pos = Position::new(self.position.offset + len);
        let range = Range::new(self.position, end_pos);
        self.document.replace(range, new_text)
    }
}

impl EditorDocument {
    /// Get fluent API for position-based operations
    pub fn at(&mut self, pos: Position) -> DocumentPosition<'_> {
        DocumentPosition {
            document: self,
            position: pos,
        }
    }
}
