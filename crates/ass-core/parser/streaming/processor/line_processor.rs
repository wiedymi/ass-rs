//! Core line processor type, dispatch, and lifecycle for streaming parsing.
//!
//! Defines [`LineProcessor`] and its state-aware dispatch entry points along
//! with construction, reset, and `Default` support.

use crate::Result;

use super::super::{
    delta::DeltaBatch,
    state::{ParserState, SectionKind, StreamingContext},
};

/// Line processor for streaming ASS parser
///
/// Handles context-aware processing of individual lines based on current
/// parser state and section type. Maintains state transitions and generates
/// appropriate parse deltas.
pub struct LineProcessor {
    /// Current parser state
    pub state: ParserState,
    /// Parsing context with line tracking
    pub context: StreamingContext,
}

impl LineProcessor {
    /// Create new line processor
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: ParserState::ExpectingSection,
            context: StreamingContext::new(),
        }
    }

    /// Process a single complete line
    ///
    /// Dispatches line processing based on current state and line content.
    /// Updates internal state and returns any generated deltas.
    ///
    /// # Errors
    ///
    /// Returns an error if the line contains malformed section headers or
    /// other unrecoverable syntax errors during processing.
    pub fn process_line(&mut self, line: &str) -> Result<DeltaBatch<'static>> {
        self.context.next_line();
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with("!:") {
            return Ok(DeltaBatch::new());
        }

        // Handle section headers
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            return Ok(self.process_section_header(trimmed));
        }

        // Handle section content based on current state
        match &self.state {
            ParserState::ExpectingSection => {
                // Content outside sections - ignore or warn
                Ok(DeltaBatch::new())
            }
            ParserState::InSection(section_kind) => {
                Ok(self.process_section_content(line, *section_kind))
            }
            ParserState::InEvent {
                section,
                fields_seen,
            } => Ok(self.process_event_continuation(line, *section, *fields_seen)),
        }
    }

    /// Process section header line
    fn process_section_header(&mut self, line: &str) -> DeltaBatch<'static> {
        let section_name = &line[1..line.len() - 1]; // Remove [ ]
        let section_kind = SectionKind::from_header(section_name);

        // Update state
        self.state.enter_section(section_kind);
        self.context.enter_section(section_kind);

        // Reset format for sections that expect it
        if section_kind.expects_format() {
            match section_kind {
                SectionKind::Events => self.context.events_format = None,
                SectionKind::Styles => self.context.styles_format = None,
                _ => {}
            }
        }

        DeltaBatch::new()
    }

    /// Process content within a section
    fn process_section_content(
        &mut self,
        line: &str,
        section_kind: SectionKind,
    ) -> DeltaBatch<'static> {
        match section_kind {
            SectionKind::ScriptInfo => Self::process_script_info_line(line),
            SectionKind::Styles => self.process_styles_line(line),
            SectionKind::Events => self.process_events_line(line),
            SectionKind::Fonts | SectionKind::Graphics => Self::process_binary_line(line),
            SectionKind::Unknown => {
                // Log unknown section content but continue parsing
                DeltaBatch::new()
            }
        }
    }

    /// Reset processor state for new parsing session
    pub fn reset(&mut self) {
        self.state = ParserState::ExpectingSection;
        self.context.reset();
    }
}

impl Default for LineProcessor {
    fn default() -> Self {
        Self::new()
    }
}
