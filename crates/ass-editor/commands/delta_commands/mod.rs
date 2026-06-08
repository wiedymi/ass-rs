//! Delta-aware commands for incremental ASS editing
//!
//! Commands that use ass-core's Delta tracking for optimal performance

mod batch;
mod event_edit;
mod insert;
mod replace;

#[cfg(test)]
mod tests;

pub use batch::DeltaBatchCommand;
pub use event_edit::IncrementalEventEditCommand;
pub use insert::IncrementalInsertCommand;
pub use replace::IncrementalReplaceCommand;

use super::CommandResult;
use crate::core::{EditorDocument, Result};

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// A command that tracks deltas for incremental updates
pub trait DeltaCommand {
    /// Execute the command and return the result with delta information
    fn execute_with_delta(&self, document: &mut EditorDocument) -> Result<super::CommandResult>;

    /// Get command description for history
    fn description(&self) -> String;

    /// Check if this command can be executed incrementally
    fn supports_incremental(&self) -> bool {
        cfg!(feature = "stream")
    }
}
