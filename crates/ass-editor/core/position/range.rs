//! Document range type built from start/end [`Position`] values.
//!
//! Defines [`Range`], a half-open interval `[start, end)` with
//! containment, overlap, union, and intersection operations.

use super::Position;
use core::cmp::{max, min};
use core::fmt;

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
