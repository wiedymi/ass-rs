//! Unit tests for SRT format construction and conversion helpers.

use super::*;
use crate::formats::{FormatExporter, FormatImporter};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec};

#[test]
fn test_srt_format_creation() {
    let format = SrtFormat::new();
    let info = FormatImporter::format_info(&format);
    assert_eq!(info.name, "SRT");
    assert!(info.supports_styling);
    assert!(!info.supports_positioning);
    assert!(format.can_import("srt"));
    assert!(format.can_export("srt"));
}

#[test]
fn test_parse_srt_time() {
    assert_eq!(
        SrtFormat::parse_srt_time("00:01:23,456").unwrap(),
        "0:01:23.45"
    );
    assert_eq!(
        SrtFormat::parse_srt_time("01:00:00,000").unwrap(),
        "1:00:00.00"
    );
    assert_eq!(
        SrtFormat::parse_srt_time("10:30:45,123").unwrap(),
        "10:30:45.12"
    );

    assert!(SrtFormat::parse_srt_time("invalid").is_err());
    assert!(SrtFormat::parse_srt_time("00:01:23").is_err());
}

#[test]
fn test_format_srt_time() {
    assert_eq!(
        SrtFormat::format_srt_time("0:01:23.45").unwrap(),
        "00:01:23,450"
    );
    assert_eq!(
        SrtFormat::format_srt_time("1:00:00.00").unwrap(),
        "01:00:00,000"
    );
    assert_eq!(
        SrtFormat::format_srt_time("10:30:45.12").unwrap(),
        "10:30:45,120"
    );

    assert!(SrtFormat::format_srt_time("invalid").is_err());
    assert!(SrtFormat::format_srt_time("00:01:23").is_err());
}

#[test]
fn test_convert_srt_to_ass_styling() {
    assert_eq!(
        SrtFormat::convert_srt_to_ass_styling("<b>Bold</b> text"),
        r"{\b1}Bold{\b0} text"
    );
    assert_eq!(
        SrtFormat::convert_srt_to_ass_styling("<i>Italic</i> and <u>underlined</u>"),
        r"{\i1}Italic{\i0} and {\u1}underlined{\u0}"
    );
    assert_eq!(
        SrtFormat::convert_srt_to_ass_styling("<font color=\"#FF0000\">Red text</font>"),
        r"{\c&HFF0000&}Red text{\c}"
    );
}

#[test]
fn test_convert_ass_to_srt_styling() {
    assert_eq!(
        SrtFormat::convert_ass_to_srt_styling(r"{\b1}Bold{\b0} text"),
        "<b>Bold</b> text"
    );
    assert_eq!(
        SrtFormat::convert_ass_to_srt_styling(r"{\i1}Italic{\i0} and {\u1}underlined{\u0}"),
        "<i>Italic</i> and <u>underlined</u>"
    );
}
