//! Unit tests for the encoding detection utilities.

use super::super::bom::BomType;
use super::*;
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn encoding_info_creation() {
    let info = EncodingInfo::new("UTF-8".to_string(), 0.95);
    assert_eq!(info.encoding, "UTF-8");
    assert!((info.confidence - 0.95).abs() < f32::EPSILON);
    assert!(!info.has_bom);
    assert!(info.is_valid);

    let info_with_bom = EncodingInfo::with_bom("UTF-8".to_string(), 1.0, BomType::Utf8);
    assert!((info_with_bom.confidence - 1.0).abs() < f32::EPSILON);
    assert!(info_with_bom.has_bom);
    assert_eq!(info_with_bom.bom_type, Some(BomType::Utf8));
}

#[test]
fn detect_utf8_encoding() {
    let text = "[Script Info]\nTitle: Test Script";
    let encoding = detect_encoding(text.as_bytes());
    assert_eq!(encoding.encoding, "UTF-8");
    assert!(encoding.confidence > 0.9); // High confidence due to ASS patterns
    assert!(!encoding.has_bom);
}

#[test]
fn detect_encoding_with_bom() {
    let text = "\u{FEFF}[Script Info]";
    let encoding = detect_encoding(text.as_bytes());
    assert_eq!(encoding.encoding, "UTF-8");
    assert!((encoding.confidence - 1.0).abs() < f32::EPSILON);
    assert!(encoding.has_bom);
    assert_eq!(encoding.bom_type, Some(BomType::Utf8));
}

#[test]
fn detect_non_utf8_encoding() {
    let invalid_bytes = &[0x80, 0x81, b'H', b'e', b'l', b'l', b'o']; // Invalid UTF-8, no BOM
    let encoding = detect_encoding(invalid_bytes);
    assert_eq!(encoding.encoding, "Windows-1252");
    assert!(encoding.confidence < 1.0);
}

#[test]
fn detect_ascii_encoding() {
    let ascii_bytes = b"Hello World"; // Pure ASCII
    let encoding = detect_encoding(ascii_bytes);
    assert_eq!(encoding.encoding, "UTF-8"); // ASCII is valid UTF-8
    assert!(encoding.confidence > 0.7);
}

#[test]
fn is_likely_ass_content_detection() {
    assert!(is_likely_ass_content("[Script Info]\nTitle: Test"));
    assert!(is_likely_ass_content("[V4+ Styles]\nFormat: Name"));
    assert!(is_likely_ass_content("Dialogue: 0,0:00:00.00"));
    assert!(is_likely_ass_content("ScriptType: v4.00+"));
    assert!(!is_likely_ass_content("This is just regular text"));
    assert!(!is_likely_ass_content("No ASS patterns here"));
}

#[test]
fn encoding_info_equality() {
    let info1 = EncodingInfo::new("UTF-8".to_string(), 0.95);
    let info2 = EncodingInfo::new("UTF-8".to_string(), 0.95);
    let info3 = EncodingInfo::new("ASCII".to_string(), 0.95);

    assert_eq!(info1, info2);
    assert_ne!(info1, info3);
}

#[test]
fn encoding_info_debug() {
    let info = EncodingInfo::new("UTF-8".to_string(), 0.95);
    let debug_str = format!("{info:?}");
    assert!(debug_str.contains("EncodingInfo"));
    assert!(debug_str.contains("UTF-8"));
}
