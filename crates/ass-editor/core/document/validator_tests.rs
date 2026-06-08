//! Tests for validator integration and format import/export

use super::*;
use crate::core::position::Position;

#[test]
fn test_validator_integration() {
    let mut doc = EditorDocument::from_content(
        "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test"
    ).unwrap();

    // Should have result after comprehensive validation
    let result = doc.validate_comprehensive().unwrap();
    assert!(result.is_valid);

    // Modify document
    doc.insert(Position::new(doc.len_bytes()), "\nComment: Test")
        .unwrap();

    // Force validate should work
    let result2 = doc.force_validate().unwrap();
    assert!(result2.is_valid);
}

#[test]
fn test_validator_configuration() {
    let mut doc = EditorDocument::new();

    // Configure validator
    let config = crate::utils::validator::ValidatorConfig {
        max_issues: 5,
        enable_performance_hints: false,
        ..Default::default()
    };
    doc.set_validator_config(config);

    // Validator should be configured
    // We can't directly check the cache anymore, but configuration should work
    assert!(doc.is_valid_cached().is_ok());
}

#[test]
fn test_validator_with_invalid_document() {
    let mut doc = EditorDocument::from_content("Invalid content").unwrap();

    // Comprehensive validation should find issues
    let result = doc.validate_comprehensive().unwrap();
    assert!(!result.issues.is_empty());

    // Should have warnings about missing sections
    let warnings =
        result.issues_with_severity(crate::utils::validator::ValidationSeverity::Warning);
    assert!(!warnings.is_empty());
}

#[test]
#[cfg(feature = "formats")]
fn test_format_import_export() {
    // Test SRT import
    let srt_content = "1\n00:00:00,000 --> 00:00:05,000\nHello world!";
    let doc = EditorDocument::import_format(
        srt_content,
        Some(crate::utils::formats::SubtitleFormat::SRT),
    )
    .unwrap();
    assert!(doc.text().contains("Hello world!"));
    assert!(doc.has_events().unwrap());

    // Test export to WebVTT
    let options = crate::utils::formats::ConversionOptions::default();
    let webvtt = doc
        .export_format(crate::utils::formats::SubtitleFormat::WebVTT, &options)
        .unwrap();
    assert!(webvtt.starts_with("WEBVTT"));
    assert!(webvtt.contains("00:00:00.000 --> 00:00:05.000"));
    assert!(webvtt.contains("Hello world!"));
}
