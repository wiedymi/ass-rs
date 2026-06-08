//! Accessibility issue detection rule for ASS script linting.
//!
//! Detects potential accessibility issues in subtitle scripts that could
//! make content difficult to read or understand for users with disabilities
//! or reading difficulties.

mod rule;

#[cfg(test)]
mod tests;

pub use rule::AccessibilityRule;
