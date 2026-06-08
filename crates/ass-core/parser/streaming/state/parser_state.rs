//! Streaming parser state machine for incremental processing.
//!
//! Defines [`ParserState`], tracking the current parsing context to handle
//! partial data and section boundaries correctly during streaming.

use super::SectionKind;

/// Streaming parser state for incremental processing
///
/// Tracks current parsing context to handle partial data and
/// section boundaries correctly during streaming.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserState {
    /// Expecting section header or document start
    ExpectingSection,
    /// Currently parsing a specific section
    InSection(SectionKind),
    /// Parsing an event with potentially incomplete data
    InEvent {
        /// Which section type we're in
        section: SectionKind,
        /// Number of fields processed so far
        fields_seen: usize,
    },
}

impl ParserState {
    /// Check if currently inside a section
    #[must_use]
    pub const fn is_in_section(&self) -> bool {
        matches!(self, Self::InSection(_) | Self::InEvent { .. })
    }

    /// Get current section kind if in a section
    #[must_use]
    pub const fn current_section(&self) -> Option<SectionKind> {
        match self {
            Self::ExpectingSection => None,
            Self::InSection(kind) => Some(*kind),
            Self::InEvent { section, .. } => Some(*section),
        }
    }

    /// Transition to new section
    pub fn enter_section(&mut self, kind: SectionKind) {
        *self = Self::InSection(kind);
    }

    /// Begin event parsing within section
    pub fn enter_event(&mut self, section: SectionKind) {
        *self = Self::InEvent {
            section,
            fields_seen: 0,
        };
    }

    /// Exit current section
    pub fn exit_section(&mut self) {
        *self = Self::ExpectingSection;
    }
}
