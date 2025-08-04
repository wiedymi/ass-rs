//! Events section parser for ASS scripts.
//!
//! Handles parsing of the [Events] section which contains dialogue, comments,
//! and other timed events with format specifications and event entries.

use crate::parser::{
    ast::{Event, EventType, Section, Span},
    errors::{IssueCategory, IssueSeverity, ParseError, ParseIssue},
    position_tracker::PositionTracker,
    sections::SectionParseResult,
    ParseResult,
};
use alloc::{format, vec::Vec};

/// Parser for `[Events]` section content
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
    /// Position tracker for accurate span generation
    tracker: PositionTracker<'a>,
    /// Parse issues and warnings collected during parsing
    issues: Vec<ParseIssue>,
    /// Format fields for the events section
    format: Option<Vec<&'a str>>,
}

impl<'a> EventsParser<'a> {
    /// Parse a single event line (make existing internal method public)
    ///
    /// Parses a single event line using the provided format specification.
    /// This method is exposed for incremental parsing support.
    ///
    /// # Arguments
    ///
    /// * `line` - The event line to parse (e.g., "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text")
    /// * `format` - The format fields from the Format line
    /// * `line_number` - The line number for error reporting
    ///
    /// # Returns
    ///
    /// Parsed Event or error if the line is malformed
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::InvalidEventType`] if the line doesn't start with a valid event type
    /// Returns [`ParseError::InsufficientFields`] if the line has fewer fields than expected by format
    pub fn parse_event_line(
        line: &'a str,
        format: &[&'a str],
        line_number: u32,
    ) -> core::result::Result<Event<'a>, ParseError> {
        // Determine event type
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
            return Err(ParseError::InvalidEventType {
                line: line_number as usize,
            });
        };

        // Parse event data
        Self::parse_event_data_static(event_type, data.trim(), format, line_number)
    }

    /// Static helper to parse event data fields
    fn parse_event_data_static(
        event_type: EventType,
        data: &'a str,
        format: &[&'a str],
        line_number: u32,
    ) -> core::result::Result<Event<'a>, ParseError> {
        let format = if format.is_empty() {
            &[
                "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV",
                "Effect", "Text",
            ]
        } else {
            format
        };

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
            return Err(ParseError::InsufficientFields {
                expected: format.len(),
                found: parts.len(),
                line: line_number as usize,
            });
        }

        let get_field = |name: &str| -> &'a str {
            format
                .iter()
                .position(|&field| field.eq_ignore_ascii_case(name))
                .and_then(|idx| parts.get(idx))
                .map_or("", |s| s.trim())
        };

        // Create span for the event (caller will need to adjust this)
        let span = Span::new(0, 0, line_number, 1);

        Ok(Event {
            event_type,
            layer: get_field("Layer"),
            start: get_field("Start"),
            end: get_field("End"),
            style: get_field("Style"),
            name: get_field("Name"),
            margin_l: get_field("MarginL"),
            margin_r: get_field("MarginR"),
            margin_v: get_field("MarginV"),
            margin_t: format
                .iter()
                .any(|&f| f.eq_ignore_ascii_case("MarginT"))
                .then(|| get_field("MarginT")),
            margin_b: format
                .iter()
                .any(|&f| f.eq_ignore_ascii_case("MarginB"))
                .then(|| get_field("MarginB")),
            effect: get_field("Effect"),
            text: get_field("Text"),
            span,
        })
    }
    /// Create new events parser for source text
    ///
    /// # Arguments
    ///
    /// * `source` - Source text to parse
    /// * `start_position` - Starting byte position in source
    /// * `start_line` - Starting line number for error reporting
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Can't be const due to Vec::new()
    pub fn new(source: &'a str, start_position: usize, start_line: usize) -> Self {
        Self {
            tracker: PositionTracker::new_at(
                source,
                start_position,
                u32::try_from(start_line).unwrap_or(u32::MAX),
                1,
            ),
            issues: Vec::new(),
            format: None,
        }
    }

    /// Create a new parser with a pre-known format for incremental parsing
    #[must_use]
    pub fn with_format(
        source: &'a str,
        format: &[&'a str],
        start_position: usize,
        start_line: u32,
    ) -> Self {
        Self {
            tracker: PositionTracker::new_at(source, start_position, start_line, 1),
            issues: Vec::new(),
            format: Some(format.to_vec()),
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

        while !self.tracker.is_at_end() && !self.at_next_section() {
            self.skip_whitespace_and_comments();

            if self.tracker.is_at_end() || self.at_next_section() {
                break;
            }

            let line_start = self.tracker.checkpoint();
            let line = self.current_line().trim();

            if line.is_empty() {
                self.tracker.skip_line();
                continue;
            }

            if line.starts_with("Format:") {
                self.parse_format_line(line);
            } else if let Some(event) = self.parse_event_line_internal(line, &line_start) {
                events.push(event);
            }

            self.tracker.skip_line();
        }

        Ok((
            Section::Events(events),
            self.format,
            self.issues,
            self.tracker.offset(),
            self.tracker.line() as usize,
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
    fn parse_event_line_internal(
        &mut self,
        line: &'a str,
        line_start: &PositionTracker<'a>,
    ) -> Option<Event<'a>> {
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

        self.parse_event_data(event_type, data.trim(), line_start)
    }

    /// Parse event data fields using format mapping
    fn parse_event_data(
        &mut self,
        event_type: EventType,
        data: &'a str,
        line_start: &PositionTracker<'a>,
    ) -> Option<Event<'a>> {
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
                line_start.line() as usize,
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

        // Calculate span for this event line
        // We need to get the original line length from parse_event_line
        // For now, use the current line method to get the full line
        let full_line = self.current_line();
        let span = line_start.span_for(full_line.len());

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
            margin_t: format
                .iter()
                .any(|&f| f.eq_ignore_ascii_case("MarginT"))
                .then(|| get_field("MarginT")),
            margin_b: format
                .iter()
                .any(|&f| f.eq_ignore_ascii_case("MarginB"))
                .then(|| get_field("MarginB")),
            effect: get_field("Effect"),
            text,
            span,
        })
    }

    /// Get current line from source
    fn current_line(&self) -> &'a str {
        let remaining = self.tracker.remaining();
        let end = remaining.find('\n').unwrap_or(remaining.len());
        &remaining[..end]
    }

    /// Check if at start of next section
    #[must_use]
    fn at_next_section(&self) -> bool {
        self.tracker.remaining().trim_start().starts_with('[')
    }

    /// Skip whitespace and comment lines
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.tracker.skip_whitespace();

            let remaining = self.tracker.remaining();
            if remaining.is_empty() {
                break;
            }

            if remaining.starts_with(';') || remaining.starts_with('#') {
                self.tracker.skip_line();
                continue;
            }

            // Check for newlines in whitespace
            if remaining.starts_with('\n') {
                self.tracker.advance(1);
                continue;
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
            // Check span
            assert!(event.span.start > 0);
            assert!(event.span.end > event.span.start);
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
    fn parse_with_position_tracking() {
        // Create a larger content that simulates a full file
        let prefix = "a".repeat(200); // 200 bytes of padding
        let section_content = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test Event\n";
        let full_content = format!("{prefix}{section_content}");

        // Parser starts at position 200
        let parser = EventsParser::new(&full_content, 200, 15);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, _, _, final_pos, final_line) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            let event = &events[0];
            assert_eq!(event.span.start, 200);
            assert_eq!(event.span.line, 15);
            assert_eq!(event.text, "Test Event");
        } else {
            panic!("Expected Events section");
        }

        assert_eq!(final_pos, 200 + section_content.len());
        assert_eq!(final_line, 16);
    }

    #[test]
    fn parse_without_format_line() {
        let content = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,No format line\n";
        let parser = EventsParser::new(content, 0, 1);
        let result = parser.parse();
        assert!(result.is_ok());

        let (section, format, ..) = result.unwrap();
        if let Section::Events(events) = section {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].text, "No format line");
        } else {
            panic!("Expected Events section");
        }
        assert!(format.is_none());
    }

    #[test]
    fn test_public_parse_event_line() {
        let format = vec![
            "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect",
            "Text",
        ];
        let line = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test text";

        let result = EventsParser::parse_event_line(line, &format, 1);
        assert!(result.is_ok());

        let event = result.unwrap();
        assert!(matches!(event.event_type, EventType::Dialogue));
        assert_eq!(event.start, "0:00:00.00");
        assert_eq!(event.end, "0:00:05.00");
        assert_eq!(event.style, "Default");
        assert_eq!(event.text, "Test text");
    }

    #[test]
    fn test_parse_event_line_invalid_type() {
        let format = vec!["Layer", "Start", "End", "Style", "Text"];
        let line = "Invalid: 0,0:00:00.00,0:00:05.00,Default,Test";

        let result = EventsParser::parse_event_line(line, &format, 1);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, ParseError::InvalidEventType { .. }));
        }
    }

    #[test]
    fn test_parse_event_line_insufficient_fields() {
        let format = vec![
            "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect",
            "Text",
        ];
        let line = "Dialogue: 0,0:00:00.00,0:00:05.00"; // Missing fields

        let result = EventsParser::parse_event_line(line, &format, 1);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, ParseError::InsufficientFields { .. }));
        }
    }

    #[test]
    fn test_parse_event_line_with_commas_in_text() {
        let format = vec![
            "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect",
            "Text",
        ];
        let line = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello, world, with commas!";

        let result = EventsParser::parse_event_line(line, &format, 1);
        assert!(result.is_ok());

        let event = result.unwrap();
        assert_eq!(event.text, "Hello, world, with commas!");
    }
}
