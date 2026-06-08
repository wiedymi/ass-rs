//! Configuration for the lazy validator.
//!
//! Defines `ValidatorConfig`, controlling validation behavior such as
//! enabled rule sets, severity thresholds, and caching intervals.

use super::ValidationSeverity;

/// Configuration for the lazy validator
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Enable automatic validation after document changes
    pub auto_validate: bool,

    /// Minimum time between validations
    #[cfg(feature = "std")]
    pub min_validation_interval: std::time::Duration,

    /// Maximum number of issues to report
    pub max_issues: usize,

    /// Severity threshold for reporting issues
    pub severity_threshold: ValidationSeverity,

    /// Enable specific validation rules
    pub enable_performance_hints: bool,
    pub enable_accessibility_checks: bool,
    pub enable_spec_compliance: bool,
    pub enable_unicode_checks: bool,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            auto_validate: true,
            #[cfg(feature = "std")]
            min_validation_interval: std::time::Duration::from_millis(500),
            max_issues: 100,
            severity_threshold: ValidationSeverity::Info,
            enable_performance_hints: true,
            enable_accessibility_checks: true,
            enable_spec_compliance: true,
            enable_unicode_checks: true,
        }
    }
}
