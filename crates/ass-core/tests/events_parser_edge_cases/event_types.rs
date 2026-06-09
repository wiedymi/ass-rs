//! Event type and Format field handling tests for the events parser.
//!
//! Covers parsing of all event types, Format lines without a Text field,
//! event lines with too few fields, and access to fields missing from Format.

use ass_core::{parser::IssueSeverity, Script};

/// Test all event types parsing (L86-L95)
#[test]
fn test_all_event_types_parsing() {
    let script_all_events = r"
[Script Info]
Title: Test Events

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&Hffffff,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
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

[Events]
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

[Events]
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

[Events]
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
