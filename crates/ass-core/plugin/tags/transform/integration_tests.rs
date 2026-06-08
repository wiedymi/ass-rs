//! Integration tests for transform handler registration and validation parity.

use super::validation::validate_numeric_arg;
use super::*;
use crate::plugin::{TagHandler, TagResult};

#[test]
fn create_transform_handlers_returns_all() {
    let handlers = create_transform_handlers();
    assert_eq!(handlers.len(), 8);

    let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();

    assert!(names.contains(&"frz"));
    assert!(names.contains(&"frx"));
    assert!(names.contains(&"fry"));
    assert!(names.contains(&"fscx"));
    assert!(names.contains(&"fscy"));
    assert!(names.contains(&"fax"));
    assert!(names.contains(&"fay"));
    assert!(names.contains(&"fsp"));
}

#[test]
fn whitespace_handling() {
    assert!(validate_numeric_arg(" 123 "));
    assert!(validate_numeric_arg("\t456\t"));
    assert!(validate_numeric_arg(" -789 "));

    // All transform handlers should handle whitespace
    let handler = RotationZTagHandler;
    assert_eq!(handler.process(" 90 "), TagResult::Processed);
}

#[test]
fn validation_consistency() {
    let handlers = create_transform_handlers();

    for handler in &handlers {
        // Ensure validate() and process() agree
        let valid = "123.45";
        let invalid = "abc";

        assert!(handler.validate(valid));
        assert_eq!(handler.process(valid), TagResult::Processed);

        assert!(!handler.validate(invalid));
        assert!(matches!(handler.process(invalid), TagResult::Failed(_)));
    }
}

#[test]
fn special_transform_values() {
    // Test special rotation values
    let rot_handler = RotationZTagHandler;
    assert_eq!(rot_handler.process("0"), TagResult::Processed); // No rotation
    assert_eq!(rot_handler.process("360"), TagResult::Processed); // Full rotation
    assert_eq!(rot_handler.process("-360"), TagResult::Processed); // Reverse full
    assert_eq!(rot_handler.process("720"), TagResult::Processed); // Double rotation

    // Test special scale values
    let scale_handler = ScaleXTagHandler;
    assert_eq!(scale_handler.process("0"), TagResult::Processed); // Invisible
    assert_eq!(scale_handler.process("100"), TagResult::Processed); // Normal
    assert_eq!(scale_handler.process("-100"), TagResult::Processed); // Flipped

    // Test special shear values
    let shear_handler = ShearXTagHandler;
    assert_eq!(shear_handler.process("0"), TagResult::Processed); // No shear
}
