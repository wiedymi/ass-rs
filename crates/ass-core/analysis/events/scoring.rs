//! Complexity scoring algorithms for ASS events
//!
//! Provides efficient calculation of animation complexity and rendering performance
//! impact scores for dialogue events. Scores are computed based on override tag
//! analysis, text content, and performance heuristics.
//!
//! # Scoring System
//!
//! - Animation Score: 0-10 scale based on tag complexity
//! - Complexity Score: 0-100 scale combining multiple factors
//! - Performance Impact: Categorical assessment for rendering optimization
//!
//! # Performance
//!
//! - Target: <0.1ms per scoring operation
//! - Memory: Zero allocations, operates on borrowed data
//! - Scalability: Linear complexity O(n) where n = tag count

use crate::analysis::events::tags::OverrideTag;

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

/// Calculate animation complexity score from override tags
///
/// Analyzes ASS override tags to determine animation complexity on a 0-10 scale.
/// Higher scores indicate more computationally expensive rendering operations.
///
/// # Scoring Rules
///
/// - Basic formatting (b, i, u, s, colors): 1 point each
/// - Positioning (pos, an, org): 2 points each
/// - Transforms (frx, fry, frz, fscx, fscy, etc.): 3 points each
/// - Movement (move): 4 points
/// - Transitions (t): 5 points
/// - Drawing (p): 8 points
///
/// # Arguments
///
/// * `tags` - Slice of parsed override tags
///
/// # Returns
///
/// Animation complexity score capped at 10
///
/// # Example
///
/// ```rust
/// # use ass_core::analysis::events::scoring::calculate_animation_score;
/// # use ass_core::analysis::events::tags::OverrideTag;
/// let tags = vec![];  // Empty for this example
/// let score = calculate_animation_score(&tags);
/// assert_eq!(score, 0);
/// ```
#[must_use]
pub fn calculate_animation_score(tags: &[OverrideTag<'_>]) -> u8 {
    tags.iter()
        .map(|tag| match tag.name() {
            "b" | "i" | "u" | "s" | "c" | "1c" | "2c" | "3c" | "4c" | "alpha" | "1a" | "2a"
            | "3a" | "4a" => 1,
            "frx" | "fry" | "frz" | "fscx" | "fscy" | "fsp" | "fad" | "fade" | "clip" | "iclip" => {
                3
            }
            "move" => 4,
            "t" | "pbo" => 5,
            "p" => 8,
            _ => 2,
        })
        .sum::<u8>()
        .min(10)
}

/// Calculate overall complexity score combining multiple factors
///
/// Computes a comprehensive complexity score (0-100) by combining animation
/// complexity, text length, and override tag count. Used for performance
/// optimization and rendering strategy selection.
///
/// # Scoring Components
///
/// - Animation score: Weighted 5x (0-50 points)
/// - Character count: Variable based on text length (0-50 points)
/// - Override count: Variable based on tag density (0-35 points)
///
/// # Arguments
///
/// * `animation_score` - Pre-calculated animation complexity (0-10)
/// * `char_count` - Number of characters in event text
/// * `override_count` - Number of override tags found
///
/// # Returns
///
/// Overall complexity score capped at 100
///
/// # Example
///
/// ```rust
/// # use ass_core::analysis::events::scoring::calculate_complexity_score;
/// let score = calculate_complexity_score(3, 100, 5);
/// assert!(score > 0 && score <= 100);
/// ```
#[must_use]
pub fn calculate_complexity_score(
    animation_score: u8,
    char_count: usize,
    override_count: usize,
) -> u8 {
    let mut score = u32::from(animation_score) * 5;

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

    (score.min(255) as u8).min(100)
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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::vec;

    #[test]
    fn test_animation_score_empty() {
        let tags = vec![];
        assert_eq!(calculate_animation_score(&tags), 0);
    }

    #[test]
    fn test_animation_score_basic_formatting() {
        // Mock tags would go here in a real implementation
        // Testing with empty for now since OverrideTag construction requires parser
        let tags = vec![];
        assert_eq!(calculate_animation_score(&tags), 0);
    }

    #[test]
    fn test_complexity_score_minimal() {
        let score = calculate_complexity_score(0, 10, 0);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_complexity_score_high() {
        let score = calculate_complexity_score(10, 1000, 50);
        assert_eq!(score, 100);
    }

    #[test]
    fn test_performance_impact_mapping() {
        assert_eq!(get_performance_impact(0), PerformanceImpact::Minimal);
        assert_eq!(get_performance_impact(30), PerformanceImpact::Low);
        assert_eq!(get_performance_impact(50), PerformanceImpact::Medium);
        assert_eq!(get_performance_impact(70), PerformanceImpact::High);
        assert_eq!(get_performance_impact(90), PerformanceImpact::Critical);
    }

    #[test]
    fn test_complexity_score_medium_char_count() {
        // Test the 501-1000 character range (line 127)
        let score = calculate_complexity_score(0, 750, 0);
        assert_eq!(score, 30); // Should match the 501..=1000 => 30 case
    }
}
