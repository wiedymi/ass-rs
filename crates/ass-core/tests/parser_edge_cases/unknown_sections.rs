//! Tests for unknown-section handling and recovery suggestions.
//!
//! Covers typo detection, content-based section suggestions, multiple
//! consecutive unknown sections, and empty section content.

use ass_core::{parser::IssueSeverity, Script};

/// Test unknown section with suggestion logic (L187-L212)
#[test]
fn test_unknown_section_with_suggestions() {
    let script_with_typo = r"
[Script Info]
Title: Test

[Event]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(script_with_typo).expect("Script parsing should work");

    // Should have warning for unknown section
    assert!(script
        .issues()
        .iter()
        .any(|issue| matches!(issue.severity, IssueSeverity::Warning)));

    // Should have info suggestion
    assert!(script
        .issues()
        .iter()
        .any(|issue| matches!(issue.severity, IssueSeverity::Info)));
}

/// Test `skip_to_next_section` suggestion logic (L245-L281)
#[test]
fn test_skip_to_next_section_suggestions() {
    // Test Style: line suggesting V4+ Styles section
    let script_style_suggestion = r"
[Unknown Section]
Style: Default,Arial,20,&Hffffff,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(script_style_suggestion).expect("Script parsing should work");

    assert!(script.issues().iter().any(|issue| {
        matches!(issue.severity, IssueSeverity::Info) && issue.message.contains("V4+ Styles")
    }));

    // Test Dialogue: line suggesting Events section
    let script_dialogue_suggestion = r"
[Wrong Events]
Dialogue: 0:00:00.00,0:00:05.00,Default,Test text
";

    let script = Script::parse(script_dialogue_suggestion).expect("Script parsing should work");

    assert!(script.issues().iter().any(|issue| {
        matches!(issue.severity, IssueSeverity::Info) && issue.message.contains("Events")
    }));

    // Test Title: line suggesting Script Info section
    let script_title_suggestion = r"
[Bad Section]
Title: My Subtitle File
";

    let script = Script::parse(script_title_suggestion).expect("Script parsing should work");

    assert!(script.issues().iter().any(|issue| {
        matches!(issue.severity, IssueSeverity::Info) && issue.message.contains("Script Info")
    }));
}

/// Test multiple consecutive unknown sections
#[test]
fn test_multiple_unknown_sections() {
    let multi_unknown = r"
[Unknown1]
Some content

[Unknown2]
More content

[Unknown3]
Even more content

[Events]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(multi_unknown).expect("Script parsing should work");

    // Should have multiple warnings for unknown sections
    assert!(
        script
            .issues()
            .iter()
            .filter(|issue| {
                matches!(issue.severity, IssueSeverity::Warning)
                    && issue.message.contains("Unknown section")
            })
            .count()
            >= 3
    );
}

/// Test section header without content
#[test]
fn test_empty_section_content() {
    let empty_content = r"
[Script Info]

[Events]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(empty_content).expect("Script parsing should work");

    // Should parse successfully even with empty sections
    assert!(!script.sections().is_empty());
}
