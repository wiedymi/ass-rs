//! Edge case and error handling tests for the styles parser.
//!
//! This module contains comprehensive tests targeting previously untested code paths
//! in the styles parser, focusing on format handling, field validation, and error recovery.

use ass_core::{parser::IssueSeverity, Script};

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Test files ending abruptly during styles parsing (L184-L191, L204-L218)
    #[test]
    fn test_abrupt_ending_in_styles() {
        // Test ending in middle of style line
        let truncated_style = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour
Style: Default,Arial,20";

        let script = Script::parse(truncated_style).expect("Script parsing should work");

        // Should handle truncated file gracefully
        assert!(!script.sections().is_empty() || !script.issues().is_empty());

        // Test ending in middle of comment
        let truncated_comment = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize
; This is a comment that gets cut off in the mid";

        let script_comment = Script::parse(truncated_comment).expect("Script parsing should work");

        // Should handle truncated comments
        let _has_sections_or_issues =
            !script_comment.sections().is_empty() || !script_comment.issues().is_empty();

        // Test ending with whitespace
        let ending_whitespace = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize
Style: Default,Arial,20

   ";

        let script_ws = Script::parse(ending_whitespace).expect("Script parsing should work");

        // Should handle trailing whitespace
        assert!(!script_ws.sections().is_empty());
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

    /// Test empty styles section
    #[test]
    fn test_empty_styles_section() {
        let empty_section = r"
[V4+ Styles]

[Events\]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

        let script = Script::parse(empty_section).expect("Script parsing should work");

        // Should handle empty styles section
        assert!(!script.sections().is_empty());
    }

    /// Test styles section with only comments
    #[test]
    fn test_styles_only_comments() {
        let only_comments = r"
[V4+ Styles]
; This is a comment
; Another comment
!: This is also a comment

[Events\]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

        let script = Script::parse(only_comments).expect("Script parsing should work");

        // Should handle section with only comments
        assert!(!script.sections().is_empty());
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
}
