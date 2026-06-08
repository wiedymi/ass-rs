//! Handler trait and channel configuration/statistics types.
//!
//! Declares the [`EventHandler`] trait implemented by observers along with the
//! [`EventStats`] and [`EventChannelConfig`] value types used by the channel.

use super::{DocumentEvent, EventFilter};
use crate::core::Result;

/// Event handler trait for responding to document events
pub trait EventHandler: Send + Sync {
    /// Handle a document event
    fn handle_event(&mut self, event: &DocumentEvent) -> Result<()>;

    /// Get the event filter for this handler
    fn event_filter(&self) -> EventFilter {
        EventFilter::new()
    }

    /// Get handler priority (higher numbers = higher priority)
    fn priority(&self) -> i32 {
        0
    }
}

/// Statistics about event handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventStats {
    /// Total number of events dispatched
    pub events_dispatched: usize,
    /// Number of handlers currently registered
    pub handlers_count: usize,
    /// Number of events dropped due to filters
    pub events_filtered: usize,
    /// Number of async events queued
    pub async_events_queued: usize,
    /// Average event processing time in microseconds
    pub avg_processing_time_us: u64,
}

/// Event channel configuration
#[derive(Debug, Clone)]
pub struct EventChannelConfig {
    /// Maximum number of handlers
    pub max_handlers: usize,
    /// Maximum size of async event queue
    pub max_async_queue_size: usize,
    /// Whether to enable event batching
    pub enable_batching: bool,
    /// Maximum batch size for event processing
    pub max_batch_size: usize,
    /// Whether to log events for debugging
    pub enable_logging: bool,
}

impl Default for EventChannelConfig {
    fn default() -> Self {
        Self {
            max_handlers: 100,
            max_async_queue_size: 1000,
            enable_batching: false,
            max_batch_size: 10,
            enable_logging: false,
        }
    }
}
