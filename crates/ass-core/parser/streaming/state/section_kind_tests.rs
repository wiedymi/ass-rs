//! Unit tests for [`SectionKind`] header parsing and section properties.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn section_kind_from_header() {
    assert_eq!(
        SectionKind::from_header("Script Info"),
        SectionKind::ScriptInfo
    );
    assert_eq!(SectionKind::from_header("V4+ Styles"), SectionKind::Styles);
    assert_eq!(SectionKind::from_header("V4 Styles"), SectionKind::Styles);
    assert_eq!(SectionKind::from_header("Events"), SectionKind::Events);
    assert_eq!(SectionKind::from_header("Fonts"), SectionKind::Fonts);
    assert_eq!(SectionKind::from_header("Graphics"), SectionKind::Graphics);
    assert_eq!(
        SectionKind::from_header("Unknown Section"),
        SectionKind::Unknown
    );
}

#[test]
fn section_kind_properties() {
    assert!(SectionKind::Styles.expects_format());
    assert!(SectionKind::Events.expects_format());
    assert!(!SectionKind::ScriptInfo.expects_format());

    assert!(SectionKind::Events.is_timed());
    assert!(!SectionKind::Styles.is_timed());

    assert!(SectionKind::Fonts.is_binary());
    assert!(SectionKind::Graphics.is_binary());
    assert!(!SectionKind::Events.is_binary());
}

#[test]
fn section_kind_all_variants() {
    let kinds = [
        SectionKind::ScriptInfo,
        SectionKind::Styles,
        SectionKind::Events,
        SectionKind::Fonts,
        SectionKind::Graphics,
        SectionKind::Unknown,
    ];

    for &kind in &kinds {
        let debug_str = format!("{kind:?}");
        assert!(!debug_str.is_empty());

        // Test Copy trait
        let copied = kind;
        assert_eq!(kind, copied);
    }
}

#[test]
fn section_kind_header_parsing_edge_cases() {
    // Test case insensitive variations
    assert_eq!(
        SectionKind::from_header("  Script Info  "),
        SectionKind::ScriptInfo
    );
    assert_eq!(
        SectionKind::from_header("\tV4+ Styles\t"),
        SectionKind::Styles
    );

    // Test empty and whitespace
    assert_eq!(SectionKind::from_header(""), SectionKind::Unknown);
    assert_eq!(SectionKind::from_header("   "), SectionKind::Unknown);

    // Test partial matches
    assert_eq!(SectionKind::from_header("Script"), SectionKind::Unknown);
    assert_eq!(SectionKind::from_header("Info"), SectionKind::Unknown);
    assert_eq!(SectionKind::from_header("Styles"), SectionKind::Unknown);

    // Test common variations
    assert_eq!(SectionKind::from_header("V4 Styles"), SectionKind::Styles);
    assert_eq!(SectionKind::from_header("V4+ Styles"), SectionKind::Styles);
}

#[test]
fn section_kind_all_properties() {
    // Test expects_format
    assert!(SectionKind::Styles.expects_format());
    assert!(SectionKind::Events.expects_format());
    assert!(!SectionKind::ScriptInfo.expects_format());
    assert!(!SectionKind::Fonts.expects_format());
    assert!(!SectionKind::Graphics.expects_format());
    assert!(!SectionKind::Unknown.expects_format());

    // Test is_timed
    assert!(SectionKind::Events.is_timed());
    assert!(!SectionKind::ScriptInfo.is_timed());
    assert!(!SectionKind::Styles.is_timed());
    assert!(!SectionKind::Fonts.is_timed());
    assert!(!SectionKind::Graphics.is_timed());
    assert!(!SectionKind::Unknown.is_timed());

    // Test is_binary
    assert!(SectionKind::Fonts.is_binary());
    assert!(SectionKind::Graphics.is_binary());
    assert!(!SectionKind::ScriptInfo.is_binary());
    assert!(!SectionKind::Styles.is_binary());
    assert!(!SectionKind::Events.is_binary());
    assert!(!SectionKind::Unknown.is_binary());
}
