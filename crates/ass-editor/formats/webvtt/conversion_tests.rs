//! Unit tests for WebVTT timestamp, styling, and cue-setting conversion.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec};

use crate::formats::{FormatExporter, FormatImporter};

#[test]
fn test_webvtt_format_creation() {
    let format = WebVttFormat::new();
    let info = FormatImporter::format_info(&format);
    assert_eq!(info.name, "WebVTT");
    assert!(info.supports_styling);
    assert!(info.supports_positioning);
    assert!(format.can_import("vtt"));
    assert!(format.can_import("webvtt"));
    assert!(format.can_export("vtt"));
}

#[test]
fn test_parse_vtt_time() {
    assert_eq!(
        WebVttFormat::parse_vtt_time("00:01:23.456").unwrap(),
        "0:01:23.45"
    );
    assert_eq!(
        WebVttFormat::parse_vtt_time("01:00:00.000").unwrap(),
        "1:00:00.00"
    );
    assert_eq!(
        WebVttFormat::parse_vtt_time("30:45.123").unwrap(),
        "0:30:45.12"
    );

    assert!(WebVttFormat::parse_vtt_time("invalid").is_err());
    assert!(WebVttFormat::parse_vtt_time("00:01:23").is_err());
}

#[test]
fn test_format_vtt_time() {
    assert_eq!(
        WebVttFormat::format_vtt_time("0:01:23.45").unwrap(),
        "00:01:23.450"
    );
    assert_eq!(
        WebVttFormat::format_vtt_time("1:00:00.00").unwrap(),
        "01:00:00.000"
    );
    assert_eq!(
        WebVttFormat::format_vtt_time("10:30:45.12").unwrap(),
        "10:30:45.120"
    );

    assert!(WebVttFormat::format_vtt_time("invalid").is_err());
    assert!(WebVttFormat::format_vtt_time("00:01:23").is_err());
}

#[test]
fn test_convert_vtt_to_ass_styling() {
    assert_eq!(
        WebVttFormat::convert_vtt_to_ass_styling("<b>Bold</b> text"),
        r"{\b1}Bold{\b0} text"
    );
    assert_eq!(
        WebVttFormat::convert_vtt_to_ass_styling("<i>Italic</i> and <u>underlined</u>"),
        r"{\i1}Italic{\i0} and {\u1}underlined{\u0}"
    );
}

#[test]
fn test_convert_ass_to_vtt_styling() {
    assert_eq!(
        WebVttFormat::convert_ass_to_vtt_styling(r"{\b1}Bold{\b0} text"),
        "<b>Bold</b> text"
    );
    assert_eq!(
        WebVttFormat::convert_ass_to_vtt_styling(r"{\i1}Italic{\i0} and {\u1}underlined{\u0}"),
        "<i>Italic</i> and <u>underlined</u>"
    );
}

#[test]
fn test_parse_cue_settings() {
    let settings = WebVttFormat::parse_cue_settings("align:center line:20% position:50%");
    assert_eq!(settings.get("align"), Some(&"center".to_string()));
    assert_eq!(settings.get("line"), Some(&"20%".to_string()));
    assert_eq!(settings.get("position"), Some(&"50%".to_string()));
}
