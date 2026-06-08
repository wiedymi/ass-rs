//! `EventType` discriminant for ASS timeline events
//!
//! Defines the [`EventType`] enum distinguishing dialogue, comments, and the
//! media/command event kinds, with parsing from and formatting to their
//! canonical ASS names.

/// Event type discriminant for different kinds of timeline events
///
/// Determines how the event is processed during subtitle rendering.
/// Different types have different behaviors during playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
