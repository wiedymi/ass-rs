//! Tests for `CoreError` construction, equality, cloning, the `Result` alias,
//! and the standard/core `Error` trait implementations.

use crate::parser::ParseError;
use crate::utils::errors::{CoreError, Result};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn core_error_parse_creation() {
    let error = CoreError::parse("test parse error");
    assert!(matches!(error, CoreError::Parse(_)));

    if let CoreError::Parse(parse_err) = error {
        match parse_err {
            ParseError::IoError { message } => {
                assert_eq!(message, "test parse error");
            }
            _ => panic!("Expected IoError variant"),
        }
    }
}

#[test]
fn core_error_internal_creation() {
    let error = CoreError::internal("internal bug");
    assert!(matches!(error, CoreError::Internal(_)));

    if let CoreError::Internal(message) = error {
        assert_eq!(message, "internal bug");
    }
}

#[test]
fn error_equality_and_cloning() {
    let error1 = CoreError::Tokenization("test".to_string());
    let error2 = CoreError::Tokenization("test".to_string());
    let error3 = CoreError::Tokenization("different".to_string());

    assert_eq!(error1, error2);
    assert_ne!(error1, error3);

    let cloned = error1.clone();
    assert_eq!(error1, cloned);
}

#[test]
fn result_type_alias() {
    fn test_function() -> i32 {
        42
    }

    fn test_error_function() -> Result<i32> {
        Err(CoreError::Analysis("test error".to_string()))
    }

    assert_eq!(test_function(), 42);
    assert!(test_error_function().is_err());
}

#[cfg(feature = "std")]
#[test]
fn std_error_trait() {
    use std::error::Error;

    let error = CoreError::Analysis("test".to_string());
    assert!(error.source().is_none());

    // Test that it implements the Error trait
    let _: &dyn Error = &error;
}

#[cfg(not(feature = "std"))]
#[test]
fn nostd_error_trait() {
    use core::error::Error;
    let error = CoreError::Analysis("test".to_string());

    // Test that it implements the core Error trait in no_std
    let _: &dyn Error = &error;
}
