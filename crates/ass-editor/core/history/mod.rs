//! History management for undo/redo operations
//!
//! Provides efficient undo/redo functionality with configurable depth limits,
//! arena-based memory pooling, and delta compression to minimize memory usage.

mod config;
mod entry;
mod manager;
mod operation;
mod stack;
mod stack_impl;

#[cfg(test)]
mod manager_tests;
#[cfg(test)]
mod stack_tests;

pub use config::UndoStackConfig;
pub use entry::HistoryEntry;
pub use manager::{HistoryStats, UndoManager};
#[cfg(feature = "stream")]
pub use operation::DeltaUndoData;
pub use operation::Operation;
pub use stack::UndoStack;
