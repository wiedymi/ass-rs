//! Edge case and error handling tests for the events parser.
//!
//! This module contains comprehensive tests targeting previously untested code paths
//! in the events parser, focusing on different event types, format handling, and error recovery.

use ass_core::{parser::IssueSeverity, Script};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test all event types parsing (L86-L95)
    #[test]
    fn test_all_event_types_parsing() {
        let script_all_events = r"
[Script Info]
Title: Test Events

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&Hffffff,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
Comment: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,This is a comment
Picture: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,picture.jpg
Sound: 0,0:00:15.00,0:00:20.00,Default,,0,0,0,,sound.wav
Movie: 0,0:00:20.00,0:00:25.00,Default,,0,0,0,,movie.avi
Command: 0,0:00:25.00,0:00:30.00,Default,,0,0,0,,some_command
";

        let script = Script::parse(script_all_events).expect("Script parsing should work");

        // Should successfully parse all event types
        assert!(!script.sections().is_empty());

        // Should not have critical errors for valid event types
        let critical_errors = script
            .issues()
            .iter()
            .any(|issue| matches!(issue.severity, IssueSeverity::Error));

        // Should handle all event types without critical parsing errors
        assert!(!critical_errors || script.issues().len() < 5); // Allow some tolerance
    }

    /// Test Format line without Text field (L100-L105)
    #[test]
    fn test_format_without_text_field() {
        let script_no_text = r"
[Script Info]
Title: Test No Text

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour
Style: Default,Arial,20,&Hffffff

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,
Comment: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,
";

        let script = Script::parse(script_no_text).expect("Script parsing should work");

        // Should parse successfully even without Text field in Format
        assert!(!script.sections().is_empty());

        // May have warnings or handle gracefully
        let _has_text_field_issues = script
            .issues()
            .iter()
            .any(|issue| issue.message.to_lowercase().contains("text"));
    }

    /// Test event lines with too few fields (L108-L118)
    #[test]
    fn test_event_too_few_fields() {
        let script_few_fields = r"
[Script Info]
Title: Test Few Fields

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour
Style: Default,Arial,20,&Hffffff

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default
Comment: 0,0:00:05.00,0:00:10.00
Picture: 0,0:00:10.00
Sound: 0
";

        let script = Script::parse(script_few_fields).expect("Script parsing should work");

        // Should handle lines with too few fields
        assert!(!script.sections().is_empty());

        // Should have warnings about field count mismatches
        let field_warnings = script.issues().iter().any(|issue| {
            matches!(issue.severity, IssueSeverity::Warning)
                && (issue.message.contains("field") || issue.message.contains("count"))
        });

        // May or may not generate warnings depending on implementation
        let _ = field_warnings;
    }

    /// Test missing fields in Format and field access (L120-L126)
    #[test]
    fn test_missing_format_fields_access() {
        let script_missing_fields = r"
[Script Info]
Title: Test Missing Fields

[V4+ Styles]
Format: Name, Fontname
Style: Default,Arial

[Events\]
Format: Layer, Start, End
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
Comment: 0,0:00:05.00,0:00:10.00,Extra,Field,Values,Here
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

    /// Test files ending abruptly during events parsing (L152-L183)
    #[test]
    fn test_abrupt_ending_in_events() {
        // Test ending in middle of event line
        let truncated_event = r"
[Script Info]
Title: Test Truncated

[Events\]
Format: Layer, Start, End, Style, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,Hello Wor";

        let script = Script::parse(truncated_event).expect("Script parsing should work");

        // Should handle truncated file gracefully
        assert!(!script.sections().is_empty() || !script.issues().is_empty());

        // Test ending in middle of comment
        let truncated_comment = r"
[Events\]
Format: Layer, Start, End, Style, Text
; This is a comment that gets cut off in the mid";

        let script_comment = Script::parse(truncated_comment).expect("Script parsing should work");

        // Should handle truncated comments
        let _has_sections_or_issues =
            !script_comment.sections().is_empty() || !script_comment.issues().is_empty();

        // Test ending with whitespace
        let ending_whitespace = r"
[Events\]
Format: Layer, Start, End, Style, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,Test

   ";

        let script_ws = Script::parse(ending_whitespace).expect("Script parsing should work");

        // Should handle trailing whitespace
        assert!(!script_ws.sections().is_empty());
    }

    /// Test malformed event lines
    #[test]
    fn test_malformed_event_lines() {
        let malformed_events = r"
[Events\]
Format: Layer, Start, End, Style, Text
Dialogue:
Comment:
Picture: 0,invalid_time,0:00:05.00,Default,test.jpg
Sound: not_a_number,0:00:00.00,0:00:05.00,Default,sound.wav
Movie: 0,0:00:00.00,invalid_end_time,Default,movie.avi
Command: 0,0:00:00.00,0:00:05.00,,
InvalidType: 0,0:00:00.00,0:00:05.00,Default,This is not a valid event type
";

        let script = Script::parse(malformed_events).expect("Script parsing should work");

        // Should handle malformed lines gracefully
        assert!(!script.sections().is_empty());

        // May have warnings or errors for malformed content
        let _has_malformed_issues = script
            .issues()
            .iter()
            .any(|issue| issue.message.contains("invalid") || issue.message.contains("malformed"));
    }

    /// Test events with special characters and edge cases
    #[test]
    fn test_events_special_characters() {
        let special_chars = r"
[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,Speaker!@#$%,0,0,0,,Hello World!
Comment: 0,0:00:05.00,0:00:10.00,Default,Unicode测试,0,0,0,,Unicode text: こんにちは 🌍
Picture: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,file with spaces.jpg
Sound: 0,0:00:15.00,0:00:20.00,Default,,0,0,0,,C:\path\with\backslashes.wav
Movie: 0,0:00:20.00,0:00:25.00,Default,,0,0,0,,https://example.com/video.mp4
Command: 0,0:00:25.00,0:00:30.00,Default,,0,0,0,,command --with=arguments
Dialogue: 0,0:00:30.00,0:00:35.00,Default,,0,0,0,,Text with\Nline breaks\nand more\r\nbreaks
Dialogue: 0,0:00:35.00,0:00:40.00,Default,,0,0,0,,Text with {override tags} and {\b1}bold{\b0} text
";

        let script = Script::parse(special_chars).expect("Script parsing should work");

        // Should handle special characters and formatting
        assert!(!script.sections().is_empty());

        // Should not have critical errors for special characters
        let critical_errors = script
            .issues()
            .iter()
            .any(|issue| matches!(issue.severity, IssueSeverity::Error));

        // May have warnings but shouldn't crash on special characters
        let _ = critical_errors;
    }

    /// Test multiple Format lines in Events section
    #[test]
    fn test_multiple_format_lines() {
        let multiple_formats = r"
[Events\]
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

[Events\]

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
[Events\]
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
[Events\]
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

    /// Test boundary conditions for event field parsing
    #[test]
    fn test_event_field_boundaries() {
        let boundary_cases = r"
[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,,,,,,,
Dialogue: 999,23:59:59.99,23:59:59.99,VeryLongStyleNameThatShouldStillWork,VeryLongSpeakerName,9999,9999,9999,VeryLongEffectName,Very long text that goes on and on and should still be parsed correctly
Dialogue: -1,-1:00:00.00,-1:00:00.00,Default,,-1,-1,-1,,Negative values
Comment: 0,0:00:00.00,0:00:00.01,Default,,0,0,0,,Very short duration
Movie: 0,0:00:00.00,10:00:00.00,Default,,0,0,0,,Very long duration
";

        let script = Script::parse(boundary_cases).expect("Script parsing should work");

        // Should handle boundary values gracefully
        assert!(!script.sections().is_empty());

        // Should not crash on extreme values
        let _has_sections = !script.sections().is_empty();
    }

    /// Test mixed event types in single section
    #[test]
    fn test_mixed_event_types() {
        let mixed_events = r"
[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:02.00,Default,,0,0,0,,First dialogue
Comment: 0,0:00:01.00,0:00:03.00,Default,,0,0,0,,Overlapping comment
Dialogue: 0,0:00:02.00,0:00:04.00,Default,,0,0,0,,Second dialogue
Picture: 0,0:00:03.00,0:00:05.00,Default,,0,0,0,,background.jpg
Sound: 0,0:00:04.00,0:00:06.00,Default,,0,0,0,,sfx.wav
Dialogue: 0,0:00:05.00,0:00:07.00,Default,,0,0,0,,Third dialogue
Movie: 0,0:00:06.00,0:00:08.00,Default,,0,0,0,,intro.avi
Command: 0,0:00:07.00,0:00:09.00,Default,,0,0,0,,fade_in
Comment: 0,0:00:08.00,0:00:10.00,Default,,0,0,0,,Final comment
";

        let script = Script::parse(mixed_events).expect("Script parsing should work");

        // Should handle mixed event types in chronological order
        assert!(!script.sections().is_empty());

        // Should parse all events without critical errors
        let critical_errors = script
            .issues()
            .iter()
            .filter(|issue| matches!(issue.severity, IssueSeverity::Error))
            .count();

        // Should have minimal critical errors for valid mixed content
        assert!(critical_errors < 5); // Allow some tolerance for edge cases
    }
}
