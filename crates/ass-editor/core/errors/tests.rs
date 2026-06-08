//! Unit tests for `EditorError` construction, classification, and display

use super::*;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
use ass_core::utils::errors::CoreError;

#[test]
fn error_conversion_from_core() {
    let core_err = CoreError::parse("test error");
    let editor_err: EditorError = core_err.into();
    assert!(matches!(editor_err, EditorError::Core(_)));
}

#[test]
fn error_recoverability() {
    assert!(EditorError::command_failed("test").is_recoverable());
    assert!(EditorError::validation("test").is_recoverable());
    assert!(!EditorError::SessionLimitExceeded {
        current: 10,
        limit: 10
    }
    .is_recoverable());
}

#[test]
fn position_error_detection() {
    assert!(EditorError::InvalidPosition { line: 1, column: 1 }.is_position_error());
    assert!(EditorError::PositionOutOfBounds {
        position: 100,
        length: 50
    }
    .is_position_error());
    assert!(!EditorError::command_failed("test").is_position_error());
}

#[test]
fn history_error_detection() {
    assert!(EditorError::NothingToUndo.is_history_error());
    assert!(EditorError::NothingToRedo.is_history_error());
    assert!(EditorError::HistoryError {
        message: "test".to_string()
    }
    .is_history_error());
    assert!(!EditorError::command_failed("test").is_history_error());
}

#[test]
fn core_error_extraction() {
    let core_err = CoreError::parse("test");
    let editor_err = EditorError::Core(core_err.clone());
    assert_eq!(editor_err.as_core_error(), Some(&core_err));
    assert_eq!(EditorError::command_failed("test").as_core_error(), None);
}

#[test]
fn new_error_types() {
    // Test builder validation error
    let builder_err = EditorError::builder_validation("Invalid field value");
    assert!(builder_err.is_recoverable());
    assert!(matches!(
        builder_err,
        EditorError::BuilderValidationError { .. }
    ));

    // Test serialization error
    let serialization_err = EditorError::serialization("Failed to serialize AST");
    assert!(serialization_err.is_recoverable());
    assert!(matches!(
        serialization_err,
        EditorError::SerializationError { .. }
    ));

    // Test format line error
    let format_err = EditorError::format_line("Invalid format specification");
    assert!(format_err.is_recoverable());
    assert!(matches!(format_err, EditorError::FormatLineError { .. }));
}

#[test]
fn error_display_new_types() {
    let builder_err = EditorError::builder_validation("test");
    assert_eq!(builder_err.to_string(), "Builder validation error: test");

    let serialization_err = EditorError::serialization("test");
    assert_eq!(serialization_err.to_string(), "Serialization error: test");

    let format_err = EditorError::format_line("test");
    assert_eq!(format_err.to_string(), "Format line error: test");
}
