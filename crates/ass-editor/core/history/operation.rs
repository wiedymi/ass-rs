//! Undoable operations and their memory accounting.
//!
//! Defines the [`Operation`] variants captured by the undo/redo system along
//! with the delta bookkeeping required to reverse incremental script updates.

use crate::core::position::{Position, Range};

#[cfg(feature = "stream")]
use ass_core::parser::ScriptDeltaOwned;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

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
