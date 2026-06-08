//! Behavioural tests for the transform [`TagHandler`] and the handler factory.

use super::*;
use crate::plugin::{TagHandler, TagResult};

#[test]
fn transform_handler_valid() {
    let handler = TransformTagHandler;
    // Simple transform with modifiers
    assert_eq!(handler.process("(\\fs20)"), TagResult::Processed);
    assert_eq!(handler.process("(\\c&H0000FF&)"), TagResult::Processed);
    // With timing
    assert_eq!(handler.process("(100,500,\\fs30)"), TagResult::Processed);
    // With timing and acceleration
    assert_eq!(
        handler.process("(0,1000,2.5,\\frz360)"),
        TagResult::Processed
    );
    // Multiple modifiers
    assert_eq!(
        handler.process("(\\fs20\\c&HFF0000&)"),
        TagResult::Processed
    );
}

#[test]
fn transform_handler_invalid() {
    let handler = TransformTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("\\fs20"), TagResult::Failed(_))); // No parentheses
    assert!(matches!(handler.process("()"), TagResult::Failed(_))); // Empty
    assert!(matches!(handler.process("(  )"), TagResult::Failed(_))); // Only whitespace

    if let TagResult::Failed(msg) = handler.process("no_parens") {
        assert_eq!(
            msg,
            "Transform tag requires (modifiers) or (t1,t2,[accel,]modifiers)"
        );
    }
}

#[test]
fn handlers_have_correct_names() {
    assert_eq!(TransformTagHandler.name(), "t");
    assert_eq!(FadeTagHandler.name(), "fade");
    assert_eq!(SimpleFadeTagHandler.name(), "fad");
}

#[test]
fn create_animation_handlers_returns_all() {
    let handlers = create_animation_handlers();
    assert_eq!(handlers.len(), 3);

    let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
    assert!(names.contains(&"t"));
    assert!(names.contains(&"fade"));
    assert!(names.contains(&"fad"));
}

#[test]
fn transform_validation() {
    let handler = TransformTagHandler;
    assert!(handler.validate("(\\fs20)"));
    assert!(handler.validate("(100,500,\\fs30)"));
    assert!(handler.validate("(0,1000,2.5,\\frz360)"));
    assert!(!handler.validate(""));
    assert!(!handler.validate("no_parens"));
    assert!(!handler.validate("()"));
}

#[test]
fn transform_complex_cases() {
    let handler = TransformTagHandler;

    // Nested parentheses (in real implementation would need proper parsing)
    assert_eq!(
        handler.process("(\\clip(0,0,100,100))"),
        TagResult::Processed
    );

    // Multiple transformations
    assert_eq!(
        handler.process("(\\fs20\\frz30\\c&HFF0000&)"),
        TagResult::Processed
    );

    // With complex timing
    assert_eq!(
        handler.process("(100,2000,0.5,\\fscx120\\fscy120)"),
        TagResult::Processed
    );
}
