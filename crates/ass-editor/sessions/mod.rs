//! Session management for multi-document editing
//!
//! Provides the `EditorSessionManager` for managing multiple documents
//! with shared resources, arenas, and extension registries. Supports
//! efficient session switching (<100µs target) and resource pooling.

#[cfg(not(feature = "std"))]
extern crate alloc;

use crate::core::{EditorDocument, EditorError, Result};

#[cfg(feature = "arena")]
use bumpalo::Bump;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[cfg(feature = "multi-thread")]
use std::sync::Arc;

#[cfg(all(feature = "plugins", not(feature = "multi-thread")))]
use std::sync::Arc;

#[cfg(not(feature = "multi-thread"))]
use core::cell::RefCell;

#[cfg(feature = "multi-thread")]
use parking_lot::Mutex;

/// Configuration for session management
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,

    /// Maximum memory usage per session in bytes
    pub max_memory_per_session: usize,

    /// Total memory limit across all sessions
    pub total_memory_limit: usize,

    /// Whether to enable automatic cleanup of unused sessions
    pub auto_cleanup: bool,

    /// Interval for arena resets (0 = never reset)
    pub arena_reset_interval: usize,

    /// Whether to share extension registry across sessions
    pub share_extensions: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_sessions: 50,
            max_memory_per_session: 100 * 1024 * 1024, // 100MB per session
            total_memory_limit: 1024 * 1024 * 1024,    // 1GB total
            auto_cleanup: true,
            arena_reset_interval: 1000, // Reset every 1000 operations
            share_extensions: true,
        }
    }
}

/// Statistics about session manager
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionStats {
    /// Number of active sessions
    pub active_sessions: usize,

    /// Total memory usage across all sessions
    pub total_memory_usage: usize,

    /// Number of operations since last cleanup
    pub operations_since_cleanup: usize,

    /// Number of arena resets performed
    pub arena_resets: usize,
}

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

/// Multi-document session manager with resource sharing
///
/// Manages multiple editing sessions with shared resources like extension
/// registries and arena allocators. Provides efficient session switching
/// and automatic resource management.
struct EditorSessionManagerInner {
    /// Configuration for this manager
    config: SessionConfig,

    /// Active editing sessions
    sessions: HashMap<String, EditorSession>,

    /// Currently active session ID
    active_session_id: Option<String>,

    /// Shared arena allocator for temporary operations
    #[cfg(feature = "arena")]
    shared_arena: Bump,

    /// Shared extension registry
    #[cfg(feature = "plugins")]
    extension_registry: Option<Arc<ass_core::plugin::ExtensionRegistry>>,

    /// Statistics tracking
    stats: SessionStats,

    /// Operations since last arena reset
    #[cfg(feature = "arena")]
    ops_since_arena_reset: usize,
}

/// Multi-document session manager with built-in thread-safety
pub struct EditorSessionManager {
    #[cfg(feature = "multi-thread")]
    inner: Arc<Mutex<EditorSessionManagerInner>>,
    #[cfg(not(feature = "multi-thread"))]
    inner: RefCell<EditorSessionManagerInner>,
}

// EditorSessionManager is cloneable when multi-thread feature is enabled
#[cfg(feature = "multi-thread")]
impl Clone for EditorSessionManager {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

// Note: EditorSessionManager does not implement Clone without multi-thread feature
// This is intentional - cloning requires Arc<Mutex<T>> which needs multi-thread

impl std::fmt::Debug for EditorSessionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "multi-thread")]
        {
            let inner = self.inner.lock();
            f.debug_struct("EditorSessionManager")
                .field("config", &inner.config)
                .field("active_session_id", &inner.active_session_id)
                .field("sessions", &inner.sessions.keys().collect::<Vec<_>>())
                .field("stats", &inner.stats)
                .finish()
        }
        #[cfg(not(feature = "multi-thread"))]
        {
            let inner = self.inner.borrow();
            f.debug_struct("EditorSessionManager")
                .field("config", &inner.config)
                .field("active_session_id", &inner.active_session_id)
                .field("sessions", &inner.sessions.keys().collect::<Vec<_>>())
                .field("stats", &inner.stats)
                .finish()
        }
    }
}

impl EditorSessionManagerInner {
    /// Create a new inner session manager
    fn new(config: SessionConfig) -> Self {
        Self {
            config,
            sessions: HashMap::new(),
            active_session_id: None,
            #[cfg(feature = "arena")]
            shared_arena: Bump::new(),
            #[cfg(feature = "plugins")]
            extension_registry: None,
            stats: SessionStats {
                active_sessions: 0,
                total_memory_usage: 0,
                operations_since_cleanup: 0,
                arena_resets: 0,
            },
            #[cfg(feature = "arena")]
            ops_since_arena_reset: 0,
        }
    }
}

impl EditorSessionManager {
    /// Helper method for accessing inner data mutably
    #[cfg(feature = "multi-thread")]
    fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut EditorSessionManagerInner) -> R,
    {
        let mut inner = self.inner.lock();
        f(&mut inner)
    }

    /// Helper method for accessing inner data immutably
    #[cfg(feature = "multi-thread")]
    fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&EditorSessionManagerInner) -> R,
    {
        let inner = self.inner.lock();
        f(&inner)
    }

    /// Helper method for accessing inner data mutably
    #[cfg(not(feature = "multi-thread"))]
    fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut EditorSessionManagerInner) -> R,
    {
        let mut inner = self.inner.borrow_mut();
        f(&mut inner)
    }

    /// Helper method for accessing inner data immutably
    #[cfg(not(feature = "multi-thread"))]
    fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&EditorSessionManagerInner) -> R,
    {
        let inner = self.inner.borrow();
        f(&inner)
    }

    /// Create a new session manager
    pub fn new() -> Self {
        Self::with_config(SessionConfig::default())
    }

    /// Create a new session manager with custom configuration
    pub fn with_config(config: SessionConfig) -> Self {
        #[cfg(feature = "multi-thread")]
        {
            Self {
                inner: Arc::new(Mutex::new(EditorSessionManagerInner::new(config))),
            }
        }
        #[cfg(not(feature = "multi-thread"))]
        {
            Self {
                inner: RefCell::new(EditorSessionManagerInner::new(config)),
            }
        }
    }

    /// Create a new session with an empty document
    pub fn create_session(&mut self, session_id: String) -> Result<()> {
        self.create_session_with_document(session_id, EditorDocument::new())
    }

    /// Create a new session with a specific document
    pub fn create_session_with_document(
        &mut self,
        session_id: String,
        document: EditorDocument,
    ) -> Result<()> {
        self.with_inner_mut(|inner| {
            // Check session limits
            if inner.sessions.len() >= inner.config.max_sessions {
                return Err(EditorError::SessionLimitExceeded {
                    current: inner.sessions.len(),
                    limit: inner.config.max_sessions,
                });
            }

            // Create new session
            let session = EditorSession::new(session_id.clone(), document);

            // Add to sessions map
            inner.sessions.insert(session_id.clone(), session);

            // Update stats
            inner.stats.active_sessions += 1;

            // Set as active if it's the first session
            if inner.active_session_id.is_none() {
                inner.active_session_id = Some(session_id);
            }

            Ok(())
        })
    }

    /// Switch to a different session
    pub fn switch_session(&mut self, session_id: &str) -> Result<()> {
        self.with_inner_mut(|inner| {
            // Check if session exists
            if !inner.sessions.contains_key(session_id) {
                return Err(EditorError::DocumentNotFound {
                    id: session_id.to_string(),
                });
            }

            // Switch active session
            inner.active_session_id = Some(session_id.to_string());

            // Touch the session to update access time
            #[cfg(feature = "std")]
            if let Some(session) = inner.sessions.get_mut(session_id) {
                session.touch();
            }

            Ok(())
        })
    }

    /// Get the currently active session
    pub fn active_session(&self) -> Result<Option<String>> {
        Ok(self.with_inner(|inner| inner.active_session_id.clone()))
    }

    /// Execute a function with a read-only reference to a session's document
    pub fn with_document<F, R>(&self, session_id: &str, f: F) -> Result<R>
    where
        F: FnOnce(&EditorDocument) -> Result<R>,
    {
        self.with_inner(|inner| {
            inner
                .sessions
                .get(session_id)
                .ok_or_else(|| EditorError::DocumentNotFound {
                    id: session_id.to_string(),
                })
                .and_then(|session| f(&session.document))
        })
    }

    /// Execute a closure with mutable access to a session's document
    pub fn with_document_mut<F, R>(&mut self, session_id: &str, f: F) -> Result<R>
    where
        F: FnOnce(&mut EditorDocument) -> Result<R>,
    {
        self.with_inner_mut(|inner| {
            let session = inner.sessions.get_mut(session_id).ok_or_else(|| {
                EditorError::DocumentNotFound {
                    id: session_id.to_string(),
                }
            })?;

            let result = f(&mut session.document)?;
            session.increment_operations();

            Ok(result)
        })
    }

    /// Remove a session
    pub fn remove_session(&mut self, session_id: &str) -> Result<EditorSession> {
        self.with_inner_mut(|inner| {
            let session =
                inner
                    .sessions
                    .remove(session_id)
                    .ok_or_else(|| EditorError::DocumentNotFound {
                        id: session_id.to_string(),
                    })?;

            // Update stats
            inner.stats.active_sessions -= 1;
            inner.stats.total_memory_usage -= session.memory_usage;

            // Clear active session if it was removed
            if inner.active_session_id.as_ref() == Some(&session_id.to_string()) {
                inner.active_session_id = None;
            }

            Ok(session)
        })
    }

    /// List all session IDs
    pub fn list_sessions(&self) -> Result<Vec<String>> {
        Ok(self.with_inner(|inner| inner.sessions.keys().cloned().collect()))
    }

    /// Get session statistics
    pub fn stats(&self) -> SessionStats {
        self.with_inner(|inner| inner.stats.clone())
    }

    /// Perform cleanup of stale sessions
    #[cfg(feature = "std")]
    pub fn cleanup_stale_sessions(&mut self, max_age: std::time::Duration) -> Result<usize> {
        // Get list of stale sessions
        let sessions_to_remove = self.with_inner(|inner| {
            if !inner.config.auto_cleanup {
                return vec![];
            }

            inner
                .sessions
                .iter()
                .filter(|(_, session)| session.is_stale(max_age))
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>()
        });

        // Remove stale sessions
        let mut removed_count = 0;
        for session_id in sessions_to_remove {
            if self.remove_session(&session_id).is_ok() {
                removed_count += 1;
            }
        }

        Ok(removed_count)
    }

    /// Reset shared arena to reclaim memory
    #[cfg(feature = "arena")]
    pub fn reset_shared_arena(&mut self) {
        self.with_inner_mut(|inner| {
            inner.shared_arena.reset();
            inner.stats.arena_resets += 1;
            inner.ops_since_arena_reset = 0;
        });
    }

    /// Set shared extension registry
    #[cfg(feature = "plugins")]
    pub fn set_extension_registry(&mut self, registry: Arc<ass_core::plugin::ExtensionRegistry>) {
        self.with_inner_mut(|inner| {
            inner.extension_registry = Some(registry);
        });
    }

    /// Get shared extension registry
    #[cfg(feature = "plugins")]
    #[must_use]
    pub fn extension_registry(&self) -> Option<Arc<ass_core::plugin::ExtensionRegistry>> {
        self.with_inner(|inner| inner.extension_registry.clone())
    }
}

impl Default for EditorSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_manager_creation() {
        let manager = EditorSessionManager::new();
        assert_eq!(manager.stats().active_sessions, 0);
        assert!(manager.active_session().unwrap().is_none());
    }

    #[test]
    fn session_creation_and_switching() {
        let mut manager = EditorSessionManager::new();

        // Create first session
        manager.create_session("session1".to_string()).unwrap();
        assert_eq!(manager.stats().active_sessions, 1);
        assert_eq!(
            manager.active_session().unwrap(),
            Some("session1".to_string())
        );

        // Create second session
        manager.create_session("session2".to_string()).unwrap();
        assert_eq!(manager.stats().active_sessions, 2);

        // Switch to second session
        manager.switch_session("session2").unwrap();
        assert_eq!(
            manager.active_session().unwrap(),
            Some("session2".to_string())
        );

        // List sessions
        let sessions = manager.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.contains(&"session1".to_string()));
        assert!(sessions.contains(&"session2".to_string()));
    }

    #[test]
    fn session_document_access() {
        let mut manager = EditorSessionManager::new();
        let doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

        manager
            .create_session_with_document("test".to_string(), doc)
            .unwrap();

        // Test document access
        manager
            .with_document("test", |doc| {
                assert!(doc.text().contains("Title: Test"));
                Ok(())
            })
            .unwrap();

        // Test document mutation
        manager
            .with_document_mut("test", |doc| {
                doc.insert(crate::core::Position::new(0), "Hello ")?;
                Ok(())
            })
            .unwrap();

        manager
            .with_document("test", |doc| {
                assert!(doc.text().starts_with("Hello "));
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn session_removal() {
        let mut manager = EditorSessionManager::new();

        manager.create_session("test".to_string()).unwrap();
        assert_eq!(manager.stats().active_sessions, 1);

        let removed_session = manager.remove_session("test").unwrap();
        assert_eq!(removed_session.id, "test");
        assert_eq!(manager.stats().active_sessions, 0);
        assert!(manager.active_session().unwrap().is_none());
    }

    #[test]
    fn session_limits() {
        let config = SessionConfig {
            max_sessions: 2,
            ..Default::default()
        };
        let mut manager = EditorSessionManager::with_config(config);

        // Create maximum allowed sessions
        manager.create_session("session1".to_string()).unwrap();
        manager.create_session("session2".to_string()).unwrap();

        // Try to create one more - should fail
        let result = manager.create_session("session3".to_string());
        assert!(matches!(
            result,
            Err(EditorError::SessionLimitExceeded { .. })
        ));
    }

    #[test]
    fn session_metadata() {
        let mut session = EditorSession::new("test".to_string(), EditorDocument::new());

        assert_eq!(session.get_metadata("key"), None);

        session.set_metadata("key".to_string(), "value".to_string());
        assert_eq!(session.get_metadata("key"), Some("value"));
    }
}
