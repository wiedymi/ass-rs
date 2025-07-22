//! Invalid tag detection rule for ASS script linting.
//!
//! Detects invalid or malformed override tags in event text that would
//! cause parsing errors or unexpected rendering behavior.

use crate::{
    analysis::{
        events::text_analysis::TextAnalysis,
        linting::{IssueCategory, IssueSeverity, LintIssue, LintRule},
    },
    parser::{Script, Section},
};
use alloc::{string::ToString, vec::Vec};

/// Rule for detecting invalid or malformed override tags in event text
///
/// Override tags control text formatting and positioning within subtitle events.
/// Malformed tags can cause parsing errors, rendering glitches, or unexpected
/// visual behavior in subtitle displays.
///
/// # Performance
///
/// - Time complexity: O(n * m) for n events and m tokens per event
/// - Memory: O(1) additional space
/// - Target: <2ms for typical scripts with 1000 events
///
/// # Example
///
/// ```rust
/// use ass_core::analysis::linting::rules::invalid_tag::InvalidTagRule;
/// use ass_core::analysis::linting::LintRule;
/// use ass_core::parser::Script;
///
/// let script = Script::parse(r#"
/// [Events]
/// Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
/// Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with {\b1\} override
/// "#).unwrap();
///
/// let rule = InvalidTagRule;
/// let issues = rule.check_script(&script);
/// assert!(!issues.is_empty()); // Should detect the empty tag after \b1
/// ```
pub struct InvalidTagRule;

impl LintRule for InvalidTagRule {
    fn id(&self) -> &'static str {
        "invalid-tag"
    }

    fn name(&self) -> &'static str {
        "Invalid Tag"
    }

    fn description(&self) -> &'static str {
        "Detects invalid or malformed override tags in event text"
    }

    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Warning
    }

    fn category(&self) -> IssueCategory {
        IssueCategory::Content
    }

    fn check_script<'a>(&self, script: &'a Script<'a>) -> Vec<LintIssue<'a>> {
        let mut issues = Vec::new();

        if let Some(Section::Events(events)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            for event in events {
                let text_analysis = match TextAnalysis::analyze(event.text) {
                    Ok(analysis) => analysis,
                    Err(_) => continue, // Skip analysis if text parsing fails
                };
                // Check for empty tags in parsed override tags
                for tag in text_analysis.override_tags() {
                    if tag.name().trim().is_empty() {
                        let issue = LintIssue::new(
                            self.default_severity(),
                            IssueCategory::Content,
                            self.id(),
                            "Empty override tag found".to_string(),
                        );
                        issues.push(issue);
                    }
                }

                // Check for malformed tags that TextAnalysis might miss
                // Look for cases like {\b1\} where there's an empty tag after a valid one
                if event.text.contains('{') && event.text.contains('}') {
                    let start = event.text.find('{').unwrap_or(0);
                    let end = event.text.rfind('}').unwrap_or(event.text.len());
                    if start < end {
                        let override_block = &event.text[start + 1..end];
                        let tag_parts: Vec<&str> = override_block.split('\\').collect();

                        // Skip the first part and check each remaining part for empty tags
                        for tag_part in tag_parts.iter().skip(1) {
                            let tag_name = tag_part.trim();
                            if tag_name.is_empty() {
                                let issue = LintIssue::new(
                                    self.default_severity(),
                                    IssueCategory::Content,
                                    self.id(),
                                    "Empty override tag found".to_string(),
                                );
                                issues.push(issue);
                            }
                        }
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
        let rule = InvalidTagRule;
        assert_eq!(rule.id(), "invalid-tag");
        assert_eq!(rule.name(), "Invalid Tag");
        assert_eq!(rule.default_severity(), IssueSeverity::Warning);
    }

    #[test]
    fn empty_script_no_issues() {
        let script_text = "[Script Info]\nTitle: Test";
        let script = Script::parse(script_text).unwrap();

        let rule = InvalidTagRule;
        let issues = rule.check_script(&script);

        assert!(issues.is_empty());
    }

    #[test]
    fn valid_tags_no_issues() {
        let script_text = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with {\i1}valid{\i0} tags"#;

        let script = Script::parse(script_text).unwrap();
        let rule = InvalidTagRule;
        let issues = rule.check_script(&script);

        assert!(issues.is_empty());
    }

    #[test]
    fn no_events_section_no_issues() {
        let script_text = r#"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF&,&H000000FF&,&H00000000&,&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1"#;

        let script = Script::parse(script_text).unwrap();
        let rule = InvalidTagRule;
        let issues = rule.check_script(&script);

        assert!(issues.is_empty());
    }

    #[test]
    fn plain_text_no_issues() {
        let script_text = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Plain text without any tags"#;

        let script = Script::parse(script_text).unwrap();
        let rule = InvalidTagRule;
        let issues = rule.check_script(&script);

        assert!(issues.is_empty());
    }

    #[test]
    fn empty_tag_after_valid_tag_detected() {
        let script_text = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with {\b1\} override"#;

        let script = Script::parse(script_text).unwrap();
        let rule = InvalidTagRule;
        let issues = rule.check_script(&script);

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|issue| issue.message().contains("Empty override tag")));
    }
}
