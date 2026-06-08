//! Fluent API entry points implemented on [`EditorDocument`].

use super::event_ops::EventOps;
use super::karaoke::KaraokeOps;
use super::media::{FontsOps, GraphicsOps};
use super::script_info::ScriptInfoOps;
use super::style::StyleOps;
use super::tag::TagOps;
use super::{AtPosition, SelectRange};
use crate::core::{EditorDocument, Position, Range};

#[cfg(feature = "rope")]
use crate::core::errors::EditorError;
#[cfg(feature = "rope")]
use crate::core::Result;

/// Extension trait to add fluent API to EditorDocument
impl EditorDocument {
    /// Start a fluent operation at a position
    pub fn at_pos(&mut self, position: Position) -> AtPosition<'_> {
        AtPosition::new(self, position)
    }

    /// Start a fluent operation at a line
    #[cfg(feature = "rope")]
    pub fn at_line(&mut self, line: usize) -> Result<AtPosition<'_>> {
        let line_idx = line.saturating_sub(1);
        if line_idx >= self.rope().len_lines() {
            return Err(EditorError::InvalidPosition { line, column: 1 });
        }

        let byte_pos = self.rope().line_to_byte(line_idx);
        Ok(AtPosition::new(self, Position::new(byte_pos)))
    }

    /// Start a fluent operation at the start of the document
    pub fn at_start(&mut self) -> AtPosition<'_> {
        AtPosition::new(self, Position::start())
    }

    /// Start a fluent operation at the end of the document
    pub fn at_end(&mut self) -> AtPosition<'_> {
        let end_pos = Position::new(self.len());
        AtPosition::new(self, end_pos)
    }

    /// Start a fluent operation on a range
    pub fn select(&mut self, range: Range) -> SelectRange<'_> {
        SelectRange::new(self, range)
    }

    /// Start fluent style operations
    pub fn styles(&mut self) -> StyleOps<'_> {
        StyleOps::new(self)
    }

    /// Start fluent event operations
    pub fn events(&mut self) -> EventOps<'_> {
        EventOps::new(self)
    }

    /// Start fluent tag operations
    pub fn tags(&mut self) -> TagOps<'_> {
        TagOps::new(self)
    }

    /// Start fluent karaoke operations
    pub fn karaoke(&mut self) -> KaraokeOps<'_> {
        KaraokeOps::new(self)
    }

    /// Start fluent script info operations
    pub fn info(&mut self) -> ScriptInfoOps<'_> {
        ScriptInfoOps::new(self)
    }

    /// Start fluent fonts operations
    pub fn fonts(&mut self) -> FontsOps<'_> {
        FontsOps::new(self)
    }

    /// Start fluent graphics operations
    pub fn graphics(&mut self) -> GraphicsOps<'_> {
        GraphicsOps::new(self)
    }

    /// Convert a Position to line/column tuple
    #[cfg(feature = "rope")]
    pub fn position_to_line_col(&self, pos: Position) -> Result<(usize, usize)> {
        if pos.offset > self.len() {
            return Err(EditorError::PositionOutOfBounds {
                position: pos.offset,
                length: self.len(),
            });
        }

        let line_idx = self.rope().byte_to_line(pos.offset);
        let line_start = self.rope().line_to_byte(line_idx);
        let col_offset = pos.offset - line_start;

        // Convert byte offset to character offset
        let line = self.rope().line(line_idx);
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

    /// Convert line/column to Position
    #[cfg(feature = "rope")]
    pub fn line_column_to_position(&self, line: usize, column: usize) -> Result<Position> {
        use crate::core::PositionBuilder;

        PositionBuilder::new()
            .line(line)
            .column(column)
            .build(self.rope())
    }
}
