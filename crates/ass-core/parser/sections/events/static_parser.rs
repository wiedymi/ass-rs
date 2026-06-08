//! Stateless event-line parsing for incremental re-parsing.
//!
//! Exposes [`EventsParser::parse_event_line`], a static entry point that parses
//! a single event line against an explicit format specification without needing
//! a live parser instance, returning [`ParseError`] on malformed input.

use super::EventsParser;
use crate::parser::{
    ast::{Event, EventType, Span},
    errors::ParseError,
};
use alloc::vec::Vec;

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
}
