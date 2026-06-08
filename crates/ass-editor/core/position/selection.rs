//! Directional text selection type.
//!
//! Defines [`Selection`], which pairs an anchor and cursor [`Position`]
//! to track selection direction and expose the covered [`Range`].

use super::{Position, Range};

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
