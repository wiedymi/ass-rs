//! Event AST node for ASS dialogue and commands
//!
//! Contains the Event struct and EventType enum representing events from the
//! [Events] section with zero-copy design and time parsing utilities.

#[cfg(debug_assertions)]
use core::ops::Range;

/// Event from [Events] section (dialogue, comments, etc.)
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
#[derive(Debug, Clone, PartialEq)]
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

    /// Vertical margin override (pixels)
    pub margin_v: &'a str,

    /// Effect specification for special rendering
    pub effect: &'a str,

    /// Text content with possible style overrides
    pub text: &'a str,
}

/// Event type discriminant for different kinds of timeline events
///
/// Determines how the event is processed during subtitle rendering.
/// Different types have different behaviors during playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Dialogue line (displayed during playback)
    Dialogue,

    /// Comment (ignored during playback)
    Comment,

    /// Picture display event
    Picture,

    /// Sound playback event
    Sound,

    /// Movie playback event
    Movie,

    /// Command execution event
    Command,
}

impl EventType {
    /// Parse event type from string
    ///
    /// Converts ASS event type names to the corresponding enum variant.
    /// Returns `None` for unrecognized event types.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::EventType;
    /// assert_eq!(EventType::parse_type("Dialogue"), Some(EventType::Dialogue));
    /// assert_eq!(EventType::parse_type("Comment"), Some(EventType::Comment));
    /// assert_eq!(EventType::parse_type("Unknown"), None);
    /// ```
    pub fn parse_type(s: &str) -> Option<Self> {
        match s.trim() {
            "Dialogue" => Some(Self::Dialogue),
            "Comment" => Some(Self::Comment),
            "Picture" => Some(Self::Picture),
            "Sound" => Some(Self::Sound),
            "Movie" => Some(Self::Movie),
            "Command" => Some(Self::Command),
            _ => None,
        }
    }

    /// Get string representation for serialization
    ///
    /// Returns the canonical ASS event type name for this variant.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dialogue => "Dialogue",
            Self::Comment => "Comment",
            Self::Picture => "Picture",
            Self::Sound => "Sound",
            Self::Movie => "Movie",
            Self::Command => "Command",
        }
    }
}

impl Event<'_> {
    /// Check if this is a dialogue event
    ///
    /// Returns `true` for events that should be displayed during playback.
    pub fn is_dialogue(&self) -> bool {
        matches!(self.event_type, EventType::Dialogue)
    }

    /// Check if this is a comment
    ///
    /// Returns `true` for events that are ignored during playback.
    pub fn is_comment(&self) -> bool {
        matches!(self.event_type, EventType::Comment)
    }

    /// Parse start time to centiseconds
    ///
    /// Converts the start time string to centiseconds for timing calculations.
    /// Uses the standard ASS time format parser.
    pub fn start_time_cs(&self) -> Result<u32, crate::utils::CoreError> {
        crate::utils::parse_ass_time(self.start)
    }

    /// Parse end time to centiseconds
    ///
    /// Converts the end time string to centiseconds for timing calculations.
    /// Uses the standard ASS time format parser.
    pub fn end_time_cs(&self) -> Result<u32, crate::utils::CoreError> {
        crate::utils::parse_ass_time(self.end)
    }

    /// Get duration in centiseconds
    ///
    /// Calculates the event duration by subtracting start time from end time.
    /// Returns 0 if start time is greater than end time.
    pub fn duration_cs(&self) -> Result<u32, crate::utils::CoreError> {
        let start = self.start_time_cs()?;
        let end = self.end_time_cs()?;
        Ok(end.saturating_sub(start))
    }

    /// Validate all spans in this Event reference valid source
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    /// Validates that all string references point to memory within
    /// the specified source range.
    ///
    /// Only available in debug builds to avoid performance overhead.
    #[cfg(debug_assertions)]
    pub fn validate_spans(&self, source_range: &Range<usize>) -> bool {
        let spans = [
            self.layer,
            self.start,
            self.end,
            self.style,
            self.name,
            self.margin_l,
            self.margin_r,
            self.margin_v,
            self.effect,
            self.text,
        ];

        spans.iter().all(|span| {
            let ptr = span.as_ptr() as usize;
            source_range.contains(&ptr)
        })
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
            effect: "",
            text: "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_parsing() {
        assert_eq!(EventType::parse_type("Dialogue"), Some(EventType::Dialogue));
        assert_eq!(EventType::parse_type("Comment"), Some(EventType::Comment));
        assert_eq!(EventType::parse_type("Picture"), Some(EventType::Picture));
        assert_eq!(EventType::parse_type("Sound"), Some(EventType::Sound));
        assert_eq!(EventType::parse_type("Movie"), Some(EventType::Movie));
        assert_eq!(EventType::parse_type("Command"), Some(EventType::Command));
        assert_eq!(EventType::parse_type("Unknown"), None);
        assert_eq!(
            EventType::parse_type("  Dialogue  "),
            Some(EventType::Dialogue)
        );
    }

    #[test]
    fn event_type_string_conversion() {
        assert_eq!(EventType::Dialogue.as_str(), "Dialogue");
        assert_eq!(EventType::Comment.as_str(), "Comment");
        assert_eq!(EventType::Picture.as_str(), "Picture");
        assert_eq!(EventType::Sound.as_str(), "Sound");
        assert_eq!(EventType::Movie.as_str(), "Movie");
        assert_eq!(EventType::Command.as_str(), "Command");
    }

    #[test]
    fn event_type_properties() {
        assert_eq!(EventType::Dialogue, EventType::Dialogue);
        assert_ne!(EventType::Dialogue, EventType::Comment);
    }

    #[test]
    fn event_dialogue_check() {
        let dialogue = Event {
            event_type: EventType::Dialogue,
            ..Event::default()
        };
        assert!(dialogue.is_dialogue());
        assert!(!dialogue.is_comment());

        let comment = Event {
            event_type: EventType::Comment,
            ..Event::default()
        };
        assert!(!comment.is_dialogue());
        assert!(comment.is_comment());
    }

    #[test]
    fn event_default() {
        let event = Event::default();
        assert_eq!(event.event_type, EventType::Dialogue);
        assert_eq!(event.layer, "0");
        assert_eq!(event.start, "0:00:00.00");
        assert_eq!(event.end, "0:00:00.00");
        assert_eq!(event.style, "Default");
        assert_eq!(event.text, "");
    }

    #[test]
    fn event_clone_eq() {
        let event = Event::default();
        let cloned = event.clone();
        assert_eq!(event, cloned);
    }
}
