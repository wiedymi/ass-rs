//! State management for streaming ASS parser
//!
//! Provides state machine components for incremental parsing with
//! proper section tracking and context management.

use alloc::string::String;

/// Streaming parser state for incremental processing
///
/// Tracks current parsing context to handle partial data and
/// section boundaries correctly during streaming.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserState {
    /// Expecting section header or document start
    ExpectingSection,
    /// Currently parsing a specific section
    InSection(SectionKind),
    /// Parsing an event with potentially incomplete data
    InEvent {
        /// Which section type we're in
        section: SectionKind,
        /// Number of fields processed so far
        fields_seen: usize,
    },
}

impl ParserState {
    /// Check if currently inside a section
    pub fn is_in_section(&self) -> bool {
        matches!(self, Self::InSection(_) | Self::InEvent { .. })
    }

    /// Get current section kind if in a section
    pub fn current_section(&self) -> Option<SectionKind> {
        match self {
            Self::ExpectingSection => None,
            Self::InSection(kind) => Some(*kind),
            Self::InEvent { section, .. } => Some(*section),
        }
    }

    /// Transition to new section
    pub fn enter_section(&mut self, kind: SectionKind) {
        *self = Self::InSection(kind);
    }

    /// Begin event parsing within section
    pub fn enter_event(&mut self, section: SectionKind) {
        *self = Self::InEvent {
            section,
            fields_seen: 0,
        };
    }

    /// Exit current section
    pub fn exit_section(&mut self) {
        *self = Self::ExpectingSection;
    }
}

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
    /// [Events] section with dialogue/timing
    Events,
    /// [Fonts] section with embedded fonts
    Fonts,
    /// [Graphics] section with embedded images
    Graphics,
    /// Unknown or unsupported section
    Unknown,
}

impl SectionKind {
    /// Parse section kind from header text
    ///
    /// Returns appropriate SectionKind for known section headers,
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
    pub fn expects_format(&self) -> bool {
        matches!(self, Self::Styles | Self::Events)
    }

    /// Check if section contains timed content
    pub fn is_timed(&self) -> bool {
        matches!(self, Self::Events)
    }

    /// Check if section contains binary data
    pub fn is_binary(&self) -> bool {
        matches!(self, Self::Fonts | Self::Graphics)
    }
}

/// Context for streaming parser state
///
/// Maintains parsing context including line tracking, current section,
/// and format information for proper incremental processing.
#[derive(Debug, Clone)]
pub struct StreamingContext {
    /// Current line number (1-based)
    pub line_number: usize,
    /// Currently active section
    pub current_section: Option<SectionKind>,
    /// Events format fields
    pub events_format: Option<String>,
    /// Styles format fields
    pub styles_format: Option<String>,
}

impl StreamingContext {
    /// Create new context with default values
    pub fn new() -> Self {
        Self {
            line_number: 0,
            current_section: None,
            events_format: None,
            styles_format: None,
        }
    }

    /// Advance to next line
    pub fn next_line(&mut self) {
        self.line_number += 1;
    }

    /// Enter new section
    pub fn enter_section(&mut self, kind: SectionKind) {
        self.current_section = Some(kind);
    }

    /// Exit current section
    pub fn exit_section(&mut self) {
        self.current_section = None;
    }

    /// Set format for events section
    pub fn set_events_format(&mut self, format: String) {
        self.events_format = Some(format);
    }

    /// Set format for styles section
    pub fn set_styles_format(&mut self, format: String) {
        self.styles_format = Some(format);
    }

    /// Reset context for new parsing session
    pub fn reset(&mut self) {
        self.line_number = 0;
        self.current_section = None;
        self.events_format = None;
        self.styles_format = None;
    }
}

impl Default for StreamingContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_state_transitions() {
        let mut state = ParserState::ExpectingSection;
        assert!(!state.is_in_section());
        assert_eq!(state.current_section(), None);

        state.enter_section(SectionKind::Events);
        assert!(state.is_in_section());
        assert_eq!(state.current_section(), Some(SectionKind::Events));

        state.enter_event(SectionKind::Events);
        assert!(state.is_in_section());
        assert_eq!(state.current_section(), Some(SectionKind::Events));

        state.exit_section();
        assert!(!state.is_in_section());
        assert_eq!(state.current_section(), None);
    }

    #[test]
    fn section_kind_from_header() {
        assert_eq!(
            SectionKind::from_header("Script Info"),
            SectionKind::ScriptInfo
        );
        assert_eq!(SectionKind::from_header("V4+ Styles"), SectionKind::Styles);
        assert_eq!(SectionKind::from_header("V4 Styles"), SectionKind::Styles);
        assert_eq!(SectionKind::from_header("Events"), SectionKind::Events);
        assert_eq!(SectionKind::from_header("Fonts"), SectionKind::Fonts);
        assert_eq!(SectionKind::from_header("Graphics"), SectionKind::Graphics);
        assert_eq!(
            SectionKind::from_header("Unknown Section"),
            SectionKind::Unknown
        );
    }

    #[test]
    fn section_kind_properties() {
        assert!(SectionKind::Styles.expects_format());
        assert!(SectionKind::Events.expects_format());
        assert!(!SectionKind::ScriptInfo.expects_format());

        assert!(SectionKind::Events.is_timed());
        assert!(!SectionKind::Styles.is_timed());

        assert!(SectionKind::Fonts.is_binary());
        assert!(SectionKind::Graphics.is_binary());
        assert!(!SectionKind::Events.is_binary());
    }

    #[test]
    fn streaming_context_operations() {
        let mut context = StreamingContext::new();
        assert_eq!(context.line_number, 0);
        assert_eq!(context.current_section, None);

        context.next_line();
        assert_eq!(context.line_number, 1);

        context.enter_section(SectionKind::Events);
        assert_eq!(context.current_section, Some(SectionKind::Events));

        context.set_events_format("Layer,Start,End,Text".to_string());
        assert!(context.events_format.is_some());

        context.reset();
        assert_eq!(context.line_number, 0);
        assert_eq!(context.current_section, None);
        assert!(context.events_format.is_none());
    }
}
