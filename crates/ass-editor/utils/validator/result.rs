//! Aggregated validation results.
//!
//! Defines `ValidationResult`, which collects validation issues along with
//! summary statistics and optional timing/caching metadata.

use super::{ValidationIssue, ValidationSeverity};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(feature = "std")]
use std::time::Instant;

/// Validation results with caching and statistics
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// All validation issues found
    pub issues: Vec<ValidationIssue>,

    /// Time taken for validation in microseconds
    #[cfg(feature = "std")]
    pub validation_time_us: u64,

    /// Whether the document passed validation
    pub is_valid: bool,

    /// Number of warnings found
    pub warning_count: usize,

    /// Number of errors found
    pub error_count: usize,

    /// Validation timestamp for cache invalidation
    #[cfg(feature = "std")]
    pub timestamp: Instant,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new(issues: Vec<ValidationIssue>) -> Self {
        let warning_count = issues
            .iter()
            .filter(|i| i.severity == ValidationSeverity::Warning)
            .count();
        let error_count = issues.iter().filter(|i| i.is_error()).count();
        let is_valid = error_count == 0;

        Self {
            issues,
            #[cfg(feature = "std")]
            validation_time_us: 0,
            is_valid,
            warning_count,
            error_count,
            #[cfg(feature = "std")]
            timestamp: Instant::now(),
        }
    }

    /// Filter issues by severity
    pub fn issues_with_severity(&self, min_severity: ValidationSeverity) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity >= min_severity)
            .collect()
    }

    /// Get summary statistics
    pub fn summary(&self) -> String {
        if self.is_valid {
            if self.warning_count > 0 {
                format!("{} warnings", self.warning_count)
            } else {
                "Valid".to_string()
            }
        } else {
            format!(
                "{} errors, {} warnings",
                self.error_count, self.warning_count
            )
        }
    }
}
