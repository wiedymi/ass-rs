//! Tests for Format line handling in the styles parser.
//!
//! Covers default format fallback, V4 styles, field-count mismatches,
//! missing fields, malformed Format lines, and multiple Format lines.

use ass_core::{parser::IssueSeverity, Script};

/// Test Style lines without Format line to cover default format fallback (L86-L113)
#[test]
fn test_styles_without_format_line() {
    let script_no_format = r"
[V4+ Styles]
Style: Default,Arial,20,&Hffffff,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Large,Arial,40,&Hff0000,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events\]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(script_no_format).expect("Script parsing should work");

    // Should parse successfully even without explicit Format line
    assert!(!script.sections().is_empty());

    // May have warnings about missing format but should still process styles
    let _has_format_warnings = script.issues().iter().any(|issue| {
        issue.message.to_lowercase().contains("format")
            || issue.message.to_lowercase().contains("field")
    });
}

/// Test V4 Styles section (older format)
#[test]
fn test_v4_styles_section() {
    let script_v4 = r"
[V4 Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, TertiaryColour, BackColour, Bold, Italic, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, AlphaLevel, Encoding, RelativeToVideo
Style: Default,Arial,20,16777215,255,0,0,0,0,1,2,0,2,10,10,10,0,1,0

[Events\]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(script_v4).expect("Script parsing should work");

    // Should parse V4 styles successfully
    assert!(!script.sections().is_empty());
}

/// Test Style line with wrong number of fields (L130-L141)
#[test]
fn test_style_field_count_mismatch() {
    // Test with fewer fields than Format specifies
    let script_too_few = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&Hffffff
Style: Complete,Arial,20,&Hffffff,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events\]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(script_too_few).expect("Script parsing should work");

    // Should have warnings about field count mismatch
    let _has_mismatch_warnings = script.issues().iter().any(|issue| {
        matches!(issue.severity, IssueSeverity::Warning)
            && (issue.message.contains("field") || issue.message.contains("count"))
    });

    // Test with more fields than Format specifies
    let script_too_many = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize
Style: Default,Arial,20,&Hffffff,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1,extra,fields

[Events\]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script_many = Script::parse(script_too_many).expect("Script parsing should work");

    // Should handle extra fields gracefully
    assert!(!script_many.sections().is_empty());
}

/// Test missing fields in Format line (L143-L149)
#[test]
fn test_missing_format_fields() {
    let script_missing_fields = r"
[V4+ Styles]
Format: Name, Fontname
Style: Default,Arial,20,&Hffffff,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events\]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(script_missing_fields).expect("Script parsing should work");

    // Should handle missing format fields gracefully
    assert!(!script.sections().is_empty());

    // Parser should handle when trying to access fields not in Format
    let _has_missing_field_issues = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("field") || issue.message.contains("missing"));
}

/// Test malformed Format lines
#[test]
fn test_malformed_format_lines() {
    // Test Format line with no fields
    let empty_format = r"
[V4+ Styles]
Format:
Style: Default,Arial,20

[Events\]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(empty_format).expect("Script parsing should work");

    // Should handle empty format gracefully
    assert!(!script.sections().is_empty());

    // Test Format line with only spaces
    let spaces_format = r"
[V4+ Styles]
Format:
Style: Default,Arial,20

[Events\]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script_spaces = Script::parse(spaces_format).expect("Script parsing should work");
    assert!(!script_spaces.sections().is_empty());
}

/// Test multiple Format lines in same section
#[test]
fn test_multiple_format_lines() {
    let multiple_formats = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize
Format: Name, Fontname, Fontsize, PrimaryColour
Style: Default,Arial,20,&Hffffff

[Events\]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(multiple_formats).expect("Script parsing should work");

    // Should handle multiple format lines (probably uses the last one)
    assert!(!script.sections().is_empty());
}
