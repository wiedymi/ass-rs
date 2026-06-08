//! Dialogue event analysis and information extraction
//!
//! Provides comprehensive analysis of individual ASS dialogue events including
//! timing validation, complexity scoring, and performance impact assessment.
//!
//! # Features
//!
//! - Zero-copy analysis using lifetime-generic design
//! - Animation complexity scoring (0-10 scale)
//! - Performance impact categorization
//! - Timing relationship detection between events
//! - Duration calculations in multiple formats
//!
//! # Performance
//!
//! - Target: <1ms per event analysis
//! - Memory: Minimal allocations via zero-copy spans
//! - Caching: Results stored for repeated queries

use crate::{analysis::events::text_analysis::TextAnalysis, parser::Event};

mod accessors;
mod analyze;

#[cfg(test)]
mod analyze_tests;
#[cfg(test)]
mod scoring_tests;
#[cfg(test)]
mod timing_tests;

/// Timing relationship between two dialogue events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingRelation {
    /// Events have no temporal overlap
    NoOverlap,
    /// Events partially overlap in time
    PartialOverlap,
    /// One event completely contains the other
    FullOverlap,
    /// Events have identical timing
    Identical,
}

/// Comprehensive analysis results for a dialogue event
///
/// Contains timing, styling, and content analysis for a single dialogue entry.
/// Uses zero-copy references to original event data where possible.
#[derive(Debug, Clone)]
pub struct DialogueInfo<'a> {
    /// Reference to original event
    event: &'a Event<'a>,
    /// Start time in centiseconds
    start_cs: u32,
    /// End time in centiseconds
    end_cs: u32,
    /// Animation complexity score (0-10)
    animation_score: u8,

    /// Overall rendering complexity (0-100)
    complexity_score: u8,
    /// Text analysis results
    text_info: TextAnalysis<'a>,
}
