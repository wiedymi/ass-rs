//! Parser error types for ASS script parsing
//!
//! Provides comprehensive error handling for ASS subtitle script parsing with
//! detailed error messages and recovery information for interactive editing.
//!
//! # Error Philosophy
//!
//! - Prefer recovery over failure where possible
//! - Provide detailed location information for editor integration
//! - Group related errors for efficient handling
//! - Include suggestions for common mistakes
//!
//! # Module Organization
//!
//! - `parse_error` - Unrecoverable parsing errors
//! - `parse_issue` - Recoverable issues and warnings
//! - `parse_result` - Result types for error handling

pub mod parse_error;
pub mod parse_issue;
pub mod parse_result;

pub use parse_error::ParseError;
pub use parse_issue::{IssueCategory, IssueSeverity, ParseIssue};
pub use parse_result::{ParseResult, ParseResultWithIssues};
