//! Event system for document changes and editor notifications
//!
//! Provides `DocumentEvent` enum for representing editor changes and
//! `EventChannel` for distributing events to observers. Supports both
//! synchronous and asynchronous event handling with filtering.

mod channel;
mod dispatch;
mod event;
mod event_builders;
mod event_query;
mod filter;
mod handler;
mod registry;

#[cfg(test)]
mod channel_tests;

#[cfg(test)]
mod event_tests;

pub use channel::EventChannel;
pub use event::DocumentEvent;
pub use filter::EventFilter;
pub use handler::{EventChannelConfig, EventHandler, EventStats};
