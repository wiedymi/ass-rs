//! Undo/redo history entries.
//!
//! [`HistoryEntry`] couples an [`Operation`] with the metadata needed to
//! restore cursor state and account for the memory each entry consumes.

use super::Operation;
use crate::commands::CommandResult;
use crate::core::position::{Position, Range};

#[cfg(feature = "stream")]
use ass_core::parser::ScriptDeltaOwned;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// A single entry in the undo/redo history
///
/// Stores the operation data needed to undo and redo changes
/// along with metadata for efficient memory management.
#[derive(Debug)]
pub struct HistoryEntry {
    /// The operation that was performed
    pub operation: Operation,

    /// Description of the operation
    pub description: String,

    /// Range that was modified by the operation
    pub modified_range: Option<Range>,

    /// Cursor position before the operation
    pub cursor_before: Option<Position>,

    /// Cursor position after the operation
    pub cursor_after: Option<Position>,

    /// Script delta for efficient incremental parsing (when available)
    #[cfg(feature = "stream")]
    pub script_delta: Option<ScriptDeltaOwned>,

    /// Timestamp when the operation was performed
    #[cfg(feature = "std")]
    pub timestamp: std::time::Instant,

    /// Memory usage of this entry (for capacity management)
    pub memory_usage: usize,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(
        operation: Operation,
        description: String,
        result: &CommandResult,
        cursor_before: Option<Position>,
    ) -> Self {
        let memory_usage = operation.memory_usage() + description.len();

        Self {
            operation,
            description,
            modified_range: result.modified_range,
            cursor_before,
            cursor_after: result.new_cursor,
            #[cfg(feature = "stream")]
            script_delta: None,
            #[cfg(feature = "std")]
            timestamp: std::time::Instant::now(),
            memory_usage,
        }
    }

    /// Create a new history entry with delta
    #[cfg(feature = "stream")]
    pub fn with_delta(
        operation: Operation,
        description: String,
        result: &CommandResult,
        cursor_before: Option<Position>,
        script_delta: ScriptDeltaOwned,
    ) -> Self {
        let delta_memory = script_delta.added.iter().map(|s| s.len()).sum::<usize>()
            + script_delta
                .modified
                .iter()
                .map(|(_, s)| s.len())
                .sum::<usize>();
        let memory_usage = operation.memory_usage() + description.len() + delta_memory;

        Self {
            operation,
            description,
            modified_range: result.modified_range,
            cursor_before,
            cursor_after: result.new_cursor,
            script_delta: Some(script_delta),
            #[cfg(feature = "std")]
            timestamp: std::time::Instant::now(),
            memory_usage,
        }
    }
}
