//! Event query and filtering result types for the fluent API.
//!
//! Defines [`EventInfo`], [`OwnedEvent`], [`EventFilter`], [`EventSortCriteria`],
//! and [`EventSortOptions`] used by the event query builder.

use crate::core::Range;
use ass_core::parser::ast::{Event, EventType};

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// Event information with index for filtering/sorting results
#[derive(Debug, Clone, PartialEq)]
pub struct EventInfo {
    /// Zero-based index in the document
    pub index: usize,
    /// Owned copy of the event data
    pub event: OwnedEvent,
    /// Line number in the document (1-based)
    pub line_number: usize,
    /// Character position range in document
    pub range: Range,
}

/// Owned version of Event for use in EventInfo
#[derive(Debug, Clone, PartialEq)]
pub struct OwnedEvent {
    /// Event type (Dialogue, Comment, etc.)
    pub event_type: EventType,
    /// Layer for drawing order (higher layers drawn on top)
    pub layer: String,
    /// Start time in ASS time format (H:MM:SS.CS)
    pub start: String,
    /// End time in ASS time format (H:MM:SS.CS)
    pub end: String,
    /// Style name reference
    pub style: String,
    /// Character name or speaker
    pub name: String,
    /// Left margin override (pixels)
    pub margin_l: String,
    /// Right margin override (pixels)
    pub margin_r: String,
    /// Vertical margin override (pixels) (V4+)
    pub margin_v: String,
    /// Top margin override (pixels) (V4++) - optional
    pub margin_t: Option<String>,
    /// Bottom margin override (pixels) (V4++) - optional
    pub margin_b: Option<String>,
    /// Effect specification for special rendering
    pub effect: String,
    /// Text content with possible style overrides
    pub text: String,
}

impl<'a> From<&Event<'a>> for OwnedEvent {
    fn from(event: &Event<'a>) -> Self {
        Self {
            event_type: event.event_type,
            layer: event.layer.to_string(),
            start: event.start.to_string(),
            end: event.end.to_string(),
            style: event.style.to_string(),
            name: event.name.to_string(),
            margin_l: event.margin_l.to_string(),
            margin_r: event.margin_r.to_string(),
            margin_v: event.margin_v.to_string(),
            margin_t: event.margin_t.map(|s| s.to_string()),
            margin_b: event.margin_b.map(|s| s.to_string()),
            effect: event.effect.to_string(),
            text: event.text.to_string(),
        }
    }
}

/// Filter criteria for events
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Filter by event type (Dialogue, Comment)
    pub event_type: Option<EventType>,
    /// Filter by style name pattern
    pub style_pattern: Option<String>,
    /// Filter by speaker/actor name pattern
    pub speaker_pattern: Option<String>,
    /// Filter by text content pattern
    pub text_pattern: Option<String>,
    /// Filter by time range (start_cs, end_cs)
    pub time_range: Option<(u32, u32)>,
    /// Filter by layer
    pub layer: Option<u32>,
    /// Filter by effect presence/pattern
    pub effect_pattern: Option<String>,
    /// Use regex for pattern matching
    pub use_regex: bool,
    /// Case sensitive matching
    pub case_sensitive: bool,
}

/// Sort criteria for events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventSortCriteria {
    /// Sort by start time (default)
    StartTime,
    /// Sort by end time
    EndTime,
    /// Sort by duration (end - start)
    Duration,
    /// Sort by style name
    Style,
    /// Sort by speaker/actor name
    Speaker,
    /// Sort by layer
    Layer,
    /// Sort by document order (original index)
    Index,
    /// Sort by text content (alphabetical)
    Text,
}

/// Sort options
#[derive(Debug, Clone)]
pub struct EventSortOptions {
    /// Primary sort criteria
    pub criteria: EventSortCriteria,
    /// Secondary sort criteria (for ties)
    pub secondary: Option<EventSortCriteria>,
    /// Sort in ascending order (default true)
    pub ascending: bool,
}

impl Default for EventSortOptions {
    fn default() -> Self {
        Self {
            criteria: EventSortCriteria::Index,
            secondary: None,
            ascending: true,
        }
    }
}
