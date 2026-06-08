//! Behavioural tests for the `\fade` and `\fad` [`TagHandler`] implementations.

use super::*;
use crate::plugin::{TagHandler, TagResult};

#[test]
fn fade_handler_valid() {
    let handler = FadeTagHandler;
    // Standard fade
    assert_eq!(
        handler.process("(0,255,0,0,500,1000,1500)"),
        TagResult::Processed
    );
    // All zeros
    assert_eq!(handler.process("(0,0,0,0,0,0,0)"), TagResult::Processed);
    // Max alpha values
    assert_eq!(
        handler.process("(255,255,255,100,200,300,400)"),
        TagResult::Processed
    );
}

#[test]
fn fade_handler_invalid() {
    let handler = FadeTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(
        handler.process("(0,0,0,0,0,0)"),
        TagResult::Failed(_)
    )); // Too few
    assert!(matches!(
        handler.process("(0,0,0,0,0,0,0,0)"),
        TagResult::Failed(_)
    )); // Too many
    assert!(matches!(
        handler.process("(256,0,0,0,0,0,0)"),
        TagResult::Failed(_)
    )); // Alpha > 255
    assert!(matches!(
        handler.process("(0,0,0,-1,0,0,0)"),
        TagResult::Failed(_)
    )); // Negative time
    assert!(matches!(
        handler.process("(a,b,c,d,e,f,g)"),
        TagResult::Failed(_)
    )); // Non-numeric

    if let TagResult::Failed(msg) = handler.process("(1,2,3)") {
        assert_eq!(
            msg,
            "Fade tag requires (a1,a2,a3,t1,t2,t3,t4) - 7 numeric parameters"
        );
    }
}

#[test]
fn simple_fade_handler_valid() {
    let handler = SimpleFadeTagHandler;
    assert_eq!(handler.process("(500,500)"), TagResult::Processed);
    assert_eq!(handler.process("(0,1000)"), TagResult::Processed);
    assert_eq!(handler.process("(1000,0)"), TagResult::Processed);
    assert_eq!(handler.process("( 100 , 200 )"), TagResult::Processed); // With spaces
}

#[test]
fn simple_fade_handler_invalid() {
    let handler = SimpleFadeTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("(500)"), TagResult::Failed(_))); // Too few
    assert!(matches!(
        handler.process("(500,500,500)"),
        TagResult::Failed(_)
    )); // Too many
    assert!(matches!(
        handler.process("(-500,500)"),
        TagResult::Failed(_)
    )); // Negative
    assert!(matches!(handler.process("(abc,def)"), TagResult::Failed(_))); // Non-numeric

    if let TagResult::Failed(msg) = handler.process("500,500") {
        assert_eq!(
            msg,
            "Simple fade tag requires (t1,t2) - fade in and out durations"
        );
    }
}

#[test]
fn fade_validation() {
    let handler = FadeTagHandler;
    assert!(handler.validate("(0,255,0,0,500,1000,1500)"));
    assert!(handler.validate("(255,255,255,0,0,0,0)"));
    assert!(!handler.validate("(0,0,0,0,0,0)")); // Too few
    assert!(!handler.validate("(256,0,0,0,0,0,0)")); // Alpha > 255
    assert!(!handler.validate("(0,0,0,-1,0,0,0)")); // Negative time
}

#[test]
fn simple_fade_validation() {
    let handler = SimpleFadeTagHandler;
    assert!(handler.validate("(500,500)"));
    assert!(handler.validate("(0,0)"));
    assert!(handler.validate("(9999,9999)"));
    assert!(!handler.validate("(500)")); // Too few
    assert!(!handler.validate("(500,500,500)")); // Too many
    assert!(!handler.validate("(-500,500)")); // Negative not allowed
}

#[test]
fn fade_edge_cases() {
    let handler = FadeTagHandler;

    // Boundary alpha values
    assert_eq!(handler.process("(0,0,0,0,0,0,0)"), TagResult::Processed);
    assert_eq!(
        handler.process("(255,255,255,0,0,0,0)"),
        TagResult::Processed
    );

    // Large time values
    assert_eq!(
        handler.process("(128,128,128,0,99999,199999,299999)"),
        TagResult::Processed
    );

    // Whitespace handling
    assert_eq!(
        handler.process("( 100 , 200 , 100 , 0 , 500 , 1000 , 1500 )"),
        TagResult::Processed
    );
}
