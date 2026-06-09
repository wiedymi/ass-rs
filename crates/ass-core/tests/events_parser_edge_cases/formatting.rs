//! Format line, empty section, and whitespace handling tests.
//!
//! Covers multiple Format lines, empty event sections, comment-only
//! sections, and event lines with unusual spacing and tab characters.

use ass_core::Script;

/// Test multiple Format lines in Events section
#[test]
fn test_multiple_format_lines() {
    let multiple_formats = r"
[Events]
Format: Layer, Start, End, Style, Text
Format: Layer, Start, End, Style, Name, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,Speaker,Hello World!
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Comment: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,This is a comment
";

    let script = Script::parse(multiple_formats).expect("Script parsing should work");

    // Should handle multiple format lines (probably uses the last one)
    assert!(!script.sections().is_empty());
}

/// Test empty events section
#[test]
fn test_empty_events_section() {
    let empty_section = r"
[Script Info]
Title: Test Empty Events

[Events]

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour
Style: Default,Arial,20,&Hffffff
";

    let script = Script::parse(empty_section).expect("Script parsing should work");

    // Should handle empty events section
    assert!(!script.sections().is_empty());
}

/// Test events section with only comments and whitespace
#[test]
fn test_events_only_comments() {
    let only_comments = r"
[Events]
; This is a comment
; Another comment
!: This is also a comment

; More comments
	; Indented comment

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour
Style: Default,Arial,20,&Hffffff
";

    let script = Script::parse(only_comments).expect("Script parsing should work");

    // Should handle section with only comments
    assert!(!script.sections().is_empty());
}

/// Test event lines with unusual spacing and formatting
#[test]
fn test_event_spacing_variations() {
    let spacing_variations = r"
[Events]
Format: Layer, Start, End, Style, Text
Dialogue:0,0:00:00.00,0:00:05.00,Default,No spaces after colon
Dialogue:   0   ,   0:00:05.00   ,   0:00:10.00   ,   Default   ,   Lots of spaces
Dialogue:	0	,	0:00:10.00	,	0:00:15.00	,	Default	,	Tabs everywhere
Dialogue: 0 , 0:00:15.00 , 0:00:20.00 , Default , Mixed   spacing	patterns
Comment:0,0:00:20.00,0:00:25.00,Default,Minimal spacing
";

    let script = Script::parse(spacing_variations).expect("Script parsing should work");

    // Should handle various spacing patterns
    assert!(!script.sections().is_empty());
}
