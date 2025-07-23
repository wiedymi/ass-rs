//! Line processing logic for streaming ASS parser
//!
//! Handles incremental processing of individual lines during streaming parsing,
//! with context-aware processing based on current parser state.

use crate::Result;

use super::{
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
    pub fn new() -> Self {
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

    /// Process line in Script Info section
    fn process_script_info_line(line: &str) -> DeltaBatch<'static> {
        let trimmed = line.trim();

        if let Some(colon_pos) = trimmed.find(':') {
            let _key = trimmed[..colon_pos].trim();
            let _value = trimmed[colon_pos + 1..].trim();
            // TODO: Handle script info fields
        }

        DeltaBatch::new()
    }

    /// Process line in Styles section
    fn process_styles_line(&mut self, line: &str) -> DeltaBatch<'static> {
        let trimmed = line.trim();

        if let Some(format_str) = trimmed.strip_prefix("Format:") {
            let format_str = format_str.trim().to_string();
            self.context.set_styles_format(format_str);
        } else if trimmed.starts_with("Style:") {
            // Style definition detected - in full parser this would create AST node
        }

        DeltaBatch::new()
    }

    /// Process line in Events section
    fn process_events_line(&mut self, line: &str) -> DeltaBatch<'static> {
        let trimmed = line.trim();

        if let Some(format_str) = trimmed.strip_prefix("Format:") {
            let format_str = format_str.trim().to_string();
            self.context.set_events_format(format_str);
            return DeltaBatch::new();
        }

        if trimmed.starts_with("Dialogue:") || trimmed.starts_with("Comment:") {
            // Begin event parsing
            self.state.enter_event(SectionKind::Events);
            // In full parser, this would parse the event fields
        }

        DeltaBatch::new()
    }

    /// Process line in binary sections (Fonts/Graphics)
    fn process_binary_line(line: &str) -> DeltaBatch<'static> {
        let trimmed = line.trim();

        if trimmed.contains(':') {
            // Font/graphic filename declaration
        } else {
            // UU-encoded data line
        }

        DeltaBatch::new()
    }

    /// Process continuation of an event
    fn process_event_continuation(
        &mut self,
        line: &str,
        section: SectionKind,
        _fields_seen: usize,
    ) -> DeltaBatch<'static> {
        let trimmed = line.trim();

        if !trimmed.is_empty() {
            // Process event continuation data
        }

        // Return to section state
        self.state = ParserState::InSection(section);
        DeltaBatch::new()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processor_creation() {
        let processor = LineProcessor::new();
        assert_eq!(processor.context.line_number, 0);
        assert!(!processor.state.is_in_section());
    }

    #[test]
    fn section_header_processing() {
        let mut processor = LineProcessor::new();
        let result = processor.process_line("[Script Info]").unwrap();
        assert!(result.is_empty());
        assert!(processor.state.is_in_section());
        assert_eq!(
            processor.state.current_section(),
            Some(SectionKind::ScriptInfo)
        );
    }

    #[test]
    fn comment_line_skipping() {
        let mut processor = LineProcessor::new();
        let result = processor.process_line("; This is a comment").unwrap();
        assert!(result.is_empty());
        assert_eq!(processor.context.line_number, 1);
    }

    #[test]
    fn format_line_processing() {
        let mut processor = LineProcessor::new();
        processor.state.enter_section(SectionKind::Events);
        processor.context.enter_section(SectionKind::Events);

        let result = processor
            .process_line("Format: Layer, Start, End, Style, Text")
            .unwrap();
        assert!(result.is_empty());
        assert!(processor.context.events_format.is_some());
    }

    #[test]
    fn processor_reset() {
        let mut processor = LineProcessor::new();
        processor.state.enter_section(SectionKind::Events);
        processor.context.next_line();

        processor.reset();
        assert!(!processor.state.is_in_section());
        assert_eq!(processor.context.line_number, 0);
    }
}
