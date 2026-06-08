//! Unit tests for `apply_resolution_scaling` behavior on `ResolvedStyle`.

use super::test_support::create_test_style;
use super::*;

#[test]
fn resolved_style_apply_resolution_scaling_symmetric() {
    let style = create_test_style();
    let mut resolved = ResolvedStyle::from_style(&style).unwrap();

    // Apply 2x scaling
    resolved.apply_resolution_scaling(2.0, 2.0);

    assert!((resolved.font_size() - 40.0).abs() < f32::EPSILON); // 20 * 2
    assert!((resolved.spacing() - 0.0).abs() < f32::EPSILON); // 0 * 2
    assert!((resolved.outline() - 4.0).abs() < f32::EPSILON); // 2 * 2
    assert!((resolved.shadow() - 0.0).abs() < f32::EPSILON); // 0 * 2
    assert_eq!(resolved.margin_l(), 20); // 10 * 2
    assert_eq!(resolved.margin_r(), 20); // 10 * 2
    assert_eq!(resolved.margin_t(), 20); // 10 * 2
    assert_eq!(resolved.margin_b(), 20); // 10 * 2
}

#[test]
fn resolved_style_apply_resolution_scaling_asymmetric() {
    let mut style = create_test_style();
    style.spacing = "4";
    style.shadow = "2";
    style.margin_l = "10";
    style.margin_r = "20";
    style.margin_v = "30";

    let mut resolved = ResolvedStyle::from_style(&style).unwrap();

    // Apply asymmetric scaling (3x horizontal, 2x vertical)
    resolved.apply_resolution_scaling(3.0, 2.0);

    // Average scale for font/outline/shadow: (3 + 2) / 2 = 2.5
    assert!((resolved.font_size() - 50.0).abs() < f32::EPSILON); // 20 * 2.5
    assert!((resolved.spacing() - 12.0).abs() < f32::EPSILON); // 4 * 3
    assert!((resolved.outline() - 5.0).abs() < f32::EPSILON); // 2 * 2.5
    assert!((resolved.shadow() - 5.0).abs() < f32::EPSILON); // 2 * 2.5
    assert_eq!(resolved.margin_l(), 30); // 10 * 3
    assert_eq!(resolved.margin_r(), 60); // 20 * 3
    assert_eq!(resolved.margin_t(), 60); // 30 * 2
    assert_eq!(resolved.margin_b(), 60); // 30 * 2
}

#[test]
fn resolved_style_apply_resolution_scaling_downscale() {
    let style = create_test_style();
    let mut resolved = ResolvedStyle::from_style(&style).unwrap();

    // Apply 0.5x scaling (downscale)
    resolved.apply_resolution_scaling(0.5, 0.5);

    assert!((resolved.font_size() - 10.0).abs() < f32::EPSILON); // 20 * 0.5
    assert!((resolved.spacing() - 0.0).abs() < f32::EPSILON); // 0 * 0.5
    assert!((resolved.outline() - 1.0).abs() < f32::EPSILON); // 2 * 0.5
    assert!((resolved.shadow() - 0.0).abs() < f32::EPSILON); // 0 * 0.5
    assert_eq!(resolved.margin_l(), 5); // 10 * 0.5
    assert_eq!(resolved.margin_r(), 5); // 10 * 0.5
    assert_eq!(resolved.margin_t(), 5); // 10 * 0.5
    assert_eq!(resolved.margin_b(), 5); // 10 * 0.5
}

#[test]
fn resolved_style_apply_resolution_scaling_updates_complexity() {
    let mut style = create_test_style();
    style.fontsize = "30"; // Not quite large enough to trigger complexity

    let mut resolved = ResolvedStyle::from_style(&style).unwrap();
    let initial_complexity = resolved.complexity_score();

    // Apply 3x scaling to push font size over complexity threshold
    resolved.apply_resolution_scaling(3.0, 3.0);

    assert!((resolved.font_size() - 90.0).abs() < f32::EPSILON); // 30 * 3
    assert!(resolved.complexity_score() > initial_complexity); // Should increase due to large font
}

#[test]
fn resolved_style_apply_resolution_scaling_preserves_other_properties() {
    let mut style = create_test_style();
    style.bold = "1";
    style.italic = "1";
    style.primary_colour = "&H00FF0000"; // Red
    style.angle = "45";

    let mut resolved = ResolvedStyle::from_style(&style).unwrap();
    let initial_color = resolved.primary_color();
    let initial_angle = resolved.angle;
    let initial_formatting = resolved.formatting();

    // Apply scaling
    resolved.apply_resolution_scaling(2.0, 2.0);

    // These properties should not be affected by scaling
    assert_eq!(resolved.primary_color(), initial_color);
    assert!((resolved.angle - initial_angle).abs() < f32::EPSILON);
    assert_eq!(resolved.formatting(), initial_formatting);
    assert!(resolved.is_bold());
    assert!(resolved.is_italic());
}
