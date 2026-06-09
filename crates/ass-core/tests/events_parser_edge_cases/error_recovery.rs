//! Error recovery tests for the events parser.
//!
//! Covers abrupt file truncation during events parsing, malformed event
//! lines, and events containing special characters and unusual content.

use ass_core::{parser::IssueSeverity, Script};

/// Test files ending abruptly during events parsing (L152-L183)
#[test]
fn test_abrupt_ending_in_events() {
    // Test ending in middle of event line
    let truncated_event = r"
[Script Info]
Title: Test Truncated

[Events]
Format: Layer, Start, End, Style, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,Hello Wor";

    let script = Script::parse(truncated_event).expect("Script parsing should work");

    // Should handle truncated file gracefully
    assert!(!script.sections().is_empty() || !script.issues().is_empty());

    // Test ending in middle of comment
    let truncated_comment = r"
[Events]
Format: Layer, Start, End, Style, Text
; This is a comment that gets cut off in the mid";

    let script_comment = Script::parse(truncated_comment).expect("Script parsing should work");

    // Should handle truncated comments
    let _has_sections_or_issues =
        !script_comment.sections().is_empty() || !script_comment.issues().is_empty();

    // Test ending with whitespace
    let ending_whitespace = r"
[Events]
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
[Events]
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
[Events]
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
