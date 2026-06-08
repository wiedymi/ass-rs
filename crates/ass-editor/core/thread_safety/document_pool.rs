//! Thread-safe pool for managing multiple synchronized documents.
//!
//! Provides [`DocumentPool`], a registry mapping document IDs to
//! [`SyncDocument`] instances for concurrent multi-document workflows.

use super::SyncDocument;
use crate::core::errors::EditorError;
use crate::core::{EditorDocument, Result};

#[cfg(feature = "std")]
use std::sync::{Arc, RwLock};

#[cfg(not(feature = "std"))]
use alloc::sync::Arc;

/// Thread-safe document pool for managing multiple documents
#[cfg(feature = "concurrency")]
#[derive(Debug, Clone)]
pub struct DocumentPool {
    /// Map of document IDs to synchronized documents
    documents: Arc<RwLock<std::collections::HashMap<String, SyncDocument>>>,
}

#[cfg(feature = "concurrency")]
impl DocumentPool {
    /// Create a new document pool
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Add a document to the pool
    pub fn add_document(&self, document: EditorDocument) -> Result<String> {
        let id = document.id().to_string();
        let sync_doc = SyncDocument::new(document);

        let mut docs = self
            .documents
            .write()
            .map_err(|_| EditorError::ThreadSafetyError {
                message: "Failed to acquire pool write lock".to_string(),
            })?;

        docs.insert(id.clone(), sync_doc);
        Ok(id)
    }

    /// Get a document from the pool
    pub fn get_document(&self, id: &str) -> Result<SyncDocument> {
        let docs = self
            .documents
            .read()
            .map_err(|_| EditorError::ThreadSafetyError {
                message: "Failed to acquire pool read lock".to_string(),
            })?;

        docs.get(id)
            .cloned()
            .ok_or_else(|| EditorError::ValidationError {
                message: format!("Document not found: {id}"),
            })
    }

    /// Remove a document from the pool
    pub fn remove_document(&self, id: &str) -> Result<()> {
        let mut docs = self
            .documents
            .write()
            .map_err(|_| EditorError::ThreadSafetyError {
                message: "Failed to acquire pool write lock".to_string(),
            })?;

        docs.remove(id);
        Ok(())
    }

    /// List all document IDs in the pool
    pub fn list_documents(&self) -> Result<Vec<String>> {
        let docs = self
            .documents
            .read()
            .map_err(|_| EditorError::ThreadSafetyError {
                message: "Failed to acquire pool read lock".to_string(),
            })?;

        Ok(docs.keys().cloned().collect())
    }

    /// Get the number of documents in the pool
    pub fn document_count(&self) -> Result<usize> {
        let docs = self
            .documents
            .read()
            .map_err(|_| EditorError::ThreadSafetyError {
                message: "Failed to acquire pool read lock".to_string(),
            })?;

        Ok(docs.len())
    }
}

#[cfg(feature = "concurrency")]
impl Default for DocumentPool {
    fn default() -> Self {
        Self::new()
    }
}
