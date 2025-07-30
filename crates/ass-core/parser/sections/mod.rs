//! Section-specific parsers for ASS script components.
//!
//! This module contains specialized parsers for each type of section in ASS scripts,
//! breaking down the monolithic parser into focused, maintainable components that
//! adhere to the 200 LOC limit while maintaining zero-copy performance.
//!
//! # Architecture
//!
//! Each section parser is responsible for:
//! - Parsing its specific section format and content
//! - Handling format specifications and field mappings
//! - Generating appropriate parse issues for validation
//! - Maintaining zero-copy lifetime-generic references
//!
//! # Performance
//!
//! All parsers are designed for optimal performance:
//! - Zero allocations via lifetime-generic spans
//! - Linear time complexity for section content
//! - Efficient format mapping and field extraction
//! - Minimal memory overhead during parsing
//!
//! # Example
//!
/// ```rust
/// use ass_core::parser::sections::{ScriptInfoParser, StylesParser, EventsParser};
///
/// let source = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname\nStyle: Default,Arial\n\n[Events]\nFormat: Layer, Start, End, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Hello";
/// let start_pos = 0;
/// let start_line = 1;
///
/// // Parse script info section
/// let info_parser = ScriptInfoParser::new(source, start_pos, start_line);
/// let (section, version, issues, pos, line) = info_parser.parse()?;
///
/// // Parse styles section
/// let styles_parser = StylesParser::new(source, start_pos, start_line);
/// let (section, format, issues, pos, line) = styles_parser.parse()?;
///
/// // Parse events section
/// let events_parser = EventsParser::new(source, start_pos, start_line);
/// let (section, format, issues, pos, line) = events_parser.parse()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub mod events;
pub mod script_info;
pub mod styles;

pub use events::EventsParser;
pub use script_info::ScriptInfoParser;
pub use styles::StylesParser;

use crate::parser::ast::{Section, SectionType};
use crate::parser::errors::{ParseError, ParseResult};
use alloc::vec::Vec;

/// Formats detected during initial parse
#[derive(Clone, Debug)]
pub struct SectionFormats<'a> {
    /// Format fields for styles section
    pub styles_format: Option<Vec<&'a str>>,
    /// Format fields for events section  
    pub events_format: Option<Vec<&'a str>>,
}

/// Parse a specific section type from text with context
///
/// # Errors
///
/// Returns [`ParseError::MissingFormat`] if required format is not provided
/// Returns [`ParseError::UnsupportedSection`] for Fonts/Graphics sections
/// Returns other parse errors from section-specific parsers
pub fn parse_section_with_context<'a>(
    section_type: SectionType,
    text: &'a str,
    line_offset: u32,
    existing_formats: &SectionFormats<'a>,
) -> ParseResult<Section<'a>> {
    match section_type {
        SectionType::ScriptInfo => {
            let parser = ScriptInfoParser::new(text, 0, line_offset as usize);
            let (section, ..) = parser.parse()?;
            Ok(section)
        }
        SectionType::Styles => {
            let format = existing_formats
                .styles_format
                .as_ref()
                .ok_or(ParseError::MissingFormat)?;
            let parser = StylesParser::with_format(text, format, 0, line_offset);
            let (section, ..) = parser.parse()?;
            Ok(section)
        }
        SectionType::Events => {
            let format = existing_formats
                .events_format
                .as_ref()
                .ok_or(ParseError::MissingFormat)?;
            let parser = EventsParser::with_format(text, format, 0, line_offset);
            let (section, ..) = parser.parse()?;
            Ok(section)
        }
        SectionType::Fonts | SectionType::Graphics => {
            // These sections are parsed as binary data
            Err(ParseError::UnsupportedSection(section_type))
        }
    }
}

/// Result type for section parsing with metadata
pub type SectionParseResult<'a> = (
    crate::parser::ast::Section<'a>,
    Option<Vec<&'a str>>,
    Vec<crate::parser::errors::ParseIssue>,
    usize,
    usize,
);

/// Result type for script info parsing with version detection
pub type ScriptInfoParseResult<'a> = (
    crate::parser::ast::Section<'a>,
    Option<crate::ScriptVersion>,
    Vec<crate::parser::errors::ParseIssue>,
    usize,
    usize,
);
