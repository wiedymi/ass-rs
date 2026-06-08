//! Accessors and timing queries for [`DialogueInfo`].
//!
//! Exposes duration, scoring, and timing-relationship helpers derived from a
//! previously analyzed dialogue event.

use super::{DialogueInfo, TimingRelation};
use crate::{
    analysis::events::{
        scoring::{get_performance_impact, PerformanceImpact},
        text_analysis::TextAnalysis,
    },
    parser::Event,
};

impl<'a> DialogueInfo<'a> {
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
