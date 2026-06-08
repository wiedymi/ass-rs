//! Performance impact categorization for complexity scores.
//!
//! Defines the [`PerformanceImpact`] category enum and the mapping from a
//! numerical complexity score to a categorical rendering impact level used
//! for rendering optimization decisions.

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

/// Determine performance impact category from complexity score
///
/// Maps numerical complexity scores to categorical performance impact levels
/// for easier rendering optimization decisions.
///
/// # Arguments
///
/// * `complexity_score` - Overall complexity score (0-100)
///
/// # Returns
///
/// Performance impact category for rendering optimization
///
/// # Example
///
/// ```rust
/// # use ass_core::analysis::events::scoring::{get_performance_impact, PerformanceImpact};
/// let impact = get_performance_impact(75);
/// assert_eq!(impact, PerformanceImpact::High);
/// ```
#[must_use]
pub const fn get_performance_impact(complexity_score: u8) -> PerformanceImpact {
    match complexity_score {
        0..=20 => PerformanceImpact::Minimal,
        21..=40 => PerformanceImpact::Low,
        41..=60 => PerformanceImpact::Medium,
        61..=80 => PerformanceImpact::High,
        _ => PerformanceImpact::Critical,
    }
}
