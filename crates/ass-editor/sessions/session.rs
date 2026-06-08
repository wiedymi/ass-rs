//! Individual editing session state.
//!
//! Defines [`EditorSession`], pairing an [`EditorDocument`] with its
//! identifier, access bookkeeping, memory accounting, and per-session
//! metadata.

use crate::core::EditorDocument;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// A single editing session containing a document and associated state
#[derive(Debug)]
pub struct EditorSession {
    /// The document being edited
    pub document: EditorDocument,

    /// Session identifier
    pub id: String,

    /// Last access timestamp for cleanup purposes
    #[cfg(feature = "std")]
    pub last_accessed: std::time::Instant,

    /// Memory usage of this session
    pub memory_usage: usize,

    /// Number of operations performed in this session
    pub operation_count: usize,

    /// Session-specific metadata
    pub metadata: HashMap<String, String>,
}

impl EditorSession {
    /// Create a new session with a document
    pub fn new(id: String, document: EditorDocument) -> Self {
        Self {
            id,
            document,
            #[cfg(feature = "std")]
            last_accessed: std::time::Instant::now(),
            memory_usage: 0,
            operation_count: 0,
            metadata: HashMap::new(),
        }
    }

    /// Update last accessed time
    #[cfg(feature = "std")]
    pub fn touch(&mut self) {
        self.last_accessed = std::time::Instant::now();
    }

    /// Check if session is stale (for cleanup)
    #[cfg(feature = "std")]
    pub fn is_stale(&self, max_age: std::time::Duration) -> bool {
        self.last_accessed.elapsed() > max_age
    }

    /// Get session metadata
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Set session metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Increment operation counter
    pub fn increment_operations(&mut self) {
        self.operation_count += 1;
    }
}
