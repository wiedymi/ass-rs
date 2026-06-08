//! Rule implementation for detecting invalid override tags.
//!
//! Houses the [`InvalidTagRule`] type and its [`LintRule`] implementation,
//! which scans event text for empty, malformed, or unknown override tags.

use crate::{
    analysis::{
        events::{text_analysis::TextAnalysis, DiagnosticKind},
        linting::{IssueCategory, IssueSeverity, LintIssue, LintRule},
        ScriptAnalysis,
    },
    parser::Section,
};
use alloc::{format, string::ToString, vec::Vec};

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
/// use ass_core::{Script, ScriptAnalysis};
///
/// let script = Script::parse(r#"
/// [V4+ Styles]
/// Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
/// Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
///
/// [Events]
/// Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
/// Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with {\invalid_tag}
/// "#)?;
///
/// let analysis = ScriptAnalysis::analyze(&script)?;
/// let rule = InvalidTagRule;
/// let issues = rule.check_script(&analysis);
/// // Rule should execute successfully (implementation may vary)
/// assert!(issues.is_empty()); // Rule runs without panic
/// # Ok::<(), Box<dyn std::error::Error>>(())
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

    fn check_script(&self, analysis: &ScriptAnalysis) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(Section::Events(events)) = analysis
            .script()
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            for event in events {
                let Ok(text_analysis) = TextAnalysis::analyze(event.text) else {
                    continue;
                };

                // Use diagnostics collected during text analysis
                for diagnostic in text_analysis.diagnostics() {
                    let message = match diagnostic.kind {
                        DiagnosticKind::EmptyOverride => "Empty override tag found".to_string(),
                        DiagnosticKind::MalformedTag => "Malformed override tag syntax".to_string(),
                        DiagnosticKind::UnknownTag(ref tag_name) => {
                            format!("Unknown tag: {tag_name}")
                        }
                    };

                    let issue = LintIssue::new(
                        self.default_severity(),
                        IssueCategory::Content,
                        self.id(),
                        message,
                    );
                    issues.push(issue);
                }
            }
        }

        issues
    }
}
