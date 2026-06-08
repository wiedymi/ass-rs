//! Events section parser for ASS scripts.
//!
//! Handles parsing of the `[Events]` section which contains dialogue, comments,
//! and other timed events with format specifications and event entries.

mod event_data;
mod parser;
mod static_parser;

#[cfg(test)]
mod static_tests;
#[cfg(test)]
mod streaming_tests;

use crate::parser::{errors::ParseIssue, position_tracker::PositionTracker};
use alloc::vec::Vec;

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
