//! Unit tests for the lazy validator and its supporting types.

use super::*;
use crate::EditorDocument;
#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec};

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
