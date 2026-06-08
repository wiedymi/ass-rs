//! Result type produced by executing editor commands.
//!
//! `CommandResult` carries success status, optional messaging, the modified
//! range, the new cursor position, and (when the `stream` feature is enabled)
//! an incremental `ScriptDeltaOwned` for partial re-parsing.

use crate::core::{Position, Range};

#[cfg(feature = "stream")]
use ass_core::parser::ScriptDeltaOwned;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Result of executing a command
///
/// Contains the modified document and optional metadata about the operation.
/// This will be used by the history system to track changes.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Whether the command was successfully executed
    pub success: bool,

    /// Optional message about the operation
    pub message: Option<String>,

    /// The range of text that was modified (for cursor updates)
    pub modified_range: Option<Range>,

    /// New cursor position after the command
    pub new_cursor: Option<Position>,

    /// Whether the document content was changed
    pub content_changed: bool,

    /// Script delta for incremental parsing (when available)
    #[cfg(feature = "stream")]
    pub script_delta: Option<ScriptDeltaOwned>,
}

impl CommandResult {
    /// Create a successful command result
    pub fn success() -> Self {
        Self {
            success: true,
            message: None,
            modified_range: None,
            new_cursor: None,
            content_changed: false,
            #[cfg(feature = "stream")]
            script_delta: None,
        }
    }

    /// Create a successful result with content change
    pub fn success_with_change(range: Range, cursor: Position) -> Self {
        Self {
            success: true,
            message: None,
            modified_range: Some(range),
            new_cursor: Some(cursor),
            content_changed: true,
            #[cfg(feature = "stream")]
            script_delta: None,
        }
    }

    /// Create a failed command result
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message: Some(message),
            modified_range: None,
            new_cursor: None,
            content_changed: false,
            #[cfg(feature = "stream")]
            script_delta: None,
        }
    }

    /// Add a script delta to the result
    #[cfg(feature = "stream")]
    #[must_use]
    pub fn with_delta(mut self, delta: ScriptDeltaOwned) -> Self {
        self.script_delta = Some(delta);
        self
    }

    /// Add a message to the result
    #[must_use]
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}
