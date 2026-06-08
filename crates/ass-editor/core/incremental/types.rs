//! Core type definitions for incremental parsing.
//!
//! Defines [`DocumentChange`], which records a single edit with delta tracking,
//! and [`IncrementalParser`], which manages incremental parsing state and delta
//! accumulation. Method implementations live in sibling submodules.

use crate::core::Range;

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::{borrow::Cow, string::String};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Represents a change to the document with delta tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentChange<'a> {
    /// The range that was affected
    pub range: Range,

    /// The new text that replaced the range
    pub new_text: Cow<'a, str>,

    /// The old text that was replaced (for undo)
    pub old_text: Cow<'a, str>,

    /// Timestamp when change occurred (for grouping)
    #[cfg(feature = "std")]
    pub timestamp: std::time::Instant,

    /// Change ID for tracking (monotonic counter)
    pub change_id: u64,
}

/// Manages incremental parsing state and delta accumulation
#[derive(Debug)]
pub struct IncrementalParser {
    /// Last successfully parsed script (cached)
    pub(super) cached_script: Option<String>,

    /// Accumulated changes since last full parse (owned to persist)
    pub(super) pending_changes: Vec<DocumentChange<'static>>,

    /// Next change ID
    pub(super) next_change_id: u64,

    /// Threshold for triggering full reparse (in bytes changed)
    pub(super) reparse_threshold: usize,

    /// Total bytes changed since last full parse
    pub(super) bytes_changed: usize,
}

impl Default for IncrementalParser {
    fn default() -> Self {
        Self::new()
    }
}
