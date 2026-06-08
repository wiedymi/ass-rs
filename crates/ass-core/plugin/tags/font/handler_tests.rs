//! Behavioural tests for the font [`TagHandler`] implementations.

use super::*;
use crate::plugin::{TagHandler, TagResult};

#[test]
fn font_name_valid() {
    let handler = FontNameTagHandler;
    assert_eq!(handler.process("Arial"), TagResult::Processed);
    assert_eq!(handler.process("Times New Roman"), TagResult::Processed);
    assert_eq!(handler.process("MS UI Gothic"), TagResult::Processed);
    assert_eq!(handler.process(" Comic Sans MS "), TagResult::Processed);
    assert_eq!(handler.process("font-family"), TagResult::Processed);
}

#[test]
fn font_name_invalid() {
    let handler = FontNameTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("  "), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("") {
        assert_eq!(msg, "Font name tag requires a font name");
    }
}

#[test]
fn font_size_valid() {
    let handler = FontSizeTagHandler;
    assert_eq!(handler.process("12"), TagResult::Processed);
    assert_eq!(handler.process("24.5"), TagResult::Processed);
    assert_eq!(handler.process("100"), TagResult::Processed);
    assert_eq!(handler.process("0.5"), TagResult::Processed);
    assert_eq!(handler.process(" 16 "), TagResult::Processed);
}

#[test]
fn font_size_invalid() {
    let handler = FontSizeTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("0"), TagResult::Failed(_)));
    assert!(matches!(handler.process("-12"), TagResult::Failed(_)));
    assert!(matches!(handler.process("abc"), TagResult::Failed(_)));
    assert!(matches!(handler.process("12px"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("invalid") {
        assert_eq!(msg, "Font size tag requires positive numeric size");
    }
}

#[test]
fn font_encoding_valid() {
    let handler = FontEncodingTagHandler;
    assert_eq!(handler.process("0"), TagResult::Processed);
    assert_eq!(handler.process("1"), TagResult::Processed);
    assert_eq!(handler.process("128"), TagResult::Processed);
    assert_eq!(handler.process("255"), TagResult::Processed);
    assert_eq!(handler.process(" 134 "), TagResult::Processed);
}

#[test]
fn font_encoding_invalid() {
    let handler = FontEncodingTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("-1"), TagResult::Failed(_)));
    assert!(matches!(handler.process("1.5"), TagResult::Failed(_)));
    assert!(matches!(handler.process("abc"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("invalid") {
        assert_eq!(msg, "Font encoding tag requires numeric charset");
    }
}

#[test]
fn handlers_have_correct_names() {
    assert_eq!(FontNameTagHandler.name(), "fn");
    assert_eq!(FontSizeTagHandler.name(), "fs");
    assert_eq!(FontEncodingTagHandler.name(), "fe");
}

#[test]
fn create_font_handlers_returns_all() {
    let handlers = create_font_handlers();
    assert_eq!(handlers.len(), 3);

    let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
    assert!(names.contains(&"fn"));
    assert!(names.contains(&"fs"));
    assert!(names.contains(&"fe"));
}

#[test]
fn font_name_validation() {
    let handler = FontNameTagHandler;
    assert!(handler.validate("Arial"));
    assert!(handler.validate("Font Name With Spaces"));
    assert!(handler.validate("  trimmed  "));
    assert!(!handler.validate(""));
    assert!(!handler.validate("   "));
}

#[test]
fn font_size_validation() {
    let handler = FontSizeTagHandler;
    assert!(handler.validate("12"));
    assert!(handler.validate("12.5"));
    assert!(handler.validate("0.1"));
    assert!(!handler.validate("0"));
    assert!(!handler.validate("-5"));
    assert!(!handler.validate(""));
    assert!(!handler.validate("abc"));
}

#[test]
fn font_encoding_validation() {
    let handler = FontEncodingTagHandler;
    assert!(handler.validate("0"));
    assert!(handler.validate("128"));
    assert!(handler.validate("255"));
    assert!(!handler.validate("-1"));
    assert!(!handler.validate("1.5"));
    assert!(!handler.validate(""));
    assert!(!handler.validate("abc"));
}

#[test]
fn font_name_edge_cases() {
    let handler = FontNameTagHandler;
    // Unicode font names
    assert_eq!(handler.process("ＭＳ ゴシック"), TagResult::Processed);
    // Font names with special characters
    assert_eq!(handler.process("Font_Name-123"), TagResult::Processed);
    // Very long font name
    assert_eq!(
        handler.process("A".repeat(100).as_str()),
        TagResult::Processed
    );
}

#[test]
fn font_size_edge_cases() {
    let handler = FontSizeTagHandler;
    // Very small but positive
    assert_eq!(handler.process("0.001"), TagResult::Processed);
    // Very large
    assert_eq!(handler.process("9999"), TagResult::Processed);
    // Scientific notation is actually accepted by parse::<f32>()
    assert_eq!(handler.process("1e2"), TagResult::Processed); // 100
}

#[test]
fn font_encoding_edge_cases() {
    let handler = FontEncodingTagHandler;
    // Common encoding values
    assert_eq!(handler.process("0"), TagResult::Processed); // ANSI
    assert_eq!(handler.process("1"), TagResult::Processed); // Default
    assert_eq!(handler.process("128"), TagResult::Processed); // Shift-JIS
    assert_eq!(handler.process("134"), TagResult::Processed); // GB2312
    assert_eq!(handler.process("136"), TagResult::Processed); // Big5
                                                              // Max u32
    assert_eq!(handler.process("4294967295"), TagResult::Processed);
}
