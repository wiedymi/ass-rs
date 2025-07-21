//! Built-in linting rules for ASS script validation.
//!
//! This module contains implementations of all built-in linting rules
//! that check for common issues in ASS subtitle scripts.

use super::{IssueCategory, IssueSeverity, LintIssue, LintRule};
use crate::{
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
            let mut sorted_events: Vec<_> = events.iter().collect();
            sorted_events.sort_by(|a, b| {
                let a_start = parse_ass_time(a.start).unwrap_or(0);
                let b_start = parse_ass_time(b.start).unwrap_or(0);
                a_start.cmp(&b_start)
            });

            for (i, event1) in sorted_events.iter().enumerate() {
                let end1 = parse_ass_time(event1.end).unwrap_or(0);

                for event2 in sorted_events.iter().skip(i + 1) {
                    let start2 = parse_ass_time(event2.start).unwrap_or(0);

                    if start2 >= end1 {
                        break;
                    }

                    if start2 < end1 {
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

        // Check styles
        if let Some(Section::Styles(styles)) = script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Styles(_)))
        {
            for style in styles {
                // Check primary color
                if parse_bgr_color(style.primary_colour).is_err() {
                    let issue = LintIssue::new(
                        self.default_severity(),
                        IssueCategory::Styling,
                        self.id(),
                        format!("Invalid primary color: {}", style.primary_colour),
                    );
                    issues.push(issue);
                }

                // Check secondary color
                if parse_bgr_color(style.secondary_colour).is_err() {
                    let issue = LintIssue::new(
                        self.default_severity(),
                        IssueCategory::Styling,
                        self.id(),
                        format!("Invalid secondary color: {}", style.secondary_colour),
                    );
                    issues.push(issue);
                }
            }
        }

        issues
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
