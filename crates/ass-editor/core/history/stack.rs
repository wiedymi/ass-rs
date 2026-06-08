//! Undo/redo stack storage.
//!
//! [`UndoStack`] owns the undo and redo queues plus the optional arena used for
//! temporary allocations; its inherent methods live in the sibling `stack_impl`
//! module, so the fields are `pub(super)` for use across the `history` subtree.

use super::{HistoryEntry, UndoStackConfig};

#[cfg(feature = "arena")]
use bumpalo::Bump;

#[cfg(feature = "std")]
use std::collections::VecDeque;

#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;

/// Undo/redo stack with efficient memory management
///
/// Uses a circular buffer design with arena allocation for optimal
/// performance. Supports configurable limits and automatic cleanup.
#[derive(Debug)]
pub struct UndoStack {
    /// Configuration for this stack
    pub(super) config: UndoStackConfig,

    /// Undo history (most recent operations first)
    pub(super) undo_stack: VecDeque<HistoryEntry>,

    /// Redo history (operations that can be redone)
    pub(super) redo_stack: VecDeque<HistoryEntry>,

    /// Current memory usage in bytes
    pub(super) current_memory: usize,

    /// Arena for temporary allocations
    #[cfg(feature = "arena")]
    pub(super) arena: Bump,

    /// Number of operations since last arena reset
    #[cfg(feature = "arena")]
    pub(super) ops_since_reset: usize,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}
