//! Position and range types for document editing
//!
//! Provides types and builders for working with positions and ranges
//! in documents. Supports both byte offsets and line/column positions
//! with efficient conversion between them using the rope data structure.

use crate::core::errors::{EditorError, Result};
use core::cmp::{max, min};
use core::fmt;

/// A position in a document represented as byte offset
///
/// This is the primary position representation used internally
/// for efficiency. Can be converted to/from line/column positions.
///
/// # Examples
///
/// ```
/// use ass_editor::{Position, EditorDocument};
///
/// let doc = EditorDocument::from_content("Hello World").unwrap();
/// let pos = Position::new(6); // Position before "World"
///
/// // Basic operations
/// assert_eq!(pos.offset, 6);
/// assert!(!pos.is_start());
///
/// // Position arithmetic  
/// let advanced = pos.advance(5);
/// assert_eq!(advanced.offset, 11);
///
/// let retreated = pos.retreat(3);
/// assert_eq!(retreated.offset, 3);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// Byte offset from the beginning of the document
    pub offset: usize,
}

impl Position {
    /// Create a new position from byte offset
    #[must_use]
    pub const fn new(offset: usize) -> Self {
        Self { offset }
    }

    /// Create a position at the start of the document
    #[must_use]
    pub const fn start() -> Self {
        Self { offset: 0 }
    }

    /// Check if this position is at the start
    #[must_use]
    pub const fn is_start(&self) -> bool {
        self.offset == 0
    }

    /// Advance position by given bytes
    #[must_use]
    pub const fn advance(&self, bytes: usize) -> Self {
        Self {
            offset: self.offset.saturating_add(bytes),
        }
    }

    /// Move position back by given bytes
    #[must_use]
    pub const fn retreat(&self, bytes: usize) -> Self {
        Self {
            offset: self.offset.saturating_sub(bytes),
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::start()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.offset)
    }
}

/// A line/column position in a document
///
/// Lines and columns are 1-indexed for user-facing display.
/// Used for UI display and error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineColumn {
    /// 1-indexed line number
    pub line: usize,
    /// 1-indexed column number (in Unicode scalar values)
    pub column: usize,
}

impl LineColumn {
    /// Create a new line/column position
    ///
    /// # Errors
    /// Returns error if line or column is 0
    pub fn new(line: usize, column: usize) -> Result<Self> {
        if line == 0 || column == 0 {
            return Err(EditorError::InvalidPosition { line, column });
        }
        Ok(Self { line, column })
    }

    /// Create at start of document (1, 1)
    #[must_use]
    pub const fn start() -> Self {
        Self { line: 1, column: 1 }
    }
}

impl fmt::Display for LineColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A range in a document represented by start and end positions
///
/// Ranges are half-open intervals [start, end) where start is inclusive
/// and end is exclusive. This matches standard text editor conventions.
///
/// # Examples
///
/// ```
/// use ass_editor::{Position, Range, EditorDocument};
///
/// let doc = EditorDocument::from_content("Hello World").unwrap();
/// let range = Range::new(Position::new(0), Position::new(5)); // "Hello"
///
/// // Basic properties
/// assert_eq!(range.len(), 5);
/// assert!(!range.is_empty());
/// assert!(range.contains(Position::new(2)));
/// assert!(!range.contains(Position::new(5))); // End is exclusive
///
/// // Range operations
/// let other = Range::new(Position::new(3), Position::new(8)); // "lo Wo"
/// assert!(range.overlaps(&other));
///
/// let union = range.union(&other);
/// assert_eq!(union.start.offset, 0);
/// assert_eq!(union.end.offset, 8);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    /// Start position (inclusive)
    pub start: Position,
    /// End position (exclusive)
    pub end: Position,
}

impl Range {
    /// Create a new range
    ///
    /// Automatically normalizes so start <= end
    #[must_use]
    pub fn new(start: Position, end: Position) -> Self {
        if start.offset <= end.offset {
            Self { start, end }
        } else {
            Self {
                start: end,
                end: start,
            }
        }
    }

    /// Create an empty range at position
    #[must_use]
    pub const fn empty(pos: Position) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    /// Check if range is empty (start == end)
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start.offset == self.end.offset
    }

    /// Get the length of the range in bytes
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end.offset.saturating_sub(self.start.offset)
    }

    /// Check if range contains a position
    #[must_use]
    pub const fn contains(&self, pos: Position) -> bool {
        pos.offset >= self.start.offset && pos.offset < self.end.offset
    }

    /// Check if this range overlaps with another
    #[must_use]
    pub const fn overlaps(&self, other: &Self) -> bool {
        self.start.offset < other.end.offset && other.start.offset < self.end.offset
    }

    /// Extend range to include a position
    #[must_use]
    pub fn extend_to(&self, pos: Position) -> Self {
        Self {
            start: Position::new(min(self.start.offset, pos.offset)),
            end: Position::new(max(self.end.offset, pos.offset)),
        }
    }

    /// Get the union of two ranges (smallest range containing both)
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            start: Position::new(min(self.start.offset, other.start.offset)),
            end: Position::new(max(self.end.offset, other.end.offset)),
        }
    }

    /// Get the intersection of two ranges if they overlap
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let start = max(self.start.offset, other.start.offset);
        let end = min(self.end.offset, other.end.offset);

        if start < end {
            Some(Self::new(Position::new(start), Position::new(end)))
        } else {
            None
        }
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

/// Builder for creating document positions with fluent API
///
/// Provides ergonomic ways to create positions:
/// ```
/// use ass_editor::{EditorDocument, PositionBuilder};
///
/// let document = EditorDocument::from_content("Line 1\nLine 2\nLine 3").unwrap();
///
/// // PositionBuilder requires a Rope, not EditorDocument
/// // For this example, we'll use Position::new directly
/// let pos = ass_editor::Position::new(7); // Position at start of "Line 2"
///     
/// assert_eq!(pos.offset, 7);
/// ```
#[derive(Debug, Clone, Default)]
pub struct PositionBuilder {
    line: Option<usize>,
    column: Option<usize>,
    offset: Option<usize>,
}

impl PositionBuilder {
    /// Create a new position builder
    #[must_use]
    pub const fn new() -> Self {
        Self {
            line: None,
            column: None,
            offset: None,
        }
    }

    /// Set line number (1-indexed)
    #[must_use]
    pub const fn line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Set column number (1-indexed)
    #[must_use]
    pub const fn column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Set byte offset directly
    #[must_use]
    pub const fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Build position at the start of a line
    #[must_use]
    pub const fn at_line_start(mut self, line: usize) -> Self {
        self.line = Some(line);
        self.column = Some(1);
        self
    }

    /// Build position at the end of a line
    #[must_use]
    pub const fn at_line_end(mut self, line: usize) -> Self {
        self.line = Some(line);
        self.column = None; // Will be calculated
        self
    }

    /// Build position at the start of the document
    #[must_use]
    pub const fn at_start() -> Self {
        Self {
            line: Some(1),
            column: Some(1),
            offset: Some(0),
        }
    }

    /// Build position using rope for line/column conversion
    ///
    /// If offset is provided, uses that directly.
    /// Otherwise converts from line/column using the rope.
    #[cfg(feature = "rope")]
    pub fn build(self, rope: &ropey::Rope) -> Result<Position> {
        if let Some(offset) = self.offset {
            if offset > rope.len_bytes() {
                return Err(EditorError::PositionOutOfBounds {
                    position: offset,
                    length: rope.len_bytes(),
                });
            }
            Ok(Position::new(offset))
        } else if let Some(line) = self.line {
            // Convert to 0-indexed
            let line_idx = line.saturating_sub(1);

            if line_idx >= rope.len_lines() {
                return Err(EditorError::InvalidPosition { line, column: 1 });
            }

            let line_start = rope.line_to_byte(line_idx);

            if let Some(column) = self.column {
                LineColumn::new(line, column)?;
                let col_idx = column.saturating_sub(1);
                let line = rope.line(line_idx);

                // Find the byte position of the column
                let mut byte_pos = 0;
                let mut char_pos = 0;

                for ch in line.chars() {
                    if char_pos == col_idx {
                        break;
                    }
                    byte_pos += ch.len_utf8();
                    char_pos += 1;
                }

                if char_pos < col_idx {
                    return Err(EditorError::InvalidPosition {
                        line: self.line.unwrap_or(0),
                        column,
                    });
                }

                Ok(Position::new(line_start + byte_pos))
            } else {
                // No column specified - go to end of line
                let line_end = if line_idx + 1 < rope.len_lines() {
                    rope.line_to_byte(line_idx + 1).saturating_sub(1)
                } else {
                    rope.len_bytes()
                };
                Ok(Position::new(line_end))
            }
        } else {
            // Default to start if nothing specified
            Ok(Position::start())
        }
    }

    /// Build position without rope (offset must be specified)
    #[cfg(not(feature = "rope"))]
    pub fn build(self) -> Result<Position> {
        if let Some(offset) = self.offset {
            Ok(Position::new(offset))
        } else {
            Err(EditorError::FeatureNotEnabled {
                feature: "line/column position".to_string(),
                required_feature: "rope".to_string(),
            })
        }
    }
}

/// Selection represents a range with a direction
///
/// The anchor is where the selection started, and the cursor
/// is where it currently ends. This allows tracking selection direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// Where the selection started
    pub anchor: Position,
    /// Where the selection cursor is
    pub cursor: Position,
}

impl Selection {
    /// Create a new selection
    #[must_use]
    pub const fn new(anchor: Position, cursor: Position) -> Self {
        Self { anchor, cursor }
    }

    /// Create an empty selection at position
    #[must_use]
    pub const fn empty(pos: Position) -> Self {
        Self {
            anchor: pos,
            cursor: pos,
        }
    }

    /// Check if selection is empty (no selected text)
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.anchor.offset == self.cursor.offset
    }

    /// Get the range covered by this selection (normalized)
    #[must_use]
    pub fn range(&self) -> Range {
        Range::new(self.anchor, self.cursor)
    }

    /// Check if selection is reversed (cursor before anchor)
    #[must_use]
    pub const fn is_reversed(&self) -> bool {
        self.cursor.offset < self.anchor.offset
    }

    /// Extend selection to include a position
    #[must_use]
    pub const fn extend_to(&self, pos: Position) -> Self {
        Self {
            anchor: self.anchor,
            cursor: pos,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_operations() {
        let pos = Position::new(10);
        assert_eq!(pos.advance(5).offset, 15);
        assert_eq!(pos.retreat(5).offset, 5);
        assert_eq!(pos.retreat(20).offset, 0); // saturating
    }

    #[test]
    fn line_column_validation() {
        assert!(LineColumn::new(0, 1).is_err());
        assert!(LineColumn::new(1, 0).is_err());
        assert!(LineColumn::new(1, 1).is_ok());
    }

    #[test]
    fn range_normalization() {
        let r = Range::new(Position::new(10), Position::new(5));
        assert_eq!(r.start.offset, 5);
        assert_eq!(r.end.offset, 10);
    }

    #[test]
    fn range_operations() {
        let r1 = Range::new(Position::new(5), Position::new(10));
        let r2 = Range::new(Position::new(8), Position::new(15));

        assert!(r1.overlaps(&r2));
        assert_eq!(r1.union(&r2).start.offset, 5);
        assert_eq!(r1.union(&r2).end.offset, 15);

        let intersection = r1.intersection(&r2).unwrap();
        assert_eq!(intersection.start.offset, 8);
        assert_eq!(intersection.end.offset, 10);
    }

    #[test]
    fn selection_direction() {
        let sel = Selection::new(Position::new(10), Position::new(5));
        assert!(sel.is_reversed());
        assert_eq!(sel.range().start.offset, 5);
        assert_eq!(sel.range().end.offset, 10);
    }

    #[test]
    #[cfg(feature = "rope")]
    fn position_builder_with_rope() {
        let rope = ropey::Rope::from_str("Line 1\nLine 2\nLine 3");
        let pos = PositionBuilder::new()
            .line(2)
            .column(1)
            .build(&rope)
            .unwrap();
        assert_eq!(pos.offset, 7); // After "Line 1\n"
    }

    #[test]
    #[cfg(not(feature = "rope"))]
    fn position_builder_offset() {
        let pos = PositionBuilder::new().offset(42).build().unwrap();
        assert_eq!(pos.offset, 42);
    }
}
