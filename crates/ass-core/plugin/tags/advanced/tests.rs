//! Behavioural tests for the advanced formatting [`TagHandler`] implementations.

use super::*;
use crate::plugin::{TagHandler, TagResult};

#[test]
fn border_handler_valid() {
    let handler = BorderTagHandler;
    assert_eq!(handler.process("0"), TagResult::Processed);
    assert_eq!(handler.process("1"), TagResult::Processed);
    assert_eq!(handler.process("2.5"), TagResult::Processed);
    assert_eq!(handler.process("10"), TagResult::Processed);
    assert_eq!(handler.process(" 3 "), TagResult::Processed);
}

#[test]
fn border_handler_invalid() {
    let handler = BorderTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("-1"), TagResult::Failed(_)));
    assert!(matches!(handler.process("abc"), TagResult::Failed(_)));
    assert!(matches!(handler.process("2px"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("-1") {
        assert_eq!(msg, "Border tag requires non-negative numeric width");
    }
}

#[test]
fn shadow_handler_valid() {
    let handler = ShadowTagHandler;
    assert_eq!(handler.process("0"), TagResult::Processed);
    assert_eq!(handler.process("1"), TagResult::Processed);
    assert_eq!(handler.process("2.5"), TagResult::Processed);
    assert_eq!(handler.process("10"), TagResult::Processed);
    assert_eq!(handler.process(" 3 "), TagResult::Processed);
}

#[test]
fn shadow_handler_invalid() {
    let handler = ShadowTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("-1"), TagResult::Failed(_)));
    assert!(matches!(handler.process("abc"), TagResult::Failed(_)));
    assert!(matches!(handler.process("2px"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("-1") {
        assert_eq!(msg, "Shadow tag requires non-negative numeric depth");
    }
}

#[test]
fn blur_edges_handler_valid() {
    let handler = BlurEdgesTagHandler;
    assert_eq!(handler.process("0"), TagResult::Processed);
    assert_eq!(handler.process("1"), TagResult::Processed);
    assert_eq!(handler.process(" 0 "), TagResult::Processed);
    assert_eq!(handler.process(" 1 "), TagResult::Processed);
}

#[test]
fn blur_edges_handler_invalid() {
    let handler = BlurEdgesTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("2"), TagResult::Failed(_)));
    assert!(matches!(handler.process("true"), TagResult::Failed(_)));
    assert!(matches!(handler.process("on"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("2") {
        assert_eq!(msg, "Blur edges tag accepts only 0 or 1");
    }
}

#[test]
fn handlers_have_correct_names() {
    assert_eq!(BorderTagHandler.name(), "bord");
    assert_eq!(ShadowTagHandler.name(), "shad");
    assert_eq!(BlurEdgesTagHandler.name(), "be");
}

#[test]
fn create_advanced_handlers_returns_all() {
    let handlers = create_advanced_handlers();
    assert_eq!(handlers.len(), 3);

    let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
    assert!(names.contains(&"bord"));
    assert!(names.contains(&"shad"));
    assert!(names.contains(&"be"));
}

#[test]
fn border_validation() {
    let handler = BorderTagHandler;
    assert!(handler.validate("0"));
    assert!(handler.validate("1.5"));
    assert!(handler.validate("100"));
    assert!(!handler.validate(""));
    assert!(!handler.validate("-1"));
    assert!(!handler.validate("abc"));
}

#[test]
fn shadow_validation() {
    let handler = ShadowTagHandler;
    assert!(handler.validate("0"));
    assert!(handler.validate("1.5"));
    assert!(handler.validate("100"));
    assert!(!handler.validate(""));
    assert!(!handler.validate("-1"));
    assert!(!handler.validate("abc"));
}

#[test]
fn blur_edges_validation() {
    let handler = BlurEdgesTagHandler;
    assert!(handler.validate("0"));
    assert!(handler.validate("1"));
    assert!(!handler.validate(""));
    assert!(!handler.validate("2"));
    assert!(!handler.validate("true"));
}

#[test]
fn edge_cases() {
    let border_handler = BorderTagHandler;
    let shadow_handler = ShadowTagHandler;

    // Zero values
    assert_eq!(border_handler.process("0"), TagResult::Processed);
    assert_eq!(shadow_handler.process("0"), TagResult::Processed);

    // Decimal values
    assert_eq!(border_handler.process("0.1"), TagResult::Processed);
    assert_eq!(shadow_handler.process("0.1"), TagResult::Processed);

    // Large values
    assert_eq!(border_handler.process("999"), TagResult::Processed);
    assert_eq!(shadow_handler.process("999"), TagResult::Processed);

    // Very precise decimals
    assert_eq!(border_handler.process("1.23456"), TagResult::Processed);
    assert_eq!(shadow_handler.process("1.23456"), TagResult::Processed);
}

#[test]
fn whitespace_handling() {
    let handlers = create_advanced_handlers();

    // Border and shadow accept numeric with whitespace
    assert_eq!(handlers[0].process(" 2 "), TagResult::Processed);
    assert_eq!(handlers[1].process(" 3 "), TagResult::Processed);

    // Blur edges accepts 0/1 with whitespace
    assert_eq!(handlers[2].process(" 0 "), TagResult::Processed);
    assert_eq!(handlers[2].process(" 1 "), TagResult::Processed);
}
