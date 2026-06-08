//! Behavioural tests for the alignment and wrapping-style [`TagHandler`]
//! implementations.

use super::*;
use crate::plugin::{TagHandler, TagResult};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn alignment_handler_valid() {
    let handler = AlignmentTagHandler;
    // Bottom row
    assert_eq!(handler.process("1"), TagResult::Processed);
    assert_eq!(handler.process("2"), TagResult::Processed);
    assert_eq!(handler.process("3"), TagResult::Processed);
    // Top row
    assert_eq!(handler.process("5"), TagResult::Processed);
    assert_eq!(handler.process("6"), TagResult::Processed);
    assert_eq!(handler.process("7"), TagResult::Processed);
    // Middle row
    assert_eq!(handler.process("9"), TagResult::Processed);
    assert_eq!(handler.process("10"), TagResult::Processed);
    assert_eq!(handler.process("11"), TagResult::Processed);
}

#[test]
fn alignment_handler_invalid() {
    let handler = AlignmentTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("0"), TagResult::Failed(_)));
    assert!(matches!(handler.process("4"), TagResult::Failed(_)));
    assert!(matches!(handler.process("8"), TagResult::Failed(_)));
    assert!(matches!(handler.process("12"), TagResult::Failed(_)));
    assert!(matches!(handler.process("abc"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("4") {
        assert_eq!(msg, "Alignment tag requires valid alignment code (1-11)");
    }
}

#[test]
fn numpad_alignment_handler_valid() {
    let handler = NumpadAlignmentTagHandler;
    for i in 1..=9i32 {
        assert_eq!(handler.process(&i.to_string()), TagResult::Processed);
    }
    // With whitespace
    assert_eq!(handler.process(" 5 "), TagResult::Processed);
}

#[test]
fn numpad_alignment_handler_invalid() {
    let handler = NumpadAlignmentTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("0"), TagResult::Failed(_)));
    assert!(matches!(handler.process("10"), TagResult::Failed(_)));
    assert!(matches!(handler.process("abc"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("0") {
        assert_eq!(msg, "Numpad alignment tag requires value 1-9");
    }
}

#[test]
fn wrapping_style_handler_valid() {
    let handler = WrappingStyleTagHandler;
    assert_eq!(handler.process("0"), TagResult::Processed);
    assert_eq!(handler.process("1"), TagResult::Processed);
    assert_eq!(handler.process("2"), TagResult::Processed);
    assert_eq!(handler.process("3"), TagResult::Processed);
    assert_eq!(handler.process(" 2 "), TagResult::Processed);
}

#[test]
fn wrapping_style_handler_invalid() {
    let handler = WrappingStyleTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("4"), TagResult::Failed(_)));
    assert!(matches!(handler.process("-1"), TagResult::Failed(_)));
    assert!(matches!(handler.process("abc"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("4") {
        assert_eq!(msg, "Wrapping style tag requires value 0-3");
    }
}

#[test]
fn handlers_have_correct_names() {
    assert_eq!(AlignmentTagHandler.name(), "a");
    assert_eq!(NumpadAlignmentTagHandler.name(), "an");
    assert_eq!(WrappingStyleTagHandler.name(), "q");
}

#[test]
fn create_alignment_handlers_returns_all() {
    let handlers = create_alignment_handlers();
    assert_eq!(handlers.len(), 3);

    let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
    assert!(names.contains(&"a"));
    assert!(names.contains(&"an"));
    assert!(names.contains(&"q"));
}

#[test]
fn alignment_validation() {
    let handler = AlignmentTagHandler;
    // Valid
    assert!(handler.validate("1"));
    assert!(handler.validate("2"));
    assert!(handler.validate("3"));
    assert!(handler.validate("5"));
    assert!(handler.validate("6"));
    assert!(handler.validate("7"));
    assert!(handler.validate("9"));
    assert!(handler.validate("10"));
    assert!(handler.validate("11"));
    // Invalid
    assert!(!handler.validate("0"));
    assert!(!handler.validate("4"));
    assert!(!handler.validate("8"));
    assert!(!handler.validate("12"));
    assert!(!handler.validate(""));
}

#[test]
fn numpad_alignment_validation() {
    let handler = NumpadAlignmentTagHandler;
    // Valid
    for i in 1..=9i32 {
        assert!(handler.validate(&i.to_string()));
    }
    // Invalid
    assert!(!handler.validate("0"));
    assert!(!handler.validate("10"));
    assert!(!handler.validate(""));
    assert!(!handler.validate("abc"));
}

#[test]
fn wrapping_style_validation() {
    let handler = WrappingStyleTagHandler;
    // Valid
    assert!(handler.validate("0"));
    assert!(handler.validate("1"));
    assert!(handler.validate("2"));
    assert!(handler.validate("3"));
    // Invalid
    assert!(!handler.validate("4"));
    assert!(!handler.validate("-1"));
    assert!(!handler.validate(""));
    assert!(!handler.validate("abc"));
}

#[test]
fn whitespace_handling() {
    let alignment = AlignmentTagHandler;
    let numpad = NumpadAlignmentTagHandler;
    let wrapping = WrappingStyleTagHandler;

    assert_eq!(alignment.process(" 1 "), TagResult::Processed);
    assert_eq!(numpad.process(" 5 "), TagResult::Processed);
    assert_eq!(wrapping.process(" 2 "), TagResult::Processed);

    assert_eq!(alignment.process("\t3\t"), TagResult::Processed);
    assert_eq!(numpad.process("\t9\t"), TagResult::Processed);
    assert_eq!(wrapping.process("\t0\t"), TagResult::Processed);
}

#[test]
fn alignment_semantics() {
    let handler = AlignmentTagHandler;

    // Bottom alignments (1-3)
    assert_eq!(handler.process("1"), TagResult::Processed); // Bottom-left
    assert_eq!(handler.process("2"), TagResult::Processed); // Bottom-center
    assert_eq!(handler.process("3"), TagResult::Processed); // Bottom-right

    // Top alignments (5-7)
    assert_eq!(handler.process("5"), TagResult::Processed); // Top-left
    assert_eq!(handler.process("6"), TagResult::Processed); // Top-center
    assert_eq!(handler.process("7"), TagResult::Processed); // Top-right

    // Middle alignments (9-11)
    assert_eq!(handler.process("9"), TagResult::Processed); // Middle-left
    assert_eq!(handler.process("10"), TagResult::Processed); // Middle-center
    assert_eq!(handler.process("11"), TagResult::Processed); // Middle-right
}
