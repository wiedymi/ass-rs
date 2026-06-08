//! Trait for implementing custom lint rules.
//!
//! Defines [`LintRule`], the extension point used by both built-in and
//! user-provided rules to inspect a [`ScriptAnalysis`] and emit issues.

use super::{IssueCategory, IssueSeverity, LintIssue};
use crate::analysis::ScriptAnalysis;
use alloc::vec::Vec;

/// Trait for implementing custom lint rules.
pub trait LintRule: Send + Sync {
    /// Unique identifier for this rule.
    fn id(&self) -> &'static str;

    /// Human-readable name.
    fn name(&self) -> &'static str;

    /// Rule description.
    fn description(&self) -> &'static str;

    /// Default severity level.
    fn default_severity(&self) -> IssueSeverity;

    /// Issue category this rule checks for.
    fn category(&self) -> IssueCategory;

    /// Check script and return issues.
    fn check_script(&self, analysis: &ScriptAnalysis) -> Vec<LintIssue>;
}
