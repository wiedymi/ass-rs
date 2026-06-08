//! Selective event filtering via [`EventFilter`].
//!
//! Builder-style filter that includes/excludes events by type name and by
//! modification or cursor/selection category for targeted handler delivery.

use super::DocumentEvent;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

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
