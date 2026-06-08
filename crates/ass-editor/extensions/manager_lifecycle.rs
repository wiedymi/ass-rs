//! Extension manager construction and context creation.
//!
//! Provides constructors for the manager and its internal storage along with
//! the `create_context` bridge used to hand editor access to extensions.

use crate::core::{EditorDocument, Result};

use super::context::{EditorContext, ExtensionContext};
use super::manager::{ExtensionManager, ExtensionManagerInner};

#[cfg(feature = "std")]
use super::extension::{MessageHandler, StdMessageHandler};

#[cfg(feature = "std")]
use super::manager::EventSender;

#[cfg(not(feature = "std"))]
use super::extension::NoOpMessageHandler;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String};

#[cfg(feature = "std")]
use std::sync::mpsc;

#[cfg(feature = "multi-thread")]
use std::sync::Arc;

#[cfg(feature = "multi-thread")]
use parking_lot::Mutex;

#[cfg(not(feature = "multi-thread"))]
use core::cell::RefCell;

#[cfg(all(not(feature = "multi-thread"), not(feature = "std")))]
use alloc::rc::Rc;

#[cfg(all(not(feature = "multi-thread"), feature = "std"))]
use std::rc::Rc;

impl ExtensionManagerInner {
    /// Create a new inner manager
    fn new() -> Self {
        #[cfg(feature = "std")]
        let (tx, _rx) = mpsc::channel();

        Self {
            extensions: HashMap::new(),
            extension_states: HashMap::new(),
            commands: HashMap::new(),
            config: HashMap::new(),
            extension_data: HashMap::new(),
            #[cfg(feature = "std")]
            event_tx: tx,
            #[cfg(feature = "std")]
            message_handler: Box::new(StdMessageHandler),
            #[cfg(not(feature = "std"))]
            message_handler: Box::new(NoOpMessageHandler),
        }
    }
}

impl ExtensionManager {
    /// Create a new extension manager
    pub fn new() -> Self {
        #[cfg(feature = "multi-thread")]
        {
            Self {
                inner: Arc::new(Mutex::new(ExtensionManagerInner::new())),
            }
        }
        #[cfg(not(feature = "multi-thread"))]
        {
            Self {
                inner: RefCell::new(ExtensionManagerInner::new()),
            }
        }
    }

    /// Create a new extension manager with custom event sender and message handler
    #[cfg(feature = "std")]
    pub fn with_event_channel(
        event_tx: EventSender,
        message_handler: Box<dyn MessageHandler>,
    ) -> Self {
        let inner = ExtensionManagerInner {
            extensions: HashMap::new(),
            extension_states: HashMap::new(),
            commands: HashMap::new(),
            config: HashMap::new(),
            extension_data: HashMap::new(),
            event_tx,
            message_handler,
        };

        #[cfg(feature = "multi-thread")]
        {
            Self {
                inner: Arc::new(Mutex::new(inner)),
            }
        }
        #[cfg(not(feature = "multi-thread"))]
        {
            Self {
                inner: RefCell::new(inner),
            }
        }
    }

    /// Create an extension context for use by extensions
    pub fn create_context<'a>(
        &'a mut self,
        extension_name: String,
        document: Option<&'a mut EditorDocument>,
    ) -> Result<Box<dyn ExtensionContext + 'a>> {
        #[cfg(feature = "multi-thread")]
        {
            Ok(Box::new(EditorContext {
                document,
                manager: self.clone(),
                extension_name,
            }))
        }

        #[cfg(not(feature = "multi-thread"))]
        {
            // In single-threaded mode, we share the config state via Rc<RefCell>
            let config_clone = self.inner.borrow().config.clone();
            let shared_config = Rc::new(RefCell::new(config_clone));

            Ok(Box::new(EditorContext {
                document,
                manager: self,
                manager_mut_state: shared_config,
                extension_name,
            }))
        }
    }
}
