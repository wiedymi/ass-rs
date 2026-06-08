//! High-level undo/redo coordination.
//!
//! [`UndoManager`] drives the [`UndoStack`], translating command results into
//! recorded operations and exposing aggregate [`HistoryStats`].

use super::{HistoryEntry, Operation, UndoStack, UndoStackConfig};
use crate::commands::CommandResult;
use crate::core::position::Position;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Undo manager that coordinates between commands and history stack
///
/// Provides high-level undo/redo operations and manages the creation
/// of inverse commands for undoing operations.
#[derive(Debug)]
pub struct UndoManager {
    /// The underlying undo stack
    stack: UndoStack,

    /// Current cursor position for context
    current_cursor: Option<Position>,
}

impl UndoManager {
    /// Create a new undo manager
    pub fn new() -> Self {
        Self {
            stack: UndoStack::new(),
            current_cursor: None,
        }
    }

    /// Create a new undo manager with custom configuration
    pub fn with_config(config: UndoStackConfig) -> Self {
        Self {
            stack: UndoStack::with_config(config),
            current_cursor: None,
        }
    }

    /// Update the configuration of the undo stack
    pub fn set_config(&mut self, config: UndoStackConfig) {
        self.stack = UndoStack::with_config(config);
    }

    /// Update the current cursor position
    pub fn set_cursor(&mut self, cursor: Option<Position>) {
        self.current_cursor = cursor;
    }

    /// Get the current cursor position
    #[must_use]
    pub fn cursor_position(&self) -> Option<Position> {
        self.current_cursor
    }

    /// Record an operation for undo purposes
    ///
    /// Stores the operation data in the history stack.
    pub fn record_operation(
        &mut self,
        operation: Operation,
        description: String,
        result: &CommandResult,
    ) {
        if result.success && result.content_changed {
            #[cfg(feature = "stream")]
            let entry = if let Some(ref delta) = result.script_delta {
                HistoryEntry::with_delta(
                    operation,
                    description,
                    result,
                    self.current_cursor,
                    delta.clone(),
                )
            } else {
                HistoryEntry::new(operation, description, result, self.current_cursor)
            };

            #[cfg(not(feature = "stream"))]
            let entry = HistoryEntry::new(operation, description, result, self.current_cursor);

            self.stack.push(entry);

            // Update cursor position
            if let Some(new_cursor) = result.new_cursor {
                self.current_cursor = Some(new_cursor);
            }
        }
    }

    /// Check if undo is available
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.stack.can_undo()
    }

    /// Check if redo is available
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.stack.can_redo()
    }

    /// Get the next undo operation description
    #[must_use]
    pub fn next_undo_description(&self) -> Option<&str> {
        self.stack.next_undo_description()
    }

    /// Get the next redo operation description
    #[must_use]
    pub fn next_redo_description(&self) -> Option<&str> {
        self.stack.next_redo_description()
    }

    /// Get history statistics
    #[must_use]
    pub fn stats(&self) -> HistoryStats {
        HistoryStats {
            undo_count: self.stack.undo_count(),
            redo_count: self.stack.redo_count(),
            memory_usage: self.stack.memory_usage(),
        }
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    /// Access to underlying stack for advanced operations
    #[must_use]
    pub fn stack(&self) -> &UndoStack {
        &self.stack
    }

    /// Mutable access to underlying stack
    pub fn stack_mut(&mut self) -> &mut UndoStack {
        &mut self.stack
    }

    /// Pop an entry from undo stack and return it
    pub fn pop_undo_entry(&mut self) -> Option<HistoryEntry> {
        self.stack.pop_undo()
    }

    /// Pop an entry from redo stack and return it
    pub fn pop_redo_entry(&mut self) -> Option<HistoryEntry> {
        self.stack.pop_redo()
    }

    /// Push an entry to redo stack
    pub fn push_redo_entry(&mut self, entry: HistoryEntry) {
        self.stack.push_redo(entry);
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the history system
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryStats {
    /// Number of operations that can be undone
    pub undo_count: usize,
    /// Number of operations that can be redone
    pub redo_count: usize,
    /// Current memory usage in bytes
    pub memory_usage: usize,
}
