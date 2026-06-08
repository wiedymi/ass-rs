//! Inherent methods for [`UndoStack`].
//!
//! Implements construction, push/pop, query, and capacity-enforcement logic
//! over the fields declared in the sibling `stack` module.

use super::{HistoryEntry, UndoStack, UndoStackConfig};

#[cfg(feature = "arena")]
use bumpalo::Bump;

#[cfg(feature = "std")]
use std::collections::VecDeque;

#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;

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
