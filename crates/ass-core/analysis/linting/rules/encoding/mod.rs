//! Encoding issue detection rule for ASS script linting.
//!
//! Detects potential encoding or character issues in subtitle scripts
//! that could cause display problems or compatibility issues.

mod rule;

#[cfg(test)]
mod tests;

pub use rule::EncodingRule;
