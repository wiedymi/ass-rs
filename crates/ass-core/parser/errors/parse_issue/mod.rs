//! Parse issue types for recoverable parsing problems
//!
//! Contains types for representing warnings, errors, and other issues that
//! can be recovered from during parsing. These allow continued parsing
//! while collecting problems for later review.

#[cfg(not(feature = "std"))]
extern crate alloc;

mod category;
mod issue;
mod severity;

#[cfg(test)]
mod tests;

pub use category::IssueCategory;
pub use issue::ParseIssue;
pub use severity::IssueSeverity;
