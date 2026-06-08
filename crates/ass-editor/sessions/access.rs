//! Document-access, statistics, and maintenance operations.
//!
//! Implements read/write access to a session's [`EditorDocument`], session
//! statistics reporting, stale-session cleanup, shared-arena resets, and
//! shared extension-registry management for [`EditorSessionManager`].

use super::config::SessionStats;
use super::manager::EditorSessionManager;
use crate::core::{EditorDocument, EditorError, Result};

#[cfg(feature = "plugins")]
use std::sync::Arc;

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec, vec::Vec};

impl EditorSessionManager {
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
