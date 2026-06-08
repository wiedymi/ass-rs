//! Thread-safe document wrapper backed by a read-write lock.
//!
//! Provides [`SyncDocument`], a synchronized wrapper around
//! [`EditorDocument`] that lets multiple threads safely read and modify
//! the document through `RwLock` synchronization.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::errors::EditorError;
use crate::core::{EditorDocument, Result};

#[cfg(feature = "std")]
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[cfg(not(feature = "std"))]
use alloc::sync::Arc;

/// Thread-safe wrapper for EditorDocument
///
/// Provides synchronized access to an EditorDocument, allowing multiple
/// threads to safely read and modify the document. Uses RwLock for
/// efficient concurrent reads.
///
/// Note: Due to Bump allocator using Cell types, EditorDocument is not Sync.
/// SyncDocument provides thread-safe access through RwLock synchronization.
/// The RwLock ensures that only one thread can write at a time, and the
/// interior mutability of Bump is safe within a single thread.
#[cfg(feature = "concurrency")]
#[derive(Debug, Clone)]
pub struct SyncDocument {
    /// The wrapped document behind a read-write lock
    inner: Arc<RwLock<EditorDocument>>,

    /// Command execution lock to ensure atomic command operations
    command_lock: Arc<Mutex<()>>,
}

#[cfg(feature = "concurrency")]
impl SyncDocument {
    /// Create a new thread-safe document
    pub fn new(document: EditorDocument) -> Self {
        Self {
            inner: Arc::new(RwLock::new(document)),
            command_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Create from an existing document
    pub fn from_document(document: EditorDocument) -> Self {
        Self::new(document)
    }

    /// Get a read-only reference to the document
    pub fn read(&self) -> Result<RwLockReadGuard<'_, EditorDocument>> {
        self.inner
            .read()
            .map_err(|_| EditorError::ThreadSafetyError {
                message: "Failed to acquire read lock".to_string(),
            })
    }

    /// Get a mutable reference to the document
    pub fn write(&self) -> Result<RwLockWriteGuard<'_, EditorDocument>> {
        self.inner
            .write()
            .map_err(|_| EditorError::ThreadSafetyError {
                message: "Failed to acquire write lock".to_string(),
            })
    }

    /// Execute a command atomically
    pub fn execute_command<C: EditorCommand>(&self, command: C) -> Result<CommandResult> {
        // Lock command execution to ensure atomicity
        let _guard = self
            .command_lock
            .lock()
            .map_err(|_| EditorError::ThreadSafetyError {
                message: "Failed to acquire command lock".to_string(),
            })?;

        // Now execute the command with write access
        let mut doc = self.write()?;
        command.execute(&mut doc)
    }

    /// Try to get a read-only reference without blocking
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, EditorDocument>> {
        self.inner.try_read().ok()
    }

    /// Try to get a mutable reference without blocking
    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, EditorDocument>> {
        self.inner.try_write().ok()
    }

    /// Get the document text safely
    pub fn text(&self) -> Result<String> {
        let doc = self.read()?;
        Ok(doc.text())
    }

    /// Get document length safely
    pub fn len(&self) -> Result<usize> {
        let doc = self.read()?;
        Ok(doc.len())
    }

    /// Check if document is empty safely
    pub fn is_empty(&self) -> Result<bool> {
        let doc = self.read()?;
        Ok(doc.is_empty())
    }

    /// Get document ID safely
    pub fn id(&self) -> Result<String> {
        let doc = self.read()?;
        Ok(doc.id().to_string())
    }

    /// Perform an operation with read access
    pub fn with_read<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&EditorDocument) -> R,
    {
        let doc = self.read()?;
        Ok(f(&doc))
    }

    /// Perform an operation with write access
    pub fn with_write<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut EditorDocument) -> Result<R>,
    {
        let mut doc = self.write()?;
        f(&mut doc)
    }

    /// Clone the underlying document
    pub fn clone_document(&self) -> Result<EditorDocument> {
        let doc = self.read()?;
        EditorDocument::from_content(&doc.text())
    }

    /// Validate the document (basic parsing check)
    pub fn validate(&self) -> Result<()> {
        let doc = self.read()?;
        doc.validate()
    }

    /// Validate comprehensive (requires mutable access for LazyValidator)
    pub fn validate_comprehensive(&self) -> Result<Vec<crate::utils::validator::ValidationIssue>> {
        self.with_write(|doc| {
            let result = doc.validate_comprehensive()?;
            Ok(result.issues)
        })
    }
}
