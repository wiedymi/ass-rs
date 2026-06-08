//! Built-in lint rule registry aggregating all rule implementations.
//!
//! Houses the [`BuiltinRules`] facade used to enumerate, filter, and look up
//! the complete set of built-in linting rules.

use alloc::{boxed::Box, vec, vec::Vec};

use super::{
    AccessibilityRule, EncodingRule, InvalidColorRule, InvalidTagRule, MissingStyleRule,
    NegativeDurationRule, PerformanceRule, TimingOverlapRule,
};
use crate::analysis::linting::{IssueCategory, LintRule};

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
    #[must_use]
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
    #[must_use]
    pub fn rules_for_category(category: IssueCategory) -> Vec<Box<dyn LintRule>> {
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
    #[must_use]
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
    #[must_use]
    pub fn all_rule_ids() -> Vec<&'static str> {
        Self::all_rules().iter().map(|rule| rule.id()).collect()
    }
}
