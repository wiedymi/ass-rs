//! Tests for error recovery in the styles parser.
//!
//! Covers abrupt file endings, empty styles sections, and sections
//! containing only comments.

use ass_core::Script;

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
