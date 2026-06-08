//! Tokenizer state management and issue reporting
//!
//! Provides context tracking and error reporting for the ASS tokenizer.
//! Maintains parsing state and accumulates issues during lexical analysis.

#[cfg(not(feature = "std"))]
extern crate alloc;

mod collector;
mod context;
mod issue;
mod level;

#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod issue_tests;

pub use collector::IssueCollector;
pub use context::TokenContext;
pub use issue::TokenIssue;
pub use level::IssueLevel;
