//! Per-section line handlers for the streaming [`LineProcessor`].
//!
//! Implements the content processors for Script Info, Styles, Events, binary
//! (Fonts/Graphics) sections, and event continuation lines.

use alloc::string::ToString;

use super::super::{
    delta::DeltaBatch,
    state::{ParserState, SectionKind},
};
use super::LineProcessor;

impl LineProcessor {
    /// Process line in Script Info section
    pub(super) fn process_script_info_line(line: &str) -> DeltaBatch<'static> {
        let trimmed = line.trim();

        if let Some(colon_pos) = trimmed.find(':') {
            let _key = trimmed[..colon_pos].trim();
            let _value = trimmed[colon_pos + 1..].trim();
            // TODO: Handle script info fields
        }

        DeltaBatch::new()
    }

    /// Process line in Styles section
    pub(super) fn process_styles_line(&mut self, line: &str) -> DeltaBatch<'static> {
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
    pub(super) fn process_events_line(&mut self, line: &str) -> DeltaBatch<'static> {
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
    pub(super) fn process_binary_line(line: &str) -> DeltaBatch<'static> {
        let trimmed = line.trim();

        if trimmed.contains(':') {
            // Font/graphic filename declaration
        } else {
            // UU-encoded data line
        }

        DeltaBatch::new()
    }

    /// Process continuation of an event
    pub(super) fn process_event_continuation(
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
}
