//! Source span tracking for ASS AST nodes
//!
//! Provides the zero-copy [`Span`] type recording byte offsets and line/column
//! position information for nodes referencing the original source text.

/// Represents a span in the source text with position information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Span {
    /// Byte offset in source where span starts
    pub start: usize,
    /// Byte offset in source where span ends
    pub end: usize,
    /// Line number (1-based) where span starts
    pub line: u32,
    /// Column number (1-based) where span starts
    pub column: u32,
}

impl Span {
    /// Create a new span with position information
    #[must_use]
    pub const fn new(start: usize, end: usize, line: u32, column: u32) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    /// Check if a byte offset is contained within this span
    #[must_use]
    pub const fn contains(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Merge two spans to create a span covering both
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        use core::cmp::Ordering;

        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            column: match self.line.cmp(&other.line) {
                Ordering::Less => self.column,
                Ordering::Greater => other.column,
                Ordering::Equal => self.column.min(other.column),
            },
        }
    }
}
