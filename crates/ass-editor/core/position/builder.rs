//! Fluent builder for constructing document [`Position`] values.
//!
//! Defines [`PositionBuilder`], which resolves line/column or byte
//! offset specifications into a [`Position`], using the rope when the
//! `rope` feature is enabled.

#[cfg(feature = "rope")]
use super::LineColumn;
use super::Position;
use crate::core::errors::{EditorError, Result};

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
