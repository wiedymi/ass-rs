//! Event dispatch for [`EventChannel`].
//!
//! Implements synchronous and batched delivery to registered handlers,
//! statistics bookkeeping, and the optional asynchronous channel plumbing
//! gated behind the `async` feature.

use super::channel::EventChannel;
use super::DocumentEvent;
use crate::core::Result;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "async")]
use futures::channel::mpsc;

impl EventChannel {
    /// Dispatch an event to all registered handlers
    pub fn dispatch(&mut self, event: DocumentEvent) -> Result<()> {
        #[cfg(feature = "std")]
        let start_time = std::time::Instant::now();

        self.stats.events_dispatched += 1;

        let mut filtered_count = 0;
        #[cfg(feature = "std")]
        let mut processed_count = 0;

        #[cfg(feature = "multi-thread")]
        {
            let mut handlers =
                self.handlers
                    .write()
                    .map_err(|_| crate::core::EditorError::ThreadSafetyError {
                        message: "Failed to acquire write lock for handlers".to_string(),
                    })?;

            for handler_info in handlers.iter_mut() {
                if handler_info.filter.matches(&event) {
                    handler_info.handler.handle_event(&event)?;
                    handler_info.events_processed += 1;
                    #[cfg(feature = "std")]
                    {
                        processed_count += 1;
                    }
                } else {
                    filtered_count += 1;
                }
            }
        }

        #[cfg(not(feature = "multi-thread"))]
        {
            let mut handlers = self.handlers.borrow_mut();

            for handler_info in handlers.iter_mut() {
                if handler_info.filter.matches(&event) {
                    handler_info.handler.handle_event(&event)?;
                    handler_info.events_processed += 1;
                    #[cfg(feature = "std")]
                    {
                        processed_count += 1;
                    }
                } else {
                    filtered_count += 1;
                }
            }
        }

        self.stats.events_filtered += filtered_count;

        // Update average processing time
        #[cfg(feature = "std")]
        {
            let processing_time = start_time.elapsed().as_micros() as u64;
            if self.stats.events_dispatched == 1 {
                self.stats.avg_processing_time_us = processing_time;
            } else {
                self.stats.avg_processing_time_us =
                    (self.stats.avg_processing_time_us + processing_time) / 2;
            }
        }

        if self.config.enable_logging {
            #[cfg(feature = "std")]
            eprintln!(
                "Event dispatched: {} -> {} handlers (filtered: {})",
                event.description(),
                processed_count,
                filtered_count
            );
        }

        Ok(())
    }

    /// Dispatch multiple events in a batch
    pub fn dispatch_batch(&mut self, events: Vec<DocumentEvent>) -> Result<()> {
        if self.config.enable_batching && events.len() <= self.config.max_batch_size {
            // Process as a batch
            for event in events {
                self.dispatch(event)?;
            }
        } else {
            // Process individually
            for event in events {
                self.dispatch(event)?;
            }
        }
        Ok(())
    }

    /// Setup async event processing (requires async feature)
    #[cfg(feature = "async")]
    pub fn setup_async(&mut self) -> mpsc::UnboundedReceiver<DocumentEvent> {
        let (sender, receiver) = mpsc::unbounded();
        self.async_sender = Some(sender);
        receiver
    }

    /// Dispatch event asynchronously (requires async feature)
    #[cfg(feature = "async")]
    pub fn dispatch_async(&mut self, event: DocumentEvent) -> Result<()> {
        if let Some(ref sender) = self.async_sender {
            sender
                .unbounded_send(event)
                .map_err(|_| crate::core::EditorError::CommandFailed {
                    message: "Failed to send async event".to_string(),
                })?;
            self.stats.async_events_queued += 1;
        } else {
            return Err(crate::core::EditorError::FeatureNotEnabled {
                feature: "async event processing".to_string(),
                required_feature: "async".to_string(),
            });
        }
        Ok(())
    }
}
