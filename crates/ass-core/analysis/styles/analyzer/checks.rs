//! Style extraction, conflict detection, and validation checks.
//!
//! Provides helpers to locate the styles section, flag duplicate style names,
//! and validate individual resolved styles against correctness and performance
//! criteria.

use super::{AnalysisOptions, StyleAnalyzer};
use crate::{
    analysis::styles::{
        resolved_style::ResolvedStyle,
        validation::{StyleConflict, StyleValidationIssue},
    },
    parser::{Section, Style},
};
use alloc::{collections::BTreeMap, vec::Vec};

impl<'a> StyleAnalyzer<'a> {
    /// Extract styles from script sections
    #[must_use]
    pub fn extract_styles(&self) -> Option<&[Style<'a>]> {
        for section in self.script.sections() {
            if let Section::Styles(styles) = section {
                return Some(styles);
            }
        }
        None
    }

    /// Detect conflicts between styles in a section
    pub(super) fn detect_style_conflicts_from_section(&mut self, styles: &[Style<'a>]) {
        let mut name_counts: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

        for style in styles {
            name_counts.entry(style.name).or_default().push(style.name);
        }

        for (_name, instances) in name_counts {
            if instances.len() > 1 {
                self.conflicts
                    .push(StyleConflict::duplicate_name(instances));
            }
        }
    }

    /// Validate style properties
    pub(super) fn validate_style_properties(
        &self,
        style: &ResolvedStyle<'a>,
    ) -> Vec<StyleValidationIssue> {
        let mut issues = Vec::new();

        if style.font_size() <= 0.0 {
            issues.push(StyleValidationIssue::error(
                "font_size",
                "Font size must be positive",
            ));
        }

        if self
            .config
            .options
            .contains(AnalysisOptions::STRICT_VALIDATION)
            && style.font_size() > 200.0
        {
            issues.push(StyleValidationIssue::warning(
                "font_size",
                "Very large font size may cause performance issues",
            ));
        }

        issues
    }

    /// Analyze style performance impact
    pub(super) fn analyze_style_performance(
        &self,
        style: &ResolvedStyle<'a>,
    ) -> Vec<StyleValidationIssue> {
        let mut issues = Vec::new();
        let thresholds = &self.config.performance_thresholds;

        if style.font_size() > thresholds.large_font_threshold {
            issues.push(StyleValidationIssue::info_with_suggestion(
                "font_size",
                "Large font size detected",
                "Consider reducing font size for better performance",
            ));
        }

        if style.has_performance_issues() {
            issues.push(StyleValidationIssue::warning(
                "complexity",
                "Style has high rendering complexity",
            ));
        }

        issues
    }
}
