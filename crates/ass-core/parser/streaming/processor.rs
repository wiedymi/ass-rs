//! Line processing logic for streaming ASS parser
//!
//! Handles incremental processing of individual lines during streaming parsing,
//! with context-aware processing based on current parser state.

use crate::Result;
use alloc::string::ToString;

use super::{
    delta::DeltaBatch,
    state::{ParserState, SectionKind, StreamingContext}};

/// Line processor for streaming ASS parser
///
/// Handles context-aware processing of individual lines based on current
/// parser state and section type. Maintains state transitions and generates
/// appropriate parse deltas.
pub struct LineProcessor {
    /// Current parser state
    pub state: ParserState,
    /// Parsing context with line tracking
    pub context: StreamingContext}

impl LineProcessor {
    /// Create new line processor
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: ParserState::ExpectingSection,
            context: StreamingContext::new()}
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
                fields_seen} => Ok(self.process_event_continuation(line, *section, *fields_seen))}
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
    #[cfg(not(feature = "std"))]
    
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

    #[test]
    fn processor_default() {
        let processor = LineProcessor::default();
        assert_eq!(processor.context.line_number, 0);
        assert!(!processor.state.is_in_section());
    }

    #[test]
    fn empty_line_processing() {
        let mut processor = LineProcessor::new();
        let result = processor.process_line("").unwrap();
        assert!(result.is_empty());
        assert_eq!(processor.context.line_number, 1);

        let result = processor.process_line("   \t  ").unwrap();
        assert!(result.is_empty());
        assert_eq!(processor.context.line_number, 2);
    }

    #[test]
    fn different_comment_formats() {
        let mut processor = LineProcessor::new();

        let result = processor.process_line("; Standard comment").unwrap();
        assert!(result.is_empty());

        let result = processor.process_line("!: Aegisub comment").unwrap();
        assert!(result.is_empty());

        assert_eq!(processor.context.line_number, 2);
    }

    #[test]
    fn all_section_headers() {
        let mut processor = LineProcessor::new();

        let sections = [
            "[Script Info]",
            "[V4+ Styles]",
            "[Events]",
            "[Fonts]",
            "[Graphics]",
            "[Unknown Section]",
        ];

        for section in &sections {
            let result = processor.process_line(section).unwrap();
            assert!(result.is_empty());
            assert!(processor.state.is_in_section());
        }
    }

    #[test]
    fn script_info_line_processing() {
        let mut processor = LineProcessor::new();
        processor.state.enter_section(SectionKind::ScriptInfo);

        let result = processor.process_line("Title: Test Script").unwrap();
        assert!(result.is_empty());

        let result = processor.process_line("Author: Test Author").unwrap();
        assert!(result.is_empty());

        let result = processor.process_line("ScriptType: v4.00+").unwrap();
        assert!(result.is_empty());

        // Malformed line without colon
        let result = processor.process_line("Malformed line").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn styles_line_processing() {
        let mut processor = LineProcessor::new();
        processor.state.enter_section(SectionKind::Styles);

        let result = processor
            .process_line("Format: Name, Fontname, Fontsize")
            .unwrap();
        assert!(result.is_empty());
        assert!(processor.context.styles_format.is_some());

        let result = processor.process_line("Style: Default,Arial,20").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn events_line_processing() {
        let mut processor = LineProcessor::new();
        processor.state.enter_section(SectionKind::Events);

        let result = processor
            .process_line("Format: Layer, Start, End, Style, Text")
            .unwrap();
        assert!(result.is_empty());
        assert!(processor.context.events_format.is_some());

        let result = processor
            .process_line("Dialogue: 0,0:00:00.00,0:00:05.00,Default,Hello")
            .unwrap();
        assert!(result.is_empty());

        let result = processor
            .process_line("Comment: 0,0:00:05.00,0:00:10.00,Default,Note")
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn binary_line_processing() {
        let mut processor = LineProcessor::new();

        // Test Fonts section
        processor.state.enter_section(SectionKind::Fonts);
        let result = processor.process_line("fontname: Arial.ttf").unwrap();
        assert!(result.is_empty());

        let result = processor
            .process_line("AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD")
            .unwrap();
        assert!(result.is_empty());

        // Test Graphics section
        processor.state.enter_section(SectionKind::Graphics);
        let result = processor.process_line("graphic: logo.png").unwrap();
        assert!(result.is_empty());

        let result = processor
            .process_line("0123456789ABCDEF0123456789ABCDEF")
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn event_continuation_processing() {
        let mut processor = LineProcessor::new();
        processor.state.enter_event(SectionKind::Events);

        let result = processor.process_line("  continuation data").unwrap();
        assert!(result.is_empty());
        // Should return to section state
        assert!(processor.state.is_in_section());
        assert_eq!(processor.state.current_section(), Some(SectionKind::Events));

        // Test with empty continuation
        processor.state.enter_event(SectionKind::Events);
        let result = processor.process_line("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn content_outside_sections() {
        let mut processor = LineProcessor::new();
        // Start in ExpectingSection state
        assert!(!processor.state.is_in_section());

        let result = processor
            .process_line("Random content outside sections")
            .unwrap();
        assert!(result.is_empty());
        // Should still not be in a section
        assert!(!processor.state.is_in_section());
    }

    #[test]
    fn section_header_edge_cases() {
        let mut processor = LineProcessor::new();

        // Section header with spaces
        let result = processor.process_line("[ Script Info ]").unwrap();
        assert!(result.is_empty());
        assert!(processor.state.is_in_section());

        // Empty section header
        let result = processor.process_line("[]").unwrap();
        assert!(result.is_empty());

        // Malformed section headers should not crash
        let result = processor.process_line("[Unclosed section").unwrap();
        assert!(result.is_empty());

        let result = processor.process_line("Unclosed section]").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn unknown_section_processing() {
        let mut processor = LineProcessor::new();
        processor.state.enter_section(SectionKind::Unknown);

        let result = processor
            .process_line("Any content in unknown section")
            .unwrap();
        assert!(result.is_empty());

        let result = processor.process_line("Key: Value").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn line_counter_increments() {
        let mut processor = LineProcessor::new();
        assert_eq!(processor.context.line_number, 0);

        processor.process_line("Line 1").unwrap();
        assert_eq!(processor.context.line_number, 1);

        processor.process_line("Line 2").unwrap();
        assert_eq!(processor.context.line_number, 2);

        processor.process_line("").unwrap();
        assert_eq!(processor.context.line_number, 3);
    }

    #[test]
    fn format_context_updates() {
        let mut processor = LineProcessor::new();

        // Test styles format
        processor.state.enter_section(SectionKind::Styles);
        processor.context.enter_section(SectionKind::Styles);

        assert!(processor.context.styles_format.is_none());
        processor
            .process_line("Format: Name, Fontname, Fontsize, Bold")
            .unwrap();
        assert!(processor.context.styles_format.is_some());

        // Test events format
        processor.state.enter_section(SectionKind::Events);
        processor.context.enter_section(SectionKind::Events);

        assert!(processor.context.events_format.is_none());
        processor
            .process_line("Format: Layer, Start, End, Style, Text")
            .unwrap();
        assert!(processor.context.events_format.is_some());
    }

    #[test]
    fn complex_processing_sequence() {
        let mut processor = LineProcessor::new();

        // Process a complete mini-script
        let lines = [
            "[Script Info]",
            "Title: Test",
            "Author: Tester",
            "",
            "[V4+ Styles]",
            "Format: Name, Fontname, Fontsize",
            "Style: Default,Arial,20",
            "",
            "[Events]",
            "Format: Layer, Start, End, Style, Text",
            "Dialogue: 0,0:00:00.00,0:00:05.00,Default,Hello World",
            "; End of script",
        ];

        for line in &lines {
            let result = processor.process_line(line).unwrap();
            assert!(result.is_empty());
        }

        assert_eq!(processor.context.line_number, lines.len());
        assert!(processor.context.events_format.is_some());
        assert!(processor.context.styles_format.is_some());
    }

    #[test]
    fn whitespace_handling() {
        let mut processor = LineProcessor::new();

        // Test various whitespace scenarios
        processor.process_line("   [Script Info]   ").unwrap();
        assert!(processor.state.is_in_section());

        processor.process_line("\t\tTitle: Test\t\t").unwrap();

        processor
            .process_line("   ; Comment with spaces   ")
            .unwrap();

        processor.process_line("\t\n").unwrap(); // Tab followed by what looks like newline
    }
}
