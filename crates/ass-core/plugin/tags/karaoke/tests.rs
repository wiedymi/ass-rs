//! Behavioural tests for the karaoke [`TagHandler`] implementations.

use super::*;
use crate::plugin::{TagHandler, TagResult};
use alloc::vec::Vec;

#[test]
fn kt_handler_name() {
    let handler = KaraokeTimingTagHandler;
    assert_eq!(handler.name(), "kt");
}

#[test]
fn kt_handler_valid_args() {
    let handler = KaraokeTimingTagHandler;
    assert_eq!(handler.process("500"), TagResult::Processed);
    assert!(handler.validate("500"));
}

#[test]
fn kt_handler_valid_zero() {
    let handler = KaraokeTimingTagHandler;
    assert_eq!(handler.process("0"), TagResult::Processed);
    assert!(handler.validate("0"));
}

#[test]
fn kt_handler_valid_large_number() {
    let handler = KaraokeTimingTagHandler;
    assert_eq!(handler.process("999999"), TagResult::Processed);
    assert!(handler.validate("999999"));
}

#[test]
fn kt_handler_invalid_args() {
    let handler = KaraokeTimingTagHandler;
    assert!(matches!(handler.process("abc"), TagResult::Failed(_)));
    assert!(!handler.validate("abc"));
}

#[test]
fn kt_handler_invalid_negative() {
    let handler = KaraokeTimingTagHandler;
    assert!(matches!(handler.process("-100"), TagResult::Failed(_)));
    assert!(!handler.validate("-100"));
}

#[test]
fn kt_handler_invalid_float() {
    let handler = KaraokeTimingTagHandler;
    assert!(matches!(handler.process("123.45"), TagResult::Failed(_)));
    assert!(!handler.validate("123.45"));
}

#[test]
fn kt_handler_invalid_empty() {
    let handler = KaraokeTimingTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(!handler.validate(""));
}

#[test]
fn kt_handler_invalid_whitespace_only() {
    let handler = KaraokeTimingTagHandler;
    assert!(matches!(handler.process("   "), TagResult::Failed(_)));
    assert!(!handler.validate("   "));
}

#[test]
fn kt_handler_whitespace_trimming() {
    let handler = KaraokeTimingTagHandler;
    assert_eq!(handler.process("  500  "), TagResult::Processed);
    assert!(handler.validate("  500  "));
}

#[test]
fn k_handler_valid_args() {
    let handler = BasicKaraokeTagHandler;
    assert_eq!(handler.process("50"), TagResult::Processed);
    assert_eq!(handler.process("0"), TagResult::Processed);
    assert_eq!(handler.process("999"), TagResult::Processed);
    assert!(handler.validate("50"));
}

#[test]
fn k_handler_invalid_args() {
    let handler = BasicKaraokeTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("abc"), TagResult::Failed(_)));
    assert!(matches!(handler.process("-10"), TagResult::Failed(_)));
    assert!(matches!(handler.process("1.5"), TagResult::Failed(_)));
}

#[test]
fn kf_handler_valid_args() {
    let handler = FillKaraokeTagHandler;
    assert_eq!(handler.process("100"), TagResult::Processed);
    assert_eq!(handler.process("0"), TagResult::Processed);
    assert!(handler.validate("100"));
}

#[test]
fn kf_handler_invalid_args() {
    let handler = FillKaraokeTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));
}

#[test]
fn ko_handler_valid_args() {
    let handler = OutlineKaraokeTagHandler;
    assert_eq!(handler.process("75"), TagResult::Processed);
    assert_eq!(handler.process("0"), TagResult::Processed);
    assert!(handler.validate("75"));
}

#[test]
fn ko_handler_invalid_args() {
    let handler = OutlineKaraokeTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(
        handler.process("not_a_number"),
        TagResult::Failed(_)
    ));
}

#[test]
fn create_karaoke_handlers_contains_all() {
    let handlers = create_karaoke_handlers();
    assert_eq!(handlers.len(), 4);

    let names: Vec<&str> = handlers.iter().map(|h| h.name()).collect();
    assert!(names.contains(&"k"));
    assert!(names.contains(&"kf"));
    assert!(names.contains(&"ko"));
    assert!(names.contains(&"kt"));
}

#[test]
fn create_karaoke_handlers_all_functional() {
    let handlers = create_karaoke_handlers();

    for handler in &handlers {
        // Test valid input
        assert_eq!(handler.process("100"), TagResult::Processed);

        // Test invalid input
        assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));
    }
}
