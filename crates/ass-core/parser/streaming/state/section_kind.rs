//! Section classification for the streaming parser state machine.
//!
//! Defines [`SectionKind`], identifying which ASS script section is currently
//! being parsed to enable context-aware processing.

/// Section types for state tracking
///
/// Identifies which ASS script section is currently being parsed
/// to enable context-aware processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    /// [Script Info] section with metadata
    ScriptInfo,
    /// [V4+ Styles] or [V4 Styles] section
    Styles,
    /// `[Events\]` section with dialogue/timing
    Events,
    /// `[Fonts\]` section with embedded fonts
    Fonts,
    /// `[Graphics\]` section with embedded images
    Graphics,
    /// Unknown or unsupported section
    Unknown,
}

impl SectionKind {
    /// Parse section kind from header text
    ///
    /// Returns appropriate `SectionKind` for known section headers,
    /// Unknown for unrecognized sections.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ass_core::parser::streaming::SectionKind;
    /// assert_eq!(SectionKind::from_header("Script Info"), SectionKind::ScriptInfo);
    /// assert_eq!(SectionKind::from_header("V4+ Styles"), SectionKind::Styles);
    /// assert_eq!(SectionKind::from_header("Unknown"), SectionKind::Unknown);
    /// ```
    #[must_use]
    pub fn from_header(header: &str) -> Self {
        match header.trim() {
            "Script Info" => Self::ScriptInfo,
            "V4+ Styles" | "V4 Styles" => Self::Styles,
            "Events" => Self::Events,
            "Fonts" => Self::Fonts,
            "Graphics" => Self::Graphics,
            _ => Self::Unknown,
        }
    }

    /// Check if section expects format line
    #[must_use]
    pub const fn expects_format(&self) -> bool {
        matches!(self, Self::Styles | Self::Events)
    }

    /// Check if section contains timed content
    #[must_use]
    pub const fn is_timed(&self) -> bool {
        matches!(self, Self::Events)
    }

    /// Check if section contains binary data
    #[must_use]
    pub const fn is_binary(&self) -> bool {
        matches!(self, Self::Fonts | Self::Graphics)
    }
}
