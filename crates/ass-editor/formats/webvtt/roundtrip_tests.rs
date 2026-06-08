//! Integration tests for WebVTT import, export, and round-tripping.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec};

use crate::formats::{FormatExporter, FormatImporter, FormatOptions};

const SAMPLE_WEBVTT: &str = r#"WEBVTT

1
00:00:00.000 --> 00:00:05.000
<b>Hello</b> <i>World</i>!

2
00:00:06.000 --> 00:00:10.000 align:center
This is a <u>subtitle</u> with <c.red>red text</c>.

3
00:12:30.500 --> 00:15:45.750 line:20% position:50%
<v Speaker>Multiple</v>
lines with positioning

"#;

#[test]
fn test_webvtt_import_from_string() {
    let format = WebVttFormat::new();
    let options = FormatOptions::default();

    let result = format.import_from_string(SAMPLE_WEBVTT, &options);
    assert!(result.is_ok());

    let (document, format_result) = result.unwrap();
    assert!(format_result.success);
    assert_eq!(format_result.lines_processed, 3); // 3 cues
    assert!(document.text().contains("Hello"));
    assert!(document.text().contains("World"));
    assert!(document.text().contains(r"{\b1}"));
    assert!(document.text().contains(r"{\i1}"));
}

#[test]
fn test_webvtt_export_to_string() {
    let format = WebVttFormat::new();
    let options = FormatOptions::default();

    // First import
    let (document, _) = format.import_from_string(SAMPLE_WEBVTT, &options).unwrap();

    // Then export
    let result = format.export_to_string(&document, &options);
    assert!(result.is_ok());

    let (exported_content, format_result) = result.unwrap();
    assert!(format_result.success);
    assert!(exported_content.contains("WEBVTT"));
    assert!(exported_content.contains("Hello"));
    assert!(exported_content.contains("<b>"));
    assert!(exported_content.contains("<i>"));
    assert!(exported_content.contains("00:00:00.000 --> 00:00:05.000"));
}

#[test]
fn test_webvtt_roundtrip_basic() {
    let format = WebVttFormat::new();
    let options = FormatOptions::default();

    let simple_vtt = "WEBVTT\n\n1\n00:00:01.000 --> 00:00:03.000\nHello World\n\n";

    // Import -> Export -> Import
    let (document1, _) = format.import_from_string(simple_vtt, &options).unwrap();
    let (exported_content, _) = format.export_to_string(&document1, &options).unwrap();

    // Verify basic structure is preserved
    assert!(exported_content.contains("WEBVTT"));
    assert!(exported_content.contains("Hello World"));
    assert!(exported_content.contains("00:00:01.000 --> 00:00:03.000"));
}

#[test]
fn test_webvtt_style_preservation() {
    let format = WebVttFormat::new();
    let options = FormatOptions::default();

    let styled_vtt =
        "WEBVTT\n\n1\n00:00:00.000 --> 00:00:02.000\n<b>Bold</b> and <i>italic</i> text\n\n";

    let (document, _) = format.import_from_string(styled_vtt, &options).unwrap();
    let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

    // Verify styles are preserved
    assert!(exported_content.contains("<b>Bold</b>"));
    assert!(exported_content.contains("<i>italic</i>"));
}

#[test]
fn test_webvtt_positioning_support() {
    let format = WebVttFormat::new();
    let options = FormatOptions::default();

    let positioned_vtt =
        "WEBVTT\n\n1\n00:00:00.000 --> 00:00:02.000 line:20% position:50%\nPositioned text\n\n";

    let (document, _) = format.import_from_string(positioned_vtt, &options).unwrap();

    // Should parse without errors and preserve positioning info in ASS format
    assert!(document.text().contains("Positioned text"));
}

#[test]
fn test_webvtt_multiline_handling() {
    let format = WebVttFormat::new();
    let options = FormatOptions::default();

    let multiline_vtt =
        "WEBVTT\n\n1\n00:00:00.000 --> 00:00:02.000\nLine one\nLine two\nLine three\n\n";

    let (document, _) = format.import_from_string(multiline_vtt, &options).unwrap();
    let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

    // Verify multiline content is preserved
    assert!(exported_content.contains("Line one"));
    assert!(exported_content.contains("Line two"));
    assert!(exported_content.contains("Line three"));
}

#[test]
fn test_webvtt_error_handling() {
    let format = WebVttFormat::new();
    let options = FormatOptions::default();

    let invalid_vtt = "Invalid WebVTT content";
    let result = format.import_from_string(invalid_vtt, &options);

    // Should handle gracefully and return warnings
    if let Ok((_, format_result)) = result {
        assert!(!format_result.warnings.is_empty());
    }
}

#[test]
fn test_webvtt_metadata_extraction() {
    let format = WebVttFormat::new();
    let options = FormatOptions::default();

    let (_, format_result) = format.import_from_string(SAMPLE_WEBVTT, &options).unwrap();

    assert_eq!(
        format_result.metadata.get("original_format"),
        Some(&"WebVTT".to_string())
    );
    assert_eq!(
        format_result.metadata.get("cues_count"),
        Some(&"3".to_string())
    );
    assert_eq!(
        format_result.metadata.get("encoding"),
        Some(&"UTF-8".to_string())
    );
}
