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

use crate::{parser::Script, Result};
use alloc::{string::String, vec::Vec};
use core::fmt;

pub mod rules;

pub use rules::BuiltinRules;

/// Severity level for lint issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IssueSeverity {
    /// Informational message - no action required
    Info,
    /// Hint for improvement - optional fix
    Hint,
    /// Warning - should be addressed but not critical
    Warning,
    /// Error - must be fixed for proper functionality
    Error,
    /// Critical error - script may not work at all
    Critical,
}

impl fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Hint => write!(f, "hint"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Category of lint issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueCategory {
    /// Timing-related issues
    Timing,
    /// Style definition problems
    Styling,
    /// Text content issues
    Content,
    /// Performance concerns
    Performance,
    /// Spec compliance violations
    Compliance,
    /// Accessibility concerns
    Accessibility,
    /// Encoding or character issues
    Encoding,
}

impl fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timing => write!(f, "timing"),
            Self::Styling => write!(f, "styling"),
            Self::Content => write!(f, "content"),
            Self::Performance => write!(f, "performance"),
            Self::Compliance => write!(f, "compliance"),
            Self::Accessibility => write!(f, "accessibility"),
            Self::Encoding => write!(f, "encoding"),
        }
    }
}

/// Location information for a lint issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueLocation<'a> {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Byte offset in source
    pub offset: usize,
    /// Length of the problematic span
    pub length: usize,
    /// The problematic text span
    pub span: &'a str,
}

/// A single lint issue found in the script.
#[derive(Debug, Clone)]
pub struct LintIssue<'a> {
    /// Severity level
    severity: IssueSeverity,
    /// Category of issue
    category: IssueCategory,
    /// Human-readable message
    message: String,
    /// Optional detailed description
    description: Option<String>,
    /// Location in source (if available)
    location: Option<IssueLocation<'a>>,
    /// Rule ID that generated this issue
    rule_id: &'static str,
    /// Suggested fix (if available)
    suggested_fix: Option<String>,
}

impl<'a> LintIssue<'a> {
    /// Create a new lint issue.
    pub fn new(
        severity: IssueSeverity,
        category: IssueCategory,
        rule_id: &'static str,
        message: String,
    ) -> Self {
        Self {
            severity,
            category,
            message,
            description: None,
            location: None,
            rule_id,
            suggested_fix: None,
        }
    }

    /// Add detailed description.
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Add location information.
    pub fn with_location(mut self, location: IssueLocation<'a>) -> Self {
        self.location = Some(location);
        self
    }

    /// Add suggested fix.
    pub fn with_suggested_fix(mut self, fix: String) -> Self {
        self.suggested_fix = Some(fix);
        self
    }

    /// Get severity level.
    pub fn severity(&self) -> IssueSeverity {
        self.severity
    }

    /// Get issue category.
    pub fn category(&self) -> IssueCategory {
        self.category
    }

    /// Get issue message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get detailed description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get location information.
    pub fn location(&self) -> Option<&IssueLocation<'a>> {
        self.location.as_ref()
    }

    /// Get rule ID.
    pub fn rule_id(&self) -> &'static str {
        self.rule_id
    }

    /// Get suggested fix.
    pub fn suggested_fix(&self) -> Option<&str> {
        self.suggested_fix.as_deref()
    }
}

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
    pub fn with_min_severity(mut self, severity: IssueSeverity) -> Self {
        self.min_severity = severity;
        self
    }

    /// Set maximum number of issues.
    pub fn with_max_issues(mut self, max: usize) -> Self {
        self.max_issues = max;
        self
    }

    /// Enable strict compliance checking.
    pub fn with_strict_compliance(mut self, enabled: bool) -> Self {
        self.strict_mode = enabled;
        self
    }

    /// Check if a rule is enabled.
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        if self.disabled_rules.contains(&rule_id) {
            return false;
        }
        self.enabled_rules.is_empty() || self.enabled_rules.contains(&rule_id)
    }

    /// Check if severity should be reported.
    pub fn should_report_severity(&self, severity: IssueSeverity) -> bool {
        severity >= self.min_severity
    }
}

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

    /// Check script and return issues.
    fn check_script<'a>(&self, script: &'a Script<'a>) -> Vec<LintIssue<'a>>;
}

/// Lint a script with the given configuration.
///
/// Runs all enabled rules against the script and returns found issues,
/// respecting the configuration limits and filters.
pub fn lint_script<'a>(script: &'a Script<'a>, config: &LintConfig) -> Result<Vec<LintIssue<'a>>> {
    let mut issues = Vec::new();
    let rules = BuiltinRules::all_rules();

    for rule in rules {
        if !config.is_rule_enabled(rule.id()) {
            continue;
        }

        let mut rule_issues = rule.check_script(script);
        rule_issues.retain(|issue| config.should_report_severity(issue.severity()));

        issues.extend(rule_issues);

        // Check max issues limit
        if config.max_issues > 0 && issues.len() >= config.max_issues {
            issues.truncate(config.max_issues);
            break;
        }
    }

    Ok(issues)
}
