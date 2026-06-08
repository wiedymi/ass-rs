//! Unit tests for `CoreError` construction and classification.

use super::*;

#[test]
fn error_creation() {
    let parse_err = CoreError::parse("test message");
    assert!(matches!(parse_err, CoreError::Parse(_)));
}

#[test]
fn internal_error() {
    let internal_err = CoreError::internal("something went wrong");
    assert!(matches!(internal_err, CoreError::Internal(_)));
    assert!(internal_err.is_internal_bug());
    assert!(!internal_err.is_recoverable());
}

#[test]
fn error_recoverability() {
    assert!(CoreError::parse("test").is_recoverable());
    assert!(!CoreError::internal("test").is_recoverable());
}

#[test]
fn internal_bug_detection() {
    assert!(CoreError::internal("test").is_internal_bug());
    assert!(!CoreError::parse("test").is_internal_bug());
}
