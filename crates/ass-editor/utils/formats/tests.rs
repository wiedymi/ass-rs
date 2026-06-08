//! Tests for subtitle format detection and conversion.

use super::*;
use crate::core::EditorDocument;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

#[test]
fn test_format_detection() {
    assert_eq!(
        SubtitleFormat::from_extension("ass"),
        Some(SubtitleFormat::ASS)
    );
    assert_eq!(
        SubtitleFormat::from_extension("srt"),
        Some(SubtitleFormat::SRT)
    );
    assert_eq!(
        SubtitleFormat::from_extension("vtt"),
        Some(SubtitleFormat::WebVTT)
    );
    assert_eq!(SubtitleFormat::from_extension("unknown"), None);

    assert_eq!(
        SubtitleFormat::from_content("[Script Info]\nTitle: Test"),
        SubtitleFormat::ASS
    );
    assert_eq!(
        SubtitleFormat::from_content("WEBVTT\n\n00:00.000 --> 00:05.000"),
        SubtitleFormat::WebVTT
    );
    assert_eq!(
        SubtitleFormat::from_content("1\n00:00:00,000 --> 00:00:05,000\nHello"),
        SubtitleFormat::SRT
    );
}

#[test]
fn test_srt_import() {
    let srt_content = r#"1
00:00:00,000 --> 00:00:05,000
Hello <i>world</i>!

2
00:00:05,000 --> 00:00:10,000
This is a <b>test</b>."#;

    let result = FormatConverter::import(srt_content, Some(SubtitleFormat::SRT)).unwrap();

    assert!(result.contains("[Script Info]"));
    assert!(result.contains("[Events]"));
    assert!(result.contains("Hello {\\i1}world{\\i0}!"));
    assert!(result.contains("This is a {\\b1}test{\\b0}."));
}

#[test]
fn test_webvtt_import() {
    let webvtt_content = r#"WEBVTT

00:00:00.000 --> 00:00:05.000
Hello <i>world</i>!

00:00:05.000 --> 00:00:10.000
This is a test."#;

    let result = FormatConverter::import(webvtt_content, Some(SubtitleFormat::WebVTT)).unwrap();

    assert!(result.contains("[Script Info]"));
    assert!(result.contains("[Events]"));
    assert!(result.contains("Hello {\\i1}world{\\i0}!"));
}

#[test]
fn test_export_srt() {
    let doc = EditorDocument::from_content(
        r#"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello {\i1}world{\i0}!
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Test line\NSecond line"#
    ).unwrap();

    let options = ConversionOptions::default();
    let result = FormatConverter::export(&doc, SubtitleFormat::SRT, &options).unwrap();

    assert!(result.contains("1\n00:00:00,000 --> 00:00:05,000"));
    assert!(result.contains("Hello <i>world</i>!"));
    assert!(result.contains("Test line\nSecond line"));
}

#[test]
fn test_export_webvtt() {
    let doc = EditorDocument::from_content(
        r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello world!"#,
    )
    .unwrap();

    let options = ConversionOptions::default();
    let result = FormatConverter::export(&doc, SubtitleFormat::WebVTT, &options).unwrap();

    assert!(result.starts_with("WEBVTT"));
    assert!(result.contains("00:00:00.000 --> 00:00:05.000"));
    assert!(result.contains("Hello world!"));
}

#[test]
fn test_strip_formatting() {
    let doc = EditorDocument::from_content(
        r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\i1}Hello{\i0} {\b1}world{\b0}!"#,
    )
    .unwrap();

    let options = ConversionOptions {
        strip_formatting: true,
        ..Default::default()
    };

    let result = FormatConverter::export(&doc, SubtitleFormat::SRT, &options).unwrap();
    assert!(result.contains("Hello world!"));
    assert!(!result.contains("<i>"));
    assert!(!result.contains("<b>"));
}
