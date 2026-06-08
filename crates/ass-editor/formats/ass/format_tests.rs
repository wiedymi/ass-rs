//! Tests for the [`AssFormat`] handler using a shared sample document.

use super::*;
use crate::formats::{FormatExporter, FormatImporter, FormatOptions};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec};
use std::io::Cursor;

const SAMPLE_ASS: &str = r#"[Script Info]
Title: Test Script
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
"#;

#[test]
fn test_ass_format_creation() {
    let format = AssFormat::new();
    let info = FormatImporter::format_info(&format);
    assert_eq!(info.name, "ASS");
    assert!(info.supports_styling);
    assert!(info.supports_positioning);
    assert!(format.can_import("ass"));
    assert!(format.can_export("ass"));
}

#[test]
fn test_ass_import_from_string() {
    let format = AssFormat::new();
    let options = FormatOptions::default();

    let result = format.import_from_string(SAMPLE_ASS, &options);
    assert!(result.is_ok());

    let (document, format_result) = result.unwrap();
    assert!(format_result.success);
    assert!(format_result.lines_processed > 0);
    assert!(document.text().contains("Hello World!"));
}

#[test]
fn test_ass_export_to_string() {
    let format = AssFormat::new();
    let options = FormatOptions::default();

    // First import
    let (document, _) = format.import_from_string(SAMPLE_ASS, &options).unwrap();

    // Then export
    let result = format.export_to_string(&document, &options);
    assert!(result.is_ok());

    let (exported_content, format_result) = result.unwrap();
    assert!(format_result.success);
    assert!(exported_content.contains("Hello World!"));
    assert!(exported_content.contains("[Script Info]"));
}

#[test]
fn test_ass_roundtrip() {
    let format = AssFormat::new();
    let options = FormatOptions::default();

    // Import -> Export -> Import
    let (document1, _) = format.import_from_string(SAMPLE_ASS, &options).unwrap();
    let (exported_content, _) = format.export_to_string(&document1, &options).unwrap();
    let (document2, _) = format
        .import_from_string(&exported_content, &options)
        .unwrap();

    // Should be equivalent
    assert_eq!(document1.text().trim(), document2.text().trim());
}

#[test]
fn test_ass_import_with_reader() {
    let format = AssFormat::new();
    let options = FormatOptions::default();
    let mut cursor = Cursor::new(SAMPLE_ASS.as_bytes());

    let result = format.import_from_reader(&mut cursor, &options);
    assert!(result.is_ok());

    let (document, format_result) = result.unwrap();
    assert!(format_result.success);
    assert!(document.text().contains("Test Script"));
}

#[test]
fn test_ass_export_with_writer() {
    let format = AssFormat::new();
    let options = FormatOptions::default();

    let (document, _) = format.import_from_string(SAMPLE_ASS, &options).unwrap();

    let mut buffer = Vec::new();
    let result = format.export_to_writer(&document, &mut buffer, &options);
    assert!(result.is_ok());

    let exported_content = String::from_utf8(buffer).unwrap();
    assert!(exported_content.contains("Hello World!"));
}

#[test]
fn test_ass_export_preserve_formatting() {
    let format = AssFormat::new();
    let options = FormatOptions {
        preserve_formatting: true,
        ..FormatOptions::default()
    };

    let (document, _) = format.import_from_string(SAMPLE_ASS, &options).unwrap();
    let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

    // Should preserve original formatting
    assert_eq!(exported_content.trim(), SAMPLE_ASS.trim());
}

#[test]
fn test_ass_metadata_extraction() {
    let format = AssFormat::new();
    let options = FormatOptions::default();

    let (_, format_result) = format.import_from_string(SAMPLE_ASS, &options).unwrap();

    assert_eq!(
        format_result.metadata.get("title"),
        Some(&"Test Script".to_string())
    );
    assert_eq!(
        format_result.metadata.get("script_type"),
        Some(&"v4.00+".to_string())
    );
    assert!(format_result.metadata.contains_key("sections"));
}
