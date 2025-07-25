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

use crate::{
    analysis::{AnalysisConfig, ScriptAnalysis},
    parser::Script,
    Result,
};
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
pub struct IssueLocation {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Byte offset in source
    pub offset: usize,
    /// Length of the problematic span
    pub length: usize,
    /// The problematic text span
    pub span: String,
}

/// A single lint issue found in the script.
#[derive(Debug, Clone)]
pub struct LintIssue {
    /// Severity level
    severity: IssueSeverity,
    /// Category of issue
    category: IssueCategory,
    /// Human-readable message
    message: String,
    /// Optional detailed description
    description: Option<String>,
    /// Location in source (if available)
    location: Option<IssueLocation>,
    /// Rule ID that generated this issue
    rule_id: &'static str,
    /// Suggested fix (if available)
    suggested_fix: Option<String>,
}

impl LintIssue {
    /// Create a new lint issue.
    #[must_use]
    pub const fn new(
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
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Add location information.
    #[must_use]
    pub fn with_location(mut self, location: IssueLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add suggested fix.
    #[must_use]
    pub fn with_suggested_fix(mut self, fix: String) -> Self {
        self.suggested_fix = Some(fix);
        self
    }

    /// Get severity level.
    #[must_use]
    pub const fn severity(&self) -> IssueSeverity {
        self.severity
    }

    /// Get issue category.
    #[must_use]
    pub const fn category(&self) -> IssueCategory {
        self.category
    }

    /// Get issue message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get detailed description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get location information.
    #[must_use]
    pub const fn location(&self) -> Option<&IssueLocation> {
        self.location.as_ref()
    }

    /// Get rule ID.
    #[must_use]
    pub const fn rule_id(&self) -> &'static str {
        self.rule_id
    }

    /// Get suggested fix.
    #[must_use]
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

/// Lint a script with the given configuration.
/// Lint script with existing analysis
///
/// Runs all enabled rules against the provided analysis and returns found issues,
/// respecting the configuration limits and filters.
/// Lint script using existing analysis
///
/// # Errors
///
/// Returns an error if linting rule execution fails.
pub fn lint_script_with_analysis(
    analysis: &ScriptAnalysis,
    config: &LintConfig,
) -> Result<Vec<LintIssue>> {
    let mut issues = Vec::new();
    let rules = BuiltinRules::all_rules();

    for rule in rules {
        if !config.is_rule_enabled(rule.id()) {
            continue;
        }

        let mut rule_issues = rule.check_script(analysis);
        rule_issues.retain(|issue| config.should_report_severity(issue.severity()));

        issues.extend(rule_issues);

        if config.max_issues > 0 && issues.len() >= config.max_issues {
            issues.truncate(config.max_issues);
            break;
        }
    }

    Ok(issues)
}

/// Lint script with configuration
///
/// Creates a minimal analysis without linting, then runs all enabled rules
/// against the script and returns found issues, respecting the configuration
/// limits and filters.
///
/// # Errors
///
/// Returns an error if script analysis or linting rule execution fails.
pub fn lint_script(script: &Script, config: &LintConfig) -> Result<Vec<LintIssue>> {
    // Create analysis without linting to avoid circular dependency
    let mut analysis = ScriptAnalysis {
        script,
        lint_issues: Vec::new(),
        resolved_styles: Vec::new(),
        dialogue_info: Vec::new(),
        config: AnalysisConfig::default(),
        #[cfg(feature = "plugins")]
        registry: None,
    };

    // Run only style resolution and event analysis (no linting)
    analysis.resolve_all_styles();
    analysis.analyze_events();

    // Now run linting with the prepared analysis
    lint_script_with_analysis(&analysis, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Script;

    #[test]
    fn issue_severity_display() {
        assert_eq!(IssueSeverity::Info.to_string(), "info");
        assert_eq!(IssueSeverity::Hint.to_string(), "hint");
        assert_eq!(IssueSeverity::Warning.to_string(), "warning");
        assert_eq!(IssueSeverity::Error.to_string(), "error");
        assert_eq!(IssueSeverity::Critical.to_string(), "critical");
    }

    #[test]
    fn issue_severity_ordering() {
        assert!(IssueSeverity::Info < IssueSeverity::Hint);
        assert!(IssueSeverity::Hint < IssueSeverity::Warning);
        assert!(IssueSeverity::Warning < IssueSeverity::Error);
        assert!(IssueSeverity::Error < IssueSeverity::Critical);
    }

    #[test]
    fn issue_category_display() {
        assert_eq!(IssueCategory::Timing.to_string(), "timing");
        assert_eq!(IssueCategory::Styling.to_string(), "styling");
        assert_eq!(IssueCategory::Content.to_string(), "content");
        assert_eq!(IssueCategory::Performance.to_string(), "performance");
        assert_eq!(IssueCategory::Compliance.to_string(), "compliance");
        assert_eq!(IssueCategory::Accessibility.to_string(), "accessibility");
        assert_eq!(IssueCategory::Encoding.to_string(), "encoding");
    }

    #[test]
    fn issue_location_creation() {
        let location = IssueLocation {
            line: 42,
            column: 10,
            offset: 1000,
            length: 5,
            span: "error".to_string(),
        };

        assert_eq!(location.line, 42);
        assert_eq!(location.column, 10);
        assert_eq!(location.offset, 1000);
        assert_eq!(location.length, 5);
        assert_eq!(location.span, "error");
    }

    #[test]
    fn lint_issue_creation() {
        let issue = LintIssue::new(
            IssueSeverity::Warning,
            IssueCategory::Timing,
            "test_rule",
            "Test message".to_string(),
        );

        assert_eq!(issue.severity(), IssueSeverity::Warning);
        assert_eq!(issue.category(), IssueCategory::Timing);
        assert_eq!(issue.message(), "Test message");
        assert_eq!(issue.rule_id(), "test_rule");
        assert!(issue.description().is_none());
        assert!(issue.location().is_none());
        assert!(issue.suggested_fix().is_none());
    }

    #[test]
    fn lint_issue_with_description() {
        let issue = LintIssue::new(
            IssueSeverity::Error,
            IssueCategory::Styling,
            "style_rule",
            "Style error".to_string(),
        )
        .with_description("Detailed description".to_string());

        assert_eq!(issue.description(), Some("Detailed description"));
    }

    #[test]
    fn lint_issue_with_location() {
        let location = IssueLocation {
            line: 5,
            column: 2,
            offset: 100,
            length: 3,
            span: "bad".to_string(),
        };

        let issue = LintIssue::new(
            IssueSeverity::Critical,
            IssueCategory::Content,
            "content_rule",
            "Content error".to_string(),
        )
        .with_location(location);

        let loc = issue.location().unwrap();
        assert_eq!(loc.line, 5);
        assert_eq!(loc.column, 2);
        assert_eq!(loc.span, "bad");
    }

    #[test]
    fn lint_issue_with_suggested_fix() {
        let issue = LintIssue::new(
            IssueSeverity::Hint,
            IssueCategory::Performance,
            "perf_rule",
            "Performance hint".to_string(),
        )
        .with_suggested_fix("Use simpler approach".to_string());

        assert_eq!(issue.suggested_fix(), Some("Use simpler approach"));
    }

    #[test]
    fn lint_config_default() {
        let config = LintConfig::default();
        assert_eq!(config.min_severity, IssueSeverity::Info);
        assert_eq!(config.max_issues, 0);
        assert!(!config.strict_mode);
        assert!(config.enabled_rules.is_empty());
        assert!(config.disabled_rules.is_empty());
    }

    #[test]
    fn lint_config_with_min_severity() {
        let config = LintConfig::default().with_min_severity(IssueSeverity::Warning);
        assert_eq!(config.min_severity, IssueSeverity::Warning);
    }

    #[test]
    fn lint_config_with_max_issues() {
        let config = LintConfig::default().with_max_issues(100);
        assert_eq!(config.max_issues, 100);
    }

    #[test]
    fn lint_config_with_strict_compliance() {
        let config = LintConfig::default().with_strict_compliance(true);
        assert!(config.strict_mode);
    }

    #[test]
    fn lint_config_is_rule_enabled_all_disabled() {
        let mut config = LintConfig::default();
        config.disabled_rules.push("test_rule");

        assert!(!config.is_rule_enabled("test_rule"));
        assert!(config.is_rule_enabled("other_rule"));
    }

    #[test]
    fn lint_config_is_rule_enabled_specific_enabled() {
        let mut config = LintConfig::default();
        config.enabled_rules.push("test_rule");

        assert!(config.is_rule_enabled("test_rule"));
        assert!(!config.is_rule_enabled("other_rule"));
    }

    #[test]
    fn lint_config_should_report_severity() {
        let config = LintConfig::default().with_min_severity(IssueSeverity::Warning);

        assert!(!config.should_report_severity(IssueSeverity::Info));
        assert!(!config.should_report_severity(IssueSeverity::Hint));
        assert!(config.should_report_severity(IssueSeverity::Warning));
        assert!(config.should_report_severity(IssueSeverity::Error));
        assert!(config.should_report_severity(IssueSeverity::Critical));
    }

    #[test]
    fn lint_script_empty_script() {
        let script_content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,16,&Hffffff,&Hffffff,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,30,30,30,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";

        let script = Script::parse(script_content).unwrap();
        let config = LintConfig::default();

        let issues = lint_script(&script, &config);
        assert!(issues.is_ok());
    }

    #[test]
    fn lint_script_with_analysis_empty() {
        let script_content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,16,&Hffffff,&Hffffff,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,30,30,30,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";

        let script = Script::parse(script_content).unwrap();
        let analysis = ScriptAnalysis {
            script: &script,
            lint_issues: Vec::new(),
            resolved_styles: Vec::new(),
            dialogue_info: Vec::new(),
            config: AnalysisConfig::default(),
            #[cfg(feature = "plugins")]
            registry: None,
        };

        let config = LintConfig::default();
        let issues = lint_script_with_analysis(&analysis, &config);
        assert!(issues.is_ok());
    }

    #[test]
    fn lint_script_with_max_issues() {
        let script_content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,16,&Hffffff,&Hffffff,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,30,30,30,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";

        let script = Script::parse(script_content).unwrap();
        let config = LintConfig::default().with_max_issues(1);

        let issues = lint_script(&script, &config);
        assert!(issues.is_ok());
        if let Ok(issues) = issues {
            assert!(issues.len() <= 1);
        }
    }

    #[test]
    fn lint_script_with_disabled_rule() {
        // Test to cover the continue statement when rules are disabled (line 318)
        let script_content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,16,&Hffffff,&Hffffff,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,30,30,30,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";

        let script = Script::parse(script_content).unwrap();
        let analysis = ScriptAnalysis {
            script: &script,
            lint_issues: Vec::new(),
            resolved_styles: Vec::new(),
            dialogue_info: Vec::new(),
            config: AnalysisConfig::default(),
            #[cfg(feature = "plugins")]
            registry: None,
        };

        // Create config with specific rules disabled to trigger continue path
        let mut config = LintConfig::default();
        config.disabled_rules.push("accessibility_contrast");
        config.disabled_rules.push("encoding_format");
        config.disabled_rules.push("invalid_color");

        let issues = lint_script_with_analysis(&analysis, &config);
        assert!(issues.is_ok());
    }
}
