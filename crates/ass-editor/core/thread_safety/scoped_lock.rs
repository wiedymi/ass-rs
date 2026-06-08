//! Scoped write-lock guard for batch document operations.
//!
//! Provides [`ScopedDocumentLock`], an RAII guard that holds a write lock
//! on a [`SyncDocument`] for the duration of a batch of edits.

use super::SyncDocument;
use crate::core::{EditorDocument, Result};

#[cfg(feature = "std")]
use std::sync::RwLockWriteGuard;

/// Scoped lock guard for batch operations
#[cfg(feature = "concurrency")]
pub struct ScopedDocumentLock<'a> {
    _guard: RwLockWriteGuard<'a, EditorDocument>,
}

#[cfg(feature = "concurrency")]
impl<'a> ScopedDocumentLock<'a> {
    /// Create a new scoped lock
    pub fn new(document: &'a SyncDocument) -> Result<Self> {
        let guard = document.write()?;
        Ok(Self { _guard: guard })
    }

    /// Get the document for this lock
    pub fn document(&mut self) -> &mut EditorDocument {
        &mut self._guard
    }
}
