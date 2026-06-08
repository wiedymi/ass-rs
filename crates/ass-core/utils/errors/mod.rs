//! Core error types for ASS-RS utilities and cross-module error handling
//!
//! Provides the main `CoreError` enum that wraps all error types from different
//! modules in the crate. Designed for easy error propagation and conversion.
//!
//! # Error Philosophy
//!
//! - Use `thiserror` for structured error handling (no `anyhow` bloat)
//! - Provide detailed context for debugging and user feedback
//! - Support error chains with source information
//! - Include suggestions for common error scenarios
//! - Maintain zero-cost error handling where possible
//!
//! # Examples
//!
//! ```rust
//! use ass_core::utils::errors::{CoreError, Result, ErrorCategory};
//!
//! // Create specific error types
//! let color_err = CoreError::invalid_color("invalid");
//! let time_err = CoreError::invalid_time("1:23", "missing seconds");
//!
//! // Check error properties
//! assert_eq!(color_err.category(), ErrorCategory::Format);
//! assert!(color_err.suggestion().is_some());
//! ```

mod category;
mod constructors;
mod conversions;
mod core;
pub mod encoding;
mod format;
pub mod resource;

// Re-export all public types to maintain API compatibility
pub use category::ErrorCategory;
pub use core::{CoreError, Result};

// Re-export utility functions from sub-modules
pub use encoding::{
    utf8_error, validate_ass_text_content, validate_bom_handling, validate_utf8_detailed,
};
pub use format::{invalid_color, invalid_numeric, invalid_time, validate_color_format};
pub use resource::{
    check_memory_limit, feature_not_supported, out_of_memory, resource_limit_exceeded,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod inline_tests;

#[cfg(test)]
mod conversion_tests;
