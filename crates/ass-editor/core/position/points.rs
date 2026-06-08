//! Single-point position types: byte offset and line/column.
//!
//! Defines [`Position`] (byte offset based) and [`LineColumn`]
//! (1-indexed line/column) along with their conversions and display
//! formatting.

use crate::core::errors::{EditorError, Result};
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
