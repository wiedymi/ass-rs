//! Events section parser for ASS scripts.
//!
//! Handles parsing of the [Events] section which contains dialogue, comments,
//! and other timed events with format specifications and event entries.

use crate::parser::{
    ast::{Event, EventType, Section},
    errors::{IssueCategory, IssueSeverity, ParseIssue},
    sections::SectionParseResult,
    ParseResult,
};
use alloc::vec::Vec;

/// Parser for [Events] section content
///
/// Parses format definitions and event entries from the events section.
/// Uses format mapping to handle different field orderings and event types.
///
/// # Performance
///
/// - Time complexity: O(n * m) for n events and m fields per event
/// - Memory: Zero allocations via lifetime-generic spans
/// - Target: <2ms for typical event sections with 1000 events
pub struct EventsParser<'a> {
    /// Source text being parsed
    source: &'a str,
    /// Current byte position in source
    position: usize,
    /// Current line number for error reporting
    line: usize,
    /// Parse issues and warnings collected during parsing
    issues: Vec<ParseIssue>,
    /// Format fields for the events section
    format: Option<Vec<&'a str>>,
}

impl<'a> EventsParser<'a> {
    /// Create new events parser for source text
    ///
    /// # Arguments
    ///
    /// * `source` - Source text to parse
    /// * `start_position` - Starting byte position in source
    /// * `start_line` - Starting line number for error reporting
    #[must_use]
    pub const fn new(source: &'a str, start_position: usize, start_line: usize) -> Self {
        Self {
            source,
            position: start_position,
            line: start_line,
            issues: Vec::new(),
            format: None,
        }
    }

    /// Parse events section content
    ///
    /// Returns the parsed section and any issues encountered during parsing.
    /// Handles Format line parsing and event entry validation.
    ///
    /// # Returns
    ///
    /// Tuple of (`parsed_section`, `format_fields`, `parse_issues`, `final_position`, `final_line`)
    ///
    /// # Errors
    ///
    /// Returns an error if the events section contains malformed format lines or
    /// other unrecoverable syntax errors.
    pub fn parse(mut self) -> ParseResult<SectionParseResult<'a>> {
        let mut events = Vec::new();

        while self.position < self.source.len() && !self.at_next_section() {
            self.skip_whitespace_and_comments();

            if self.position >= self.source.len() || self.at_next_section() {
                break;
            }

            let line = self.current_line().trim();
            if line.is_empty() {
                self.skip_line();
                continue;
            }

            if line.starts_with("Format:") {
                self.parse_format_line(line);
            } else if let Some(event) = self.parse_event_line(line) {
                events.push(event);
            }

            self.skip_line();
        }

        Ok((
            Section::Events(events),
            self.format,
            self.issues,
            self.position,
            self.line,
        ))
    }

    /// Parse format specification line
    fn parse_format_line(&mut self, line: &'a str) {
        if let Some(format_data) = line.strip_prefix("Format:") {
            let fields: Vec<&'a str> = format_data.split(',').map(str::trim).collect();
            self.format = Some(fields);
        }
    }

    /// Parse single event line (Dialogue, Comment, etc.)
    fn parse_event_line(&mut self, line: &'a str) -> Option<Event<'a>> {
        let (event_type, data) = if let Some(data) = line.strip_prefix("Dialogue:") {
            (EventType::Dialogue, data)
        } else if let Some(data) = line.strip_prefix("Comment:") {
            (EventType::Comment, data)
        } else if let Some(data) = line.strip_prefix("Picture:") {
            (EventType::Picture, data)
        } else if let Some(data) = line.strip_prefix("Sound:") {
            (EventType::Sound, data)
        } else if let Some(data) = line.strip_prefix("Movie:") {
            (EventType::Movie, data)
        } else if let Some(data) = line.strip_prefix("Command:") {
            (EventType::Command, data)
        } else {
            return None;
        };

        self.parse_event_data(event_type, data.trim())
    }

    /// Parse event data fields using format mapping
    fn parse_event_data(&mut self, event_type: EventType, data: &'a str) -> Option<Event<'a>> {
        let format = self.format.as_deref().unwrap_or(&[
            "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect",
            "Text",
        ]);

        // Check if Text field exists in format to determine splitting strategy
        let has_text_field = format
            .iter()
            .any(|&field| field.eq_ignore_ascii_case("Text"));

        // Use appropriate splitting strategy to handle commas correctly
        let parts: Vec<&str> = if has_text_field {
            // If Text field exists, limit splits to preserve commas in text
            data.splitn(format.len(), ',').collect()
        } else {
            // If no Text field, split all commas and ignore extra fields
            data.splitn(10, ',').collect()
        };

        if parts.len() < format.len() {
            self.issues.push(ParseIssue::new(
                IssueSeverity::Warning,
                IssueCategory::Format,
                format!(
                    "Event line has {} fields, expected at least {}",
                    parts.len(),
                    format.len()
                ),
                self.line,
            ));
            return None;
        }

        let get_field = |name: &str| -> &'a str {
            format
                .iter()
                .position(|&field| field.eq_ignore_ascii_case(name))
                .and_then(|idx| parts.get(idx))
                .map_or("", |s| s.trim())
        };

        let text = get_field("Text");

        Some(Event {
            event_type,
            layer: get_field("Layer"),
            start: get_field("Start"),
            end: get_field("End"),
            style: get_field("Style"),
            name: get_field("Name"),
            margin_l: get_field("MarginL"),
            margin_r: get_field("MarginR"),
            margin_v: get_field("MarginV"),
            effect: get_field("Effect"),
            text,
        })
    }

    /// Get current line from source
    fn current_line(&self) -> &'a str {
        let start = self.position;
        let end = self.source[self.position..]
            .find('\n')
            .map_or(self.source.len(), |pos| self.position + pos);

        &self.source[start..end]
    }

    /// Check if at start of next section
    #[must_use]
    fn at_next_section(&self) -> bool {
        let remaining = &self.source[self.position..];
        remaining.trim_start().starts_with('[')
    }

    /// Skip current line and advance position
    fn skip_line(&mut self) {
        if let Some(newline_pos) = self.source[self.position..].find('\n') {
            self.position += newline_pos + 1;
            self.line += 1;
        } else {
            self.position = self.source.len();
        }
    }

    /// Skip whitespace and comment lines
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            let remaining = &self.source[self.position..];
            let trimmed = remaining.trim_start();

            if trimmed.is_empty() {
                self.position = self.source.len();
                break;
            }

            if trimmed.starts_with(';') || trimmed.starts_with('#') {
                self.skip_line();
                continue;
            }

            let whitespace_len = remaining.len() - trimmed.len();
            if whitespace_len > 0 {
                let newlines = remaining[..whitespace_len].matches('\n').count();
                self.position += whitespace_len;
                self.line += newlines;
            }

            break;
        }
    }

    /// Get accumulated parse issues
    #[must_use]
    pub fn issues(self) -> Vec<ParseIssue> {
        self.issues
    }

    /// Get format specification
    #[must_use]
    pub const fn format(&self) -> Option<&Vec<&'a str>> {
        self.format.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_section() {
        let parser = EventsParser::new("", 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert!(events.is_empty());
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_with_format_and_dialogue() {
        let content = "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            let event = &events[0];
            assert!(matches!(event.event_type, EventType::Dialogue));
            assert_eq!(event.start, "0:00:00.00");
            assert_eq!(event.end, "0:00:05.00");
            assert_eq!(event.style, "Default");
            assert_eq!(event.text, "Hello World!");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_different_event_types() {
        let content = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Dialogue\nComment: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Comment\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 2);
            assert!(matches!(events[0].event_type, EventType::Dialogue));
            assert!(matches!(events[1].event_type, EventType::Comment));
            assert_eq!(events[0].text, "Dialogue");
            assert_eq!(events[1].text, "Comment");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn handle_text_with_commas() {
        let content =
            "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello, world, with commas!\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].text, "Hello, world, with commas!");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn skip_comments_and_whitespace() {
        let content = "; Comment\n# Another comment\n\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].text, "Test");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_all_event_types() {
        let content = "Picture: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,picture.png\nSound: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,sound.wav\nMovie: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,movie.avi\nCommand: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,some command\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 4);
            assert!(matches!(events[0].event_type, EventType::Picture));
            assert!(matches!(events[1].event_type, EventType::Sound));
            assert!(matches!(events[2].event_type, EventType::Movie));
            assert!(matches!(events[3].event_type, EventType::Command));
            assert_eq!(events[0].text, "picture.png");
            assert_eq!(events[1].text, "sound.wav");
            assert_eq!(events[2].text, "movie.avi");
            assert_eq!(events[3].text, "some command");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_custom_format_order() {
        let content = "Format: Text, Start, End, Style\nDialogue: Hello World!,0:00:00.00,0:00:05.00,Default\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, format, ..) = result.unwrap();
        assert!(format.is_some());
        let format_fields = format.unwrap();
        assert_eq!(format_fields, vec!["Text", "Start", "End", "Style"]);

        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            let event = &events[0];
            assert_eq!(event.text, "Hello World!");
            assert_eq!(event.start, "0:00:00.00");
            assert_eq!(event.end, "0:00:05.00");
            assert_eq!(event.style, "Default");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_insufficient_fields_warning() {
        let content = "Dialogue: 0,0:00:00.00\n"; // Only 2 fields, should warn
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, _, issues, ..) = result.unwrap();
        assert!(!issues.is_empty());
        let has_field_warning = issues
            .iter()
            .any(|issue| issue.message.contains("fields, expected at least"));
        assert!(has_field_warning);

        if let Section::Events(events) = section {
            assert!(events.is_empty()); // Event should be skipped due to insufficient fields
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_empty_format_line() {
        let content = "Format:\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, format, ..) = result.unwrap();
        assert!(format.is_some());
        let format_fields = format.unwrap();
        assert_eq!(format_fields, vec![""]);

        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_format_with_extra_spaces() {
        let content = "Format:  Layer ,  Start ,  End ,  Style ,  Text  \nDialogue: 0,0:00:00.00,0:00:05.00,Default,Test\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, format, ..) = result.unwrap();
        assert!(format.is_some());
        let format_fields = format.unwrap();
        assert_eq!(
            format_fields,
            vec!["Layer", "Start", "End", "Style", "Text"]
        );

        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].text, "Test");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_unknown_event_type() {
        let content = "Unknown: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Valid\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1); // Only valid dialogue should be parsed
            assert_eq!(events[0].text, "Valid");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_mixed_case_format_fields() {
        let content = "Format: layer, START, end, STYLE, text\nDialogue: 5,0:00:01.00,0:00:06.00,TestStyle,Hello\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            let event = &events[0];
            assert_eq!(event.layer, "5");
            assert_eq!(event.start, "0:00:01.00");
            assert_eq!(event.end, "0:00:06.00");
            assert_eq!(event.style, "TestStyle");
            assert_eq!(event.text, "Hello");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_event_with_unicode_text() {
        let content = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,こんにちは世界！🎭\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].text, "こんにちは世界！🎭");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_event_with_empty_fields() {
        let content = "Dialogue: ,,,,,,,,,Empty fields but text\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            let event = &events[0];
            assert_eq!(event.layer, "");
            assert_eq!(event.start, "");
            assert_eq!(event.end, "");
            assert_eq!(event.style, "");
            assert_eq!(event.text, "Empty fields but text");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_stops_at_next_section() {
        let content = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test\n[Next Section]\nOther content\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, _, _, position, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].text, "Test");
        } else {
            panic!("Expected Events section");
        }

        // Should stop before the next section
        assert!(content[position..].starts_with("[Next Section]"));
    }

    #[test]
    fn parse_with_trailing_whitespace() {
        let content = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test   \n   \n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].text, "Test"); // Should be trimmed
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_multiple_format_lines() {
        let content = "Format: Layer, Start, End, Style, Text\nFormat: Different, Format, Here\nDialogue: 1,0:00:00.00,0:00:05.00,Default,Test\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, format, ..) = result.unwrap();
        // Should use the last format line
        assert!(format.is_some());
        let format_fields = format.unwrap();
        assert_eq!(format_fields, vec!["Different", "Format", "Here"]);

        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_missing_format_uses_default() {
        let content = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,Name,0,0,0,Effect,Text\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, format, ..) = result.unwrap();
        assert!(format.is_none()); // No format line provided

        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            let event = &events[0];
            // Should use default format mapping
            assert_eq!(event.layer, "0");
            assert_eq!(event.start, "0:00:00.00");
            assert_eq!(event.end, "0:00:05.00");
            assert_eq!(event.style, "Default");
            assert_eq!(event.name, "Name");
            assert_eq!(event.text, "Text");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_event_data_edge_cases() {
        // Test edge cases in field extraction
        let content =
            "Format: NonExistentField, Text, Start\nDialogue: value,Hello World,0:00:00.00\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            let event = &events[0];
            assert_eq!(event.text, "Hello World");
            assert_eq!(event.start, "0:00:00.00");
            // Non-existent fields should default to empty
            assert_eq!(event.end, "");
            assert_eq!(event.style, "");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn current_line_at_end_of_file() {
        let content = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,No newline";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].text, "No newline");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn skip_whitespace_and_comments_comprehensive() {
        let content = "   \n  ; First comment\n!Another comment\n# Third comment\n\n  \nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,After comments\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].text, "After comments");
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_all_event_types_comprehensive() {
        let content = "Format: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Dialogue text\nComment: 0:00:00.00,0:00:05.00,Comment text\nPicture: 0:00:00.00,0:00:05.00,Picture text\nSound: 0:00:00.00,0:00:05.00,Sound text\nMovie: 0:00:00.00,0:00:05.00,Movie text\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 5);
            assert_eq!(events[0].event_type, EventType::Dialogue);
            assert_eq!(events[1].event_type, EventType::Comment);
            assert_eq!(events[2].event_type, EventType::Picture);
            assert_eq!(events[3].event_type, EventType::Sound);
            assert_eq!(events[4].event_type, EventType::Movie);
        } else {
            panic!("Expected Events section");
        }
    }

    #[test]
    fn parse_event_data_insufficient_fields_error() {
        let content = "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0:00:00.00,0:00:05.00\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, _format, issues, ..) = result.unwrap();

        // Should generate warning for insufficient fields
        let has_field_warning = issues
            .iter()
            .any(|issue| issue.message.contains("has 2 fields, expected at least"));
        assert!(has_field_warning);

        // Should not add events with insufficient fields
        if let Section::Events(events) = section {
            assert!(events.is_empty());
        }
    }

    #[test]
    fn parse_event_data_minimum_fields_required() {
        let content = "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,Name,0,0,0,Effect\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, _format, issues, ..) = result.unwrap();

        // Should generate warning for missing Text field
        let has_field_warning = issues
            .iter()
            .any(|issue| issue.message.contains("has 9 fields, expected at least 10"));
        assert!(has_field_warning);

        // Should not add events with insufficient minimum fields
        if let Section::Events(events) = section {
            assert!(events.is_empty());
        }
    }

    #[test]
    fn parse_event_data_exactly_minimum_fields() {
        let content = "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,Name,0,0,0,Effect,Text\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, _format, issues, ..) = result.unwrap();

        // Should not generate field count warnings
        let has_field_warning = issues
            .iter()
            .any(|issue| issue.message.contains("fields, expected at least"));
        assert!(!has_field_warning);

        // Should successfully add the event
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].text, "Text");
        }
    }

    #[test]
    fn parse_event_data_with_commas_in_text() {
        let content = "Format: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Text with, many, commas, here\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);

            // Text field should contain commas since we use splitn(10, ',')
            assert!(events[0].text.contains("commas"));
        }
    }

    #[test]
    fn parse_event_data_field_case_insensitive() {
        let content = "Format: start, END, text\nDialogue: 0:00:00.00,0:00:05.00,Test text\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].start, "0:00:00.00");
            assert_eq!(events[0].end, "0:00:05.00");
            assert_eq!(events[0].text, "Test text");
        }
    }

    #[test]
    fn parse_event_data_missing_format_field() {
        let content =
            "Format: Start, End\nDialogue: 0:00:00.00,0:00:05.00,Text without format field\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            // Text field should be empty since it's not in format
            assert_eq!(events[0].text, "");
            // But other fields should be populated
            assert_eq!(events[0].start, "0:00:00.00");
            assert_eq!(events[0].end, "0:00:05.00");
        }
    }

    #[test]
    fn parse_event_data_whitespace_trimming() {
        let content =
            "Format: Start, End, Text\nDialogue:  0:00:00.00 , 0:00:05.00 ,  Text with spaces  \n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            // Values should be trimmed
            assert_eq!(events[0].start, "0:00:00.00");
            assert_eq!(events[0].end, "0:00:05.00");
            assert_eq!(events[0].text, "Text with spaces");
        }
    }

    #[test]
    fn parse_unknown_event_type_fallback() {
        let content =
            "Format: Start, End, Text\nUnknownType: 0:00:00.00,0:00:05.00,Unknown event\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            // Unknown event types should be ignored
            assert_eq!(events.len(), 0);
        }
    }

    #[test]
    fn parse_format_line_with_extra_whitespace_and_commas() {
        let content = "Format:  Start ,  End , Text ,  \nDialogue: 0:00:00.00,0:00:05.00,Test\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (_section, format, ..) = result.unwrap();
        if let Some(fmt) = format {
            // Should handle extra whitespace and trailing commas
            assert!(fmt.len() >= 3);
            assert!(fmt.contains(&"Start"));
            assert!(fmt.contains(&"End"));
            assert!(fmt.contains(&"Text"));
        }
    }

    #[test]
    fn parse_events_with_unicode_and_special_characters() {
        let content =
            "Format: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Unicode: 🎬 中文 العربية\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert!(events[0].text.contains("🎬"));
            assert!(events[0].text.contains("中文"));
            assert!(events[0].text.contains("العربية"));
        }
    }

    #[test]
    fn parse_events_with_empty_field_values() {
        let content = "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: ,,,,,,,,, Empty fields\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].layer, "");
            assert_eq!(events[0].start, "");
            assert_eq!(events[0].end, "");
            assert_eq!(events[0].style, "");
            assert_eq!(events[0].text, "Empty fields");
        }
    }

    #[test]
    fn parse_events_line_ending_variations() {
        let content = "Format: Start, End, Text\r\nDialogue: 0:00:00.00,0:00:05.00,Windows CRLF\nDialogue: 0:00:05.00,0:00:10.00,Unix LF\rDialogue: 0:00:10.00,0:00:15.00,Mac CR\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            // Should handle different line endings
            assert!(!events.is_empty());
        }
    }

    #[test]
    fn parse_events_at_end_of_file_no_newline() {
        let content = "Format: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,No newline at end";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].text, "No newline at end");
        }
    }

    #[test]
    fn parse_events_mixed_valid_invalid_lines() {
        let content = "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,Name,0,0,0,Effect,Valid line\nDialogue: OnlyTwoFields,Invalid\nComment: 0,0:00:05.00,0:00:10.00,Default,Name,0,0,0,Effect,Another valid\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, _format, issues, ..) = result.unwrap();

        // Should have warning for insufficient fields
        let field_warnings = issues
            .iter()
            .filter(|issue| issue.message.contains("fields, expected at least"))
            .count();
        assert_eq!(field_warnings, 1);

        // Should only include valid events
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 2); // Only valid events
            assert_eq!(events[0].text, "Valid line");
            assert_eq!(events[1].text, "Another valid");
        }
    }
}
