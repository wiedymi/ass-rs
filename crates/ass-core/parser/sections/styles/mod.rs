//! Styles section parser for ASS scripts.
//!
//! Handles parsing of the [V4+ Styles] section which contains style definitions
//! with format specifications and style entries.

use crate::parser::{errors::ParseIssue, position_tracker::PositionTracker};
use alloc::vec::Vec;

mod parse_internal;
mod parse_line;
mod parser;

#[cfg(test)]
mod parse_tests;
#[cfg(test)]
mod style_line_tests;

/// Parser for [V4+ Styles] section content
///
/// Parses format definitions and style entries from the styles section.
/// Uses format mapping to handle different field orderings and missing fields.
///
/// # Performance
///
/// - Time complexity: O(n * m) for n styles and m fields per style
/// - Memory: Zero allocations via lifetime-generic spans
/// - Target: <1ms for typical style sections with 50 styles
pub struct StylesParser<'a> {
    /// Position tracker for accurate span generation
    tracker: PositionTracker<'a>,
    /// Parse issues and warnings collected during parsing
    issues: Vec<ParseIssue>,
    /// Format fields for the styles section
    format: Option<Vec<&'a str>>,
}
