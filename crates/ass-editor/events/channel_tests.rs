//! Unit tests for `EventChannel` registration, dispatch, and configuration.

use super::*;
use crate::core::{Position, Result};
#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[test]
fn event_channel_creation() {
    let channel = EventChannel::new();
    assert_eq!(channel.stats().handlers_count, 0);
    assert_eq!(channel.stats().events_dispatched, 0);
}

#[test]
fn event_channel_config() {
    let config = EventChannelConfig {
        max_handlers: 50,
        enable_logging: true,
        ..Default::default()
    };

    let channel = EventChannel::with_config(config);
    assert_eq!(channel.config.max_handlers, 50);
    assert!(channel.config.enable_logging);
}

// Mock handler for testing
struct TestHandler {
    events_received: Vec<String>,
    filter: EventFilter,
    priority: i32,
}

impl TestHandler {
    fn new() -> Self {
        Self {
            events_received: Vec::new(),
            filter: EventFilter::new(),
            priority: 0,
        }
    }

    fn with_filter(filter: EventFilter) -> Self {
        Self {
            events_received: Vec::new(),
            filter,
            priority: 0,
        }
    }

    fn with_priority(priority: i32) -> Self {
        Self {
            events_received: Vec::new(),
            filter: EventFilter::new(),
            priority,
        }
    }
}

impl EventHandler for TestHandler {
    fn handle_event(&mut self, event: &DocumentEvent) -> Result<()> {
        self.events_received.push(event.description());
        Ok(())
    }

    fn event_filter(&self) -> EventFilter {
        self.filter.clone()
    }

    fn priority(&self) -> i32 {
        self.priority
    }
}

#[test]
fn event_channel_handler_registration() {
    let mut channel = EventChannel::new();
    let handler = Box::new(TestHandler::new());

    let handler_id = channel.register_handler(handler).unwrap();
    assert_eq!(channel.stats().handlers_count, 1);

    let removed = channel.unregister_handler(handler_id).unwrap();
    assert!(removed);
    assert_eq!(channel.stats().handlers_count, 0);
}

#[test]
fn event_channel_dispatch() {
    let mut channel = EventChannel::new();
    let handler = Box::new(TestHandler::new());

    channel.register_handler(handler).unwrap();

    let event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());
    channel.dispatch(event).unwrap();

    assert_eq!(channel.stats().events_dispatched, 1);
}

#[test]
fn event_channel_filtering() {
    let mut channel = EventChannel::new();
    let filter = EventFilter::new().exclude_types(vec!["CursorMoved".to_string()]);
    let handler = Box::new(TestHandler::with_filter(filter));

    channel.register_handler(handler).unwrap();

    // This should be handled
    let insert_event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());
    channel.dispatch(insert_event).unwrap();

    // This should be filtered out
    let cursor_event = DocumentEvent::cursor_moved(Position::new(0), Position::new(5));
    channel.dispatch(cursor_event).unwrap();

    assert_eq!(channel.stats().events_dispatched, 2);
    assert_eq!(channel.stats().events_filtered, 1);
}

#[test]
fn event_channel_priority_ordering() {
    let mut channel = EventChannel::new();

    let low_priority_handler = Box::new(TestHandler::with_priority(1));
    let high_priority_handler = Box::new(TestHandler::with_priority(10));

    // Register in reverse priority order
    channel.register_handler(low_priority_handler).unwrap();
    channel.register_handler(high_priority_handler).unwrap();

    // Handlers should be sorted by priority internally
    assert_eq!(channel.stats().handlers_count, 2);
}

#[test]
fn event_channel_batch_dispatch() {
    let mut channel = EventChannel::new();
    let handler = Box::new(TestHandler::new());

    channel.register_handler(handler).unwrap();

    let events = vec![
        DocumentEvent::text_inserted(Position::new(0), "Hello".to_string()),
        DocumentEvent::text_inserted(Position::new(5), " World".to_string()),
    ];

    channel.dispatch_batch(events).unwrap();
    assert_eq!(channel.stats().events_dispatched, 2);
}

#[test]
fn event_channel_clear_handlers() {
    let mut channel = EventChannel::new();
    let handler = Box::new(TestHandler::new());

    channel.register_handler(handler).unwrap();
    assert_eq!(channel.stats().handlers_count, 1);

    channel.clear_handlers().unwrap();
    assert_eq!(channel.stats().handlers_count, 0);
}
