//! Behavioural tests for the color and alpha [`TagHandler`] implementations.

use super::*;
use crate::plugin::{TagHandler, TagResult};

#[test]
fn color_handlers_valid() {
    let color_handlers: [&dyn TagHandler; 5] = [
        &PrimaryColorTagHandler,
        &Color1TagHandler,
        &Color2TagHandler,
        &Color3TagHandler,
        &Color4TagHandler,
    ];

    for handler in &color_handlers {
        assert_eq!(handler.process("&H000000&"), TagResult::Processed);
        assert_eq!(handler.process("&HFFFFFF&"), TagResult::Processed);
        assert_eq!(handler.process("&H123ABC&"), TagResult::Processed);
    }
}

#[test]
fn color_handlers_invalid() {
    let handler = PrimaryColorTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("&H00&"), TagResult::Failed(_)));
    assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("&H00&") {
        assert_eq!(msg, "Color tag requires &Hbbggrr& format");
    }
}

#[test]
fn alpha_handlers_valid() {
    let alpha_handlers: [&dyn TagHandler; 5] = [
        &AlphaTagHandler,
        &Alpha1TagHandler,
        &Alpha2TagHandler,
        &Alpha3TagHandler,
        &Alpha4TagHandler,
    ];

    for handler in &alpha_handlers {
        assert_eq!(handler.process("&H00&"), TagResult::Processed);
        assert_eq!(handler.process("&HFF&"), TagResult::Processed);
        assert_eq!(handler.process("&H7F&"), TagResult::Processed);
    }
}

#[test]
fn alpha_handlers_invalid() {
    let handler = AlphaTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("&H000000&"), TagResult::Failed(_)));
    assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("&H000&") {
        assert_eq!(msg, "Alpha tag requires &Haa& format");
    }
}

#[test]
fn handlers_have_correct_names() {
    assert_eq!(PrimaryColorTagHandler.name(), "c");
    assert_eq!(Color1TagHandler.name(), "1c");
    assert_eq!(Color2TagHandler.name(), "2c");
    assert_eq!(Color3TagHandler.name(), "3c");
    assert_eq!(Color4TagHandler.name(), "4c");
    assert_eq!(AlphaTagHandler.name(), "alpha");
    assert_eq!(Alpha1TagHandler.name(), "1a");
    assert_eq!(Alpha2TagHandler.name(), "2a");
    assert_eq!(Alpha3TagHandler.name(), "3a");
    assert_eq!(Alpha4TagHandler.name(), "4a");
}

#[test]
fn create_color_handlers_returns_all() {
    let handlers = create_color_handlers();
    assert_eq!(handlers.len(), 10);

    let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();

    assert!(names.contains(&"c"));
    assert!(names.contains(&"1c"));
    assert!(names.contains(&"2c"));
    assert!(names.contains(&"3c"));
    assert!(names.contains(&"4c"));
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"1a"));
    assert!(names.contains(&"2a"));
    assert!(names.contains(&"3a"));
    assert!(names.contains(&"4a"));
}

#[test]
fn validation_consistency() {
    let color_handler = PrimaryColorTagHandler;
    let alpha_handler = AlphaTagHandler;

    // Ensure validate() and process() agree
    let color_valid = "&H123456&";
    let color_invalid = "&H12&";
    let alpha_valid = "&H12&";
    let alpha_invalid = "&H123456&";

    assert!(color_handler.validate(color_valid));
    assert_eq!(color_handler.process(color_valid), TagResult::Processed);

    assert!(!color_handler.validate(color_invalid));
    assert!(matches!(
        color_handler.process(color_invalid),
        TagResult::Failed(_)
    ));

    assert!(alpha_handler.validate(alpha_valid));
    assert_eq!(alpha_handler.process(alpha_valid), TagResult::Processed);

    assert!(!alpha_handler.validate(alpha_invalid));
    assert!(matches!(
        alpha_handler.process(alpha_invalid),
        TagResult::Failed(_)
    ));
}
