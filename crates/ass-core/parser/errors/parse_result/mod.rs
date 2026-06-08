//! Parse result types for error handling and issue collection
//!
//! Provides result types that can carry both successful parsing results
//! and accumulated issues/warnings. Enables partial recovery parsing
//! where some errors can be worked around while still collecting problems.

use super::parse_error::ParseError;

mod result_with_issues;

#[cfg(test)]
mod tests;

pub use result_with_issues::ParseResultWithIssues;

/// Result type for operations that can produce parse issues
///
/// Standard Result type using `ParseError` for the error case.
/// Use `ParseResultWithIssues` for operations that need to collect
/// warnings and recoverable errors alongside the main result.
pub type ParseResult<T> = Result<T, ParseError>;
