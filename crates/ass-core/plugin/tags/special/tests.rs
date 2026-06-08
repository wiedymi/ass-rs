//! Behavioural tests for the special-character [`TagHandler`] implementations.

use super::*;
use crate::plugin::{TagHandler, TagResult};

#[test]
fn soft_line_break_valid() {
    let handler = SoftLineBreakTagHandler;
    assert_eq!(handler.process(""), TagResult::Processed);
    assert_eq!(handler.process("  "), TagResult::Processed);
    assert_eq!(handler.process("\t"), TagResult::Processed);
}

#[test]
fn soft_line_break_invalid() {
    let handler = SoftLineBreakTagHandler;
    assert!(matches!(handler.process("arg"), TagResult::Failed(_)));
    assert!(matches!(handler.process("123"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("invalid") {
        assert_eq!(msg, "Line break tag takes no arguments");
    }
}

#[test]
fn hard_line_break_valid() {
    let handler = HardLineBreakTagHandler;
    assert_eq!(handler.process(""), TagResult::Processed);
    assert_eq!(handler.process("  "), TagResult::Processed);
    assert_eq!(handler.process("\t"), TagResult::Processed);
}

#[test]
fn hard_line_break_invalid() {
    let handler = HardLineBreakTagHandler;
    assert!(matches!(handler.process("arg"), TagResult::Failed(_)));
    assert!(matches!(handler.process("123"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("invalid") {
        assert_eq!(msg, "Line break tag takes no arguments");
    }
}

#[test]
fn hard_space_valid() {
    let handler = HardSpaceTagHandler;
    assert_eq!(handler.process(""), TagResult::Processed);
    assert_eq!(handler.process("  "), TagResult::Processed);
    assert_eq!(handler.process("\t"), TagResult::Processed);
}

#[test]
fn hard_space_invalid() {
    let handler = HardSpaceTagHandler;
    assert!(matches!(handler.process("arg"), TagResult::Failed(_)));
    assert!(matches!(handler.process("123"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("invalid") {
        assert_eq!(msg, "Hard space tag takes no arguments");
    }
}

#[test]
fn handlers_have_correct_names() {
    assert_eq!(SoftLineBreakTagHandler.name(), "n");
    assert_eq!(HardLineBreakTagHandler.name(), "N");
    assert_eq!(HardSpaceTagHandler.name(), "h");
}

#[test]
fn create_special_handlers_returns_all() {
    let handlers = create_special_handlers();
    assert_eq!(handlers.len(), 3);

    let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
    assert!(names.contains(&"n"));
    assert!(names.contains(&"N"));
    assert!(names.contains(&"h"));
}

#[test]
fn validation_consistency() {
    let handlers = create_special_handlers();

    for handler in &handlers {
        // All special handlers accept empty args
        assert!(handler.validate(""));
        assert!(handler.validate("  "));
        assert_eq!(handler.process(""), TagResult::Processed);

        // All reject non-empty args
        assert!(!handler.validate("something"));
        assert!(matches!(handler.process("something"), TagResult::Failed(_)));
    }
}

#[test]
fn whitespace_handling() {
    let handlers = create_special_handlers();

    for handler in &handlers {
        // Various whitespace should be accepted
        assert_eq!(handler.process(""), TagResult::Processed);
        assert_eq!(handler.process(" "), TagResult::Processed);
        assert_eq!(handler.process("  "), TagResult::Processed);
        assert_eq!(handler.process("\t"), TagResult::Processed);
        assert_eq!(handler.process("\n"), TagResult::Processed);
        assert_eq!(handler.process(" \t\n "), TagResult::Processed);
    }
}
