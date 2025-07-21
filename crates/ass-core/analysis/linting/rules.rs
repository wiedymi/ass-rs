//! Built-in linting rules for ASS script validation.
//!
//! This module contains implementations of all built-in linting rules
//! that check for common issues in ASS subtitle scripts.

use super::{IssueCategory, IssueSeverity, LintIssue, LintRule};
use crate::{
    analysis::events::find_overlapping_events,
    parser::{Script, Section},
    tokenizer::{AssTokenizer, TokenType},
    utils::{parse_ass_time, parse_bgr_color},
};
use alloc::{boxed::Box, format, vec::Vec};

/// Built-in lint rules registry.
pub struct BuiltinRules;

impl BuiltinRules {
    /// Get all built-in rules.
    pub fn all_rules() -> Vec<Box<dyn LintRule>> {
        vec![
            Box::new(TimingOverlapRule),
            Box::new(NegativeDurationRule),
            Box::new(InvalidColorRule),
            Box::new(MissingStyleRule),
            Box::new(InvalidTagRule),
            Box::new(PerformanceRule),
            Box::new(EncodingRule),
            Box::new(AccessibilityRule),
        ]
    }
}

/// Rule: Check for timing overlaps between events.
struct TimingOverlapRule;

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

    fn check_script<'a>(&self, script: &'a Script<'a>) -> Vec<LintIssue<'a>> {
        let mut issues = Vec::new();

        if let Some(Section::Events(events)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            // Use efficient O(n log n) overlap detection
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
                    // If overlap detection fails, add a general warning
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

/// Rule: Check for negative or zero durations.
struct NegativeDurationRule;

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

    fn check_script<'a>(&self, script: &'a Script<'a>) -> Vec<LintIssue<'a>> {
        let mut issues = Vec::new();

        if let Some(Section::Events(events)) = script
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

/// Rule: Check for invalid color formats.
struct InvalidColorRule;

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

    fn check_script<'a>(&self, script: &'a Script<'a>) -> Vec<LintIssue<'a>> {
        let mut issues = Vec::new();

        // Check styles - all four color fields
        if let Some(Section::Styles(styles)) = script
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

        // Check color override tags in event text
        if let Some(Section::Events(events)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            for event in events {
                let mut tokenizer = AssTokenizer::new(event.text);
                while let Ok(Some(token)) = tokenizer.next_token() {
                    if let TokenType::OverrideBlock = token.token_type {
                        self.check_color_override_tags(&mut issues, token.span);
                    }
                }
            }
        }

        issues
    }
}

impl InvalidColorRule {
    /// Check a single style color field
    fn check_style_color(&self, issues: &mut Vec<LintIssue>, field_name: &str, color_value: &str) {
        if parse_bgr_color(color_value).is_err() {
            let issue = LintIssue::new(
                self.default_severity(),
                IssueCategory::Styling,
                self.id(),
                format!("Invalid {} color: {}", field_name, color_value),
            );
            issues.push(issue);
        }
    }

    /// Check color override tags in an override block
    fn check_color_override_tags(&self, issues: &mut Vec<LintIssue>, span: &str) {
        // Override block content comes without braces from tokenizer
        // Split by backslashes to find individual tags
        for tag_part in span.split('\\').skip(1) {
            if tag_part.is_empty() {
                continue;
            }

            if let Some((tag_name, color_value)) = self.parse_color_tag(tag_part) {
                if parse_bgr_color(&format!("&H{}&", color_value)).is_err() {
                    let issue = LintIssue::new(
                        self.default_severity(),
                        IssueCategory::Styling,
                        self.id(),
                        format!(
                            "Invalid color override tag: \\{}&H{}&",
                            tag_name, color_value
                        ),
                    );
                    issues.push(issue);
                }
            }
        }
    }

    /// Parse a color tag and return (tag_name, color_value) if valid
    fn parse_color_tag(&self, tag_part: &str) -> Option<(String, String)> {
        // Handle \c&H...& format
        if tag_part.starts_with("c&H") || tag_part.starts_with("c&h") {
            let color_part = tag_part
                .strip_prefix("c&H")
                .or_else(|| tag_part.strip_prefix("c&h"))?;
            let color_end = color_part.find('&')?;
            let color_value = &color_part[..color_end];
            return Some(("c".to_string(), color_value.to_string()));
        }

        // Handle \1c&H...&, \2c&H...&, \3c&H...&, \4c&H...& formats
        if tag_part.len() >= 3 {
            let first_char = &tag_part[..1];
            if ["1", "2", "3", "4"].contains(&first_char) && tag_part[1..].starts_with("c&H") {
                let color_part = tag_part[1..].strip_prefix("c&H")?;
                let color_end = color_part.find('&')?;
                let color_value = &color_part[..color_end];
                return Some((format!("{}c", first_char), color_value.to_string()));
            }
        }

        None
    }
}

/// Rule: Check for references to missing styles.
struct MissingStyleRule;

impl LintRule for MissingStyleRule {
    fn id(&self) -> &'static str {
        "missing-style"
    }

    fn name(&self) -> &'static str {
        "Missing Style"
    }

    fn description(&self) -> &'static str {
        "Detects events referencing non-existent styles"
    }

    fn default_severity(&self) -> IssueSeverity {
        IssueSeverity::Error
    }

    fn check_script<'a>(&self, script: &'a Script<'a>) -> Vec<LintIssue<'a>> {
        let mut issues = Vec::new();

        // Collect available style names
        let mut style_names = Vec::new();
        if let Some(Section::Styles(styles)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Styles(_)))
        {
            for style in styles {
                style_names.push(style.name);
            }
        }

        // Check events for missing style references
        if let Some(Section::Events(events)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            for event in events {
                if !style_names.contains(&event.style) {
                    let issue = LintIssue::new(
                        self.default_severity(),
                        IssueCategory::Styling,
                        self.id(),
                        format!("Event references undefined style: {}", event.style),
                    )
                    .with_suggested_fix(format!(
                        "Define style '{}' or use an existing style",
                        event.style
                    ));

                    issues.push(issue);
                }
            }
        }

        issues
    }
}

/// Rule: Check for invalid override tags.
struct InvalidTagRule;

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

    fn check_script<'a>(&self, script: &'a Script<'a>) -> Vec<LintIssue<'a>> {
        let mut issues = Vec::new();

        if let Some(Section::Events(events)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            for event in events {
                let mut tokenizer = AssTokenizer::new(event.text);
                while let Ok(Some(token)) = tokenizer.next_token() {
                    match token.token_type {
                        TokenType::OverrideBlock => {
                            let span = token.span;
                            // Parse tag name from the override block
                            if let Some(stripped) =
                                span.strip_prefix('{').and_then(|s| s.strip_suffix('}'))
                            {
                                if let Some(tag_content) = stripped.strip_prefix('\\') {
                                    let tag_name =
                                        tag_content.split_whitespace().next().unwrap_or("");

                                    // Check for known invalid patterns
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
                        _ => continue,
                    }
                }
            }
        }

        issues
    }
}

/// Rule: Check for performance issues.
struct PerformanceRule;

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

    fn check_script<'a>(&self, script: &'a Script<'a>) -> Vec<LintIssue<'a>> {
        let mut issues = Vec::new();

        if let Some(Section::Events(events)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            if events.len() > 1000 {
                let issue = LintIssue::new(
                    self.default_severity(),
                    IssueCategory::Performance,
                    self.id(),
                    format!("Script has {} events, consider optimization", events.len()),
                )
                .with_description(
                    "Large number of events may impact rendering performance".to_string(),
                );

                issues.push(issue);
            }
        }

        issues
    }
}

/// Rule: Check for encoding issues.
struct EncodingRule;

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

    fn check_script<'a>(&self, script: &'a Script<'a>) -> Vec<LintIssue<'a>> {
        let mut issues = Vec::new();

        if let Some(Section::Events(events)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            for event in events {
                // Check for non-printable characters
                if event
                    .text
                    .chars()
                    .any(|c| c.is_control() && c != '\n' && c != '\r')
                {
                    let issue = LintIssue::new(
                        self.default_severity(),
                        IssueCategory::Encoding,
                        self.id(),
                        "Event contains non-printable characters".to_string(),
                    );
                    issues.push(issue);
                }
            }
        }

        issues
    }
}

/// Rule: Check for accessibility issues.
struct AccessibilityRule;

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

    fn check_script<'a>(&self, script: &'a Script<'a>) -> Vec<LintIssue<'a>> {
        let mut issues = Vec::new();

        if let Some(Section::Events(events)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            for event in events {
                // Check for very short display duration
                if let (Ok(start), Ok(end)) =
                    (parse_ass_time(event.start), parse_ass_time(event.end))
                {
                    let duration_ms = end - start;
                    if duration_ms < 500 {
                        // Less than 500ms
                        let issue = LintIssue::new(
                            self.default_severity(),
                            IssueCategory::Accessibility,
                            self.id(),
                            format!("Very short event duration: {}ms", duration_ms),
                        )
                        .with_description("Short durations may be difficult to read".to_string());

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
    use crate::parser::Script;

    #[test]
    fn enhanced_color_validation() {
        let script_text = r#"[Script Info]
Title: Color Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: ValidStyle,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: InvalidPrimary,Arial,20,INVALID,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: InvalidOutline,Arial,20,&H00FFFFFF,&H000000FF,BADCOLOR,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,ValidStyle,,0,0,0,,Normal text
Dialogue: 0,0:00:05.00,0:00:10.00,ValidStyle,,0,0,0,,{\c&H00FF00&}Valid color override
Dialogue: 0,0:00:10.00,0:00:15.00,ValidStyle,,0,0,0,,{\c&HINVALID&}Invalid color override
Dialogue: 0,0:00:15.00,0:00:20.00,ValidStyle,,0,0,0,,{\1c&H123456&}Valid numbered color
Dialogue: 0,0:00:20.00,0:00:25.00,ValidStyle,,0,0,0,,{\2c&HBADVAL&}Invalid numbered color
"#;

        let script = Script::parse(script_text).unwrap();
        let rule = InvalidColorRule;
        let issues = rule.check_script(&script);

        // Should find several color validation issues
        assert!(!issues.is_empty(), "Should detect color validation issues");

        // Check for style color issues
        let style_issues: Vec<_> = issues
            .iter()
            .filter(|issue| {
                issue.message().contains("Invalid") && issue.message().contains("color: ")
            })
            .collect();
        assert!(
            style_issues.len() >= 2,
            "Should detect invalid style colors"
        );

        // Check for override tag issues
        let override_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.message().contains("color override tag"))
            .collect();
        assert!(
            override_issues.len() >= 2,
            "Should detect invalid color override tags"
        );

        // Verify specific issues are found
        assert!(issues
            .iter()
            .any(|issue| issue.message().contains("INVALID")));
        assert!(issues
            .iter()
            .any(|issue| issue.message().contains("BADCOLOR")));
        assert!(issues
            .iter()
            .any(|issue| issue.message().contains("BADVAL")));
    }
}
