//! Lazy validation wrapper around ass-core's ScriptAnalysis
//!
//! Provides on-demand validation and linting for editor documents,
//! wrapping ass-core's analysis capabilities with caching and
//! incremental update support for better editor performance.

use crate::core::{EditorDocument, Result, errors::EditorError};

#[cfg(feature = "analysis")]
use ass_core::analysis::{AnalysisConfig, ScriptAnalysis, ScriptAnalysisOptions};

#[cfg(feature = "analysis")]
use ass_core::analysis::linting::IssueSeverity;

#[cfg(not(feature = "std"))]
use alloc::{format, string::{String, ToString}, vec::Vec};

#[cfg(feature = "std")]
use std::time::Instant;

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// Informational message
    Info,
    /// Warning that doesn't prevent script execution
    Warning,
    /// Error that may cause rendering issues
    Error,
    /// Critical error that prevents script execution
    Critical,
}

impl Default for ValidationSeverity {
    fn default() -> Self {
        Self::Info
    }
}

/// A validation issue found in the document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    /// Severity of the issue
    pub severity: ValidationSeverity,

    /// Line number where the issue occurs (1-indexed)
    pub line: Option<usize>,

    /// Column number where the issue occurs (1-indexed)  
    pub column: Option<usize>,

    /// Human-readable description of the issue
    pub message: String,

    /// Rule or check that generated this issue
    pub rule: String,

    /// Suggested fix for the issue (if available)
    pub suggestion: Option<String>,
}

impl ValidationIssue {
    /// Create a new validation issue
    ///
    /// # Examples
    ///
    /// ```
    /// use ass_editor::utils::validator::{ValidationIssue, ValidationSeverity};
    ///
    /// let issue = ValidationIssue::new(
    ///     ValidationSeverity::Warning,
    ///     "Missing subtitle end time".to_string(),
    ///     "timing_check".to_string()
    /// )
    /// .at_location(10, 25)
    /// .with_suggestion("Add explicit end time".to_string());
    ///
    /// assert_eq!(issue.line, Some(10));
    /// assert_eq!(issue.column, Some(25));
    /// assert!(!issue.is_error());
    /// ```
    pub fn new(severity: ValidationSeverity, message: String, rule: String) -> Self {
        Self {
            severity,
            line: None,
            column: None,
            message,
            rule,
            suggestion: None,
        }
    }

    /// Set the location of this issue
    #[must_use]
    pub fn at_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Add a suggestion for fixing this issue
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    /// Check if this is an error or critical issue
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(
            self.severity,
            ValidationSeverity::Error | ValidationSeverity::Critical
        )
    }

    /// Check if this is a warning or higher
    #[must_use]
    pub const fn is_warning_or_higher(&self) -> bool {
        matches!(
            self.severity,
            ValidationSeverity::Warning | ValidationSeverity::Error | ValidationSeverity::Critical
        )
    }
}

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

/// Lazy validator that wraps ass-core's ScriptAnalysis
///
/// Provides on-demand validation with caching and incremental updates
/// as specified in the architecture (line 164).
#[derive(Debug)]
pub struct LazyValidator {
    /// Configuration for validation behavior
    config: ValidatorConfig,

    /// Cached validation result
    cached_result: Option<ValidationResult>,

    /// Hash of last validated content
    content_hash: u64,

    /// Last validation timestamp
    #[cfg(feature = "std")]
    last_validation: Option<Instant>,

    /// Core analysis configuration
    #[cfg(feature = "analysis")]
    analysis_config: AnalysisConfig,
}

impl LazyValidator {
    /// Create a new lazy validator with default configuration
    pub fn new() -> Self {
        Self::with_config(ValidatorConfig::default())
    }

    /// Create a new lazy validator with custom configuration
    pub fn with_config(config: ValidatorConfig) -> Self {
        Self {
            #[cfg(feature = "analysis")]
            analysis_config: AnalysisConfig {
                options: {
                    let mut options = ScriptAnalysisOptions::empty();
                    if config.enable_unicode_checks {
                        options |= ScriptAnalysisOptions::UNICODE_LINEBREAKS;
                    }
                    if config.enable_performance_hints {
                        options |= ScriptAnalysisOptions::PERFORMANCE_HINTS;
                    }
                    if config.enable_spec_compliance {
                        options |= ScriptAnalysisOptions::STRICT_COMPLIANCE;
                    }
                    if config.enable_accessibility_checks {
                        options |= ScriptAnalysisOptions::BIDI_ANALYSIS;
                    }
                    options
                },
                max_events_threshold: 1000,
            },
            config,
            cached_result: None,
            content_hash: 0,
            #[cfg(feature = "std")]
            last_validation: None,
        }
    }

    /// Validate document using ass-core's ScriptAnalysis
    pub fn validate(&mut self, document: &EditorDocument) -> Result<&ValidationResult> {
        let content = document.text();
        let content_hash = self.calculate_hash(&content);

        // Check if we can use cached result
        if self.should_use_cache(content_hash) {
            return self.cached_result.as_ref()
                .ok_or_else(|| EditorError::command_failed("Cache validation inconsistency: cached result expected but not found"));
        }

        #[cfg(feature = "std")]
        let start_time = Instant::now();

        // Perform validation using ass-core
        let issues = self.validate_with_core(&content, document)?;

        // Update cache
        let mut result = ValidationResult::new(issues);

        #[cfg(feature = "std")]
        {
            result.validation_time_us = start_time.elapsed().as_micros() as u64;
        }

        self.cached_result = Some(result);
        self.content_hash = content_hash;

        #[cfg(feature = "std")]
        {
            self.last_validation = Some(Instant::now());
        }

        self.cached_result.as_ref()
            .ok_or_else(|| EditorError::command_failed("Validation completed but cached result is missing"))
    }

    /// Force validation even if cached result exists
    pub fn force_validate(&mut self, document: &EditorDocument) -> Result<&ValidationResult> {
        self.cached_result = None; // Clear cache
        self.validate(document)
    }

    /// Check if document is valid (quick check using cache if available)
    pub fn is_valid(&mut self, document: &EditorDocument) -> Result<bool> {
        Ok(self.validate(document)?.is_valid)
    }

    /// Get cached validation result without revalidating
    pub fn cached_result(&self) -> Option<&ValidationResult> {
        self.cached_result.as_ref()
    }

    /// Clear validation cache
    pub fn clear_cache(&mut self) {
        self.cached_result = None;
        self.content_hash = 0;
        #[cfg(feature = "std")]
        {
            self.last_validation = None;
        }
    }

    /// Update configuration
    pub fn set_config(&mut self, config: ValidatorConfig) {
        self.config = config;
        self.clear_cache(); // Config change invalidates cache

        #[cfg(feature = "analysis")]
        {
            self.analysis_config = AnalysisConfig {
                options: {
                    let mut options = ScriptAnalysisOptions::empty();
                    if self.config.enable_unicode_checks {
                        options |= ScriptAnalysisOptions::UNICODE_LINEBREAKS;
                    }
                    if self.config.enable_performance_hints {
                        options |= ScriptAnalysisOptions::PERFORMANCE_HINTS;
                    }
                    if self.config.enable_spec_compliance {
                        options |= ScriptAnalysisOptions::STRICT_COMPLIANCE;
                    }
                    if self.config.enable_accessibility_checks {
                        options |= ScriptAnalysisOptions::BIDI_ANALYSIS;
                    }
                    options
                },
                max_events_threshold: 1000,
            };
        }
    }

    /// Validate using ass-core's ScriptAnalysis
    #[cfg(feature = "analysis")]
    fn validate_with_core(
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
    fn validate_with_core(
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

    /// Check if cached result can be used
    fn should_use_cache(&self, content_hash: u64) -> bool {
        if self.cached_result.is_none() || self.content_hash != content_hash {
            return false;
        }

        #[cfg(feature = "std")]
        {
            if let Some(last_validation) = self.last_validation {
                return last_validation.elapsed() < self.config.min_validation_interval;
            }
        }

        true
    }

    /// Calculate hash of content for cache invalidation
    fn calculate_hash(&self, content: &str) -> u64 {
        // Simple FNV hash
        let mut hash = 0xcbf29ce484222325u64;
        for byte in content.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}

impl Default for LazyValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EditorDocument;

    #[test]
    fn test_validation_issue_creation() {
        let issue = ValidationIssue::new(
            ValidationSeverity::Warning,
            "Test issue".to_string(),
            "test_rule".to_string(),
        )
        .at_location(10, 5)
        .with_suggestion("Fix this".to_string());

        assert_eq!(issue.severity, ValidationSeverity::Warning);
        assert_eq!(issue.line, Some(10));
        assert_eq!(issue.column, Some(5));
        assert_eq!(issue.suggestion, Some("Fix this".to_string()));
        assert!(issue.is_warning_or_higher());
        assert!(!issue.is_error());
    }

    #[test]
    fn test_validation_result() {
        let issues = vec![
            ValidationIssue::new(
                ValidationSeverity::Warning,
                "Warning".to_string(),
                "rule1".to_string(),
            ),
            ValidationIssue::new(
                ValidationSeverity::Error,
                "Error".to_string(),
                "rule2".to_string(),
            ),
        ];

        let result = ValidationResult::new(issues);
        assert!(!result.is_valid);
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.error_count, 1);
        assert!(result.summary().contains("1 errors"));
    }

    #[test]
    fn test_lazy_validator() {
        let content = r#"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:05.00,0:00:10.00,Default,John,0,0,0,,Hello"#;

        let document = EditorDocument::from_content(content).unwrap();
        let mut validator = LazyValidator::new();

        let result = validator.validate(&document).unwrap();
        // Should pass basic validation
        assert!(result.is_valid);
        let issues_count = result.issues.len();

        // Test caching
        let result2 = validator.validate(&document).unwrap();
        assert_eq!(issues_count, result2.issues.len());
    }

    #[test]
    fn test_validator_config() {
        let config = ValidatorConfig {
            enable_performance_hints: false,
            max_issues: 5,
            severity_threshold: ValidationSeverity::Warning,
            ..Default::default()
        };

        let mut validator = LazyValidator::with_config(config);

        // Test config update
        let new_config = ValidatorConfig {
            max_issues: 10,
            ..Default::default()
        };
        validator.set_config(new_config);

        // Cache should be cleared
        assert!(validator.cached_result().is_none());
    }

    #[test]
    fn test_validation_with_missing_sections() {
        let content = "Title: Incomplete";
        let document = EditorDocument::from_content(content).unwrap();
        let mut validator = LazyValidator::new();

        let result = validator.validate(&document).unwrap();
        // Should have warnings about missing sections
        assert!(result.warning_count > 0);
        let warnings = result.issues_with_severity(ValidationSeverity::Warning);
        assert!(!warnings.is_empty());
    }
}
