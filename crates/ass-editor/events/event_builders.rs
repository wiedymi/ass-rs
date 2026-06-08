//! Convenience constructors for common [`DocumentEvent`] variants.
//!
//! Factory helpers that build text, cursor, save, and validation events with
//! derived fields (e.g. inserted text length) computed automatically.

use super::DocumentEvent;
use crate::core::{Position, Range};

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Convenience functions for creating common events
impl DocumentEvent {
    /// Create a text insertion event
    pub fn text_inserted(position: Position, text: String) -> Self {
        let length = text.len();
        Self::TextInserted {
            position,
            text,
            length,
        }
    }

    /// Create a text deletion event
    pub fn text_deleted(range: Range, deleted_text: String) -> Self {
        Self::TextDeleted {
            range,
            deleted_text,
        }
    }

    /// Create a text replacement event
    pub fn text_replaced(range: Range, old_text: String, new_text: String) -> Self {
        Self::TextReplaced {
            range,
            old_text,
            new_text,
        }
    }

    /// Create a cursor moved event
    pub fn cursor_moved(old_position: Position, new_position: Position) -> Self {
        Self::CursorMoved {
            old_position,
            new_position,
        }
    }

    /// Create a document saved event
    pub fn document_saved(file_path: String, save_as: bool) -> Self {
        Self::DocumentSaved { file_path, save_as }
    }

    /// Create a validation completed event
    pub fn validation_completed(
        issues_count: usize,
        error_count: usize,
        warning_count: usize,
        validation_time_ms: u64,
    ) -> Self {
        Self::ValidationCompleted {
            issues_count,
            error_count,
            warning_count,
            validation_time_ms,
        }
    }
}
