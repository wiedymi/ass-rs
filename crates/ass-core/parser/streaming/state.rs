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
    #[must_use]
    pub const fn is_in_section(&self) -> bool {
        matches!(self, Self::InSection(_) | Self::InEvent { .. })
    }

    /// Get current section kind if in a section
    #[must_use]
    pub const fn current_section(&self) -> Option<SectionKind> {
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
    #[must_use]
    pub const fn new() -> Self {
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

    #[test]
    fn parser_state_debug_and_clone() {
        let state = ParserState::ExpectingSection;
        let debug_str = format!("{state:?}");
        assert!(debug_str.contains("ExpectingSection"));

        let cloned = state.clone();
        assert_eq!(state, cloned);

        let section_state = ParserState::InSection(SectionKind::Events);
        let section_debug = format!("{section_state:?}");
        assert!(section_debug.contains("InSection"));
        assert!(section_debug.contains("Events"));

        let event_state = ParserState::InEvent {
            section: SectionKind::Events,
            fields_seen: 3,
        };
        let event_debug = format!("{event_state:?}");
        assert!(event_debug.contains("InEvent"));
        assert!(event_debug.contains("fields_seen"));
    }

    #[test]
    fn parser_state_equality() {
        let state1 = ParserState::ExpectingSection;
        let state2 = ParserState::ExpectingSection;
        assert_eq!(state1, state2);

        let state3 = ParserState::InSection(SectionKind::Events);
        let state4 = ParserState::InSection(SectionKind::Events);
        assert_eq!(state3, state4);

        let state5 = ParserState::InEvent {
            section: SectionKind::Events,
            fields_seen: 2,
        };
        let state6 = ParserState::InEvent {
            section: SectionKind::Events,
            fields_seen: 2,
        };
        assert_eq!(state5, state6);

        // Test inequality
        assert_ne!(state1, state3);
        assert_ne!(state3, state5);

        let state7 = ParserState::InEvent {
            section: SectionKind::Events,
            fields_seen: 3,
        };
        assert_ne!(state5, state7);
    }

    #[test]
    fn parser_state_all_variants() {
        // Test ExpectingSection
        let expecting = ParserState::ExpectingSection;
        assert!(!expecting.is_in_section());
        assert_eq!(expecting.current_section(), None);

        // Test InSection for all section kinds
        for &kind in &[
            SectionKind::ScriptInfo,
            SectionKind::Styles,
            SectionKind::Events,
            SectionKind::Fonts,
            SectionKind::Graphics,
            SectionKind::Unknown,
        ] {
            let in_section = ParserState::InSection(kind);
            assert!(in_section.is_in_section());
            assert_eq!(in_section.current_section(), Some(kind));
        }

        // Test InEvent
        let in_event = ParserState::InEvent {
            section: SectionKind::Events,
            fields_seen: 5,
        };
        assert!(in_event.is_in_section());
        assert_eq!(in_event.current_section(), Some(SectionKind::Events));
    }

    #[test]
    fn section_kind_all_variants() {
        let kinds = [
            SectionKind::ScriptInfo,
            SectionKind::Styles,
            SectionKind::Events,
            SectionKind::Fonts,
            SectionKind::Graphics,
            SectionKind::Unknown,
        ];

        for &kind in &kinds {
            let debug_str = format!("{kind:?}");
            assert!(!debug_str.is_empty());

            // Test Copy trait
            let copied = kind;
            assert_eq!(kind, copied);
        }
    }

    #[test]
    fn section_kind_header_parsing_edge_cases() {
        // Test case insensitive variations
        assert_eq!(
            SectionKind::from_header("  Script Info  "),
            SectionKind::ScriptInfo
        );
        assert_eq!(
            SectionKind::from_header("\tV4+ Styles\t"),
            SectionKind::Styles
        );

        // Test empty and whitespace
        assert_eq!(SectionKind::from_header(""), SectionKind::Unknown);
        assert_eq!(SectionKind::from_header("   "), SectionKind::Unknown);

        // Test partial matches
        assert_eq!(SectionKind::from_header("Script"), SectionKind::Unknown);
        assert_eq!(SectionKind::from_header("Info"), SectionKind::Unknown);
        assert_eq!(SectionKind::from_header("Styles"), SectionKind::Unknown);

        // Test common variations
        assert_eq!(SectionKind::from_header("V4 Styles"), SectionKind::Styles);
        assert_eq!(SectionKind::from_header("V4+ Styles"), SectionKind::Styles);
    }

    #[test]
    fn section_kind_all_properties() {
        // Test expects_format
        assert!(SectionKind::Styles.expects_format());
        assert!(SectionKind::Events.expects_format());
        assert!(!SectionKind::ScriptInfo.expects_format());
        assert!(!SectionKind::Fonts.expects_format());
        assert!(!SectionKind::Graphics.expects_format());
        assert!(!SectionKind::Unknown.expects_format());

        // Test is_timed
        assert!(SectionKind::Events.is_timed());
        assert!(!SectionKind::ScriptInfo.is_timed());
        assert!(!SectionKind::Styles.is_timed());
        assert!(!SectionKind::Fonts.is_timed());
        assert!(!SectionKind::Graphics.is_timed());
        assert!(!SectionKind::Unknown.is_timed());

        // Test is_binary
        assert!(SectionKind::Fonts.is_binary());
        assert!(SectionKind::Graphics.is_binary());
        assert!(!SectionKind::ScriptInfo.is_binary());
        assert!(!SectionKind::Styles.is_binary());
        assert!(!SectionKind::Events.is_binary());
        assert!(!SectionKind::Unknown.is_binary());
    }

    #[test]
    fn streaming_context_default() {
        let context = StreamingContext::default();
        assert_eq!(context.line_number, 0);
        assert_eq!(context.current_section, None);
        assert!(context.events_format.is_none());
        assert!(context.styles_format.is_none());
    }

    #[test]
    fn streaming_context_debug_and_clone() {
        let context = StreamingContext::new();
        let debug_str = format!("{context:?}");
        assert!(debug_str.contains("StreamingContext"));
        assert!(debug_str.contains("line_number"));

        let mut context_with_data = StreamingContext::new();
        context_with_data.next_line();
        context_with_data.enter_section(SectionKind::Events);
        context_with_data.set_events_format("Test Format".to_string());

        let cloned = context_with_data.clone();
        assert_eq!(cloned.line_number, context_with_data.line_number);
        assert_eq!(cloned.current_section, context_with_data.current_section);
        assert_eq!(cloned.events_format, context_with_data.events_format);
    }

    #[test]
    fn streaming_context_format_management() {
        let mut context = StreamingContext::new();

        // Test events format
        assert!(context.events_format.is_none());
        context.set_events_format("Layer, Start, End, Style, Text".to_string());
        assert!(context.events_format.is_some());
        assert_eq!(
            context.events_format.as_ref().unwrap(),
            "Layer, Start, End, Style, Text"
        );

        // Test styles format
        assert!(context.styles_format.is_none());
        context.set_styles_format("Name, Fontname, Fontsize".to_string());
        assert!(context.styles_format.is_some());
        assert_eq!(
            context.styles_format.as_ref().unwrap(),
            "Name, Fontname, Fontsize"
        );

        // Test reset clears formats
        context.reset();
        assert!(context.events_format.is_none());
        assert!(context.styles_format.is_none());
    }

    #[test]
    fn streaming_context_section_management() {
        let mut context = StreamingContext::new();
        assert_eq!(context.current_section, None);

        context.enter_section(SectionKind::ScriptInfo);
        assert_eq!(context.current_section, Some(SectionKind::ScriptInfo));

        context.enter_section(SectionKind::Events);
        assert_eq!(context.current_section, Some(SectionKind::Events));

        context.exit_section();
        assert_eq!(context.current_section, None);
    }

    #[test]
    fn streaming_context_line_tracking() {
        let mut context = StreamingContext::new();
        assert_eq!(context.line_number, 0);

        for expected_line in 1..=100 {
            context.next_line();
            assert_eq!(context.line_number, expected_line);
        }

        context.reset();
        assert_eq!(context.line_number, 0);
    }

    #[test]
    fn parser_state_transition_sequences() {
        let mut state = ParserState::ExpectingSection;

        // Test complete transition sequence
        state.enter_section(SectionKind::Events);
        assert!(state.is_in_section());
        assert_eq!(state.current_section(), Some(SectionKind::Events));

        state.enter_event(SectionKind::Events);
        assert!(state.is_in_section());
        assert_eq!(state.current_section(), Some(SectionKind::Events));

        state.exit_section();
        assert!(!state.is_in_section());
        assert_eq!(state.current_section(), None);

        // Test direct event entry
        state.enter_event(SectionKind::Styles);
        assert!(state.is_in_section());
        assert_eq!(state.current_section(), Some(SectionKind::Styles));
    }

    #[test]
    fn complex_state_context_interaction() {
        let mut state = ParserState::ExpectingSection;
        let mut context = StreamingContext::new();

        // Simulate processing script
        context.next_line(); // Line 1
        state.enter_section(SectionKind::ScriptInfo);
        context.enter_section(SectionKind::ScriptInfo);

        context.next_line(); // Line 2
        context.next_line(); // Line 3

        state.enter_section(SectionKind::Events);
        context.enter_section(SectionKind::Events);
        context.set_events_format("Layer, Start, End, Text".to_string());

        context.next_line(); // Line 4
        state.enter_event(SectionKind::Events);

        assert_eq!(context.line_number, 4);
        assert!(context.events_format.is_some());
        assert_eq!(context.current_section, Some(SectionKind::Events));
        assert_eq!(state.current_section(), Some(SectionKind::Events));
    }
}
