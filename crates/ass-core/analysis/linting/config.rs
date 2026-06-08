//! Configuration for linting behavior.
//!
//! Defines [`LintConfig`], controlling which rules run, the minimum
//! severity reported, and the maximum number of issues collected.

use super::IssueSeverity;
use alloc::vec::Vec;

/// Configuration for linting behavior.
#[derive(Debug, Clone)]
pub struct LintConfig {
    /// Minimum severity level to report
    pub min_severity: IssueSeverity,
    /// Maximum number of issues to report (0 = unlimited)
    pub max_issues: usize,
    /// Enable strict compliance mode
    pub strict_mode: bool,
    /// Enabled rule IDs (empty = all enabled)
    pub enabled_rules: Vec<&'static str>,
    /// Disabled rule IDs
    pub disabled_rules: Vec<&'static str>,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            min_severity: IssueSeverity::Info,
            max_issues: 0, // Unlimited
            strict_mode: false,
            enabled_rules: Vec::new(),
            disabled_rules: Vec::new(),
        }
    }
}

impl LintConfig {
    /// Set minimum severity level.
    #[must_use]
    pub const fn with_min_severity(mut self, severity: IssueSeverity) -> Self {
        self.min_severity = severity;
        self
    }

    /// Set maximum number of issues.
    #[must_use]
    pub const fn with_max_issues(mut self, max: usize) -> Self {
        self.max_issues = max;
        self
    }

    /// Enable strict compliance checking.
    #[must_use]
    pub const fn with_strict_compliance(mut self, enabled: bool) -> Self {
        self.strict_mode = enabled;
        self
    }

    /// Check if a rule is enabled.
    #[must_use]
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        if self.disabled_rules.contains(&rule_id) {
            return false;
        }
        self.enabled_rules.is_empty() || self.enabled_rules.contains(&rule_id)
    }

    /// Check if severity should be reported.
    #[must_use]
    pub fn should_report_severity(&self, severity: IssueSeverity) -> bool {
        severity >= self.min_severity
    }
}
