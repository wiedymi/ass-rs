//! `DocumentEvent` enum describing editor changes.
//!
//! Defines the event variants emitted when a document mutates, covering text
//! edits, cursor/selection updates, lifecycle transitions, and custom events.

use crate::core::{Position, Range};

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Types of events that can occur in the editor
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentEvent {
    /// Text was inserted at a position
    TextInserted {
        /// Position where text was inserted
        position: Position,
        /// Text that was inserted
        text: String,
        /// Length of inserted text in bytes
        length: usize,
    },

    /// Text was deleted from a range
    TextDeleted {
        /// Range where text was deleted
        range: Range,
        /// Text that was deleted (for undo purposes)
        deleted_text: String,
    },

    /// Text was replaced in a range
    TextReplaced {
        /// Range where text was replaced
        range: Range,
        /// Old text that was replaced
        old_text: String,
        /// New text that replaced the old text
        new_text: String,
    },

    /// Selection changed
    SelectionChanged {
        /// Previous selection range (if any)
        old_selection: Option<Range>,
        /// New selection range (if any)
        new_selection: Option<Range>,
    },

    /// Cursor position changed
    CursorMoved {
        /// Previous cursor position
        old_position: Position,
        /// New cursor position
        new_position: Position,
    },

    /// Document was saved to a file
    DocumentSaved {
        /// File path where document was saved
        file_path: String,
        /// Whether this was a "save as" operation
        save_as: bool,
    },

    /// Document was loaded from a file
    DocumentLoaded {
        /// File path from which document was loaded
        file_path: String,
        /// Size of loaded document in bytes
        size: usize,
    },

    /// Undo operation was performed
    UndoPerformed {
        /// Description of the undone action
        action_description: String,
        /// Number of changes undone
        changes_count: usize,
    },

    /// Redo operation was performed
    RedoPerformed {
        /// Description of the redone action
        action_description: String,
        /// Number of changes redone
        changes_count: usize,
    },

    /// Validation completed with results
    ValidationCompleted {
        /// Number of issues found
        issues_count: usize,
        /// Number of errors found
        error_count: usize,
        /// Number of warnings found
        warning_count: usize,
        /// Time taken for validation in milliseconds
        validation_time_ms: u64,
    },

    /// Search operation completed
    SearchCompleted {
        /// Search pattern that was used
        pattern: String,
        /// Number of matches found
        matches_count: usize,
        /// Whether the search hit the result limit
        hit_limit: bool,
        /// Time taken for search in microseconds
        search_time_us: u64,
    },

    /// Document parsing completed
    ParsingCompleted {
        /// Whether parsing was successful
        success: bool,
        /// Number of sections parsed
        sections_count: usize,
        /// Time taken for parsing in milliseconds
        parse_time_ms: u64,
        /// Any error message if parsing failed
        error_message: Option<String>,
    },

    /// Extension was loaded or unloaded
    ExtensionChanged {
        /// Name of the extension
        extension_name: String,
        /// Whether the extension was loaded (true) or unloaded (false)
        loaded: bool,
    },

    /// Configuration setting changed
    ConfigChanged {
        /// Name of the configuration key
        key: String,
        /// Old value (if any)
        old_value: Option<String>,
        /// New value
        new_value: String,
    },

    /// Generic custom event for extensions
    CustomEvent {
        /// Event type identifier
        event_type: String,
        /// Event data as key-value pairs
        data: HashMap<String, String>,
    },
}
