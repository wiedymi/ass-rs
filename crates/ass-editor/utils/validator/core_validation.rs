//! Core validation logic bridging ass-core analysis and basic checks.
//!
//! Implements the analysis-backed and fallback validation paths for
//! `LazyValidator`, plus structural sanity checks applied to all documents.

use super::{LazyValidator, ValidationIssue, ValidationSeverity};
use crate::core::{EditorDocument, Result};

#[cfg(feature = "analysis")]
use ass_core::analysis::linting::IssueSeverity;
#[cfg(feature = "analysis")]
use ass_core::analysis::ScriptAnalysis;

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

impl LazyValidator {
    /// Validate using ass-core's ScriptAnalysis
    #[cfg(feature = "analysis")]
    pub(super) fn validate_with_core(
        &self,
        content: &str,
        document: &EditorDocument,
    ) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Parse and analyze with ass-core
        document.parse_script_with(|script| {
            // Create ScriptAnalysis with our configuration
            match ScriptAnalysis::analyze_with_config(script, self.analysis_config.clone()) {
                Ok(analysis) => {
                    // Convert core lint issues to our format
                    for lint_issue in analysis.lint_issues() {
                        let severity = match lint_issue.severity() {
                            IssueSeverity::Hint => ValidationSeverity::Info,
                            IssueSeverity::Info => ValidationSeverity::Info,
                            IssueSeverity::Warning => ValidationSeverity::Warning,
                            IssueSeverity::Error => ValidationSeverity::Error,
                            IssueSeverity::Critical => ValidationSeverity::Critical,
                        };

                        let (line, column) = if let Some(location) = lint_issue.location() {
                            (Some(location.line), Some(location.column))
                        } else {
                            (None, None)
                        };

                        let issue = ValidationIssue {
                            severity,
                            line,
                            column,
                            message: lint_issue.message().to_string(),
                            rule: lint_issue.rule_id().to_string(),
                            suggestion: lint_issue.suggested_fix().map(|s| s.to_string()),
                        };

                        issues.push(issue);
                    }
                }
                Err(_) => {
                    // If analysis fails, add a basic error
                    issues.push(ValidationIssue::new(
                        ValidationSeverity::Error,
                        "Failed to analyze script".to_string(),
                        "analyzer".to_string(),
                    ));
                }
            }
        })?;

        // Add basic structural checks even with analysis feature
        self.add_basic_checks(content, &mut issues);

        // Apply severity threshold filter
        issues.retain(|issue| issue.severity >= self.config.severity_threshold);

        // Apply max issues limit
        if self.config.max_issues > 0 && issues.len() > self.config.max_issues {
            issues.truncate(self.config.max_issues);
        }

        Ok(issues)
    }

    /// Fallback validation without ass-core analysis
    #[cfg(not(feature = "analysis"))]
    pub(super) fn validate_with_core(
        &self,
        content: &str,
        _document: &EditorDocument,
    ) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Basic validation without core analysis
        // Note: We can't do full parsing validation without the analysis feature,
        // so we do basic structural checks only
        self.add_basic_checks(content, &mut issues);

        // Apply severity threshold filter
        issues.retain(|issue| issue.severity >= self.config.severity_threshold);

        // Apply max issues limit
        if self.config.max_issues > 0 && issues.len() > self.config.max_issues {
            issues.truncate(self.config.max_issues);
        }

        Ok(issues)
    }

    /// Add basic structural checks that work regardless of analysis feature
    fn add_basic_checks(&self, content: &str, issues: &mut Vec<ValidationIssue>) {
        // Basic checks
        if content.is_empty() {
            issues.push(ValidationIssue::new(
                ValidationSeverity::Warning,
                "Document is empty".to_string(),
                "basic".to_string(),
            ));
        }

        if !content.contains("[Script Info]") {
            issues.push(ValidationIssue::new(
                ValidationSeverity::Warning,
                "Missing [Script Info] section".to_string(),
                "structure".to_string(),
            ));
        }

        if !content.contains("[Events]") {
            issues.push(ValidationIssue::new(
                ValidationSeverity::Warning,
                "Missing [Events] section".to_string(),
                "structure".to_string(),
            ));
        }
    }
}
