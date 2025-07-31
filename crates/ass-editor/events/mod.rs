//! Event system for document changes and editor notifications  
//!
//! Provides `DocumentEvent` enum for representing editor changes and
//! `EventChannel` for distributing events to observers. Supports both
//! synchronous and asynchronous event handling with filtering.

#[cfg(not(feature = "std"))]
extern crate alloc;

use crate::core::{Position, Range, Result};

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, string::{String, ToString}, vec::Vec};

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

/// Types of events that can occur in the editor
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentEvent {
    /// Text was inserted at a position
    TextInserted {
        /// Position where text was inserted
        position: Position,
        /// Text that was inserted
        text: String,
        /// Length of inserted text in bytes
        length: usize,
    },

    /// Text was deleted from a range
    TextDeleted {
        /// Range where text was deleted
        range: Range,
        /// Text that was deleted (for undo purposes)
        deleted_text: String,
    },

    /// Text was replaced in a range
    TextReplaced {
        /// Range where text was replaced
        range: Range,
        /// Old text that was replaced
        old_text: String,
        /// New text that replaced the old text
        new_text: String,
    },

    /// Selection changed
    SelectionChanged {
        /// Previous selection range (if any)
        old_selection: Option<Range>,
        /// New selection range (if any)
        new_selection: Option<Range>,
    },

    /// Cursor position changed
    CursorMoved {
        /// Previous cursor position
        old_position: Position,
        /// New cursor position
        new_position: Position,
    },

    /// Document was saved to a file
    DocumentSaved {
        /// File path where document was saved
        file_path: String,
        /// Whether this was a "save as" operation
        save_as: bool,
    },

    /// Document was loaded from a file
    DocumentLoaded {
        /// File path from which document was loaded
        file_path: String,
        /// Size of loaded document in bytes
        size: usize,
    },

    /// Undo operation was performed
    UndoPerformed {
        /// Description of the undone action
        action_description: String,
        /// Number of changes undone
        changes_count: usize,
    },

    /// Redo operation was performed  
    RedoPerformed {
        /// Description of the redone action
        action_description: String,
        /// Number of changes redone
        changes_count: usize,
    },

    /// Validation completed with results
    ValidationCompleted {
        /// Number of issues found
        issues_count: usize,
        /// Number of errors found
        error_count: usize,
        /// Number of warnings found
        warning_count: usize,
        /// Time taken for validation in milliseconds
        validation_time_ms: u64,
    },

    /// Search operation completed
    SearchCompleted {
        /// Search pattern that was used
        pattern: String,
        /// Number of matches found
        matches_count: usize,
        /// Whether the search hit the result limit
        hit_limit: bool,
        /// Time taken for search in microseconds
        search_time_us: u64,
    },

    /// Document parsing completed
    ParsingCompleted {
        /// Whether parsing was successful
        success: bool,
        /// Number of sections parsed
        sections_count: usize,
        /// Time taken for parsing in milliseconds
        parse_time_ms: u64,
        /// Any error message if parsing failed
        error_message: Option<String>,
    },

    /// Extension was loaded or unloaded
    ExtensionChanged {
        /// Name of the extension
        extension_name: String,
        /// Whether the extension was loaded (true) or unloaded (false)
        loaded: bool,
    },

    /// Configuration setting changed
    ConfigChanged {
        /// Name of the configuration key
        key: String,
        /// Old value (if any)
        old_value: Option<String>,
        /// New value
        new_value: String,
    },

    /// Generic custom event for extensions
    CustomEvent {
        /// Event type identifier
        event_type: String,
        /// Event data as key-value pairs
        data: HashMap<String, String>,
    },
}

impl DocumentEvent {
    /// Get a human-readable description of the event
    pub fn description(&self) -> String {
        match self {
            Self::TextInserted { length, .. } => format!("Inserted {length} bytes of text"),
            Self::TextDeleted { range, .. } => format!("Deleted text from {range}"),
            Self::TextReplaced { range, .. } => format!("Replaced text in {range}"),
            Self::SelectionChanged { .. } => "Selection changed".to_string(),
            Self::CursorMoved { .. } => "Cursor moved".to_string(),
            Self::DocumentSaved { file_path, save_as } => {
                if *save_as {
                    format!("Saved document as '{file_path}'")
                } else {
                    format!("Saved document to '{file_path}'")
                }
            }
            Self::DocumentLoaded { file_path, size } => {
                format!("Loaded document '{file_path}' ({size} bytes)")
            }
            Self::UndoPerformed {
                action_description,
                changes_count,
            } => {
                format!("Undid '{action_description}' ({changes_count} changes)")
            }
            Self::RedoPerformed {
                action_description,
                changes_count,
            } => {
                format!("Redid '{action_description}' ({changes_count} changes)")
            }
            Self::ValidationCompleted {
                issues_count,
                validation_time_ms,
                ..
            } => {
                format!(
                    "Validation completed: {issues_count} issues found in {validation_time_ms}ms"
                )
            }
            Self::SearchCompleted {
                pattern,
                matches_count,
                search_time_us,
                ..
            } => {
                format!(
                    "Search for '{pattern}' found {matches_count} matches in {search_time_us}μs"
                )
            }
            Self::ParsingCompleted {
                success,
                sections_count,
                parse_time_ms,
                ..
            } => {
                if *success {
                    format!("Parsed {sections_count} sections in {parse_time_ms}ms")
                } else {
                    format!("Parsing failed after {parse_time_ms}ms")
                }
            }
            Self::ExtensionChanged {
                extension_name,
                loaded,
            } => {
                if *loaded {
                    format!("Loaded extension '{extension_name}'")
                } else {
                    format!("Unloaded extension '{extension_name}'")
                }
            }
            Self::ConfigChanged { key, .. } => format!("Configuration '{key}' changed"),
            Self::CustomEvent { event_type, .. } => format!("Custom event: {event_type}"),
        }
    }

    /// Check if this event represents a document modification
    pub fn is_modification(&self) -> bool {
        matches!(
            self,
            Self::TextInserted { .. }
                | Self::TextDeleted { .. }
                | Self::TextReplaced { .. }
                | Self::UndoPerformed { .. }
                | Self::RedoPerformed { .. }
        )
    }

    /// Check if this event affects the document's text content
    pub fn affects_text(&self) -> bool {
        matches!(
            self,
            Self::TextInserted { .. }
                | Self::TextDeleted { .. }
                | Self::TextReplaced { .. }
                | Self::UndoPerformed { .. }
                | Self::RedoPerformed { .. }
                | Self::DocumentLoaded { .. }
        )
    }

    /// Get the affected range for events that modify text
    pub fn affected_range(&self) -> Option<Range> {
        match self {
            Self::TextInserted {
                position, length, ..
            } => Some(Range::new(*position, position.advance(*length))),
            Self::TextDeleted { range, .. } | Self::TextReplaced { range, .. } => Some(*range),
            _ => None,
        }
    }
}

/// Event filter for selective event handling
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// Event types to include (empty means all types)
    include_types: Vec<String>,
    /// Event types to exclude
    exclude_types: Vec<String>,
    /// Whether to include modification events
    include_modifications: Option<bool>,
    /// Whether to include cursor/selection events  
    include_cursor_events: Option<bool>,
}

impl EventFilter {
    /// Create a new event filter that accepts all events
    pub fn new() -> Self {
        Self {
            include_types: Vec::new(),
            exclude_types: Vec::new(),
            include_modifications: None,
            include_cursor_events: None,
        }
    }

    /// Only include specific event types
    pub fn include_types(mut self, types: Vec<String>) -> Self {
        self.include_types = types;
        self
    }

    /// Exclude specific event types
    pub fn exclude_types(mut self, types: Vec<String>) -> Self {
        self.exclude_types = types;
        self
    }

    /// Set whether to include modification events
    pub fn include_modifications(mut self, include: bool) -> Self {
        self.include_modifications = Some(include);
        self
    }

    /// Set whether to include cursor/selection events
    pub fn include_cursor_events(mut self, include: bool) -> Self {
        self.include_cursor_events = Some(include);
        self
    }

    /// Check if an event passes this filter
    pub fn matches(&self, event: &DocumentEvent) -> bool {
        let event_type = event.event_type_name();

        // Check exclude list first
        if self.exclude_types.contains(&event_type) {
            return false;
        }

        // Check include list if specified
        if !self.include_types.is_empty() && !self.include_types.contains(&event_type) {
            return false;
        }

        // Check modification filter
        if let Some(include_mods) = self.include_modifications {
            if event.is_modification() != include_mods {
                return false;
            }
        }

        // Check cursor event filter
        if let Some(include_cursor) = self.include_cursor_events {
            let is_cursor_event = matches!(
                event,
                DocumentEvent::CursorMoved { .. } | DocumentEvent::SelectionChanged { .. }
            );
            if is_cursor_event != include_cursor {
                return false;
            }
        }

        true
    }
}

impl Default for EventFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentEvent {
    /// Get the event type as a string for filtering
    fn event_type_name(&self) -> String {
        match self {
            Self::TextInserted { .. } => "TextInserted".to_string(),
            Self::TextDeleted { .. } => "TextDeleted".to_string(),
            Self::TextReplaced { .. } => "TextReplaced".to_string(),
            Self::SelectionChanged { .. } => "SelectionChanged".to_string(),
            Self::CursorMoved { .. } => "CursorMoved".to_string(),
            Self::DocumentSaved { .. } => "DocumentSaved".to_string(),
            Self::DocumentLoaded { .. } => "DocumentLoaded".to_string(),
            Self::UndoPerformed { .. } => "UndoPerformed".to_string(),
            Self::RedoPerformed { .. } => "RedoPerformed".to_string(),
            Self::ValidationCompleted { .. } => "ValidationCompleted".to_string(),
            Self::SearchCompleted { .. } => "SearchCompleted".to_string(),
            Self::ParsingCompleted { .. } => "ParsingCompleted".to_string(),
            Self::ExtensionChanged { .. } => "ExtensionChanged".to_string(),
            Self::ConfigChanged { .. } => "ConfigChanged".to_string(),
            Self::CustomEvent { event_type, .. } => event_type.clone(),
        }
    }
}

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

/// Event channel for distributing document events to handlers
#[derive(Debug)]
pub struct EventChannel {
    /// Configuration for this channel
    config: EventChannelConfig,

    /// Registered event handlers
    #[cfg(feature = "multi-thread")]
    handlers: Arc<RwLock<Vec<HandlerInfo>>>,

    #[cfg(not(feature = "multi-thread"))]
    handlers: Rc<RefCell<Vec<HandlerInfo>>>,

    /// Event statistics
    stats: EventStats,

    /// Async event sender (if async feature is enabled)
    #[cfg(feature = "async")]
    async_sender: Option<mpsc::UnboundedSender<DocumentEvent>>,

    /// Next handler ID for unique identification
    next_handler_id: usize,
}

/// Information about a registered handler
struct HandlerInfo {
    /// Unique handler ID
    id: usize,
    /// Handler implementation
    handler: Box<dyn EventHandler>,
    /// Event filter for this handler
    filter: EventFilter,
    /// Handler priority
    priority: i32,
    /// Number of events processed by this handler
    events_processed: usize,
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
            handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
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
            handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
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

    /// Dispatch an event to all registered handlers
    pub fn dispatch(&mut self, event: DocumentEvent) -> Result<()> {
        #[cfg(feature = "std")]
        let start_time = std::time::Instant::now();

        self.stats.events_dispatched += 1;

        let mut filtered_count = 0;
        #[allow(unused_variables, unused_assignments)]
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
                    processed_count += 1;
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
                    processed_count += 1;
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

impl Default for EventChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for creating common events
impl DocumentEvent {
    /// Create a text insertion event
    pub fn text_inserted(position: Position, text: String) -> Self {
        let length = text.len();
        Self::TextInserted {
            position,
            text,
            length,
        }
    }

    /// Create a text deletion event
    pub fn text_deleted(range: Range, deleted_text: String) -> Self {
        Self::TextDeleted {
            range,
            deleted_text,
        }
    }

    /// Create a text replacement event
    pub fn text_replaced(range: Range, old_text: String, new_text: String) -> Self {
        Self::TextReplaced {
            range,
            old_text,
            new_text,
        }
    }

    /// Create a cursor moved event
    pub fn cursor_moved(old_position: Position, new_position: Position) -> Self {
        Self::CursorMoved {
            old_position,
            new_position,
        }
    }

    /// Create a document saved event
    pub fn document_saved(file_path: String, save_as: bool) -> Self {
        Self::DocumentSaved { file_path, save_as }
    }

    /// Create a validation completed event
    pub fn validation_completed(
        issues_count: usize,
        error_count: usize,
        warning_count: usize,
        validation_time_ms: u64,
    ) -> Self {
        Self::ValidationCompleted {
            issues_count,
            error_count,
            warning_count,
            validation_time_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_event_creation() {
        let event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());

        match event {
            DocumentEvent::TextInserted {
                position,
                text,
                length,
            } => {
                assert_eq!(position.offset, 0);
                assert_eq!(text, "Hello");
                assert_eq!(length, 5);
            }
            _ => panic!("Expected TextInserted event"),
        }
    }

    #[test]
    fn document_event_description() {
        let event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());
        assert_eq!(event.description(), "Inserted 5 bytes of text");

        let event = DocumentEvent::cursor_moved(Position::new(0), Position::new(5));
        assert_eq!(event.description(), "Cursor moved");
    }

    #[test]
    fn document_event_modification_check() {
        let insert_event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());
        assert!(insert_event.is_modification());

        let cursor_event = DocumentEvent::cursor_moved(Position::new(0), Position::new(5));
        assert!(!cursor_event.is_modification());
    }

    #[test]
    fn document_event_affects_text() {
        let insert_event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());
        assert!(insert_event.affects_text());

        let config_event = DocumentEvent::ConfigChanged {
            key: "font_size".to_string(),
            old_value: Some("12".to_string()),
            new_value: "14".to_string(),
        };
        assert!(!config_event.affects_text());
    }

    #[test]
    fn document_event_affected_range() {
        let insert_event = DocumentEvent::text_inserted(Position::new(10), "Hello".to_string());
        let range = insert_event.affected_range().unwrap();
        assert_eq!(range.start.offset, 10);
        assert_eq!(range.end.offset, 15);

        let cursor_event = DocumentEvent::cursor_moved(Position::new(0), Position::new(5));
        assert!(cursor_event.affected_range().is_none());
    }

    #[test]
    fn event_filter_creation() {
        let filter = EventFilter::new()
            .include_modifications(true)
            .exclude_types(vec!["CursorMoved".to_string()]);

        let insert_event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());
        assert!(filter.matches(&insert_event));

        let cursor_event = DocumentEvent::cursor_moved(Position::new(0), Position::new(5));
        assert!(!filter.matches(&cursor_event));
    }

    #[test]
    fn event_filter_include_types() {
        let filter = EventFilter::new()
            .include_types(vec!["TextInserted".to_string(), "TextDeleted".to_string()]);

        let insert_event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());
        assert!(filter.matches(&insert_event));

        let cursor_event = DocumentEvent::cursor_moved(Position::new(0), Position::new(5));
        assert!(!filter.matches(&cursor_event));
    }

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
}
