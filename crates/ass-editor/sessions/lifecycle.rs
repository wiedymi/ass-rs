//! Session lifecycle operations for [`EditorSessionManager`].
//!
//! Implements creation, switching, removal, and enumeration of sessions,
//! tracking the active session and enforcing configured session limits.

use super::config::SessionConfig;
use super::manager::EditorSessionManager;
use super::session::EditorSession;
use crate::core::{EditorDocument, EditorError, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

impl EditorSessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self::with_config(SessionConfig::default())
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
}

impl Default for EditorSessionManager {
    fn default() -> Self {
        Self::new()
    }
}
