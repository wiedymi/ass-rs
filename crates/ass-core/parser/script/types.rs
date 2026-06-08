//! Supporting types for incremental script editing and change tracking.
//!
//! Defines the line-content, batch-operation, and change-tracking types used by
//! [`Script`](super::Script)'s mutation and incremental-parsing APIs.

use alloc::{boxed::Box, vec::Vec};

use crate::parser::ast::{Event, Section, SectionType, Style};
use crate::parser::errors::ParseError;

/// Parsed line content for context-aware parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineContent<'a> {
    /// A style definition line
    Style(Box<Style<'a>>),
    /// An event (dialogue, comment, etc.) line
    Event(Box<Event<'a>>),
    /// A script info field (key-value pair)
    Field(&'a str, &'a str),
}

/// A batch update operation
#[derive(Debug, Clone)]
pub struct UpdateOperation<'a> {
    /// Byte offset of the line to update
    pub offset: usize,
    /// New line content
    pub new_line: &'a str,
    /// Line number for error reporting
    pub line_number: u32,
}

/// Result of a batch update operation
#[derive(Debug)]
pub struct BatchUpdateResult<'a> {
    /// Successfully updated lines with their old content
    pub updated: Vec<(usize, LineContent<'a>)>,
    /// Failed updates with error information
    pub failed: Vec<(usize, ParseError)>,
}

/// A batch of style additions
#[derive(Debug, Clone)]
pub struct StyleBatch<'a> {
    /// Styles to add
    pub styles: Vec<Style<'a>>,
}

/// A batch of event additions
#[derive(Debug, Clone)]
pub struct EventBatch<'a> {
    /// Events to add
    pub events: Vec<Event<'a>>,
}

/// Represents a change in the script
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change<'a> {
    /// A line was added
    Added {
        /// Byte offset where the line was added
        offset: usize,
        /// The content that was added
        content: LineContent<'a>,
        /// Line number
        line_number: u32,
    },
    /// A line was removed
    Removed {
        /// Byte offset where the line was removed
        offset: usize,
        /// The section type that contained the removed line
        section_type: SectionType,
        /// Line number
        line_number: u32,
    },
    /// A line was modified
    Modified {
        /// Byte offset of the modification
        offset: usize,
        /// Previous content
        old_content: LineContent<'a>,
        /// New content
        new_content: LineContent<'a>,
        /// Line number
        line_number: u32,
    },
    /// A section was added
    SectionAdded {
        /// The section that was added
        section: Section<'a>,
        /// Index in the sections array
        index: usize,
    },
    /// A section was removed
    SectionRemoved {
        /// The section type that was removed
        section_type: SectionType,
        /// Index in the sections array
        index: usize,
    },
}

/// Tracks changes made to the script
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ChangeTracker<'a> {
    /// List of changes in the order they were made
    changes: Vec<Change<'a>>,
    /// Whether change tracking is enabled
    enabled: bool,
}

impl<'a> ChangeTracker<'a> {
    /// Enable change tracking
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable change tracking
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if tracking is enabled
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Record a change
    pub fn record(&mut self, change: Change<'a>) {
        if self.enabled {
            self.changes.push(change);
        }
    }

    /// Get all recorded changes
    #[must_use]
    pub fn changes(&self) -> &[Change<'a>] {
        &self.changes
    }

    /// Clear all recorded changes
    pub fn clear(&mut self) {
        self.changes.clear();
    }

    /// Get the number of recorded changes
    #[must_use]
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Check if there are no recorded changes
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}
