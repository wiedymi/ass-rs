//! Core [`PositionTracker`] type and its position-advancing operations.
//!
//! Holds the cursor state (byte offset, line, column) over a source string and
//! exposes span-construction helpers used throughout the parser.

use crate::parser::ast::Span;

/// Tracks current position in source text with line/column information
#[derive(Debug, Clone)]
pub struct PositionTracker<'a> {
    /// Source text being tracked
    source: &'a str,
    /// Current byte offset in source
    offset: usize,
    /// Current line number (1-based)
    line: u32,
    /// Current column number (1-based)
    column: u32,
    /// Byte offset of current line start
    line_start: usize,
}

impl<'a> PositionTracker<'a> {
    /// Create a new position tracker for source text
    #[must_use]
    pub const fn new(source: &'a str) -> Self {
        Self {
            source,
            offset: 0,
            line: 1,
            column: 1,
            line_start: 0,
        }
    }

    /// Create a tracker starting at a specific position
    #[must_use]
    pub const fn new_at(source: &'a str, offset: usize, line: u32, column: u32) -> Self {
        Self {
            source,
            offset,
            line,
            column,
            line_start: offset.saturating_sub((column - 1) as usize),
        }
    }

    /// Get current byte offset
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// Get current line number (1-based)
    #[must_use]
    pub const fn line(&self) -> u32 {
        self.line
    }

    /// Get current column number (1-based)
    #[must_use]
    pub const fn column(&self) -> u32 {
        self.column
    }

    /// Advance position by a given number of bytes
    pub fn advance(&mut self, bytes: usize) {
        let end = (self.offset + bytes).min(self.source.len());

        while self.offset < end {
            if self.source.as_bytes().get(self.offset) == Some(&b'\n') {
                self.offset += 1;
                self.line += 1;
                self.column = 1;
                self.line_start = self.offset;
            } else {
                self.offset += 1;
                self.column += 1;
            }
        }
    }

    /// Advance to a specific byte offset
    pub fn advance_to(&mut self, target_offset: usize) {
        if target_offset > self.offset {
            self.advance(target_offset - self.offset);
        }
    }

    /// Skip whitespace and update position
    pub fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.source.as_bytes().get(self.offset) {
            if ch == b' ' || ch == b'\t' || ch == b'\r' {
                self.advance(1);
            } else {
                break;
            }
        }
    }

    /// Skip to end of current line
    pub fn skip_line(&mut self) {
        while let Some(&ch) = self.source.as_bytes().get(self.offset) {
            self.advance(1);
            if ch == b'\n' {
                break;
            }
        }
    }

    /// Get remaining source text from current position
    #[must_use]
    pub fn remaining(&self) -> &'a str {
        &self.source[self.offset..]
    }

    /// Check if at end of source
    #[must_use]
    pub const fn is_at_end(&self) -> bool {
        self.offset >= self.source.len()
    }

    /// Create a span from a start position to current position
    #[must_use]
    pub const fn span_from(&self, start: &PositionTracker) -> Span {
        Span::new(start.offset, self.offset, start.line, start.column)
    }

    /// Create a span for a range of bytes from current position
    #[must_use]
    pub const fn span_for(&self, length: usize) -> Span {
        Span::new(self.offset, self.offset + length, self.line, self.column)
    }

    /// Clone current position state
    #[must_use]
    pub const fn checkpoint(&self) -> Self {
        PositionTracker {
            source: self.source,
            offset: self.offset,
            line: self.line,
            column: self.column,
            line_start: self.line_start,
        }
    }
}
