//! Tests for ASS serialization via `to_ass_string` across section combinations.

use super::*;
use crate::parser::ast::Section;
#[cfg(not(feature = "std"))]
use alloc::vec;

// Helper function to create a test script with all sections
fn create_test_script() -> Script<'static> {
    use crate::parser::ast::{Event, EventType, Font, Graphic, ScriptInfo, Span, Style};
    use crate::ScriptVersion;

    let sections = vec![
        Section::ScriptInfo(ScriptInfo {
            fields: vec![
                ("Title", "Test Script"),
                ("ScriptType", "v4.00+"),
                ("WrapStyle", "0"),
                ("ScaledBorderAndShadow", "yes"),
                ("YCbCr Matrix", "None"),
            ],
            span: Span::new(0, 0, 0, 0),
        }),
        Section::Styles(vec![Style::default()]),
        Section::Events(vec![
            Event {
                event_type: EventType::Dialogue,
                text: "Hello, world!",
                ..Event::default()
            },
            Event {
                event_type: EventType::Comment,
                start: "0:00:05.00",
                end: "0:00:10.00",
                text: "This is a comment",
                ..Event::default()
            },
        ]),
        Section::Fonts(vec![Font {
            filename: "custom.ttf",
            data_lines: vec!["begin 644 custom.ttf", "M'XL...", "end"],
            span: Span::new(0, 0, 0, 0),
        }]),
        Section::Graphics(vec![Graphic {
            filename: "logo.png",
            data_lines: vec!["begin 644 logo.png", "M89PNG...", "end"],
            span: Span::new(0, 0, 0, 0),
        }]),
    ];

    Script {
        source: "",
        version: ScriptVersion::AssV4Plus,
        sections,
        issues: vec![],
        styles_format: Some(vec![
            "Name",
            "Fontname",
            "Fontsize",
            "PrimaryColour",
            "SecondaryColour",
            "OutlineColour",
            "BackColour",
            "Bold",
            "Italic",
            "Underline",
            "StrikeOut",
            "ScaleX",
            "ScaleY",
            "Spacing",
            "Angle",
            "BorderStyle",
            "Outline",
            "Shadow",
            "Alignment",
            "MarginL",
            "MarginR",
            "MarginV",
            "Encoding",
        ]),
        events_format: Some(vec![
            "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect",
            "Text",
        ]),
        change_tracker: ChangeTracker::default(),
    }
}

#[test]
fn script_to_ass_string_complete() {
    let script = create_test_script();
    let ass_string = script.to_ass_string();

    // Verify all sections are present with correct content
    assert!(ass_string.contains("[Script Info]\n"));
    assert!(ass_string.contains("Title: Test Script\n"));
    assert!(ass_string.contains("\n[V4+ Styles]\n"));
    assert!(ass_string.contains("Format: Name, Fontname, Fontsize"));
    assert!(ass_string.contains("\n[Events]\n"));
    assert!(ass_string.contains("Format: Layer, Start, End, Style"));
    assert!(ass_string.contains("Dialogue: 0,0:00:00.00,0:00:00.00,Default,,0,0,0,,Hello, world!"));
    assert!(
        ass_string.contains("Comment: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,This is a comment")
    );
    assert!(ass_string.contains("\n[Fonts]\n"));
    assert!(ass_string.contains("fontname: custom.ttf\n"));
    assert!(ass_string.contains("\n[Graphics]\n"));
    assert!(ass_string.contains("filename: logo.png\n"));
}

#[test]
fn script_to_ass_string_minimal() {
    use crate::parser::ast::{ScriptInfo, Span};
    use crate::ScriptVersion;

    let script = Script {
        source: "",
        version: ScriptVersion::AssV4Plus,
        sections: vec![Section::ScriptInfo(ScriptInfo {
            fields: vec![("Title", "Minimal")],
            span: Span::new(0, 0, 0, 0),
        })],
        issues: vec![],
        styles_format: None,
        events_format: None,
        change_tracker: ChangeTracker::default(),
    };

    let ass_string = script.to_ass_string();

    assert!(ass_string.contains("[Script Info]\n"));
    assert!(ass_string.contains("Title: Minimal\n"));
    assert!(!ass_string.contains("[V4+ Styles]"));
    assert!(!ass_string.contains("[Events]"));
    assert!(!ass_string.contains("[Fonts]"));
    assert!(!ass_string.contains("[Graphics]"));
}

#[test]
fn script_to_ass_string_empty() {
    use crate::ScriptVersion;

    let script = Script {
        source: "",
        version: ScriptVersion::AssV4Plus,
        sections: vec![],
        issues: vec![],
        styles_format: None,
        events_format: None,
        change_tracker: ChangeTracker::default(),
    };

    let ass_string = script.to_ass_string();

    // Empty script should produce empty string
    assert_eq!(ass_string, "");
}

#[test]
fn script_to_ass_string_with_custom_format_lines() {
    use crate::parser::ast::{Event, EventType, Span};
    use crate::ScriptVersion;

    let script = Script {
        source: "",
        version: ScriptVersion::AssV4Plus,
        sections: vec![Section::Events(vec![Event {
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
            text: "Test",
            span: Span::new(0, 0, 0, 0),
        }])],
        issues: vec![],
        styles_format: None,
        events_format: Some(vec!["Start", "End", "Text"]),
        change_tracker: ChangeTracker::default(),
    };

    let ass_string = script.to_ass_string();

    assert!(ass_string.contains("[Events]\n"));
    assert!(ass_string.contains("Format: Start, End, Text\n"));
    assert!(ass_string.contains("Dialogue: 0:00:00.00,0:00:05.00,Test\n"));
}
