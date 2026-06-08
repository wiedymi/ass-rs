//! Unit tests for formatting flags and property accessors of `ResolvedStyle`.

use super::test_support::create_test_style;
use super::*;

#[test]
fn text_formatting_flags_comprehensive() {
    let mut style = create_test_style();

    // Test all formatting combinations
    style.bold = "1";
    style.italic = "0";
    style.underline = "0";
    style.strikeout = "0";
    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.is_bold());
    assert!(!resolved.is_italic());
    assert!(!resolved.is_underline());
    assert!(!resolved.is_strike_out());
    assert_eq!(resolved.formatting(), TextFormatting::BOLD);

    // Test italic only
    style.bold = "0";
    style.italic = "1";
    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(!resolved.is_bold());
    assert!(resolved.is_italic());
    assert_eq!(resolved.formatting(), TextFormatting::ITALIC);

    // Test underline only
    style.italic = "0";
    style.underline = "1";
    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.is_underline());
    assert_eq!(resolved.formatting(), TextFormatting::UNDERLINE);

    // Test strikeout only
    style.underline = "0";
    style.strikeout = "1";
    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.is_strike_out());
    assert_eq!(resolved.formatting(), TextFormatting::STRIKE_OUT);

    // Test all flags combined
    style.bold = "1";
    style.italic = "1";
    style.underline = "1";
    style.strikeout = "1";
    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!(resolved.is_bold());
    assert!(resolved.is_italic());
    assert!(resolved.is_underline());
    assert!(resolved.is_strike_out());
    let expected = TextFormatting::BOLD
        | TextFormatting::ITALIC
        | TextFormatting::UNDERLINE
        | TextFormatting::STRIKE_OUT;
    assert_eq!(resolved.formatting(), expected);
}

#[test]
fn resolved_style_empty_font_name_uses_default() {
    let mut style = create_test_style();
    style.fontname = "";

    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert_eq!(resolved.font_name(), "Arial");
}

#[test]
#[allow(clippy::float_cmp)]
fn resolved_style_getters_comprehensive() {
    let style = create_test_style();
    let resolved = ResolvedStyle::from_style(&style).unwrap();

    // Test all getter methods
    assert_eq!(resolved.font_name(), "Arial");
    assert_eq!(resolved.font_size(), 20.0);
    assert_eq!(resolved.primary_color(), [255, 255, 255, 0]); // &H00FFFFFF
    assert!(!resolved.has_performance_issues()); // Low complexity

    let formatting = resolved.formatting();
    assert!(!resolved.is_bold());
    assert!(!resolved.is_italic());
    assert!(!resolved.is_underline());
    assert!(!resolved.is_strike_out());
    assert_eq!(formatting, TextFormatting::empty());
}

#[test]
fn resolved_style_spacing_getter() {
    let mut style = create_test_style();
    style.spacing = "5.5";

    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!((resolved.spacing() - 5.5).abs() < f32::EPSILON);
}

#[test]
fn resolved_style_angle_getter() {
    let mut style = create_test_style();
    style.angle = "45.5";

    let resolved = ResolvedStyle::from_style(&style).unwrap();
    assert!((resolved.angle() - 45.5).abs() < f32::EPSILON);
}
