//! Event AST node for ASS dialogue and commands
//!
//! Contains the Event struct and `EventType` enum representing events from the
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
    #[must_use]
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
    #[must_use]
    pub const fn as_str(self) -> &'static str {
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

    /// Validate all spans in this Event reference valid source
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    /// Validates that all string references point to memory within
    /// the specified source range.
    ///
    /// Only available in debug builds to avoid performance overhead.
    #[cfg(debug_assertions)]
    #[must_use]
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

    #[test]
    fn event_time_parsing() {
        let event = Event {
            start: "0:01:30.50",
            end: "0:01:35.00",
            ..Event::default()
        };

        // Test start time parsing
        assert_eq!(event.start_time_cs().unwrap(), 9050); // 1*60*100 + 30*100 + 50

        // Test end time parsing
        assert_eq!(event.end_time_cs().unwrap(), 9500); // 1*60*100 + 35*100

        // Test duration calculation
        assert_eq!(event.duration_cs().unwrap(), 450); // 9500 - 9050
    }

    #[test]
    fn event_time_parsing_edge_cases() {
        // Test zero time
        let zero_event = Event {
            start: "0:00:00.00",
            end: "0:00:00.00",
            ..Event::default()
        };
        assert_eq!(zero_event.start_time_cs().unwrap(), 0);
        assert_eq!(zero_event.end_time_cs().unwrap(), 0);
        assert_eq!(zero_event.duration_cs().unwrap(), 0);

        // Test negative duration (end before start)
        let negative_event = Event {
            start: "0:01:00.00",
            end: "0:00:30.00",
            ..Event::default()
        };
        assert_eq!(negative_event.duration_cs().unwrap(), 0); // saturating_sub returns 0
    }

    #[test]
    fn event_time_parsing_errors() {
        // Test invalid time formats
        let invalid_start = Event {
            start: "invalid",
            end: "0:00:05.00",
            ..Event::default()
        };
        assert!(invalid_start.start_time_cs().is_err());
        assert!(invalid_start.duration_cs().is_err());

        let invalid_end = Event {
            start: "0:00:00.00",
            end: "invalid",
            ..Event::default()
        };
        assert!(invalid_end.end_time_cs().is_err());
        assert!(invalid_end.duration_cs().is_err());
    }

    #[test]
    fn event_all_types() {
        // Test all event types
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

        let picture = Event {
            event_type: EventType::Picture,
            ..Event::default()
        };
        assert!(!picture.is_dialogue());
        assert!(!picture.is_comment());

        let sound = Event {
            event_type: EventType::Sound,
            ..Event::default()
        };
        assert!(!sound.is_dialogue());
        assert!(!sound.is_comment());

        let movie = Event {
            event_type: EventType::Movie,
            ..Event::default()
        };
        assert!(!movie.is_dialogue());
        assert!(!movie.is_comment());

        let command = Event {
            event_type: EventType::Command,
            ..Event::default()
        };
        assert!(!command.is_dialogue());
        assert!(!command.is_comment());
    }

    #[test]
    fn event_comprehensive_creation() {
        let event = Event {
            event_type: EventType::Dialogue,
            layer: "5",
            start: "0:02:15.75",
            end: "0:02:20.25",
            style: "MainStyle",
            name: "Character",
            margin_l: "10",
            margin_r: "20",
            margin_v: "15",
            effect: "fadeIn",
            text: "Hello, world!",
        };

        assert_eq!(event.event_type, EventType::Dialogue);
        assert_eq!(event.layer, "5");
        assert_eq!(event.start, "0:02:15.75");
        assert_eq!(event.end, "0:02:20.25");
        assert_eq!(event.style, "MainStyle");
        assert_eq!(event.name, "Character");
        assert_eq!(event.margin_l, "10");
        assert_eq!(event.margin_r, "20");
        assert_eq!(event.margin_v, "15");
        assert_eq!(event.effect, "fadeIn");
        assert_eq!(event.text, "Hello, world!");
    }

    #[test]
    fn event_debug_output() {
        let event = Event {
            event_type: EventType::Dialogue,
            text: "Test text",
            ..Event::default()
        };

        let debug_str = format!("{event:?}");
        assert!(debug_str.contains("Event"));
        assert!(debug_str.contains("Dialogue"));
        assert!(debug_str.contains("Test text"));
    }

    #[test]
    fn event_equality() {
        let event1 = Event {
            event_type: EventType::Dialogue,
            text: "Same text",
            ..Event::default()
        };

        let event2 = Event {
            event_type: EventType::Dialogue,
            text: "Same text",
            ..Event::default()
        };

        assert_eq!(event1, event2);

        let event3 = Event {
            event_type: EventType::Comment,
            text: "Same text",
            ..Event::default()
        };

        assert_ne!(event1, event3);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn event_validate_spans() {
        let source = "Dialogue,0,0:00:05.00,0:00:10.00,Default,Character,0,0,0,,Hello world";
        let source_start = source.as_ptr() as usize;
        let source_end = source_start + source.len();
        let source_range = source_start..source_end;

        let fields: Vec<&str> = source.split(',').collect();
        let event = Event {
            event_type: EventType::Dialogue,
            layer: fields[1],
            start: fields[2],
            end: fields[3],
            style: fields[4],
            name: fields[5],
            margin_l: fields[6],
            margin_r: fields[7],
            margin_v: fields[8],
            effect: fields[9],
            text: fields[10],
        };

        assert!(event.validate_spans(&source_range));
        assert_eq!(event.layer, "0");
        assert_eq!(event.start, "0:00:05.00");
        assert_eq!(event.end, "0:00:10.00");
        assert_eq!(event.style, "Default");
        assert_eq!(event.name, "Character");
        assert_eq!(event.text, "Hello world");
    }

    #[cfg(debug_assertions)]
    #[test]
    fn event_validate_spans_invalid() {
        let source1 = "Dialogue,0,0:00:05.00,0:00:10.00,Default";
        let source2 = "Other,Character,Hello";
        let source1_start = source1.as_ptr() as usize;
        let source1_end = source1_start + source1.len();
        let source1_range = source1_start..source1_end;

        let event = Event {
            event_type: EventType::Dialogue,
            layer: "0",
            start: "0:00:05.00",
            end: "0:00:10.00",
            style: "Default",
            name: &source2[6..15],  // "Character" from different source
            text: &source2[16..21], // "Hello" from different source
            ..Event::default()
        };

        // Should fail since some fields are from different source
        assert!(!event.validate_spans(&source1_range));
    }

    #[test]
    fn event_type_parse_edge_cases() {
        // Test case sensitivity
        assert_eq!(EventType::parse_type("dialogue"), None);
        assert_eq!(EventType::parse_type("DIALOGUE"), None);

        // Test empty and whitespace
        assert_eq!(EventType::parse_type(""), None);
        assert_eq!(EventType::parse_type("   "), None);

        // Test with extra whitespace
        assert_eq!(
            EventType::parse_type("  Comment  "),
            Some(EventType::Comment)
        );
        assert_eq!(
            EventType::parse_type("\tPicture\n"),
            Some(EventType::Picture)
        );
    }

    #[test]
    fn event_mixed_defaults() {
        let event = Event {
            event_type: EventType::Picture,
            start: "0:01:00.00",
            text: "Custom text",
            ..Event::default()
        };

        // Custom fields
        assert_eq!(event.event_type, EventType::Picture);
        assert_eq!(event.start, "0:01:00.00");
        assert_eq!(event.text, "Custom text");

        // Default fields
        assert_eq!(event.layer, "0");
        assert_eq!(event.end, "0:00:00.00");
        assert_eq!(event.style, "Default");
        assert_eq!(event.name, "");
        assert_eq!(event.effect, "");
    }
}
