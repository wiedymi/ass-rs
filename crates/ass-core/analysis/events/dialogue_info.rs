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
    analysis::events::text_analysis::TextAnalysis,
    parser::Event,
    utils::{parse_ass_time, CoreError},
    Result,
};
use alloc::format;

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

/// Performance impact category for rendering complexity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PerformanceImpact {
    /// Minimal impact - simple static text
    Minimal,
    /// Low impact - basic formatting
    Low,
    /// Medium impact - animations or complex styling
    Medium,
    /// High impact - many animations or large text
    High,
    /// Critical impact - may cause performance issues
    Critical,
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
    /// Number of override blocks
    #[allow(dead_code)]
    override_count: usize,
    /// Overall rendering complexity (0-100)
    complexity_score: u8,
    /// Text analysis results
    text_info: TextAnalysis<'a>,
}

impl<'a> DialogueInfo<'a> {
    /// Analyze a dialogue event comprehensively
    ///
    /// Performs timing parsing, text analysis, and complexity scoring.
    /// Results are cached within the returned DialogueInfo instance.
    ///
    /// # Arguments
    ///
    /// * `event` - Dialogue event to analyze
    ///
    /// # Returns
    ///
    /// DialogueInfo with complete analysis results, or error if parsing fails.
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
        let start_cs = parse_ass_time(event.start)
            .map_err(|e| CoreError::parse(format!("Invalid start time: {}", e)))?;

        let end_cs = parse_ass_time(event.end)
            .map_err(|e| CoreError::parse(format!("Invalid end time: {}", e)))?;

        if start_cs >= end_cs {
            return Err(CoreError::parse("Start time must be before end time"));
        }

        let text_info = TextAnalysis::analyze(event.text)?;
        let animation_score = Self::calculate_animation_score(text_info.override_tags());
        let complexity_score = Self::calculate_complexity_score(
            animation_score,
            text_info.char_count(),
            text_info.override_tags().len(),
        );

        Ok(Self {
            event,
            start_cs,
            end_cs,
            animation_score,
            override_count: text_info.override_tags().len(),
            complexity_score,
            text_info,
        })
    }

    /// Get event duration in milliseconds
    pub fn duration_ms(&self) -> u32 {
        (self.end_cs - self.start_cs) * 10
    }

    /// Get event duration in centiseconds
    pub fn duration_cs(&self) -> u32 {
        self.end_cs - self.start_cs
    }

    /// Get start time in centiseconds
    pub fn start_time_cs(&self) -> u32 {
        self.start_cs
    }

    /// Get end time in centiseconds
    pub fn end_time_cs(&self) -> u32 {
        self.end_cs
    }

    /// Get animation complexity score (0-10)
    pub fn animation_score(&self) -> u8 {
        self.animation_score
    }

    /// Get overall complexity score (0-100)
    pub fn complexity_score(&self) -> u8 {
        self.complexity_score
    }

    /// Get text analysis results
    pub fn text_analysis(&self) -> &TextAnalysis<'a> {
        &self.text_info
    }

    /// Get performance impact category
    pub fn performance_impact(&self) -> PerformanceImpact {
        match self.complexity_score {
            0..=20 => PerformanceImpact::Minimal,
            21..=40 => PerformanceImpact::Low,
            41..=60 => PerformanceImpact::Medium,
            61..=80 => PerformanceImpact::High,
            _ => PerformanceImpact::Critical,
        }
    }

    /// Check timing relationship with another event
    pub fn timing_relation(&self, other: &DialogueInfo<'_>) -> TimingRelation {
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
    pub fn overlaps_time_range(&self, start_cs: u32, end_cs: u32) -> bool {
        !(self.end_cs <= start_cs || end_cs <= self.start_cs)
    }

    /// Get reference to original event
    pub fn event(&self) -> &'a Event<'a> {
        self.event
    }

    /// Calculate animation complexity from override tags
    fn calculate_animation_score(
        tags: &[crate::analysis::events::text_analysis::OverrideTag<'_>],
    ) -> u8 {
        tags.iter()
            .map(|tag| match tag.name() {
                "b" | "i" | "u" | "s" | "c" | "1c" | "2c" | "3c" | "4c" | "alpha" | "1a" | "2a"
                | "3a" | "4a" => 1,
                "pos" | "an" | "a" | "org" => 2,
                "frx" | "fry" | "frz" | "fscx" | "fscy" | "fsp" | "fad" | "fade" | "clip"
                | "iclip" => 3,
                "move" => 4,
                "t" => 5,
                "pbo" => 5,
                "p" => 8,
                _ => 2,
            })
            .sum::<u8>()
            .min(10)
    }

    /// Calculate overall complexity score
    fn calculate_complexity_score(
        animation_score: u8,
        char_count: usize,
        override_count: usize,
    ) -> u8 {
        let mut score = animation_score as u32 * 5;

        score += match char_count {
            0..=50 => 0,
            51..=200 => 5,
            201..=500 => 15,
            501..=1000 => 30,
            _ => 50,
        };

        score += match override_count {
            0 => 0,
            1..=5 => 5,
            6..=15 => 15,
            16..=30 => 25,
            _ => 35,
        };

        (score as u8).min(100)
    }
}
