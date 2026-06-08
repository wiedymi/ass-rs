//! Construction and cache-state management for [`IncrementalParser`].
//!
//! Provides constructors, threshold configuration, cache priming, and the
//! lightweight accessors used to inspect accumulated incremental state.

use super::{DocumentChange, IncrementalParser};
use crate::core::errors::EditorError;
use crate::core::Result;
use ass_core::parser::Script;

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

impl IncrementalParser {
    /// Create a new incremental parser
    pub fn new() -> Self {
        Self {
            cached_script: None,
            pending_changes: Vec::new(),
            next_change_id: 1,
            reparse_threshold: 10_000, // 10KB of changes triggers full reparse
            bytes_changed: 0,
        }
    }

    /// Set the reparse threshold in bytes
    pub fn set_reparse_threshold(&mut self, threshold: usize) {
        self.reparse_threshold = threshold;
    }

    /// Initialize the cache with the current document content
    /// This primes the incremental parser for efficient subsequent edits
    pub fn initialize_cache(&mut self, content: &str) {
        self.cached_script = Some(content.to_string());
        self.pending_changes.clear();
        self.bytes_changed = 0;
    }

    /// Check if there's a cached script available
    pub fn has_cached_script(&self) -> bool {
        self.cached_script.is_some()
    }

    /// Execute a function with the cached script (avoids re-parsing)
    pub fn with_cached_script<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Script) -> Result<R>,
    {
        let cached = self
            .cached_script
            .as_ref()
            .ok_or_else(|| EditorError::command_failed("No cached script available"))?;
        let script = Script::parse(cached).map_err(EditorError::from)?;
        f(&script)
    }

    /// Clear all cached state
    pub fn clear_cache(&mut self) {
        self.cached_script = None;
        self.pending_changes.clear();
        self.bytes_changed = 0;
        self.next_change_id = 1;
    }

    /// Get accumulated changes since last full parse
    pub fn pending_changes(&self) -> &[DocumentChange<'static>] {
        &self.pending_changes
    }

    /// Check if a full reparse is recommended
    pub fn should_reparse(&self) -> bool {
        self.bytes_changed >= self.reparse_threshold || self.pending_changes.len() > 50
    }
}
