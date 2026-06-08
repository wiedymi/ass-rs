//! Tests for [`AssFormat`] export normalization behavior.

use super::*;
use crate::formats::{FormatExporter, FormatImporter, FormatOptions};

#[test]
fn test_ass_export_normalized() {
    let format = AssFormat::new();
    let options = FormatOptions {
        preserve_formatting: false,
        ..FormatOptions::default()
    };

    // Import with some non-standard formatting
    let messy_ass = r#"[Script Info]

Title: Test Script
ScriptType: v4.00+


[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
"#;

    let (document, _) = format.import_from_string(messy_ass, &options).unwrap();
    let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

    // Should be normalized (no extra blank lines)
    assert!(exported_content.contains("[Script Info]\nTitle: Test Script"));
    assert!(!exported_content.contains("\n\n\n")); // No triple newlines
    assert!(exported_content.contains("Hello World!"));
}

#[test]
fn test_ass_export_normalized_format_lines() {
    let format = AssFormat::new();
    let options = FormatOptions {
        preserve_formatting: false,
        ..FormatOptions::default()
    };

    // ASS without format lines - Script::to_ass_string() should add default ones
    let minimal_ass = r#"[Script Info]
Title: Test Script

[V4+ Styles]
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
"#;

    let (document, _) = format.import_from_string(minimal_ass, &options).unwrap();
    let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

    // The normalized output should NOT include format lines if the parser
    // didn't preserve them (ass-core's default behavior)
    assert!(exported_content.contains("[V4+ Styles]\n"));
    assert!(exported_content.contains("Style: Default,Arial,20"));
    assert!(exported_content.contains("[Events]\n"));
    assert!(exported_content.contains("Dialogue: 0,0:00:00.00,0:00:05.00"));
}
