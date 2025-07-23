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

use crate::{
    analysis::events::{
        scoring::{
            calculate_animation_score, calculate_complexity_score, get_performance_impact,
            PerformanceImpact,
        },
        text_analysis::TextAnalysis,
    },
    parser::Event,
    utils::{parse_ass_time, CoreError},
    Result,
};

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

impl<'a> DialogueInfo<'a> {
    /// Analyze a dialogue event comprehensively
    ///
    /// Performs timing parsing, text analysis, and complexity scoring.
    /// Results are cached within the returned `DialogueInfo` instance.
    ///
    /// # Arguments
    ///
    /// * `event` - Dialogue event to analyze
    ///
    /// # Returns
    ///
    /// `DialogueInfo` with complete analysis results, or error if parsing fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ass_core::analysis::events::dialogue_info::DialogueInfo;
    /// # use ass_core::parser::Event;
    /// let event = Event {
    ///     start: "0:00:00.00",
    ///     end: "0:00:05.00",
    ///     text: "Hello {\\b1}World{\\b0}!",
    ///     ..Default::default()
    /// };
    ///
    /// let info = DialogueInfo::analyze(&event)?;
    /// assert_eq!(info.duration_ms(), 5000);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn analyze(event: &'a Event<'a>) -> Result<Self> {
        let start_cs = parse_ass_time(event.start)?;

        let end_cs = parse_ass_time(event.end)?;

        if start_cs >= end_cs {
            return Err(CoreError::parse("Start time must be before end time"));
        }

        let text_info = TextAnalysis::analyze(event.text)?;
        let animation_score = calculate_animation_score(text_info.override_tags());
        let complexity_score = calculate_complexity_score(
            animation_score,
            text_info.char_count(),
            text_info.override_tags().len(),
        );

        Ok(Self {
            event,
            start_cs,
            end_cs,
            animation_score,

            complexity_score,
            text_info,
        })
    }

    /// Get event duration in milliseconds
    #[must_use]
    pub const fn duration_ms(&self) -> u32 {
        (self.end_cs - self.start_cs) * 10
    }

    /// Get event duration in centiseconds
    #[must_use]
    pub const fn duration_cs(&self) -> u32 {
        self.end_cs - self.start_cs
    }

    /// Get start time in centiseconds
    #[must_use]
    pub const fn start_time_cs(&self) -> u32 {
        self.start_cs
    }

    /// Get end time in centiseconds
    #[must_use]
    pub const fn end_time_cs(&self) -> u32 {
        self.end_cs
    }

    /// Get animation complexity score (0-10)
    #[must_use]
    pub const fn animation_score(&self) -> u8 {
        self.animation_score
    }

    /// Get overall complexity score (0-100)
    #[must_use]
    pub const fn complexity_score(&self) -> u8 {
        self.complexity_score
    }

    /// Get text analysis results
    #[must_use]
    pub const fn text_analysis(&self) -> &TextAnalysis<'a> {
        &self.text_info
    }

    /// Get performance impact category
    #[must_use]
    pub const fn performance_impact(&self) -> PerformanceImpact {
        get_performance_impact(self.complexity_score)
    }

    /// Check timing relationship with another event
    #[must_use]
    pub const fn timing_relation(&self, other: &DialogueInfo<'_>) -> TimingRelation {
        if self.start_cs == other.start_cs && self.end_cs == other.end_cs {
            TimingRelation::Identical
        } else if self.end_cs <= other.start_cs || other.end_cs <= self.start_cs {
            TimingRelation::NoOverlap
        } else if (self.start_cs <= other.start_cs && self.end_cs >= other.end_cs)
            || (other.start_cs <= self.start_cs && other.end_cs >= self.end_cs)
        {
            TimingRelation::FullOverlap
        } else {
            TimingRelation::PartialOverlap
        }
    }

    /// Check if event overlaps with time range
    #[must_use]
    pub const fn overlaps_time_range(&self, start_cs: u32, end_cs: u32) -> bool {
        !(self.end_cs <= start_cs || end_cs <= self.start_cs)
    }

    /// Get override tag count from text analysis
    #[must_use]
    pub fn override_count(&self) -> usize {
        self.text_info.override_tags().len()
    }

    /// Get reference to original event
    #[must_use]
    pub const fn event(&self) -> &'a Event<'a> {
        self.event
    }
}
