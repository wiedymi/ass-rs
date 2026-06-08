//! Accessors and performance queries for [`ScriptAnalysis`].
//!
//! Exposes the cached analysis results (lint issues, resolved styles, dialogue
//! information) and derives the aggregate [`PerformanceSummary`] from them.

use super::{
    count_overlapping_dialogue_events, linting, DialogueInfo, LintIssue, PerformanceSummary,
    ResolvedStyle, ScriptAnalysis,
};
use crate::parser::Script;

impl<'a> ScriptAnalysis<'a> {
    /// Get all lint issues found during analysis
    #[must_use]
    pub fn lint_issues(&self) -> &[LintIssue] {
        &self.lint_issues
    }

    /// Get resolved styles
    #[must_use]
    pub fn resolved_styles(&self) -> &[ResolvedStyle<'a>] {
        &self.resolved_styles
    }

    /// Get dialogue analysis results
    #[must_use]
    pub fn dialogue_info(&self) -> &[DialogueInfo<'a>] {
        &self.dialogue_info
    }

    /// Get reference to the analyzed script
    #[must_use]
    pub const fn script(&self) -> &'a Script<'a> {
        self.script
    }

    /// Find resolved style by name
    #[must_use]
    pub fn resolve_style(&self, name: &str) -> Option<&ResolvedStyle<'a>> {
        self.resolved_styles.iter().find(|style| style.name == name)
    }

    /// Check if script has any critical issues
    #[must_use]
    pub fn has_critical_issues(&self) -> bool {
        self.lint_issues
            .iter()
            .any(|issue| issue.severity() == linting::IssueSeverity::Critical)
    }

    /// Get performance summary
    #[must_use]
    pub fn performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            total_events: self.dialogue_info.len(),
            overlapping_events: self.count_overlapping_events(),
            complex_animations: self.count_complex_animations(),
            large_fonts: self.count_large_fonts(),
            performance_score: self.calculate_performance_score(),
        }
    }

    /// Count overlapping events using efficient O(n log n) algorithm
    fn count_overlapping_events(&self) -> usize {
        count_overlapping_dialogue_events(&self.dialogue_info)
    }

    /// Count complex animations (transforms, etc.)
    fn count_complex_animations(&self) -> usize {
        self.dialogue_info
            .iter()
            .filter(|info| info.animation_score() > 3)
            .count()
    }

    /// Count fonts larger than reasonable size
    fn count_large_fonts(&self) -> usize {
        self.resolved_styles
            .iter()
            .filter(|style| style.font_size() > 72.0)
            .count()
    }

    /// Calculate overall performance score (0-100)
    fn calculate_performance_score(&self) -> u8 {
        let mut score = 100u8;

        if self.dialogue_info.len() > 1000 {
            score = score.saturating_sub(20);
        } else if self.dialogue_info.len() > 500 {
            score = score.saturating_sub(10);
        }

        let overlaps = self.count_overlapping_events();
        if overlaps > 50 {
            score = score.saturating_sub(15);
        } else if overlaps > 20 {
            score = score.saturating_sub(8);
        }

        let animations = self.count_complex_animations();
        if animations > 100 {
            score = score.saturating_sub(10);
        } else if animations > 50 {
            score = score.saturating_sub(5);
        }

        let large_fonts = self.count_large_fonts();
        if large_fonts > 10 {
            score = score.saturating_sub(5);
        }

        score
    }
}
