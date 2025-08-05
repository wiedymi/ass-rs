//! Core types and structures for the ass-editor
//!
//! This module contains the fundamental building blocks of the editor:
//! - `EditorDocument`: Main document type holding Script and Rope
//! - Position and range types for cursor/selection management
//! - Error types for editor operations
//! - History management for undo/redo

pub mod builders;
pub mod document;
pub mod errors;
pub mod fluent;
pub mod history;
#[cfg(feature = "stream")]
pub mod incremental;
pub mod position;

#[cfg(feature = "concurrency")]
pub mod thread_safety;

// Re-export commonly used types
pub use builders::{EventBuilder, StyleBuilder};
pub use document::{DocumentPosition, EditorDocument};
pub use errors::{EditorError, Result};
pub use fluent::{
    AtPosition, EventAccessor, EventFilter, EventInfo, EventQuery, EventSortCriteria,
    EventSortOptions, OwnedEvent, SelectRange,
};
pub use history::{HistoryEntry, HistoryStats, UndoManager, UndoStack, UndoStackConfig};
#[cfg(feature = "stream")]
pub use incremental::{DocumentChange, IncrementalParser};
pub use position::{LineColumn, Position, PositionBuilder, Range, Selection};

#[cfg(feature = "concurrency")]
pub use thread_safety::{DocumentPool, ScopedDocumentLock, SyncDocument};
