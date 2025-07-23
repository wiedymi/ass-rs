//! Encoding issue detection rule for ASS script linting.
//!
//! Detects potential encoding or character issues in subtitle scripts
//! that could cause display problems or compatibility issues.

use crate::{
    analysis::{
        linting::{IssueCategory, IssueSeverity, LintIssue, LintRule},
        ScriptAnalysis,
    },
    parser::Section,
};
use alloc::{string::ToString, vec::Vec};

/// Rule for detecting encoding and character issues in subtitle scripts
///
/// Analyzes scripts for problematic characters that may cause encoding
/// issues, display problems, or compatibility issues across different
/// subtitle renderers and media players.
///
/// # Encoding Checks
///
/// - Non-printable characters: Detects control characters that shouldn't appear in text
/// - Invalid UTF-8 sequences: Identifies corrupted character data
/// - Suspicious character patterns: Warns about potentially problematic sequences
///
/// # Performance
///
/// - Time complexity: O(n * m) for n events and m characters per event
/// - Memory: O(1) additional space
/// - Target: <2ms for typical scripts with 1000 events
///
/// # Example
///
/// ```rust
/// use ass_core::analysis::linting::rules::encoding::EncodingRule;
/// use ass_core::analysis::linting::LintRule;
/// use ass_core::{Script, ScriptAnalysis};
///
/// let script_text = format!("[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with{}invalid character", '\x00');
/// let script = Script::parse(&script_text)?;
///
/// let rule = EncodingRule;
/// let analysis = ScriptAnalysis::analyze(&script).unwrap();
/// let issues = rule.check_script(&analysis);
/// assert!(!issues.is_empty()); // Should detect the control character
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct EncodingRule;

impl LintRule for EncodingRule {
    fn id(&self) -> &'static str {
        "encoding"
    }

    fn name(&self) -> &'static str {
        "Encoding"
    }

    fn description(&self) -> &'static str {
        "Detects potential encoding or character issues"
    }

    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Warning
    }

    fn category(&self) -> IssueCategory {
        IssueCategory::Encoding
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
                self.check_event_encoding(&mut issues, event);
            }
        }

        self.check_script_info_encoding(&mut issues, analysis.script());

        issues
    }
}

impl EncodingRule {
    /// Check encoding issues in a single event
    fn check_event_encoding(&self, issues: &mut Vec<LintIssue>, event: &crate::parser::Event) {
        if event
            .text
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
        {
            let issue = LintIssue::new(
                self.default_severity(),
                IssueCategory::Encoding,
                self.id(),
                "Event contains non-printable control characters".to_string(),
            )
            .with_description(
                "Control characters may cause display issues in subtitle renderers".to_string(),
            )
            .with_suggested_fix(
                "Remove or replace control characters with appropriate text".to_string(),
            );
            issues.push(issue);
        }

        if event.text.contains('\u{FFFD}') {
            let issue = LintIssue::new(
                self.default_severity(),
                IssueCategory::Encoding,
                self.id(),
                "Event contains Unicode replacement character (�)".to_string(),
            )
            .with_description(
                "Replacement characters indicate corrupted or invalid encoding".to_string(),
            )
            .with_suggested_fix("Check source file encoding and re-import".to_string());
            issues.push(issue);
        }

        let char_count = event.text.chars().count();
        let byte_count = event.text.len();

        // Check for heavy multi-byte character usage
        if char_count > 0 && byte_count > char_count * 3 {
            let issue = LintIssue::new(
                IssueSeverity::Hint,
                IssueCategory::Encoding,
                self.id(),
                "Event contains many multi-byte characters".to_string(),
            )
            .with_description(
                "Heavy use of multi-byte characters may impact performance".to_string(),
            );
            issues.push(issue);
        }
    }

    /// Check encoding issues in script info section
    fn check_script_info_encoding(
        &self,
        issues: &mut Vec<LintIssue>,
        script: &crate::parser::Script,
    ) {
        if let Some(Section::ScriptInfo(info)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::ScriptInfo(_)))
        {
            for (key, value) in &info.fields {
                if value
                    .chars()
                    .any(|c| c.is_control() && c != '\n' && c != '\r')
                {
                    let issue = LintIssue::new(
                        self.default_severity(),
                        IssueCategory::Encoding,
                        self.id(),
                        format!("Script info field '{key}' contains control characters"),
                    );
                    issues.push(issue);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_metadata_correct() {
        let rule = EncodingRule;
        assert_eq!(rule.id(), "encoding");
        assert_eq!(rule.name(), "Encoding");
        assert_eq!(rule.default_severity(), IssueSeverity::Warning);
    }

    #[test]
    fn empty_script_no_issues() {
        let script_text = "[Script Info]\nTitle: Test";
        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();

        let rule = EncodingRule;
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn valid_text_no_issues() {
        let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Valid text with unicode: ñáéíóú";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let rule = EncodingRule;
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn newlines_allowed() {
        let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with\Nline break";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let rule = EncodingRule;
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn tabs_allowed() {
        let script_text = "[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with\ttab";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let rule = EncodingRule;
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn replacement_character_detected() {
        let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with � replacement";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let rule = EncodingRule;
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let issues = rule.check_script(&analysis);

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|issue| issue.message().contains("replacement character")));
    }

    #[test]
    fn control_character_in_script_info() {
        let script_text = "[Script Info]\nTitle: Test\x00\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let rule = EncodingRule;
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let issues = rule.check_script(&analysis);

        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|issue| issue.message().contains("control characters")));
    }

    #[test]
    fn no_events_section_no_issues() {
        let script_text = r"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF&,&H000000FF&,&H00000000&,&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let rule = EncodingRule;
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn multibyte_characters_hint() {
        let heavy_unicode = "🎵🎶🎵🎶".repeat(20);
        let script_text = format!(
            r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{}",
            heavy_unicode
        );

        let script = crate::parser::Script::parse(&script_text).unwrap();
        let rule = EncodingRule;
        let analysis = ScriptAnalysis::analyze(&script).unwrap();
        let issues = rule.check_script(&analysis);

        assert!(issues
            .iter()
            .any(|issue| issue.message().contains("multi-byte characters")));
    }
}
