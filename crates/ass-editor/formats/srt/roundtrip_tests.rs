//! Integration tests for SRT import, export, and round-tripping.

use super::*;
use crate::formats::{FormatExporter, FormatImporter, FormatOptions};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec};

const SAMPLE_SRT: &str = "1\n00:00:00,000 --> 00:00:05,000\n<b>Hello</b> <i>World</i>!\n\n2\n00:00:06,000 --> 00:00:10,000\nThis is a <u>subtitle</u> with <font color=\"#FF0000\">red text</font>.\n\n3\n00:00:12,500 --> 00:00:15,750\nMultiple\nlines\nhere\n\n";

#[test]
fn test_srt_import_from_string() {
    let format = SrtFormat::new();
    let options = FormatOptions::default();

    let result = format.import_from_string(SAMPLE_SRT, &options);
    assert!(result.is_ok());

    let (document, format_result) = result.unwrap();
    assert!(format_result.success);
    assert_eq!(format_result.lines_processed, 3); // 3 subtitles
    assert!(document.text().contains("Hello"));
    assert!(document.text().contains("World"));
    assert!(document.text().contains(r"{\b1}"));
    assert!(document.text().contains(r"{\i1}"));
}

#[test]
fn test_srt_export_to_string() {
    let format = SrtFormat::new();
    let options = FormatOptions::default();

    // First import
    let (document, _) = format.import_from_string(SAMPLE_SRT, &options).unwrap();

    // Then export
    let result = format.export_to_string(&document, &options);
    assert!(result.is_ok());

    let (exported_content, format_result) = result.unwrap();
    assert!(format_result.success);
    assert!(exported_content.contains("Hello"));
    assert!(exported_content.contains("<b>"));
    assert!(exported_content.contains("<i>"));
    assert!(exported_content.contains("00:00:00,000 --> 00:00:05,000"));
}

#[test]
fn test_srt_roundtrip_basic() {
    let format = SrtFormat::new();
    let options = FormatOptions::default();

    let simple_srt = "1\n00:00:01,000 --> 00:00:03,000\nHello World\n\n";

    // Import -> Export -> Import
    let (document1, _) = format.import_from_string(simple_srt, &options).unwrap();
    let (exported_content, _) = format.export_to_string(&document1, &options).unwrap();

    // Verify basic structure is preserved
    assert!(exported_content.contains("Hello World"));
    assert!(exported_content.contains("00:00:01,000 --> 00:00:03,000"));
}

#[test]
fn test_srt_style_preservation() {
    let format = SrtFormat::new();
    let options = FormatOptions::default();

    let styled_srt = r#"1
00:00:00,000 --> 00:00:02,000
<b>Bold</b> and <i>italic</i> text

"#;

    let (document, _) = format.import_from_string(styled_srt, &options).unwrap();
    let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

    // Verify styles are preserved
    assert!(exported_content.contains("<b>Bold</b>"));
    assert!(exported_content.contains("<i>italic</i>"));
}

#[test]
fn test_srt_multiline_handling() {
    let format = SrtFormat::new();
    let options = FormatOptions::default();

    let multiline_srt = r#"1
00:00:00,000 --> 00:00:02,000
Line one
Line two
Line three

"#;

    let (document, _) = format.import_from_string(multiline_srt, &options).unwrap();
    let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

    // Verify multiline content is preserved
    assert!(exported_content.contains("Line one"));
    assert!(exported_content.contains("Line two"));
    assert!(exported_content.contains("Line three"));
}

#[test]
fn test_srt_error_handling() {
    let format = SrtFormat::new();
    let options = FormatOptions::default();

    let invalid_srt = "Invalid SRT content";
    let result = format.import_from_string(invalid_srt, &options);

    // Should handle gracefully and return warnings
    if let Ok((_, format_result)) = result {
        assert!(!format_result.warnings.is_empty());
    }
}

#[test]
fn test_srt_metadata_extraction() {
    let format = SrtFormat::new();
    let options = FormatOptions::default();

    let (_, format_result) = format.import_from_string(SAMPLE_SRT, &options).unwrap();

    assert_eq!(
        format_result.metadata.get("original_format"),
        Some(&"SRT".to_string())
    );
    assert_eq!(
        format_result.metadata.get("subtitles_count"),
        Some(&"3".to_string())
    );
    assert_eq!(
        format_result.metadata.get("encoding"),
        Some(&"UTF-8".to_string())
    );
}
