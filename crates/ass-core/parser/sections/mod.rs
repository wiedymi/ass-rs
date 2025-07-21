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
//! ```rust
//! use ass_core::parser::sections::{ScriptInfoParser, StylesParser, EventsParser};
//!
//! // Parse script info section
//! let info_parser = ScriptInfoParser::new(source, start_pos, start_line);
//! let (section, version) = info_parser.parse()?;
//!
//! // Parse styles section
//! let styles_parser = StylesParser::new(source, start_pos, start_line);
//! let section = styles_parser.parse()?;
//!
//! // Parse events section
//! let events_parser = EventsParser::new(source, start_pos, start_line);
//! let section = events_parser.parse()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod events;
pub mod script_info;
pub mod styles;

pub use events::EventsParser;
pub use script_info::ScriptInfoParser;
pub use styles::StylesParser;

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
