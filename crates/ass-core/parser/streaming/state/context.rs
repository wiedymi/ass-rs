//! Streaming parser context tracking line numbers, sections, and formats.
//!
//! Defines [`StreamingContext`], maintaining parsing context including line
//! tracking, current section, and format information for incremental processing.

use alloc::string::String;

use super::SectionKind;

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
