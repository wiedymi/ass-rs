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
//! use ass_core::parser::Script;
//!
//! let script = crate::parser::Script::parse("...")?;
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

use super::LintRule;
use alloc::{boxed::Box, vec::Vec};

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

/// Built-in lint rules registry
///
/// Provides access to all built-in rules that check for common issues
/// in ASS subtitle scripts. Rules are organized by category and can be
/// used individually or as a complete set.
///
/// # Performance
///
/// All rules are designed for efficient execution with minimal memory
/// overhead. Most rules have O(n) or O(n log n) time complexity.
///
/// # Rule List
///
/// - `TimingOverlapRule`: Detects overlapping dialogue events
/// - `NegativeDurationRule`: Finds events with invalid durations
/// - `InvalidColorRule`: Validates color formats in styles and tags
/// - `MissingStyleRule`: Checks for undefined style references
/// - `InvalidTagRule`: Detects malformed override tags
/// - `PerformanceRule`: Identifies performance-impacting patterns
/// - `EncodingRule`: Validates text encoding and character usage
/// - `AccessibilityRule`: Ensures readability and compatibility
pub struct BuiltinRules;

impl BuiltinRules {
    /// Get all built-in linting rules
    ///
    /// Returns a vector of all available built-in rules ready for use.
    /// Rules are returned in their default configuration with standard
    /// severity levels and categories.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ass_core::analysis::linting::rules::BuiltinRules;
    ///
    /// let rules = BuiltinRules::all_rules();
    /// assert_eq!(rules.len(), 8); // All built-in rules
    /// ```
    pub fn all_rules() -> Vec<Box<dyn LintRule>> {
        vec![
            Box::new(TimingOverlapRule),
            Box::new(NegativeDurationRule),
            Box::new(InvalidColorRule),
            Box::new(MissingStyleRule),
            Box::new(InvalidTagRule),
            Box::new(PerformanceRule),
            Box::new(EncodingRule),
            Box::new(AccessibilityRule),
        ]
    }

    /// Get rules by category
    ///
    /// Returns only rules that check issues in the specified category.
    /// Useful for focused linting or when only certain types of issues
    /// need to be checked.
    ///
    /// # Arguments
    ///
    /// * `category` - The issue category to filter by
    ///
    /// # Example
    ///
    /// ```rust
    /// use ass_core::analysis::linting::{IssueCategory, rules::BuiltinRules};
    ///
    /// let timing_rules = BuiltinRules::rules_for_category(IssueCategory::Timing);
    /// // Returns timing-related rules only
    /// ```
    pub fn rules_for_category(category: super::IssueCategory) -> Vec<Box<dyn LintRule>> {
        Self::all_rules()
            .into_iter()
            .filter(|rule| rule.category() == category)
            .collect()
    }

    /// Get rule by ID
    ///
    /// Returns the rule with the specified ID, or None if no such rule exists.
    /// Rule IDs are unique identifiers used for configuration and reporting.
    ///
    /// # Arguments
    ///
    /// * `id` - The rule ID to search for
    ///
    /// # Example
    ///
    /// ```rust
    /// use ass_core::analysis::linting::rules::BuiltinRules;
    ///
    /// let rule = BuiltinRules::rule_by_id("timing-overlap");
    /// assert!(rule.is_some());
    /// assert_eq!(rule.unwrap().id(), "timing-overlap");
    /// ```
    pub fn rule_by_id(id: &str) -> Option<Box<dyn LintRule>> {
        Self::all_rules().into_iter().find(|rule| rule.id() == id)
    }

    /// Get all rule IDs
    ///
    /// Returns a vector of all available rule IDs for configuration
    /// and reporting purposes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ass_core::analysis::linting::rules::BuiltinRules;
    ///
    /// let ids = BuiltinRules::all_rule_ids();
    /// assert!(ids.contains(&"timing-overlap"));
    /// assert!(ids.contains(&"negative-duration"));
    /// ```
    pub fn all_rule_ids() -> Vec<&'static str> {
        Self::all_rules().iter().map(|rule| rule.id()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_rules_count_correct() {
        let rules = BuiltinRules::all_rules();
        assert_eq!(rules.len(), 8);
    }

    #[test]
    fn all_rules_have_unique_ids() {
        let rules = BuiltinRules::all_rules();
        let mut ids = Vec::new();

        for rule in rules {
            let id = rule.id();
            assert!(!ids.contains(&id), "Duplicate rule ID: {}", id);
            ids.push(id);
        }
    }

    #[test]
    fn rule_by_id_works() {
        let rule = BuiltinRules::rule_by_id("timing-overlap");
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().id(), "timing-overlap");

        let missing = BuiltinRules::rule_by_id("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn all_rule_ids_complete() {
        let ids = BuiltinRules::all_rule_ids();
        let expected_ids = [
            "timing-overlap",
            "negative-duration",
            "invalid-color",
            "missing-style",
            "invalid-tag",
            "performance",
            "encoding",
            "accessibility",
        ];

        for expected_id in expected_ids {
            assert!(
                ids.contains(&expected_id),
                "Missing rule ID: {}",
                expected_id
            );
        }
    }
}
