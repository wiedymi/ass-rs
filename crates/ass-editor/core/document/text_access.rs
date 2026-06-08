//! Text content queries and byte/line-column conversions
//!
//! Read-only accessors for length, emptiness, raw text, rope access, range
//! extraction, and position-to-line/column mapping.

use super::EditorDocument;
use crate::core::errors::{EditorError, Result};
use crate::core::position::{LineColumn, Position, Range};

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

impl EditorDocument {
    /// Get total length in bytes
    #[must_use]
    pub fn len_bytes(&self) -> usize {
        #[cfg(feature = "rope")]
        {
            self.text_rope.len_bytes()
        }
        #[cfg(not(feature = "rope"))]
        {
            self.text_content.len()
        }
    }

    /// Get total number of lines
    #[must_use]
    pub fn len_lines(&self) -> usize {
        #[cfg(feature = "rope")]
        {
            self.text_rope.len_lines()
        }
        #[cfg(not(feature = "rope"))]
        {
            self.text_content.lines().count().max(1)
        }
    }

    /// Check if document is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len_bytes() == 0
    }

    /// Get text content as string
    #[must_use]
    pub fn text(&self) -> String {
        #[cfg(feature = "rope")]
        {
            self.text_rope.to_string()
        }
        #[cfg(not(feature = "rope"))]
        {
            self.text_content.clone()
        }
    }

    /// Get direct access to the rope for advanced operations
    #[cfg(feature = "rope")]
    #[must_use]
    pub fn rope(&self) -> &ropey::Rope {
        &self.text_rope
    }

    /// Get the length of the document in bytes
    #[must_use]
    pub fn len(&self) -> usize {
        #[cfg(feature = "rope")]
        {
            self.text_rope.len_bytes()
        }
        #[cfg(not(feature = "rope"))]
        {
            self.text_content.len()
        }
    }

    /// Get text content for a range
    pub fn text_range(&self, range: Range) -> Result<String> {
        let start = range.start.offset;
        let end = range.end.offset;

        if end > self.len_bytes() {
            return Err(EditorError::InvalidRange {
                start,
                end,
                length: self.len_bytes(),
            });
        }

        #[cfg(feature = "rope")]
        {
            // Convert byte offsets to char indices for rope operations
            let start_char = self.text_rope.byte_to_char(start);
            let end_char = self.text_rope.byte_to_char(end);
            Ok(self.text_rope.slice(start_char..end_char).to_string())
        }
        #[cfg(not(feature = "rope"))]
        {
            Ok(self.text_content[start..end].to_string())
        }
    }

    /// Convert byte position to line/column
    #[cfg(feature = "rope")]
    pub fn position_to_line_column(&self, pos: Position) -> Result<LineColumn> {
        if pos.offset > self.len_bytes() {
            return Err(EditorError::PositionOutOfBounds {
                position: pos.offset,
                length: self.len_bytes(),
            });
        }

        let line_idx = self.text_rope.byte_to_line(pos.offset);
        let line_start = self.text_rope.line_to_byte(line_idx);
        let col_offset = pos.offset - line_start;

        // Convert byte offset to character offset within line
        let line = self.text_rope.line(line_idx);
        let mut char_col = 0;
        let mut byte_count = 0;

        for ch in line.chars() {
            if byte_count >= col_offset {
                break;
            }
            byte_count += ch.len_utf8();
            char_col += 1;
        }

        // Convert to 1-indexed
        LineColumn::new(line_idx + 1, char_col + 1)
    }

    /// Convert byte position to line/column (without rope)
    #[cfg(not(feature = "rope"))]
    pub fn position_to_line_column(&self, pos: Position) -> Result<LineColumn> {
        if pos.offset > self.len_bytes() {
            return Err(EditorError::PositionOutOfBounds {
                position: pos.offset,
                length: self.len_bytes(),
            });
        }

        let mut line = 1;
        let mut col = 1;
        let mut byte_pos = 0;

        for ch in self.text_content.chars() {
            if byte_pos >= pos.offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }

            byte_pos += ch.len_utf8();
        }

        LineColumn::new(line, col)
    }
}
