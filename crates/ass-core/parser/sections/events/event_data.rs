//! Streaming event-line parsing for the `[Events]` section.
//!
//! Detects the event type for a single line and maps its comma-separated fields
//! onto the active format specification, collecting parse issues for malformed
//! entries while building zero-copy [`Event`] values.

use super::EventsParser;
use crate::parser::{
    ast::{Event, EventType},
    errors::{IssueCategory, IssueSeverity, ParseIssue},
    position_tracker::PositionTracker,
};
use alloc::{format, vec::Vec};

impl<'a> EventsParser<'a> {
    /// Parse single event line (Dialogue, Comment, etc.)
    pub(super) fn parse_event_line_internal(
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
}
