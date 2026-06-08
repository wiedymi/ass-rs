//! Async-friendly wrapper for non-blocking document operations.
//!
//! Provides [`AsyncDocument`], a thin async-aware wrapper around
//! [`SyncDocument`] for use in asynchronous runtimes.

use super::SyncDocument;
use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Result};

/// Async-friendly wrapper for non-blocking operations
#[cfg(all(feature = "concurrency", feature = "async"))]
pub struct AsyncDocument {
    sync_doc: SyncDocument,
}

#[cfg(all(feature = "concurrency", feature = "async"))]
impl AsyncDocument {
    /// Create a new async document wrapper
    pub fn new(document: EditorDocument) -> Self {
        Self {
            sync_doc: SyncDocument::new(document),
        }
    }

    /// Async-friendly text retrieval
    pub async fn text_async(&self) -> Result<String> {
        // In a real async implementation, this would use tokio::task::spawn_blocking
        // For now, we just wrap the sync version
        self.sync_doc.text()
    }

    /// Async-friendly command execution
    pub async fn execute_command_async<C: EditorCommand + Send + 'static>(
        &self,
        command: C,
    ) -> Result<CommandResult> {
        // In a real async implementation, this would use tokio::task::spawn_blocking
        self.sync_doc.execute_command(command)
    }
}
