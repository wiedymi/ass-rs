//! Extension manager type definitions and shared internals.
//!
//! Houses the `ExtensionManager` handle, its internal storage, the inner-access
//! helpers shared across the manager implementation, and the event-sender alias.

use core::fmt;

#[cfg(feature = "std")]
use crate::events::DocumentEvent;

use super::command::{ExtensionCommand, ExtensionState};
use super::extension::{EditorExtension, MessageHandler};

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};

#[cfg(feature = "multi-thread")]
use std::sync::Arc;

#[cfg(not(feature = "multi-thread"))]
use core::cell::RefCell;

/// Extension manager with built-in thread safety
#[cfg(feature = "multi-thread")]
use parking_lot::Mutex;

#[cfg(feature = "std")]
use std::sync::mpsc::Sender;

/// Event sender type for channel communication
#[cfg(feature = "std")]
pub type EventSender = Sender<DocumentEvent>;

#[cfg(not(feature = "std"))]
pub type EventSender = (); // No-op for no_std environments

/// Extension manager for loading and managing extensions
/// Internal storage for ExtensionManager data
pub(super) struct ExtensionManagerInner {
    /// Loaded extensions
    pub(super) extensions: HashMap<String, Box<dyn EditorExtension>>,

    /// Extension states
    pub(super) extension_states: HashMap<String, ExtensionState>,

    /// Available commands from all extensions
    pub(super) commands: HashMap<String, (String, ExtensionCommand)>, // command_id -> (extension_name, command)

    /// Configuration storage
    pub(super) config: HashMap<String, String>,

    /// Inter-extension data storage
    pub(super) extension_data: HashMap<String, HashMap<String, String>>,

    /// Event channel for sending events
    #[cfg(feature = "std")]
    #[allow(dead_code)]
    pub(super) event_tx: EventSender,

    /// Message handler for user notifications
    #[allow(dead_code)]
    pub(super) message_handler: Box<dyn MessageHandler>,
}

/// Single unified ExtensionManager that is always thread-safe when multi-thread feature is enabled
pub struct ExtensionManager {
    #[cfg(feature = "multi-thread")]
    pub(super) inner: Arc<Mutex<ExtensionManagerInner>>,
    #[cfg(not(feature = "multi-thread"))]
    pub(super) inner: RefCell<ExtensionManagerInner>,
}

// ExtensionManager is cloneable when multi-thread feature is enabled
#[cfg(feature = "multi-thread")]
impl Clone for ExtensionManager {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

// Note: ExtensionManager does not implement Clone without multi-thread feature
// This is intentional - cloning requires Arc<Mutex<T>> which needs multi-thread

impl fmt::Debug for ExtensionManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "multi-thread")]
        {
            let inner = self.inner.lock();
            f.debug_struct("ExtensionManager")
                .field("extension_states", &inner.extension_states)
                .field("commands", &inner.commands.keys().collect::<Vec<_>>())
                .field("config", &inner.config)
                .field("extension_data", &inner.extension_data)
                .field("extensions", &"<HashMap<String, Box<dyn EditorExtension>>>")
                .finish()
        }
        #[cfg(not(feature = "multi-thread"))]
        {
            let inner = self.inner.borrow();
            f.debug_struct("ExtensionManager")
                .field("extension_states", &inner.extension_states)
                .field("commands", &inner.commands.keys().collect::<Vec<_>>())
                .field("config", &inner.config)
                .field("extension_data", &inner.extension_data)
                .field("extensions", &"<HashMap<String, Box<dyn EditorExtension>>>")
                .finish()
        }
    }
}

impl ExtensionManager {
    /// Helper method for accessing inner data mutably
    #[cfg(feature = "multi-thread")]
    pub(super) fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ExtensionManagerInner) -> R,
    {
        let mut inner = self.inner.lock();
        f(&mut inner)
    }

    /// Helper method for accessing inner data immutably
    #[cfg(feature = "multi-thread")]
    pub(super) fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ExtensionManagerInner) -> R,
    {
        let inner = self.inner.lock();
        f(&inner)
    }

    /// Helper method for accessing inner data mutably
    #[cfg(not(feature = "multi-thread"))]
    pub(super) fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ExtensionManagerInner) -> R,
    {
        let mut inner = self.inner.borrow_mut();
        f(&mut inner)
    }

    /// Helper method for accessing inner data immutably
    #[cfg(not(feature = "multi-thread"))]
    pub(super) fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ExtensionManagerInner) -> R,
    {
        let inner = self.inner.borrow();
        f(&inner)
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}
