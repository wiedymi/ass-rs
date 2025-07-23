//! Performance issue detection rule for ASS script linting.
//!
//! Detects potential performance issues in subtitle scripts that could
//! impact rendering speed, memory usage, or playback smoothness.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_metadata_correct() {
        let rule = PerformanceRule;
        assert_eq!(rule.id(), "performance");
        assert_eq!(rule.name(), "Performance");
        assert_eq!(rule.default_severity(), IssueSeverity::Hint);
    }

    #[test]
    fn empty_script_no_issues() {
        let script_text = "[Script Info]\nTitle: Test";
        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();

        let rule = PerformanceRule;
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn small_script_no_issues() {
        let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Short text
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Another short text";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let rule = PerformanceRule;
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn no_events_section_no_issues() {
        let script_text = r"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF&,&H000000FF&,&H00000000&,&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let rule = PerformanceRule;
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn long_text_event_detected() {
        let long_text = "a".repeat(600);
        let script_text = format!(
            r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{long_text}"
        );

        let script = crate::parser::Script::parse(&script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let rule = PerformanceRule;
        let issues = rule.check_script(&analysis);

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|issue| issue.message().contains("long text")));
    }

    #[test]
    fn many_override_tags_detected() {
        let mut text_with_tags = String::new();
        for i in 0..25 {
            use std::fmt::Write;
            write!(text_with_tags, "{{\\i{}}}text", i % 2).unwrap();
        }

        let script_text = format!(
            r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{text_with_tags}"
        );

        let script = crate::parser::Script::parse(&script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let rule = PerformanceRule;
        let issues = rule.check_script(&analysis);

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|issue| issue.message().contains("override tags")));
    }
}
