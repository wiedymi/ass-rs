//! Negative duration detection rule for ASS script linting.
//!
//! Detects events with negative or zero duration that would cause
//! rendering issues or indicate timing errors in subtitle scripts.

use crate::{
    analysis::{
        linting::{IssueCategory, IssueSeverity, LintIssue, LintRule},
        ScriptAnalysis,
    },
    parser::Section,
    utils::parse_ass_time,
};
use alloc::{format, string::ToString, vec::Vec};

/// Rule for detecting events with negative or zero duration
///
/// Events with start time >= end time are invalid and will not display
/// properly in most subtitle renderers. This rule catches timing errors
/// that could result from manual editing mistakes or conversion issues.
///
/// # Performance
///
/// - Time complexity: O(n) for n events
/// - Memory: O(1) additional space
/// - Target: <0.5ms for 1000 events
///
/// # Example
///
/// ```rust
/// use ass_core::analysis::linting::rules::negative_duration::NegativeDurationRule;
/// use ass_core::analysis::linting::LintRule;
/// use ass_core::analysis::ScriptAnalysis;
/// use ass_core::parser::Script;
///
/// let script = crate::parser::Script::parse(r#"
/// [Events]
/// Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
/// Dialogue: 0,0:00:05.00,0:00:02.00,Default,,0,0,0,,Invalid event
/// "#)?;
///
/// let analysis = ScriptAnalysis::analyze(&script)?;
/// let rule = NegativeDurationRule;
/// let issues = rule.check_script(&analysis);
/// assert!(!issues.is_empty()); // Should detect the negative duration
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct NegativeDurationRule;

impl LintRule for NegativeDurationRule {
    fn id(&self) -> &'static str {
        "negative-duration"
    }

    fn name(&self) -> &'static str {
        "Negative Duration"
    }

    fn description(&self) -> &'static str {
        "Detects events with negative or zero duration"
    }

    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Error
    }

    fn category(&self) -> IssueCategory {
        IssueCategory::Timing
    }

    fn check_script(&self, analysis: &ScriptAnalysis) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(Section::Events(events)) = analysis
            .script()
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            for event in events {
                if let (Ok(start), Ok(end)) =
                    (parse_ass_time(event.start), parse_ass_time(event.end))
                {
                    if start >= end {
                        let issue = LintIssue::new(
                            self.default_severity(),
                            IssueCategory::Timing,
                            self.id(),
                            format!(
                                "Invalid duration: start {} >= end {}",
                                event.start, event.end
                            ),
                        )
                        .with_description(
                            "Events must have positive duration for proper display".to_string(),
                        );

                        issues.push(issue);
                    }
                }
            }
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_metadata_correct() {
        let rule = NegativeDurationRule;
        assert_eq!(rule.id(), "negative-duration");
        assert_eq!(rule.name(), "Negative Duration");
        assert_eq!(rule.default_severity(), IssueSeverity::Error);
    }

    #[test]
    fn empty_script_no_issues() {
        let script_text = "[Script Info]\nTitle: Test";
        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();

        let rule = NegativeDurationRule;
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn valid_duration_no_issues() {
        let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Valid event";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let rule = NegativeDurationRule;
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn negative_duration_detected() {
        let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:05.00,0:00:02.00,Default,,0,0,0,,Invalid event";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let rule = NegativeDurationRule;
        let issues = rule.check_script(&analysis);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity(), IssueSeverity::Error);
        assert_eq!(issues[0].category(), IssueCategory::Timing);
    }

    #[test]
    fn zero_duration_detected() {
        let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:05.00,0:00:05.00,Default,,0,0,0,,Zero duration";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let rule = NegativeDurationRule;
        let issues = rule.check_script(&analysis);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity(), IssueSeverity::Error);
    }

    #[test]
    fn multiple_invalid_durations() {
        let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:05.00,0:00:02.00,Default,,0,0,0,,First invalid
Dialogue: 0,0:00:01.00,0:00:06.00,Default,,0,0,0,,Valid event
Dialogue: 0,0:00:10.00,0:00:10.00,Default,,0,0,0,,Second invalid";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let rule = NegativeDurationRule;
        let issues = rule.check_script(&analysis);

        assert_eq!(issues.len(), 2);
    }
}
