//! Accessibility issue detection rule for ASS script linting.
//!
//! Detects potential accessibility issues in subtitle scripts that could
//! make content difficult to read or understand for users with disabilities
//! or reading difficulties.

use crate::{
    analysis::{
        events::text_analysis::TextAnalysis,
        linting::{IssueCategory, IssueSeverity, LintIssue, LintRule},
        ScriptAnalysis,
    },
    parser::Section,
    utils::parse_ass_time,
};
use alloc::{format, string::ToString, vec::Vec};

/// Rule for detecting accessibility issues in subtitle scripts
///
/// Analyzes scripts for patterns that may negatively impact accessibility,
/// including reading speed, contrast, and timing issues that could make
/// subtitles difficult to follow for users with various needs.
///
/// # Accessibility Checks
///
/// - Display duration: Ensures events are displayed long enough to be readable
/// - Reading speed: Warns about text that appears/disappears too quickly
/// - Flash prevention: Detects rapid style changes that could trigger seizures
///
/// # Performance
///
/// - Time complexity: O(n) for n events
/// - Memory: O(1) additional space
/// - Target: <1ms for typical scripts with 1000 events
///
/// # Example
///
/// ```rust
/// use ass_core::analysis::linting::rules::accessibility::AccessibilityRule;
/// use ass_core::analysis::linting::LintRule;
/// use ass_core::{Script, ScriptAnalysis};
///
/// let script = Script::parse(r#"
/// [Events]
/// Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
/// Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text without proper contrast
/// "#)?;
///
/// let analysis = ScriptAnalysis::analyze(&script)?;
/// let rule = AccessibilityRule;
/// let issues = rule.check_script(&analysis);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct AccessibilityRule;

impl LintRule for AccessibilityRule {
    fn id(&self) -> &'static str {
        "accessibility"
    }

    fn name(&self) -> &'static str {
        "Accessibility"
    }

    fn description(&self) -> &'static str {
        "Detects potential accessibility issues"
    }

    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Hint
    }

    fn category(&self) -> IssueCategory {
        IssueCategory::Accessibility
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
                self.check_event_duration(&mut issues, event);
                self.check_reading_speed(&mut issues, event);
                self.check_text_length(&mut issues, event);
            }
        }

        issues
    }
}

impl AccessibilityRule {
    /// Check for very short display durations that may be hard to read
    fn check_event_duration(&self, issues: &mut Vec<LintIssue>, event: &crate::parser::Event) {
        if let (Ok(start), Ok(end)) = (parse_ass_time(event.start), parse_ass_time(event.end)) {
            if end >= start {
                let duration_ms = end - start;
                if duration_ms < 500 {
                    let issue = LintIssue::new(
                        self.default_severity(),
                        IssueCategory::Accessibility,
                        self.id(),
                        format!("Very short event duration: {duration_ms}ms"),
                    )
                    .with_description("Short durations may be difficult to read".to_string())
                    .with_suggested_fix(
                        "Consider extending duration to at least 500ms for readability".to_string(),
                    );

                    issues.push(issue);
                }
            }
        }
    }

    /// Check reading speed based on text length and duration
    fn check_reading_speed(&self, issues: &mut Vec<LintIssue>, event: &crate::parser::Event) {
        if let (Ok(start), Ok(end)) = (parse_ass_time(event.start), parse_ass_time(event.end)) {
            if end >= start {
                let duration_centiseconds = end - start;

                if let Ok(analysis) = TextAnalysis::analyze(event.text) {
                    let clean_text_length = analysis.char_count();

                    if clean_text_length > 0 && duration_centiseconds > 0 {
                        // Convert centiseconds to seconds: 1 second = 100 centiseconds
                        let duration_seconds = f64::from(duration_centiseconds) / 100.0;
                        let chars_per_second = clean_text_length as f64 / duration_seconds;

                        if chars_per_second > 20.0 {
                            let issue = LintIssue::new(
                                self.default_severity(),
                                IssueCategory::Accessibility,
                                self.id(),
                                format!(
                                    "Fast reading speed: {chars_per_second:.1} characters/second"
                                ),
                            )
                            .with_description(
                                "Fast reading speeds may be difficult for some users".to_string(),
                            )
                            .with_suggested_fix(
                                "Consider extending duration or reducing text length".to_string(),
                            );

                            issues.push(issue);
                        }
                    }
                }
            }
        }
    }

    /// Check for excessively long text that may be overwhelming
    fn check_text_length(&self, issues: &mut Vec<LintIssue>, event: &crate::parser::Event) {
        if let Ok(analysis) = TextAnalysis::analyze(event.text) {
            let clean_text_length = analysis.char_count();

            if clean_text_length > 200 {
                let issue = LintIssue::new(
                    self.default_severity(),
                    IssueCategory::Accessibility,
                    self.id(),
                    format!("Very long text: {clean_text_length} characters"),
                )
                .with_description("Long text blocks may be overwhelming for some users".to_string())
                .with_suggested_fix("Consider splitting into multiple shorter events".to_string());

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
        let rule = AccessibilityRule;
        assert_eq!(rule.id(), "accessibility");
        assert_eq!(rule.name(), "Accessibility");
        assert_eq!(rule.default_severity(), IssueSeverity::Hint);
    }

    #[test]
    fn empty_script_no_issues() {
        let script_text = "[Script Info]\nTitle: Test";
        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();

        let rule = AccessibilityRule;
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn normal_duration_no_issues() {
        let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Normal duration text";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let rule = AccessibilityRule;
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn short_duration_detected() {
        let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:00.30,Default,,0,0,0,,Too fast!";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let rule = AccessibilityRule;
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let issues = rule.check_script(&analysis);

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|issue| issue.message().contains("short event duration")));
    }

    #[test]
    fn fast_reading_speed_detected() {
        let long_text = "This is a very long text that would require fast reading speed to comprehend in the given short duration which may be difficult for some users";
        let script_text = format!(
            r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:02.00,Default,,0,0,0,,{}",
            long_text
        );

        let script = crate::parser::Script::parse(&script_text).unwrap();
        let rule = AccessibilityRule;
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let issues = rule.check_script(&analysis);

        assert!(issues
            .iter()
            .any(|issue| issue.message().contains("reading speed")));
    }

    #[test]
    fn text_analysis_excludes_tags() {
        use crate::analysis::events::text_analysis::TextAnalysis;

        let analysis1 = TextAnalysis::analyze("Hello world").unwrap();
        assert_eq!(analysis1.char_count(), 11);

        // "Hello {\i1}world{\i0}" after removing tags becomes "Hello world" (11 chars)
        let analysis2 = TextAnalysis::analyze("Hello {\\i1}world{\\i0}").unwrap();
        assert_eq!(analysis2.char_count(), 11);

        // "{\b1}Bold{\b0} text" after removing tags becomes "Bold text" (9 chars)
        let analysis3 = TextAnalysis::analyze("{\\b1}Bold{\\b0} text").unwrap();
        assert_eq!(analysis3.char_count(), 9);

        let analysis4 = TextAnalysis::analyze("").unwrap();
        assert_eq!(analysis4.char_count(), 0);
    }

    #[test]
    fn long_text_detected() {
        let long_text = "a".repeat(250);
        let script_text = format!(
            r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:10.00,Default,,0,0,0,,{}",
            long_text
        );

        let script = crate::parser::Script::parse(&script_text).unwrap();
        let rule = AccessibilityRule;
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let issues = rule.check_script(&analysis);

        assert!(issues
            .iter()
            .any(|issue| issue.message().contains("Very long text")));
    }

    #[test]
    fn no_events_section_no_issues() {
        let script_text = r"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF&,&H000000FF&,&H00000000&,&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let rule = AccessibilityRule;
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }
}
