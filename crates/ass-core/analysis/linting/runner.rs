//! Entry points for running linting over a script.
//!
//! Provides [`lint_script`] and [`lint_script_with_analysis`], which execute
//! all enabled built-in rules and collect issues subject to the supplied
//! [`LintConfig`].

use super::{rules::BuiltinRules, LintConfig, LintIssue};
use crate::{
    analysis::{AnalysisConfig, ScriptAnalysis},
    parser::Script,
    Result,
};
use alloc::vec::Vec;

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
