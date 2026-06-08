//! Core [`EventChannel`] state and lifecycle.
//!
//! Holds the channel struct, the internal handler record, construction helpers,
//! statistics access, and handler clearing. Registration and dispatch live in
//! the sibling `registry` and `dispatch` modules.

use super::{EventChannelConfig, EventFilter, EventHandler, EventStats};
use crate::core::Result;

#[cfg(feature = "async")]
use super::DocumentEvent;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};

#[cfg(feature = "multi-thread")]
use std::sync::{Arc, RwLock};

#[cfg(not(feature = "multi-thread"))]
use core::cell::RefCell;

#[cfg(all(not(feature = "multi-thread"), not(feature = "std")))]
use alloc::rc::Rc;

#[cfg(all(not(feature = "multi-thread"), feature = "std"))]
use std::rc::Rc;

#[cfg(feature = "async")]
use futures::channel::mpsc;

/// Event channel for distributing document events to handlers
#[derive(Debug)]
pub struct EventChannel {
    /// Configuration for this channel
    pub(super) config: EventChannelConfig,

    /// Registered event handlers
    #[cfg(feature = "multi-thread")]
    pub(super) handlers: Arc<RwLock<Vec<HandlerInfo>>>,

    #[cfg(not(feature = "multi-thread"))]
    pub(super) handlers: Rc<RefCell<Vec<HandlerInfo>>>,

    /// Event statistics
    pub(super) stats: EventStats,

    /// Async event sender (if async feature is enabled)
    #[cfg(feature = "async")]
    pub(super) async_sender: Option<mpsc::UnboundedSender<DocumentEvent>>,

    /// Next handler ID for unique identification
    pub(super) next_handler_id: usize,
}

/// Information about a registered handler
pub(super) struct HandlerInfo {
    /// Unique handler ID
    pub(super) id: usize,
    /// Handler implementation
    pub(super) handler: Box<dyn EventHandler>,
    /// Event filter for this handler
    pub(super) filter: EventFilter,
    /// Handler priority
    pub(super) priority: i32,
    /// Number of events processed by this handler
    pub(super) events_processed: usize,
}

impl core::fmt::Debug for HandlerInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HandlerInfo")
            .field("id", &self.id)
            .field("filter", &self.filter)
            .field("priority", &self.priority)
            .field("events_processed", &self.events_processed)
            .field("handler", &"<EventHandler>")
            .finish()
    }
}

impl EventChannel {
    /// Create a new event channel with default configuration
    pub fn new() -> Self {
        Self::with_config(EventChannelConfig::default())
    }

    /// Create a new event channel with custom configuration
    pub fn with_config(config: EventChannelConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "multi-thread")]
            handlers: Arc::new(RwLock::new(Vec::new())),
            #[cfg(not(feature = "multi-thread"))]
            handlers: Rc::new(RefCell::new(Vec::new())),
            stats: EventStats {
                events_dispatched: 0,
                handlers_count: 0,
                events_filtered: 0,
                async_events_queued: 0,
                avg_processing_time_us: 0,
            },
            #[cfg(feature = "async")]
            async_sender: None,
            next_handler_id: 0,
        }
    }

    /// Get event statistics
    pub fn stats(&self) -> &EventStats {
        &self.stats
    }

    /// Clear all event handlers
    pub fn clear_handlers(&mut self) -> Result<()> {
        #[cfg(feature = "multi-thread")]
        {
            let mut handlers =
                self.handlers
                    .write()
                    .map_err(|_| crate::core::EditorError::ThreadSafetyError {
                        message: "Failed to acquire write lock for handlers".to_string(),
                    })?;
            handlers.clear();
        }

        #[cfg(not(feature = "multi-thread"))]
        {
            let mut handlers = self.handlers.borrow_mut();
            handlers.clear();
        }

        self.stats.handlers_count = 0;
        Ok(())
    }
}

impl Default for EventChannel {
    fn default() -> Self {
        Self::new()
    }
}
