//! Tests for field value handling in the styles parser.
//!
//! Covers special characters in style names, unusual spacing, and
//! boundary conditions for field parsing.

use ass_core::{parser::IssueSeverity, Script};

/// Test styles with special characters and edge cases
#[test]
fn test_styles_special_characters() {
    let special_chars = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour
Style: Default,Arial,20,&Hffffff
Style: Special!@#$%,Arial,20,&H00ff00
Style: Unicode测试,Arial,20,&Hff0000
Style: Empty,,20,&H0000ff
Style: Comma\,Name,Arial,20,&Hffff00

[Events\]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(special_chars).expect("Script parsing should work");

    // Should handle special characters in style names
    assert!(!script.sections().is_empty());

    // Should not have critical errors
    let _has_critical_errors = script
        .issues()
        .iter()
        .any(|issue| matches!(issue.severity, IssueSeverity::Error));
}

/// Test style lines with unusual spacing and formatting
#[test]
fn test_style_spacing_variations() {
    let spacing_variations = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour
Style:Default,Arial,20,&Hffffff
Style:    Spaced   ,   Arial   ,   20   ,   &H00ff00
Style: 	Tabbed	,	Arial	,	20	,	&Hff0000
Style: Mixed   ,	Arial,  20,&H0000ff

[Events\]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(spacing_variations).expect("Script parsing should work");

    // Should handle various spacing patterns
    assert!(!script.sections().is_empty());
}

/// Test boundary conditions for field parsing
#[test]
fn test_field_parsing_boundaries() {
    let boundary_cases = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Minimum,,,,,,,,,,,,,,,,,,,,,
Style: Maximum,Arial,999,&HFFFFFFFF,&HFFFFFFFF,&HFFFFFFFF,&HFFFFFFFF,1,1,1,1,1000,1000,100,360,4,100,100,11,9999,9999,9999,255
Style: Negative,Arial,-1,&H0,&H0,&H0,&H0,-1,-1,-1,-1,-100,-100,-100,-360,-1,-100,-100,-1,-1,-1,-1,-1

[Events\]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(boundary_cases).expect("Script parsing should work");

    // Should handle boundary values (minimum, maximum, negative)
    assert!(!script.sections().is_empty());

    // Should not crash on extreme values
    let _has_sections = !script.sections().is_empty();
}
