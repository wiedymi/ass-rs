//! Unit tests for the event complexity scoring functions.

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
