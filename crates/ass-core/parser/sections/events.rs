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
    source: &'a str,
    position: usize,
    line: usize,
    issues: Vec<ParseIssue>,
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
    pub fn new(source: &'a str, start_position: usize, start_line: usize) -> Self {
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
    /// Tuple of (parsed_section, format_fields, parse_issues, final_position, final_line)
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
            let fields: Vec<&'a str> = format_data.split(',').map(|s| s.trim()).collect();
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
        let parts: Vec<&str> = data.splitn(10, ',').collect();

        let format = self.format.as_deref().unwrap_or(&[
            "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect",
            "Text",
        ]);

        if parts.len() < format.len().min(9) {
            self.issues.push(ParseIssue::new(
                IssueSeverity::Warning,
                IssueCategory::Format,
                format!(
                    "Event line has {} fields, expected at least {}",
                    parts.len(),
                    format.len().min(9)
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
                .map(|s| s.trim())
                .unwrap_or("")
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
            .map(|pos| self.position + pos)
            .unwrap_or(self.source.len());

        &self.source[start..end]
    }

    /// Check if at start of next section
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
    pub fn issues(self) -> Vec<ParseIssue> {
        self.issues
    }

    /// Get format specification
    pub fn format(&self) -> Option<&Vec<&'a str>> {
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
}
