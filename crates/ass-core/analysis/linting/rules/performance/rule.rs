//! `PerformanceRule` implementation and its performance heuristics.
//!
//! Houses the [`PerformanceRule`] type along with its [`LintRule`] trait
//! implementation and the inherent helper checks for event count and
//! complex per-event patterns.

use crate::analysis::{
    linting::{IssueCategory, IssueSeverity, LintIssue, LintRule},
    ScriptAnalysis,
};
use alloc::{format, string::ToString, vec::Vec};

/// Rule for detecting potential performance issues in subtitle scripts
///
/// Analyzes scripts for patterns that may negatively impact rendering
/// performance, memory usage, or playback smoothness. Helps identify
/// optimization opportunities in large or complex subtitle files.
///
/// # Performance Checks
///
/// - Event count: Warns when script has excessive number of events
/// - Complex text: Detects events with very long text content
/// - Frequent styling: Identifies excessive use of override tags
///
/// # Performance
///
/// - Time complexity: O(n) for n events
/// - Memory: O(1) additional space
/// - Target: <1ms for typical scripts with 1000+ events
///
/// # Example
///
/// ```rust
/// use ass_core::analysis::linting::rules::performance::PerformanceRule;
/// use ass_core::analysis::linting::LintRule;
/// use ass_core::{Script, ScriptAnalysis};
///
/// let script = Script::parse(r#"
/// [Events]
/// Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
/// "#)?; // Script with many events would trigger warnings
///
/// let analysis = ScriptAnalysis::analyze(&script)?;
/// let rule = PerformanceRule;
/// let issues = rule.check_script(&analysis);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct PerformanceRule;

impl LintRule for PerformanceRule {
    fn id(&self) -> &'static str {
        "performance"
    }

    fn name(&self) -> &'static str {
        "Performance"
    }

    fn description(&self) -> &'static str {
        "Detects potential performance issues in the script"
    }

    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Hint
    }

    fn category(&self) -> IssueCategory {
        IssueCategory::Performance
    }

    fn check_script(&self, analysis: &ScriptAnalysis) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        let dialogue_info = analysis.dialogue_info();
        self.check_event_count(&mut issues, dialogue_info.len());
        self.check_complex_events(&mut issues, dialogue_info);

        issues
    }
}

impl PerformanceRule {
    /// Check for excessive number of events
    fn check_event_count(&self, issues: &mut Vec<LintIssue>, event_count: usize) {
        if event_count > 1000 {
            let issue = LintIssue::new(
                self.default_severity(),
                IssueCategory::Performance,
                self.id(),
                format!("Script has {event_count} events, consider optimization"),
            )
            .with_description("Large number of events may impact rendering performance".to_string())
            .with_suggested_fix(
                "Consider splitting into multiple files or optimizing timing".to_string(),
            );

            issues.push(issue);
        }
    }

    /// Check for performance-impacting patterns in individual events
    fn check_complex_events(
        &self,
        issues: &mut Vec<LintIssue>,
        dialogue_info: &[crate::analysis::DialogueInfo],
    ) {
        for info in dialogue_info {
            let text_analysis = info.text_analysis();
            let text_length = text_analysis.char_count();

            if text_length > 500 {
                let issue = LintIssue::new(
                    self.default_severity(),
                    IssueCategory::Performance,
                    self.id(),
                    format!("Event has very long text ({text_length} characters)"),
                )
                .with_description(
                    "Very long event text may impact rendering performance".to_string(),
                )
                .with_suggested_fix(
                    "Consider splitting long text into multiple events".to_string(),
                );

                issues.push(issue);
            }

            let override_count = text_analysis.override_tags().len();
            if override_count > 20 {
                let issue = LintIssue::new(
                    self.default_severity(),
                    IssueCategory::Performance,
                    self.id(),
                    format!("Event has many override tags ({override_count} blocks)"),
                )
                .with_description(
                    "Excessive override tags may impact rendering performance".to_string(),
                )
                .with_suggested_fix(
                    "Consider using styles instead of many override tags".to_string(),
                );

                issues.push(issue);
            }
        }
    }
}
