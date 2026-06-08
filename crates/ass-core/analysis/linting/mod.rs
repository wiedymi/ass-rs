//! Linting and validation for ASS subtitle scripts.
//!
//! Provides comprehensive linting capabilities to detect common issues, spec violations,
//! and performance problems in ASS scripts. Designed for editor integration with
//! configurable severity levels and extensible rule system.
//!
//! # Features
//!
//! - **Comprehensive validation**: Timing, styling, formatting, and spec compliance
//! - **Configurable severity**: Error, warning, info, and hint levels
//! - **Extensible rules**: Trait-based system for custom linting rules
//! - **Performance optimized**: Zero-copy analysis with <1ms per rule
//! - **Editor integration**: Rich diagnostic information with precise locations
//!
//! # Built-in Rules
//!
//! - Timing validation: Overlaps, negative durations, unrealistic timing
//! - Style validation: Missing styles, invalid colors, font issues
//! - Text validation: Encoding issues, malformed tags, accessibility
//! - Performance: Complex animations, large fonts, excessive overlaps
//! - Spec compliance: Invalid sections, deprecated features, compatibility

mod category;
mod config;
mod issue;
mod rule;
pub mod rules;
mod runner;
mod severity;

#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod issue_tests;
#[cfg(test)]
mod runner_tests;

pub use category::IssueCategory;
pub use config::LintConfig;
pub use issue::{IssueLocation, LintIssue};
pub use rule::LintRule;
pub use rules::BuiltinRules;
pub use runner::{lint_script, lint_script_with_analysis};
pub use severity::IssueSeverity;
