//! Behavioural tests for the miscellaneous [`TagHandler`] implementations.

use super::*;
use crate::plugin::{TagHandler, TagResult};

#[test]
fn reset_handler_valid() {
    let handler = ResetTagHandler;
    // Empty (reset to default)
    assert_eq!(handler.process(""), TagResult::Processed);
    // Style name
    assert_eq!(handler.process("Default"), TagResult::Processed);
    assert_eq!(handler.process("Main"), TagResult::Processed);
    assert_eq!(handler.process("Subtitle-Style"), TagResult::Processed);
    assert_eq!(handler.process(" Karaoke "), TagResult::Processed);
}

#[test]
fn reset_handler_always_valid() {
    let handler = ResetTagHandler;
    // Reset accepts any input
    assert!(handler.validate(""));
    assert!(handler.validate("StyleName"));
    assert!(handler.validate("123"));
    assert!(handler.validate("Style With Spaces"));
}

#[test]
fn short_rotation_handler_valid() {
    let handler = ShortRotationTagHandler;
    assert_eq!(handler.process("0"), TagResult::Processed);
    assert_eq!(handler.process("90"), TagResult::Processed);
    assert_eq!(handler.process("-90"), TagResult::Processed);
    assert_eq!(handler.process("360"), TagResult::Processed);
    assert_eq!(handler.process("45.5"), TagResult::Processed);
    assert_eq!(handler.process(" 180 "), TagResult::Processed);
}

#[test]
fn short_rotation_handler_invalid() {
    let handler = ShortRotationTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("abc"), TagResult::Failed(_)));
    assert!(matches!(handler.process("90deg"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("invalid") {
        assert_eq!(msg, "Rotation tag requires numeric degrees");
    }
}

#[test]
fn origin_handler_valid() {
    let handler = OriginTagHandler;
    assert_eq!(handler.process("(100,200)"), TagResult::Processed);
    assert_eq!(handler.process("(0,0)"), TagResult::Processed);
    assert_eq!(handler.process("(-50,100)"), TagResult::Processed);
    assert_eq!(handler.process("(640,360)"), TagResult::Processed);
    assert_eq!(handler.process("(100.5,200.5)"), TagResult::Processed);
    assert_eq!(handler.process("( 50 , 100 )"), TagResult::Processed);
}

#[test]
fn origin_handler_invalid() {
    let handler = OriginTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("100,200"), TagResult::Failed(_))); // No parentheses
    assert!(matches!(handler.process("(100)"), TagResult::Failed(_))); // Only one coord
    assert!(matches!(
        handler.process("(100,200,300)"),
        TagResult::Failed(_)
    )); // Too many
    assert!(matches!(handler.process("(abc,def)"), TagResult::Failed(_))); // Non-numeric

    if let TagResult::Failed(msg) = handler.process("no_parens") {
        assert_eq!(msg, "Origin tag requires (x,y) coordinates");
    }
}

#[test]
fn handlers_have_correct_names() {
    assert_eq!(ResetTagHandler.name(), "r");
    assert_eq!(ShortRotationTagHandler.name(), "fr");
    assert_eq!(OriginTagHandler.name(), "org");
}

#[test]
fn create_misc_handlers_returns_all() {
    let handlers = create_misc_handlers();
    assert_eq!(handlers.len(), 3);

    let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
    assert!(names.contains(&"r"));
    assert!(names.contains(&"fr"));
    assert!(names.contains(&"org"));
}

#[test]
fn reset_validation() {
    let handler = ResetTagHandler;
    // Always returns true
    assert!(handler.validate(""));
    assert!(handler.validate("Default"));
    assert!(handler.validate("Any String"));
    assert!(handler.validate("123"));
    assert!(handler.validate("!@#$%^&*()"));
}

#[test]
fn short_rotation_validation() {
    let handler = ShortRotationTagHandler;
    assert!(handler.validate("0"));
    assert!(handler.validate("90"));
    assert!(handler.validate("-180"));
    assert!(handler.validate("45.5"));
    assert!(!handler.validate(""));
    assert!(!handler.validate("abc"));
    assert!(!handler.validate("90deg"));
}

#[test]
fn origin_validation() {
    let handler = OriginTagHandler;
    assert!(handler.validate("(0,0)"));
    assert!(handler.validate("(100,200)"));
    assert!(handler.validate("(-50,50)"));
    assert!(handler.validate("(1.5,2.5)"));
    assert!(!handler.validate(""));
    assert!(!handler.validate("0,0")); // No parentheses
    assert!(!handler.validate("(0)")); // Too few
    assert!(!handler.validate("(0,0,0)")); // Too many
}

#[test]
fn reset_style_names() {
    let handler = ResetTagHandler;
    // Various valid style names
    assert_eq!(handler.process("Default"), TagResult::Processed);
    assert_eq!(handler.process("Main-Style"), TagResult::Processed);
    assert_eq!(handler.process("Style_123"), TagResult::Processed);
    assert_eq!(handler.process("日本語スタイル"), TagResult::Processed); // Unicode
    assert_eq!(handler.process("Style With Spaces"), TagResult::Processed);
}

#[test]
fn origin_coordinate_ranges() {
    let handler = OriginTagHandler;
    // Screen coordinates
    assert_eq!(handler.process("(0,0)"), TagResult::Processed); // Top-left
    assert_eq!(handler.process("(1920,1080)"), TagResult::Processed); // Full HD
    assert_eq!(handler.process("(3840,2160)"), TagResult::Processed); // 4K
                                                                      // Negative coordinates (off-screen)
    assert_eq!(handler.process("(-100,-100)"), TagResult::Processed);
    // Center points
    assert_eq!(handler.process("(640,360)"), TagResult::Processed); // 720p center
    assert_eq!(handler.process("(960,540)"), TagResult::Processed); // 1080p center
}
