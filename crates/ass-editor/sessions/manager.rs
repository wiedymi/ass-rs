//! Multi-document session manager core.
//!
//! Defines [`EditorSessionManager`] and its private interior state, including
//! construction, thread-safe interior-access helpers, and the [`Clone`] and
//! [`core::fmt::Debug`] implementations. Session lifecycle and document-access
//! operations live in the sibling [`lifecycle`](super::lifecycle) and
//! [`access`](super::access) modules.

use super::config::{SessionConfig, SessionStats};
use super::session::EditorSession;

#[cfg(feature = "arena")]
use bumpalo::Bump;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "multi-thread")]
use std::sync::Arc;

#[cfg(all(feature = "plugins", not(feature = "multi-thread")))]
use std::sync::Arc;

#[cfg(not(feature = "multi-thread"))]
use core::cell::RefCell;

#[cfg(feature = "multi-thread")]
use parking_lot::Mutex;

/// Multi-document session manager with resource sharing
///
/// Manages multiple editing sessions with shared resources like extension
/// registries and arena allocators. Provides efficient session switching
/// and automatic resource management.
pub(super) struct EditorSessionManagerInner {
    /// Configuration for this manager
    pub(super) config: SessionConfig,

    /// Active editing sessions
    pub(super) sessions: HashMap<String, EditorSession>,

    /// Currently active session ID
    pub(super) active_session_id: Option<String>,

    /// Shared arena allocator for temporary operations
    #[cfg(feature = "arena")]
    pub(super) shared_arena: Bump,

    /// Shared extension registry
    #[cfg(feature = "plugins")]
    pub(super) extension_registry: Option<Arc<ass_core::plugin::ExtensionRegistry>>,

    /// Statistics tracking
    pub(super) stats: SessionStats,

    /// Operations since last arena reset
    #[cfg(feature = "arena")]
    pub(super) ops_since_arena_reset: usize,
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
    pub(super) fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut EditorSessionManagerInner) -> R,
    {
        let mut inner = self.inner.lock();
        f(&mut inner)
    }

    /// Helper method for accessing inner data immutably
    #[cfg(feature = "multi-thread")]
    pub(super) fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&EditorSessionManagerInner) -> R,
    {
        let inner = self.inner.lock();
        f(&inner)
    }

    /// Helper method for accessing inner data mutably
    #[cfg(not(feature = "multi-thread"))]
    pub(super) fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut EditorSessionManagerInner) -> R,
    {
        let mut inner = self.inner.borrow_mut();
        f(&mut inner)
    }

    /// Helper method for accessing inner data immutably
    #[cfg(not(feature = "multi-thread"))]
    pub(super) fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&EditorSessionManagerInner) -> R,
    {
        let inner = self.inner.borrow();
        f(&inner)
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
}
