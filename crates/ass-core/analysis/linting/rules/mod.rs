//! Built-in linting rules for ASS script validation.
//!
//! This module contains implementations of all built-in linting rules
//! that check for common issues in ASS subtitle scripts. Each rule
//! is implemented in a separate module for better maintainability.
//!
//! # Rule Categories
//!
//! - **Timing Rules**: Check for overlaps, negative durations, and timing issues
//! - **Style Rules**: Validate style references and color formats
//! - **Content Rules**: Check tag validity and text formatting
//! - **Performance Rules**: Detect performance-impacting patterns
//! - **Accessibility Rules**: Ensure compatibility and readability
//!
//! # Example
//!
//! ```rust
//! use ass_core::analysis::linting::rules::BuiltinRules;
//! use ass_core::analysis::linting::LintRule;
//! use ass_core::{Script, ScriptAnalysis};
//!
//! let script = Script::parse("...")?;
//! let rules = BuiltinRules::all_rules();
//!
//! for rule in rules {
//!     let analysis = ScriptAnalysis::analyze(&script).unwrap();
//!     let issues = rule.check_script(&analysis);
//!     for issue in issues {
//!         println!("{}: {}", rule.name(), issue.message());
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod accessibility;
pub mod encoding;
pub mod invalid_color;
pub mod invalid_tag;
pub mod missing_style;
pub mod negative_duration;
pub mod performance;
pub mod timing_overlap;

pub use accessibility::AccessibilityRule;
pub use encoding::EncodingRule;
pub use invalid_color::InvalidColorRule;
pub use invalid_tag::InvalidTagRule;
pub use missing_style::MissingStyleRule;
pub use negative_duration::NegativeDurationRule;
pub use performance::PerformanceRule;
pub use timing_overlap::TimingOverlapRule;

mod builtin;

pub use builtin::BuiltinRules;

#[cfg(test)]
mod builtin_tests;
