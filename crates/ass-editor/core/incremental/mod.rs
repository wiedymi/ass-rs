//! Incremental parsing integration with ass-core
//!
//! Provides efficient incremental parsing by leveraging Script::parse_partial()
//! to achieve <1ms edit times and <5ms reparse times. Tracks deltas for proper
//! undo/redo integration and maintains consistency with the rope structure.

mod apply;
mod reparse;
mod state;
mod types;

#[cfg(test)]
mod tests;

pub use types::{DocumentChange, IncrementalParser};
