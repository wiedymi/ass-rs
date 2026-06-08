//! Behavioural tests for the transform [`TagHandler`] implementations.

use super::*;
use crate::plugin::{TagHandler, TagResult};

#[test]
fn rotation_handlers_valid() {
    let handlers: [&dyn TagHandler; 3] = [
        &RotationZTagHandler,
        &RotationXTagHandler,
        &RotationYTagHandler,
    ];

    for handler in &handlers {
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("90"), TagResult::Processed);
        assert_eq!(handler.process("-90"), TagResult::Processed);
        assert_eq!(handler.process("360"), TagResult::Processed);
        assert_eq!(handler.process("45.5"), TagResult::Processed);
    }
}

#[test]
fn rotation_handlers_invalid() {
    let handler = RotationZTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("abc"), TagResult::Failed(_)));
    assert!(matches!(handler.process("90deg"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("invalid") {
        assert_eq!(msg, "Rotation tag requires numeric degrees");
    }
}

#[test]
fn scale_handlers_valid() {
    let handlers: [&dyn TagHandler; 2] = [&ScaleXTagHandler, &ScaleYTagHandler];

    for handler in &handlers {
        assert_eq!(handler.process("100"), TagResult::Processed);
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("200"), TagResult::Processed);
        assert_eq!(handler.process("50.5"), TagResult::Processed);
        assert_eq!(handler.process("-100"), TagResult::Processed); // Negative scale flips
    }
}

#[test]
fn scale_handlers_invalid() {
    let handler = ScaleXTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("100%"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("invalid") {
        assert_eq!(msg, "Scale tag requires numeric percent");
    }
}

#[test]
fn shear_handlers_valid() {
    let handlers: [&dyn TagHandler; 2] = [&ShearXTagHandler, &ShearYTagHandler];

    for handler in &handlers {
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("0.5"), TagResult::Processed);
        assert_eq!(handler.process("-0.5"), TagResult::Processed);
        assert_eq!(handler.process("2"), TagResult::Processed);
    }
}

#[test]
fn shear_handlers_invalid() {
    let handler = ShearXTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("invalid") {
        assert_eq!(msg, "Shear tag requires numeric factor");
    }
}

#[test]
fn spacing_handler_valid() {
    let handler = SpacingTagHandler;
    assert_eq!(handler.process("0"), TagResult::Processed);
    assert_eq!(handler.process("5"), TagResult::Processed);
    assert_eq!(handler.process("-2"), TagResult::Processed);
    assert_eq!(handler.process("1.5"), TagResult::Processed);
}

#[test]
fn spacing_handler_invalid() {
    let handler = SpacingTagHandler;
    assert!(matches!(handler.process(""), TagResult::Failed(_)));
    assert!(matches!(handler.process("5px"), TagResult::Failed(_)));

    if let TagResult::Failed(msg) = handler.process("invalid") {
        assert_eq!(msg, "Spacing tag requires numeric pixels");
    }
}

#[test]
fn handlers_have_correct_names() {
    assert_eq!(RotationZTagHandler.name(), "frz");
    assert_eq!(RotationXTagHandler.name(), "frx");
    assert_eq!(RotationYTagHandler.name(), "fry");
    assert_eq!(ScaleXTagHandler.name(), "fscx");
    assert_eq!(ScaleYTagHandler.name(), "fscy");
    assert_eq!(ShearXTagHandler.name(), "fax");
    assert_eq!(ShearYTagHandler.name(), "fay");
    assert_eq!(SpacingTagHandler.name(), "fsp");
}
