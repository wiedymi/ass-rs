//! Fluent API builder for operations on a selected range.

use crate::core::{EditorDocument, Range, Result};

#[cfg(feature = "rope")]
use crate::core::Position;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[cfg(all(not(feature = "std"), feature = "rope"))]
use alloc::vec::Vec;

/// Fluent API builder for operations on a selected range
pub struct SelectRange<'a> {
    document: &'a mut EditorDocument,
    range: Range,
}

impl<'a> SelectRange<'a> {
    /// Create a new fluent API for range
    pub(crate) fn new(document: &'a mut EditorDocument, range: Range) -> Self {
        Self { document, range }
    }

    /// Replace the selected range with text
    pub fn replace_with(self, text: &str) -> Result<&'a mut EditorDocument> {
        self.document.replace(self.range, text)?;
        Ok(self.document)
    }

    /// Delete the selected range
    pub fn delete(self) -> Result<&'a mut EditorDocument> {
        self.document.delete(self.range)?;
        Ok(self.document)
    }

    /// Wrap the selection with ASS tags
    pub fn wrap_with_tag(self, open_tag: &str, close_tag: &str) -> Result<&'a mut EditorDocument> {
        // Get the selected text
        let selected = self
            .document
            .rope()
            .byte_slice(self.range.start.offset..self.range.end.offset);
        let mut wrapped =
            String::with_capacity(open_tag.len() + selected.len_bytes() + close_tag.len());
        wrapped.push_str(open_tag);
        wrapped.push_str(&selected.to_string());
        wrapped.push_str(close_tag);

        self.document.replace(self.range, &wrapped)?;
        Ok(self.document)
    }

    /// Indent the selected lines
    #[cfg(feature = "rope")]
    pub fn indent(self, spaces: usize) -> Result<&'a mut EditorDocument> {
        // Get line information before mutating
        let start_line = self.document.rope().byte_to_line(self.range.start.offset);
        let end_line = self.document.rope().byte_to_line(self.range.end.offset);
        let indent = " ".repeat(spaces);

        // Collect line positions
        let mut line_positions = Vec::new();
        for line_idx in (start_line..=end_line).rev() {
            let line_start = self.document.rope().line_to_byte(line_idx);
            line_positions.push(line_start);
        }

        // Apply indentation
        for line_start in line_positions {
            let pos = Position::new(line_start);
            let range = Range::empty(pos);
            self.document.replace(range, &indent)?;
        }

        Ok(self.document)
    }

    /// Unindent the selected lines
    #[cfg(feature = "rope")]
    pub fn unindent(self, spaces: usize) -> Result<&'a mut EditorDocument> {
        // Get line information before mutating
        let start_line = self.document.rope().byte_to_line(self.range.start.offset);
        let end_line = self.document.rope().byte_to_line(self.range.end.offset);

        // Collect unindent operations
        let mut unindent_ops = Vec::new();
        for line_idx in (start_line..=end_line).rev() {
            let line_start = self.document.rope().line_to_byte(line_idx);
            let line = self.document.rope().line(line_idx);

            // Count spaces to remove
            let mut space_count = 0;
            for ch in line.chars().take(spaces) {
                if ch == ' ' {
                    space_count += 1;
                } else {
                    break;
                }
            }

            if space_count > 0 {
                unindent_ops.push((line_start, space_count));
            }
        }

        // Apply unindent operations
        for (line_start, space_count) in unindent_ops {
            let range = Range::new(
                Position::new(line_start),
                Position::new(line_start + space_count),
            );
            self.document.delete(range)?;
        }

        Ok(self.document)
    }

    /// Get the selected text
    pub fn text(&self) -> String {
        self.document
            .rope()
            .byte_slice(self.range.start.offset..self.range.end.offset)
            .to_string()
    }

    /// Get the range
    pub const fn range(&self) -> Range {
        self.range
    }
}
