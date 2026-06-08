//! Handler registration for [`EventChannel`].
//!
//! Implements registering and unregistering [`EventHandler`] instances,
//! enforcing the configured handler limit and keeping handlers sorted by
//! descending priority.

use super::channel::{EventChannel, HandlerInfo};
use super::EventHandler;
use crate::core::Result;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format};

impl EventChannel {
    /// Register an event handler
    pub fn register_handler(&mut self, handler: Box<dyn EventHandler>) -> Result<usize> {
        let handler_id = self.next_handler_id;
        self.next_handler_id += 1;

        let filter = handler.event_filter();
        let priority = handler.priority();

        let handler_info = HandlerInfo {
            id: handler_id,
            handler,
            filter,
            priority,
            events_processed: 0,
        };

        #[cfg(feature = "multi-thread")]
        {
            let mut handlers =
                self.handlers
                    .write()
                    .map_err(|_| crate::core::EditorError::ThreadSafetyError {
                        message: "Failed to acquire write lock for handlers".to_string(),
                    })?;

            if handlers.len() >= self.config.max_handlers {
                return Err(crate::core::EditorError::CommandFailed {
                    message: format!("Handler limit reached: {}", self.config.max_handlers),
                });
            }

            handlers.push(handler_info);
            // Sort by priority (highest first)
            handlers.sort_by_key(|h| core::cmp::Reverse(h.priority));
        }

        #[cfg(not(feature = "multi-thread"))]
        {
            let mut handlers = self.handlers.borrow_mut();

            if handlers.len() >= self.config.max_handlers {
                return Err(crate::core::EditorError::CommandFailed {
                    message: format!("Handler limit reached: {}", self.config.max_handlers),
                });
            }

            handlers.push(handler_info);
            // Sort by priority (highest first)
            handlers.sort_by_key(|h| core::cmp::Reverse(h.priority));
        }

        self.stats.handlers_count += 1;
        Ok(handler_id)
    }

    /// Unregister an event handler by ID
    pub fn unregister_handler(&mut self, handler_id: usize) -> Result<bool> {
        #[cfg(feature = "multi-thread")]
        {
            let mut handlers =
                self.handlers
                    .write()
                    .map_err(|_| crate::core::EditorError::ThreadSafetyError {
                        message: "Failed to acquire write lock for handlers".to_string(),
                    })?;

            if let Some(pos) = handlers.iter().position(|h| h.id == handler_id) {
                handlers.remove(pos);
                self.stats.handlers_count -= 1;
                Ok(true)
            } else {
                Ok(false)
            }
        }

        #[cfg(not(feature = "multi-thread"))]
        {
            let mut handlers = self.handlers.borrow_mut();

            if let Some(pos) = handlers.iter().position(|h| h.id == handler_id) {
                handlers.remove(pos);
                self.stats.handlers_count -= 1;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}
