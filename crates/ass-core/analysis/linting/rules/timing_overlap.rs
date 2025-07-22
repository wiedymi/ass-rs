//! Timing overlap detection rule for ASS script linting.
//!
//! Detects overlapping dialogue events that may cause rendering conflicts
//! using efficient O(n log n) sweep-line algorithm.

use crate::{
    analysis::{
        events::find_overlapping_events,
        linting::{IssueCategory, IssueSeverity, LintIssue, LintRule},
        ScriptAnalysis,
    },
    parser::Section,
};
use alloc::{format, string::ToString, vec::Vec};

/// Rule for detecting timing overlaps between dialogue events
///
/// Uses sweep-line algorithm for efficient O(n log n) overlap detection.
/// Overlapping events can cause rendering conflicts where multiple subtitles
/// appear simultaneously, potentially causing readability issues.
///
/// # Performance
///
/// - Time complexity: O(n log n) via sweep-line algorithm
/// - Memory: O(n) for temporary data structures
/// - Target: <1ms for 1000 events on typical hardware
///
/// # Example
///
/// ```rust
/// use ass_core::analysis::linting::rules::timing_overlap::TimingOverlapRule;
/// use ass_core::analysis::linting::LintRule;
/// use ass_core::{Script, ScriptAnalysis};
///
/// let script = Script::parse(r#"
/// [Events]
/// Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
/// Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,First event
/// Dialogue: 0,0:00:03.00,0:00:08.00,Default,,0,0,0,,Overlapping event
/// "#)?;
///
/// let analysis = ScriptAnalysis::analyze(&script)?;
/// let rule = TimingOverlapRule;
/// let issues = rule.check_script(&analysis);
/// assert!(!issues.is_empty()); // Should detect the overlap
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct TimingOverlapRule;

impl LintRule for TimingOverlapRule {
    fn id(&self) -> &'static str {
        "timing-overlap"
    }

    fn name(&self) -> &'static str {
        "Timing Overlap"
    }

    fn description(&self) -> &'static str {
        "Detects overlapping dialogue events that may cause rendering conflicts"
    }

    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Warning
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
            match find_overlapping_events(events) {
                Ok(overlaps) => {
                    for (i, j) in overlaps {
                        let event1 = &events[i];
                        let event2 = &events[j];

                        let issue = LintIssue::new(
                            self.default_severity(),
                            IssueCategory::Timing,
                            self.id(),
                            format!(
                                "Event overlaps: {} to {} overlaps with {} to {}",
                                event1.start, event1.end, event2.start, event2.end
                            ),
                        )
                        .with_description(
                            "Overlapping events may cause rendering conflicts".to_string(),
                        );

                        issues.push(issue);
                    }
                }
                Err(_) => {
                    let issue = LintIssue::new(
                        IssueSeverity::Warning,
                        IssueCategory::Timing,
                        self.id(),
                        "Could not analyze event overlaps due to timing parse errors".to_string(),
                    );
                    issues.push(issue);
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
        let rule = TimingOverlapRule;
        assert_eq!(rule.id(), "timing-overlap");
        assert_eq!(rule.name(), "Timing Overlap");
        assert_eq!(rule.default_severity(), IssueSeverity::Warning);
    }

    #[test]
    fn empty_script_no_issues() {
        let script_text = "[Script Info]\nTitle: Test";
        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();

        let rule = TimingOverlapRule;
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn non_overlapping_events_no_issues() {
        let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Second event";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let rule = TimingOverlapRule;
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }
}
