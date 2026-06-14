//! `Event` struct with construction defaults and timing accessors
//!
//! Defines the zero-copy [`Event`] node for the `[Events]` section together
//! with its dialogue/comment predicates, centisecond time helpers, and the
//! [`Default`] template implementation.

use super::{EventType, Span};

/// Event from `[Events\]` section (dialogue, comments, etc.)
///
/// Represents a single event in the subtitle timeline. Events can be dialogue
/// lines, comments, or other commands with associated timing and styling.
/// All fields use zero-copy string references for maximum efficiency.
///
/// # Examples
///
/// ```rust
/// use ass_core::parser::ast::{Event, EventType};
///
/// let event = Event {
///     event_type: EventType::Dialogue,
///     start: "0:00:05.00",
///     end: "0:00:10.00",
///     text: "Hello, world!",
///     ..Event::default()
/// };
///
/// assert!(event.is_dialogue());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Event<'a> {
    /// Event type (Dialogue, Comment, etc.)
    pub event_type: EventType,

    /// Layer for drawing order (higher layers drawn on top)
    pub layer: &'a str,

    /// Start time in ASS time format (H:MM:SS.CS)
    pub start: &'a str,

    /// End time in ASS time format (H:MM:SS.CS)
    pub end: &'a str,

    /// Style name reference
    pub style: &'a str,

    /// Character name or speaker
    pub name: &'a str,

    /// Left margin override (pixels)
    pub margin_l: &'a str,

    /// Right margin override (pixels)
    pub margin_r: &'a str,

    /// Vertical margin override (pixels) (V4+)
    pub margin_v: &'a str,

    /// Top margin override (pixels) (V4++)
    pub margin_t: Option<&'a str>,

    /// Bottom margin override (pixels) (V4++)
    pub margin_b: Option<&'a str>,

    /// Effect specification for special rendering
    pub effect: &'a str,

    /// Text content with possible style overrides
    pub text: &'a str,

    /// Span in source text where this event is defined
    pub span: Span,
}

impl Event<'_> {
    /// Check if this is a dialogue event
    ///
    /// Returns `true` for events that should be displayed during playback.
    #[must_use]
    pub const fn is_dialogue(&self) -> bool {
        matches!(self.event_type, EventType::Dialogue)
    }

    /// Check if this is a comment event
    ///
    /// Returns `true` for events that are comments and not displayed.
    #[must_use]
    pub const fn is_comment(&self) -> bool {
        matches!(self.event_type, EventType::Comment)
    }

    /// Parse start time to centiseconds
    ///
    /// Converts the start time string to centiseconds for timing calculations.
    /// Uses the standard ASS time format parser.
    ///
    /// # Errors
    ///
    /// Returns an error if the time format is invalid or cannot be parsed.
    pub fn start_time_cs(&self) -> Result<u32, crate::utils::CoreError> {
        crate::utils::parse_ass_time(self.start)
    }

    /// Parse end time to centiseconds
    ///
    /// Converts the end time string to centiseconds for timing calculations.
    /// Uses the standard ASS time format parser.
    ///
    /// # Errors
    ///
    /// Returns an error if the time format is invalid or cannot be parsed.
    pub fn end_time_cs(&self) -> Result<u32, crate::utils::CoreError> {
        crate::utils::parse_ass_time(self.end)
    }

    /// Get duration in centiseconds
    ///
    /// Calculates the event duration by subtracting start time from end time.
    /// Returns 0 if start time is greater than end time.
    ///
    /// # Errors
    ///
    /// Returns an error if either start or end time format is invalid.
    pub fn duration_cs(&self) -> Result<u32, crate::utils::CoreError> {
        let start = self.start_time_cs()?;
        let end = self.end_time_cs()?;
        Ok(end.saturating_sub(start))
    }
}

impl Default for Event<'_> {
    /// Create default event with safe initial values
    ///
    /// Provides a valid default event that can be used as a template
    /// or for testing purposes.
    fn default() -> Self {
        Self {
            event_type: EventType::Dialogue,
            layer: "0",
            start: "0:00:00.00",
            end: "0:00:00.00",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            text: "",
            span: Span::new(0, 0, 0, 0),
        }
    }
}
