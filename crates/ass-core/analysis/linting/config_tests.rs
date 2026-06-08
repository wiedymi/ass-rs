//! Unit tests for the [`LintConfig`] type.

use super::*;

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
