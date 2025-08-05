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
            PerformanceImpact},
        text_analysis::TextAnalysis},
    parser::Event,
    utils::{parse_ass_time, CoreError},
    Result};

#[cfg(feature = "plugins")]
use crate::plugin::ExtensionRegistry;

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
    Identical}

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
    text_info: TextAnalysis<'a>}

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
    ///
    /// # Errors
    ///
    /// Returns an error if the event times are invalid or cannot be parsed.
    pub fn analyze(event: &'a Event<'a>) -> Result<Self> {
        #[cfg(feature = "plugins")]
        return Self::analyze_with_registry(event, None);
        #[cfg(not(feature = "plugins"))]
        return Self::analyze_impl(event);
    }

    /// Analyze a dialogue event with extension registry support
    ///
    /// Same as [`analyze`](Self::analyze) but allows custom tag handlers via registry.
    /// Unhandled tags fall back to standard processing.
    ///
    /// # Arguments
    ///
    /// * `event` - Dialogue event to analyze
    /// * `registry` - Optional registry for custom tag handlers
    ///
    /// # Returns
    ///
    /// `DialogueInfo` with complete analysis results, or error if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if the event times are invalid or cannot be parsed.
    #[cfg(feature = "plugins")]
    pub fn analyze_with_registry(
        event: &'a Event<'a>,
        registry: Option<&ExtensionRegistry>,
    ) -> Result<Self> {
        Self::analyze_impl_with_registry(event, registry)
    }

    /// Internal implementation without plugins support
    #[cfg(not(feature = "plugins"))]
    fn analyze_impl(event: &'a Event<'a>) -> Result<Self> {
        Self::analyze_impl_with_registry(event)
    }

    /// Internal implementation that supports optional registry
    fn analyze_impl_with_registry(
        event: &'a Event<'a>,
        #[cfg(feature = "plugins")] registry: Option<&ExtensionRegistry>,
    ) -> Result<Self> {
        let start_cs = parse_ass_time(event.start)?;

        let end_cs = parse_ass_time(event.end)?;

        if start_cs >= end_cs {
            return Err(CoreError::parse("Start time must be before end time"));
        }

        #[cfg(feature = "plugins")]
        let text_info = if let Some(registry) = registry {
            TextAnalysis::analyze_with_registry(event.text, Some(registry))?
        } else {
            TextAnalysis::analyze(event.text)?
        };

        #[cfg(not(feature = "plugins"))]
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
            text_info})
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Event, EventType, Span};
    #[cfg(not(feature = "std"))]
    
    #[test]
    fn dialogue_info_analyze_valid() {
        let event = Event {
            event_type: EventType::Dialogue,
            start: "0:00:00.00",
            end: "0:00:05.00",
            text: "Hello world!",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let info = DialogueInfo::analyze(&event).unwrap();
        assert_eq!(info.duration_ms(), 5000);
        assert_eq!(info.duration_cs(), 500);
        assert_eq!(info.start_time_cs(), 0);
        assert_eq!(info.end_time_cs(), 500);
    }

    #[test]
    fn dialogue_info_analyze_with_override_tags() {
        let event = Event {
            event_type: EventType::Dialogue,
            start: "0:00:00.00",
            end: "0:00:05.00",
            text: "Hello {\\b1}bold{\\b0} world",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let info = DialogueInfo::analyze(&event).unwrap();
        assert_eq!(info.duration_ms(), 5000);
        assert_eq!(info.duration_cs(), 500);
        assert_eq!(info.start_time_cs(), 0);
        assert_eq!(info.end_time_cs(), 500);
        assert!(!info.text_analysis().override_tags().is_empty());
    }

    #[test]
    fn dialogue_info_analyze_invalid_timing_start_after_end() {
        let event = Event {
            event_type: EventType::Dialogue,
            start: "0:00:05.00",
            end: "0:00:02.00",
            text: "Invalid timing",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        assert!(DialogueInfo::analyze(&event).is_err());
    }

    #[test]
    fn dialogue_info_analyze_invalid_timing_equal() {
        let event = Event {
            event_type: EventType::Dialogue,
            start: "0:00:03.00",
            end: "0:00:03.00",
            text: "Zero duration",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        assert!(DialogueInfo::analyze(&event).is_err());
    }

    #[test]
    fn dialogue_info_timing_relation_no_overlap() {
        let event1 = Event {
            event_type: EventType::Dialogue,
            start: "0:00:00.00",
            end: "0:00:05.00",
            text: "First event",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let event2 = Event {
            event_type: EventType::Dialogue,
            start: "0:00:06.00",
            end: "0:00:10.00",
            text: "Second event",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let info1 = DialogueInfo::analyze(&event1).unwrap();
        let info2 = DialogueInfo::analyze(&event2).unwrap();

        assert_eq!(info1.timing_relation(&info2), TimingRelation::NoOverlap);
        assert_eq!(info2.timing_relation(&info1), TimingRelation::NoOverlap);
    }

    #[test]
    fn dialogue_info_timing_relation_partial_overlap() {
        let event1 = Event {
            event_type: EventType::Dialogue,
            start: "0:00:02.50",
            end: "0:00:07.50",
            text: "Event on layer 1",
            layer: "1",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let event2 = Event {
            event_type: EventType::Dialogue,
            start: "0:00:02.00",
            end: "0:00:04.00",
            text: "Second",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let info1 = DialogueInfo::analyze(&event1).unwrap();
        let info2 = DialogueInfo::analyze(&event2).unwrap();

        assert_eq!(
            info1.timing_relation(&info2),
            TimingRelation::PartialOverlap
        );
        assert_eq!(
            info2.timing_relation(&info1),
            TimingRelation::PartialOverlap
        );
    }

    #[test]
    fn dialogue_info_timing_relation_full_overlap() {
        let event1 = Event {
            event_type: EventType::Dialogue,
            start: "0:00:00.00",
            end: "0:00:05.00",
            text: "Complex {\\b1}bold{\\b0} text",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let event2 = Event {
            event_type: EventType::Dialogue,
            start: "0:00:01.00",
            end: "0:00:03.00",
            text: "Inner",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let info1 = DialogueInfo::analyze(&event1).unwrap();
        let info2 = DialogueInfo::analyze(&event2).unwrap();

        assert_eq!(info1.timing_relation(&info2), TimingRelation::FullOverlap);
        assert_eq!(info2.timing_relation(&info1), TimingRelation::FullOverlap);
    }

    #[test]
    fn dialogue_info_timing_relation_identical() {
        let event1 = Event {
            event_type: EventType::Dialogue,
            start: "0:00:00.00",
            end: "0:00:05.00",
            text: "Container",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let event2 = Event {
            event_type: EventType::Dialogue,
            start: "0:00:00.00",
            end: "0:00:05.00",
            text: "Second",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let info1 = DialogueInfo::analyze(&event1).unwrap();
        let info2 = DialogueInfo::analyze(&event2).unwrap();

        assert_eq!(info1.timing_relation(&info2), TimingRelation::Identical);
        assert_eq!(info2.timing_relation(&info1), TimingRelation::Identical);
    }

    #[test]
    fn dialogue_info_overlaps_time_range() {
        let event = Event {
            event_type: EventType::Dialogue,
            start: "0:00:02.00",
            end: "0:00:05.00",
            text: "Test",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let info = DialogueInfo::analyze(&event).unwrap();

        // Event is 200-500 cs
        assert!(info.overlaps_time_range(100, 300)); // Overlaps start
        assert!(info.overlaps_time_range(400, 600)); // Overlaps end
        assert!(info.overlaps_time_range(300, 400)); // Contained within
        assert!(info.overlaps_time_range(100, 600)); // Contains event
        assert!(!info.overlaps_time_range(0, 200)); // Before event
        assert!(!info.overlaps_time_range(500, 600)); // After event
    }

    #[test]
    fn dialogue_info_animation_and_complexity_scoring() {
        let simple_event = Event {
            event_type: EventType::Dialogue,
            start: "0:00:00.00",
            end: "0:00:05.00",
            text: "Simple text",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let complex_event = Event {
            event_type: EventType::Dialogue,
            start: "0:00:00.00",
            end: "0:00:05.00",
            text: "Complex {\\b1}bold{\\b0} {\\i1}italic{\\i0} {\\u1}underline{\\u0} text",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let simple_info = DialogueInfo::analyze(&simple_event).unwrap();
        let complex_info = DialogueInfo::analyze(&complex_event).unwrap();

        assert!(simple_info.animation_score() < complex_info.animation_score());
        assert!(simple_info.complexity_score() < complex_info.complexity_score());
    }

    #[test]
    fn dialogue_info_override_count() {
        let event = Event {
            event_type: EventType::Dialogue,
            start: "0:00:00.00",
            end: "0:00:05.00",
            text: "{\\b1}Bold {\\k50}ka{\\k100}ra{\\k75}o{\\i1}ke {\\b0}text{\\i0}",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let info = DialogueInfo::analyze(&event).unwrap();
        assert_eq!(info.override_count(), 7);
    }

    #[test]
    fn dialogue_info_performance_impact() {
        let low_impact_event = Event {
            event_type: EventType::Dialogue,
            start: "0:00:00.00",
            end: "0:00:02.00",
            text: "Hi",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let info = DialogueInfo::analyze(&low_impact_event).unwrap();
        // Performance impact depends on complexity score
        let _impact = info.performance_impact();
    }

    #[test]
    fn dialogue_info_event_reference() {
        let event = Event {
            event_type: EventType::Dialogue,
            start: "0:00:00.00",
            end: "0:00:05.00",
            text: "Fast {\\b1}dynamic{\\b0} {\\i1}text{\\i0}!",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        let info = DialogueInfo::analyze(&event).unwrap();
        assert_eq!(
            info.event().text,
            "Fast {\\b1}dynamic{\\b0} {\\i1}text{\\i0}!"
        );
        assert_eq!(info.event().start, "0:00:00.00");
        assert_eq!(info.event().end, "0:00:05.00");
    }

    #[test]
    fn dialogue_info_analyze_invalid_start_time() {
        let event = Event {
            event_type: EventType::Dialogue,
            start: "invalid_time",
            end: "0:00:05.00",
            text: "Test",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        assert!(DialogueInfo::analyze(&event).is_err());
    }

    #[test]
    fn dialogue_info_analyze_invalid_end_time() {
        let event = Event {
            event_type: EventType::Dialogue,
            start: "0:00:00.00",
            end: "invalid_time",
            text: "Test",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            span: Span::new(0, 0, 0, 0)};

        assert!(DialogueInfo::analyze(&event).is_err());
    }
}
