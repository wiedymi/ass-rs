//! Unit tests for `ResolvedStyle` construction and complexity scoring.

use super::test_support::create_test_style;
use super::*;

#[test]
fn resolved_style_creation() {
    let style = create_test_style();
    let resolved = ResolvedStyle::from_style(&style).unwrap();

    assert_eq!(resolved.name, "Test");
    assert_eq!(resolved.font_name(), "Arial");
    assert!((resolved.font_size() - 20.0).abs() < f32::EPSILON);
    assert_eq!(resolved.primary_color(), [255, 255, 255, 0]);
}

#[test]
fn complexity_scoring() {
    let mut style = create_test_style();

    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.complexity_score() < 50);

    style.fontsize = "100";
    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.complexity_score() >= 20);
}

#[test]
fn performance_issues_detection() {
    let mut style = create_test_style();

    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(!resolved.has_performance_issues());

    // Create a style with multiple performance-affecting properties
    style.fontsize = "120"; // >72: +20 points
    style.outline = "8"; // >4: +15 points
    style.shadow = "5"; // >3: +10 points
    style.angle = "45"; // !=0: +15 points
    style.scale_x = "150"; // !=100: +10 points
    style.bold = "1"; // +2 points
    style.italic = "1"; // +2 points
    style.underline = "1"; // +5 points
                           // Total: 79 points > 70 threshold

    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.has_performance_issues());
}

#[test]
fn resolved_style_from_style_with_invalid_values() {
    let mut style = create_test_style();

    // Test with invalid font size - should return error
    style.fontsize = "-10";
    assert!(ResolvedStyle::from_style(&style).is_err());

    style.fontsize = "abc";
    assert!(ResolvedStyle::from_style(&style).is_err());

    // Test with invalid color - should return error
    style.fontsize = "20"; // Reset to valid
    style.primary_colour = "invalid_color";
    assert!(ResolvedStyle::from_style(&style).is_err());

    // Test with invalid boolean flag - should return error
    style.primary_colour = "&HFFFFFF"; // Reset to valid
    style.bold = "2";
    assert!(ResolvedStyle::from_style(&style).is_err());
}

#[test]
fn complexity_calculation_all_branches() {
    let mut style = create_test_style();

    // Test baseline complexity
    let resolved = ResolvedStyle::from_style(&style).unwrap();
    let baseline_score = resolved.complexity_score();

    // Test font size increases complexity
    style.fontsize = "100"; // Large font size
    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.complexity_score() > baseline_score);

    // Test outline increases complexity
    style = create_test_style(); // Reset
    style.outline = "5"; // Large outline
    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.complexity_score() > baseline_score);

    // Test shadow increases complexity
    style = create_test_style(); // Reset
    style.shadow = "5"; // Large shadow
    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.complexity_score() > baseline_score);

    // Test scaling increases complexity
    style = create_test_style(); // Reset
    style.scale_x = "200"; // Non-default scaling
    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.complexity_score() > baseline_score);

    // Test angle increases complexity
    style = create_test_style(); // Reset
    style.angle = "45"; // Rotation
    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.complexity_score() > baseline_score);

    // Test formatting flags increase complexity
    style = create_test_style(); // Reset
    style.bold = "1";
    style.italic = "1";
    style.underline = "1";
    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.complexity_score() > baseline_score);
}

#[test]
fn complexity_score_capped_at_100() {
    let mut style = create_test_style();

    // Set all properties to maximum complexity values
    style.fontsize = "200"; // Large font
    style.outline = "10"; // Large outline
    style.shadow = "10"; // Large shadow
    style.scale_x = "200"; // Large scaling
    style.angle = "180"; // Large rotation
    style.bold = "1";
    style.italic = "1";
    style.underline = "1";
    style.strikeout = "1";

    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.complexity_score() <= 100); // Should be capped at 100
    assert!(resolved.complexity_score() > 50); // Should be high complexity
}
