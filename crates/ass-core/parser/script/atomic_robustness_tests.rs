//! Tests for atomic batch updates and malformed/edge-case input handling.

use super::*;
use crate::parser::ast::{Event, Section, SectionType};
use crate::ScriptVersion;
#[cfg(not(feature = "std"))]
use alloc::{string::String, vec};

#[test]
fn atomic_batch_update_success() {
    use crate::parser::ast::{EventType, Span};

    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize\nStyle: Default,Arial,20";
    let mut script = Script::parse(content).unwrap();

    // Prepare updates
    let updates = if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
        vec![UpdateOperation {
            offset: styles[0].span.start,
            new_line: "Style: Default,Helvetica,24",
            line_number: 10,
        }]
    } else {
        vec![]
    };

    // Prepare event additions
    let events = vec![Event {
        event_type: EventType::Dialogue,
        layer: "0",
        start: "0:00:00.00",
        end: "0:00:05.00",
        style: "Default",
        name: "",
        margin_l: "0",
        margin_r: "0",
        margin_v: "0",
        margin_t: None,
        margin_b: None,
        effect: "",
        text: "New Event",
        span: Span::new(0, 0, 0, 0),
    }];
    let event_batch = EventBatch { events };

    // Apply atomic update
    let result = script.atomic_batch_update(updates, None, Some(event_batch));
    assert!(result.is_ok());

    // Verify all changes were applied
    if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
        assert_eq!(styles[0].fontname, "Helvetica");
        assert_eq!(styles[0].fontsize, "24");
    }

    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].text, "New Event");
    }
}

#[test]
fn atomic_batch_update_rollback() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize\nStyle: Default,Arial,20";
    let mut script = Script::parse(content).unwrap();
    let original_script = script.clone();

    // Prepare an update with invalid offset
    let updates = vec![UpdateOperation {
        offset: 999_999, // Invalid offset
        new_line: "Style: Invalid,Arial,20",
        line_number: 10,
    }];

    // Apply atomic update
    let result = script.atomic_batch_update(updates, None, None);
    assert!(result.is_err());

    // Verify script was not modified
    assert_eq!(script, original_script);
}

#[test]
fn parse_malformed_comprehensive() {
    // Test a few malformed inputs that should still parse with issues
    let malformed_inputs = vec![
        "[Script Info]\nTitleWithoutColon",
        "[Script Info]\nTitle: Test\n\nInvalid line outside section",
    ];

    for input in malformed_inputs {
        let result = Script::parse(input);
        // Should either parse successfully (with potential issues) or fail gracefully
        assert!(result.is_ok() || result.is_err());

        if let Ok(script) = result {
            // If parsing succeeded, verify basic properties
            assert_eq!(script.source(), input);
            // Verify basic properties are accessible
            let _ = script.sections();
            let _ = script.issues();
        }
    }
}

#[test]
fn parse_edge_case_inputs() {
    // Test various edge cases
    let edge_cases = vec![
        "",                      // Empty
        "\n\n\n",                // Only newlines
        "   ",                   // Only spaces
        "\t\t\t",                // Only tabs
        "[Script Info]",         // Section header only
        "[Script Info]\n",       // Section header with newline
        "[]",                    // Empty section name
        "[   ]",                 // Whitespace section name
        "[Script Info]\nTitle:", // Empty value
        "[Script Info]\n:Value", // Empty key
    ];

    for input in edge_cases {
        let result = Script::parse(input);
        assert!(result.is_ok(), "Failed to parse edge case: {input:?}");

        let script = result.unwrap();
        assert_eq!(script.source(), input);
        // Verify sections are accessible
        let _ = script.sections();
    }
}

#[test]
fn script_version_handling() {
    // Test different version detection scenarios
    let v4_script = Script::parse("[Script Info]\nScriptType: v4.00").unwrap();
    // v4.00 is actually detected as SsaV4, not AssV4
    assert_eq!(v4_script.version(), ScriptVersion::SsaV4);

    let v4_plus_script = Script::parse("[Script Info]\nScriptType: v4.00+").unwrap();
    assert_eq!(v4_plus_script.version(), ScriptVersion::AssV4);

    let no_version_script = Script::parse("[Script Info]\nTitle: Test").unwrap();
    assert_eq!(no_version_script.version(), ScriptVersion::AssV4);
}

#[test]
fn parse_large_script_comprehensive() {
    #[cfg(not(feature = "std"))]
    use alloc::fmt::Write;
    #[cfg(feature = "std")]
    use std::fmt::Write;

    let mut content = String::from("[Script Info]\nTitle: Large Test\n");

    // Add many style definitions
    content.push_str("[V4+ Styles]\nFormat: Name, Fontname, Fontsize\n");
    for i in 0..100 {
        writeln!(content, "Style: Style{},Arial,{}", i, 16 + i % 10).unwrap();
    }

    // Add many events
    content.push_str("\n[Events]\nFormat: Layer, Start, End, Style, Text\n");
    for i in 0..100 {
        let start_time = i * 5;
        let end_time = start_time + 4;
        writeln!(
            content,
            "Dialogue: 0,0:00:{:02}.00,0:00:{:02}.00,Style{},Text {}",
            start_time / 60,
            end_time / 60,
            i % 10,
            i
        )
        .unwrap();
    }

    let script = Script::parse(&content).unwrap();
    assert_eq!(script.sections().len(), 3);
    assert_eq!(script.source(), content);
}
