//! History management for undo/redo operations
//!
//! Provides efficient undo/redo functionality with configurable depth limits,
//! arena-based memory pooling, and delta compression to minimize memory usage.

use super::position::{Position, Range};
use crate::commands::CommandResult;

#[cfg(feature = "stream")]
use ass_core::parser::ScriptDeltaOwned;

#[cfg(feature = "arena")]
use bumpalo::Bump;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "std")]
use std::collections::VecDeque;

#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;

/// Data needed to undo a delta operation
#[cfg(feature = "stream")]
#[derive(Debug, Clone)]
pub struct DeltaUndoData {
    /// Sections that were removed (index and content)
    pub removed_sections: Vec<(usize, String)>,
    /// Sections that were modified (index and original content)
    pub modified_sections: Vec<(usize, String)>,
}

/// Represents an operation that can be undone and redone
#[derive(Debug, Clone)]
pub enum Operation {
    /// Text was inserted
    Insert { position: Position, text: String },
    /// Text was deleted
    Delete { range: Range, deleted_text: String },
    /// Text was replaced
    Replace {
        range: Range,
        old_text: String,
        new_text: String,
    },
    /// Delta was applied (for incremental updates)
    #[cfg(feature = "stream")]
    Delta {
        /// The forward delta to apply
        forward: ScriptDeltaOwned,
        /// Data needed to undo this delta
        undo_data: DeltaUndoData,
    },
}

impl Operation {
    /// Get memory usage of this operation
    pub fn memory_usage(&self) -> usize {
        match self {
            Self::Insert { text, .. } => core::mem::size_of::<Self>() + text.len(),
            Self::Delete { deleted_text, .. } => core::mem::size_of::<Self>() + deleted_text.len(),
            Self::Replace {
                old_text, new_text, ..
            } => core::mem::size_of::<Self>() + old_text.len() + new_text.len(),
            #[cfg(feature = "stream")]
            Self::Delta { forward, undo_data } => {
                core::mem::size_of::<Self>()
                    + forward.added.iter().map(|s| s.len()).sum::<usize>()
                    + forward.modified.iter().map(|(_, s)| s.len()).sum::<usize>()
                    + undo_data
                        .removed_sections
                        .iter()
                        .map(|(_, s)| s.len())
                        .sum::<usize>()
                    + undo_data
                        .modified_sections
                        .iter()
                        .map(|(_, s)| s.len())
                        .sum::<usize>()
            }
        }
    }
}

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

/// Configuration for undo stack behavior
#[derive(Debug, Clone)]
pub struct UndoStackConfig {
    /// Maximum number of undo entries to keep
    pub max_entries: usize,

    /// Maximum memory usage in bytes (0 = unlimited)
    pub max_memory: usize,

    /// Whether to enable compression of old entries
    pub enable_compression: bool,

    /// Interval for arena resets (0 = never reset)
    pub arena_reset_interval: usize,
}

impl Default for UndoStackConfig {
    fn default() -> Self {
        Self {
            max_entries: 50, // Set a sensible default, can be overridden programmatically
            max_memory: 10 * 1024 * 1024, // 10MB default
            enable_compression: true,
            arena_reset_interval: 100, // Reset arena every 100 operations
        }
    }
}

/// Undo/redo stack with efficient memory management
///
/// Uses a circular buffer design with arena allocation for optimal
/// performance. Supports configurable limits and automatic cleanup.
#[derive(Debug)]
pub struct UndoStack {
    /// Configuration for this stack
    config: UndoStackConfig,

    /// Undo history (most recent operations first)
    undo_stack: VecDeque<HistoryEntry>,

    /// Redo history (operations that can be redone)
    redo_stack: VecDeque<HistoryEntry>,

    /// Current memory usage in bytes
    current_memory: usize,

    /// Arena for temporary allocations
    #[cfg(feature = "arena")]
    arena: Bump,

    /// Number of operations since last arena reset
    #[cfg(feature = "arena")]
    ops_since_reset: usize,
}

impl UndoStack {
    /// Create a new undo stack with default configuration
    pub fn new() -> Self {
        Self::with_config(UndoStackConfig::default())
    }

    /// Create a new undo stack with custom configuration
    pub fn with_config(config: UndoStackConfig) -> Self {
        Self {
            config,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            current_memory: 0,
            #[cfg(feature = "arena")]
            arena: Bump::new(),
            #[cfg(feature = "arena")]
            ops_since_reset: 0,
        }
    }

    /// Push a new entry onto the undo stack
    ///
    /// This clears the redo stack as new operations invalidate
    /// previously undone operations.
    pub fn push(&mut self, entry: HistoryEntry) {
        // Clear redo stack - new operations invalidate undone changes
        self.clear_redo_stack();

        // Add memory usage
        self.current_memory += entry.memory_usage;

        // Add to undo stack
        self.undo_stack.push_front(entry);

        // Enforce limits
        self.enforce_limits();

        // Periodic arena reset
        #[cfg(feature = "arena")]
        {
            self.ops_since_reset += 1;
            if self.config.arena_reset_interval > 0
                && self.ops_since_reset >= self.config.arena_reset_interval
            {
                self.reset_arena();
            }
        }
    }

    /// Pop the most recent entry from the undo stack
    pub fn pop_undo(&mut self) -> Option<HistoryEntry> {
        if let Some(entry) = self.undo_stack.pop_front() {
            self.current_memory -= entry.memory_usage;
            Some(entry)
        } else {
            None
        }
    }

    /// Push an entry onto the redo stack
    pub fn push_redo(&mut self, entry: HistoryEntry) {
        self.current_memory += entry.memory_usage;
        self.redo_stack.push_front(entry);
    }

    /// Pop an entry from the redo stack
    pub fn pop_redo(&mut self) -> Option<HistoryEntry> {
        if let Some(entry) = self.redo_stack.pop_front() {
            self.current_memory -= entry.memory_usage;
            Some(entry)
        } else {
            None
        }
    }

    /// Check if undo is available
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of undo entries available
    #[must_use]
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redo entries available
    #[must_use]
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Get current memory usage in bytes
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        self.current_memory
    }

    /// Get description of the next undo operation
    #[must_use]
    pub fn next_undo_description(&self) -> Option<&str> {
        self.undo_stack
            .front()
            .map(|entry| entry.description.as_str())
    }

    /// Get description of the next redo operation
    #[must_use]
    pub fn next_redo_description(&self) -> Option<&str> {
        self.redo_stack
            .front()
            .map(|entry| entry.description.as_str())
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_memory = 0;

        #[cfg(feature = "arena")]
        {
            self.reset_arena();
        }
    }

    /// Clear only the redo stack (called when new operations are performed)
    fn clear_redo_stack(&mut self) {
        for entry in self.redo_stack.drain(..) {
            self.current_memory -= entry.memory_usage;
        }
    }

    /// Enforce memory and count limits
    fn enforce_limits(&mut self) {
        // Enforce entry count limit
        while self.undo_stack.len() > self.config.max_entries {
            if let Some(entry) = self.undo_stack.pop_back() {
                self.current_memory -= entry.memory_usage;
            }
        }

        // Enforce memory limit
        while self.config.max_memory > 0
            && self.current_memory > self.config.max_memory
            && !self.undo_stack.is_empty()
        {
            if let Some(entry) = self.undo_stack.pop_back() {
                self.current_memory -= entry.memory_usage;
            }
        }
    }

    /// Reset the arena allocator to reclaim memory
    #[cfg(feature = "arena")]
    fn reset_arena(&mut self) {
        self.arena.reset();
        self.ops_since_reset = 0;
    }

    /// Get arena allocator reference
    #[cfg(feature = "arena")]
    #[must_use]
    pub fn arena(&self) -> &Bump {
        &self.arena
    }

    /// Get mutable arena allocator reference
    #[cfg(feature = "arena")]
    pub fn arena_mut(&mut self) -> &mut Bump {
        &mut self.arena
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_stack_basic_operations() {
        let mut stack = UndoStack::new();
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());

        // Create a dummy history entry
        let operation = Operation::Insert {
            position: Position::new(0),
            text: "test".to_string(),
        };
        let result = CommandResult::success();
        let entry = HistoryEntry::new(operation, "Test".to_string(), &result, None);

        stack.push(entry);
        assert!(stack.can_undo());
        assert!(!stack.can_redo());

        let popped = stack.pop_undo().unwrap();
        assert_eq!(popped.description, "Test");
        assert!(!stack.can_undo());
    }

    #[test]
    fn undo_stack_memory_limits() {
        let config = UndoStackConfig {
            max_entries: 2,
            max_memory: 1000,
            enable_compression: false,
            arena_reset_interval: 0,
        };

        let mut stack = UndoStack::with_config(config);

        // Add entries beyond limit
        for i in 0..5 {
            let operation = Operation::Insert {
                position: Position::new(0),
                text: format!("test{i}"),
            };
            let result = CommandResult::success();
            let entry = HistoryEntry::new(operation, format!("Test {i}"), &result, None);
            stack.push(entry);
        }

        // Should be limited to 2 entries
        assert_eq!(stack.undo_count(), 2);
    }

    #[test]
    fn undo_manager_operations() {
        let mut manager = UndoManager::new();

        // Simulate an insert operation
        let operation = Operation::Insert {
            position: Position::new(0),
            text: "Hello".to_string(),
        };
        let mut result = CommandResult::success();
        result.content_changed = true;

        // Record the operation
        manager.record_operation(operation, "Insert text".to_string(), &result);

        assert!(manager.can_undo());
        assert_eq!(manager.next_undo_description(), Some("Insert text"));
    }

    #[test]
    fn history_stats() {
        let manager = UndoManager::new();
        let stats = manager.stats();

        assert_eq!(stats.undo_count, 0);
        assert_eq!(stats.redo_count, 0);
        assert_eq!(stats.memory_usage, 0);
    }

    #[test]
    fn operation_memory_usage() {
        let insert_op = Operation::Insert {
            position: Position::new(0),
            text: "Hello".to_string(),
        };
        assert!(insert_op.memory_usage() >= 5);

        let delete_op = Operation::Delete {
            range: Range::new(Position::new(0), Position::new(5)),
            deleted_text: "Hello".to_string(),
        };
        assert!(delete_op.memory_usage() >= 5);

        let replace_op = Operation::Replace {
            range: Range::new(Position::new(0), Position::new(5)),
            old_text: "Hello".to_string(),
            new_text: "World".to_string(),
        };
        assert!(replace_op.memory_usage() >= 10);
    }

    #[test]
    fn programmatic_undo_limit_configuration() {
        // Test custom undo limit configuration
        let custom_config = UndoStackConfig {
            max_entries: 3,
            max_memory: 1000,
            enable_compression: false,
            arena_reset_interval: 0,
        };

        let mut manager = UndoManager::with_config(custom_config);

        // Add more operations than the limit
        for i in 0..5 {
            let operation = Operation::Insert {
                position: Position::new(i * 10),
                text: format!("test{i}"),
            };
            let mut result = CommandResult::success();
            result.content_changed = true;
            manager.record_operation(operation, format!("Insert {i}"), &result);
        }

        // Should only keep the last 3 operations due to max_entries limit
        let stats = manager.stats();
        assert_eq!(stats.undo_count, 3);

        // Check that the correct operations are kept (most recent ones)
        assert_eq!(manager.next_undo_description(), Some("Insert 4"));

        // Test undo operations respect the limit
        manager.pop_undo_entry();
        assert_eq!(manager.next_undo_description(), Some("Insert 3"));

        manager.pop_undo_entry();
        assert_eq!(manager.next_undo_description(), Some("Insert 2"));

        manager.pop_undo_entry();
        assert_eq!(manager.next_undo_description(), None);
    }

    #[test]
    fn undo_stack_config_default() {
        // Test that default configuration is sensible
        let default_config = UndoStackConfig::default();
        assert_eq!(default_config.max_entries, 50);
        assert_eq!(default_config.max_memory, 10 * 1024 * 1024); // 10MB
        assert!(default_config.enable_compression);
        assert_eq!(default_config.arena_reset_interval, 100);
    }

    #[test]
    fn undo_manager_config_update() {
        // Test that UndoManager can have its configuration updated
        let mut manager = UndoManager::new();

        // Add an operation with default config
        let operation = Operation::Insert {
            position: Position::new(0),
            text: "test".to_string(),
        };
        let mut result = CommandResult::success();
        result.content_changed = true;
        manager.record_operation(operation, "Initial".to_string(), &result);

        assert_eq!(manager.stats().undo_count, 1);

        // Update to a more restrictive config
        let restrictive_config = UndoStackConfig {
            max_entries: 0, // No undo history allowed
            max_memory: 0,
            enable_compression: false,
            arena_reset_interval: 0,
        };

        manager.set_config(restrictive_config);

        // The stack should be recreated, so previous operations should be gone
        assert_eq!(manager.stats().undo_count, 0);

        // New operations should not be recorded due to 0 limit
        let operation = Operation::Insert {
            position: Position::new(5),
            text: "test2".to_string(),
        };
        manager.record_operation(operation, "Should not record".to_string(), &result);
        assert_eq!(manager.stats().undo_count, 0);
    }

    #[test]
    fn memory_limit_enforcement() {
        // Test that memory limits are enforced
        let memory_limited_config = UndoStackConfig {
            max_entries: 100, // High entry limit
            max_memory: 50,   // Very low memory limit (50 bytes)
            enable_compression: false,
            arena_reset_interval: 0,
        };

        let mut manager = UndoManager::with_config(memory_limited_config);

        // Add operations that exceed memory limit
        for i in 0..5 {
            let operation = Operation::Insert {
                position: Position::new(i * 10),
                text: format!(
                    "This is a long text string for operation {i} that should consume memory"
                ),
            };
            let mut result = CommandResult::success();
            result.content_changed = true;
            manager.record_operation(operation, format!("Long operation {i}"), &result);
        }

        // Should have fewer operations due to memory constraint
        let stats = manager.stats();
        assert!(stats.undo_count < 5);
        assert!(stats.memory_usage <= 50 || stats.undo_count == 0);
    }
}
