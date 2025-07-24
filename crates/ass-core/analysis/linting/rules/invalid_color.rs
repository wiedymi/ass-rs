//! Invalid color format detection rule for ASS script linting.
//!
//! Detects invalid color formats in both style definitions and override tags
//! that would cause rendering errors or unexpected visual results.

use crate::{
    analysis::{
        events::text_analysis::TextAnalysis,
        linting::{IssueCategory, IssueSeverity, LintIssue, LintRule},
        ScriptAnalysis,
    },
    parser::Section,
    utils::parse_bgr_color,
};
use alloc::{format, string::ToString, vec::Vec};

/// Rule for detecting invalid color formats in styles and override tags
///
/// Validates color values in style definitions (primary, secondary, outline, back)
/// and color override tags in event text. Invalid colors cause rendering errors
/// or unexpected visual results in subtitle displays.
///
/// # Performance
///
/// - Time complexity: O(n) for n styles + O(m) for m override tags
/// - Memory: O(1) additional space
/// - Target: <2ms for typical scripts with 100 styles and 1000 events
///
/// # Example
///
/// ```rust
/// use ass_core::analysis::linting::rules::invalid_color::InvalidColorRule;
/// use ass_core::analysis::linting::LintRule;
/// use ass_core::{Script, ScriptAnalysis};
///
/// let script = Script::parse(r#"
/// [V4+ Styles]
/// Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
/// Style: Default,Arial,20,&HINVALID&,&H000000FF&,&H00000000&,&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
/// "#)?;
///
/// let analysis = ScriptAnalysis::analyze(&script)?;
/// let rule = InvalidColorRule;
/// let issues = rule.check_script(&analysis);
/// assert!(!issues.is_empty()); // Should detect the invalid color
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct InvalidColorRule;

impl LintRule for InvalidColorRule {
    fn id(&self) -> &'static str {
        "invalid-color"
    }

    fn name(&self) -> &'static str {
        "Invalid Color"
    }

    fn description(&self) -> &'static str {
        "Detects invalid color format in styles and override tags"
    }

    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Error
    }

    fn category(&self) -> IssueCategory {
        IssueCategory::Styling
    }

    fn check_script(&self, analysis: &ScriptAnalysis) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        if let Some(Section::Styles(styles)) = analysis
            .script()
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Styles(_)))
        {
            for style in styles {
                self.check_style_color(&mut issues, "primary", style.primary_colour);
                self.check_style_color(&mut issues, "secondary", style.secondary_colour);
                self.check_style_color(&mut issues, "outline", style.outline_colour);
                self.check_style_color(&mut issues, "back", style.back_colour);
            }
        }

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
                for tag in text_analysis.override_tags() {
                    let tag_content = format!("\\{}{}", tag.name(), tag.args());
                    self.check_color_override_tags(&mut issues, &tag_content);
                }
            }
        }

        issues
    }
}

impl InvalidColorRule {
    /// Check a single style color field for validity
    fn check_style_color(&self, issues: &mut Vec<LintIssue>, field_name: &str, color_value: &str) {
        if parse_bgr_color(color_value).is_err() {
            let issue = LintIssue::new(
                self.default_severity(),
                IssueCategory::Styling,
                self.id(),
                format!("Invalid {field_name} color: {color_value}"),
            );
            issues.push(issue);
        }
    }

    /// Check color override tags in an override block
    fn check_color_override_tags(&self, issues: &mut Vec<LintIssue>, span: &str) {
        for tag_part in span.split('\\').skip(1) {
            if tag_part.is_empty() {
                continue;
            }

            if let Some((tag_name, color_value)) = Self::parse_color_tag(tag_part) {
                if parse_bgr_color(&format!("&H{color_value}&")).is_err() {
                    let issue = LintIssue::new(
                        self.default_severity(),
                        IssueCategory::Styling,
                        self.id(),
                        format!("Invalid color override tag: \\{tag_name}&H{color_value}&"),
                    );
                    issues.push(issue);
                }
            }
        }
    }

    /// Parse a color tag and return (`tag_name`, `color_value`) if valid
    fn parse_color_tag(tag_part: &str) -> Option<(String, String)> {
        if tag_part.starts_with("c&H") || tag_part.starts_with("c&h") {
            let color_part = tag_part
                .strip_prefix("c&H")
                .or_else(|| tag_part.strip_prefix("c&h"))?;
            let color_end = color_part.find('&')?;
            let color_value = &color_part[..color_end];
            return Some(("c".to_string(), color_value.to_string()));
        }

        if tag_part.len() >= 3 {
            let first_char = &tag_part[..1];
            if ["1", "2", "3", "4"].contains(&first_char) && tag_part[1..].starts_with("c&H") {
                let color_part = tag_part[1..].strip_prefix("c&H")?;
                let color_end = color_part.find('&')?;
                let color_value = &color_part[..color_end];
                return Some((format!("{first_char}c"), color_value.to_string()));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_metadata_correct() {
        let rule = InvalidColorRule;
        assert_eq!(rule.id(), "invalid-color");
        assert_eq!(rule.name(), "Invalid Color");
        assert_eq!(
            rule.description(),
            "Detects invalid color format in styles and override tags"
        );
        assert_eq!(rule.default_severity(), IssueSeverity::Error);
        assert_eq!(rule.category(), IssueCategory::Styling);
    }

    #[test]
    fn empty_script_no_issues() {
        let script_text = "[Script Info]\nTitle: Test";
        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();

        let rule = InvalidColorRule;
        let issues = rule.check_script(&analysis);

        assert!(issues.is_empty());
    }

    #[test]
    fn parse_color_tag_formats() {
        assert_eq!(
            InvalidColorRule::parse_color_tag("c&H00FF00&"),
            Some(("c".to_string(), "00FF00".to_string()))
        );

        assert_eq!(
            InvalidColorRule::parse_color_tag("1c&H00FF00&"),
            Some(("1c".to_string(), "00FF00".to_string()))
        );

        assert!(InvalidColorRule::parse_color_tag("invalid").is_none());
    }
}
