//! Script analysis and linting for ASS subtitle scripts
//!
//! Provides comprehensive analysis capabilities including style resolution,
//! linting for common issues, and performance optimization suggestions.
//! Designed for editor integration and script validation.
//!
//! # Features
//!
//! - Style resolution: Compute effective styles from base + overrides
//! - Linting rules: Detect common problems and spec violations
//! - Performance analysis: Identify rendering bottlenecks
//! - Unicode handling: Bidirectional text and linebreak analysis
//! - Timing validation: Overlap detection and duration checks
//!
//! # Performance
//!
//! - Target: <2ms analysis for typical scripts
//! - Memory: Lazy evaluation to avoid allocation spikes
//! - Thread-safe: Immutable analysis results
//!
//! # Example
//!
//! ```rust
//! use ass_core::{Script, analysis::ScriptAnalysis};
//!
//! let script_text = r#"
//! [Script Info]
//! Title: Test
//!
//! [V4+ Styles]
//! Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
//! Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
//!
//! [Events]
//! Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
//! Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
//! "#;
//!
//! let script = Script::parse(script_text)?;
//! let analysis = ScriptAnalysis::analyze(&script)?;
//!
//! // Check for issues
//! for issue in analysis.lint_issues() {
//!     println!("Warning: {}", issue.message());
//! }
//!
//! // Get resolved styles
//! if let Some(style) = analysis.resolve_style("Default") {
//!     println!("Font: {}", style.font_name());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::{
    parser::{Script, Section},
    Result,
};
use alloc::vec::Vec;

pub mod events;
pub mod linting;
pub mod styles;

pub use events::{
    count_overlapping_dialogue_events, count_overlapping_events, find_overlapping_events,
    DialogueInfo,
};
pub use linting::{lint_script, LintConfig, LintIssue, LintRule};
pub use styles::{ResolvedStyle, StyleAnalyzer};

/// Comprehensive analysis of an ASS script
///
/// Provides linting, style resolution, and performance analysis.
/// Results are cached for efficient repeated access.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScriptAnalysis<'a> {
    /// Reference to analyzed script
    pub script: &'a Script<'a>,

    /// Detected lint issues
    lint_issues: Vec<LintIssue>,

    /// Resolved styles cache
    resolved_styles: Vec<ResolvedStyle<'a>>,

    /// Dialogue analysis results
    dialogue_info: Vec<DialogueInfo<'a>>,

    /// Analysis configuration
    config: AnalysisConfig,
}

/// Configuration for script analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Enable Unicode linebreak analysis (libass 0.17.4+)
    pub unicode_linebreaks: bool,

    /// Enable performance warnings
    pub performance_hints: bool,

    /// Enable strict spec compliance checking
    pub strict_compliance: bool,

    /// Maximum allowed events for performance warnings
    pub max_events_threshold: usize,

    /// Enable bidirectional text analysis
    pub bidi_analysis: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            unicode_linebreaks: true,
            performance_hints: true,
            strict_compliance: false,
            max_events_threshold: 1000,
            bidi_analysis: true,
        }
    }
}

impl<'a> ScriptAnalysis<'a> {
    /// Analyze script with default configuration
    ///
    /// Performs comprehensive analysis including linting, style resolution,
    /// and event analysis. Results are cached for efficient access.
    ///
    /// # Performance
    ///
    /// Target <2ms for typical scripts. Uses lazy evaluation for expensive
    /// operations like Unicode analysis.
    pub fn analyze(script: &'a Script<'a>) -> Result<Self> {
        Self::analyze_with_config(script, AnalysisConfig::default())
    }

    /// Analyze script with custom configuration
    ///
    /// Allows fine-tuning analysis behavior for specific use cases.
    pub fn analyze_with_config(script: &'a Script<'a>, config: AnalysisConfig) -> Result<Self> {
        let mut analysis = Self {
            script,
            lint_issues: Vec::new(),
            resolved_styles: Vec::new(),
            dialogue_info: Vec::new(),
            config,
        };

        analysis.resolve_all_styles()?;
        analysis.analyze_events()?;
        analysis.run_linting()?;

        Ok(analysis)
    }

    /// Get all lint issues found during analysis
    #[must_use] pub fn lint_issues(&self) -> &[LintIssue] {
        &self.lint_issues
    }

    /// Get resolved styles
    #[must_use] pub fn resolved_styles(&self) -> &[ResolvedStyle<'a>] {
        &self.resolved_styles
    }

    /// Get dialogue analysis results
    #[must_use] pub fn dialogue_info(&self) -> &[DialogueInfo<'a>] {
        &self.dialogue_info
    }

    /// Get reference to the analyzed script
    #[must_use] pub const fn script(&self) -> &'a Script<'a> {
        self.script
    }

    /// Find resolved style by name
    #[must_use] pub fn resolve_style(&self, name: &str) -> Option<&ResolvedStyle<'a>> {
        self.resolved_styles.iter().find(|style| style.name == name)
    }

    /// Check if script has any critical issues
    #[must_use] pub fn has_critical_issues(&self) -> bool {
        self.lint_issues
            .iter()
            .any(|issue| issue.severity() == linting::IssueSeverity::Critical)
    }

    /// Get performance summary
    #[must_use] pub fn performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            total_events: self.dialogue_info.len(),
            overlapping_events: self.count_overlapping_events(),
            complex_animations: self.count_complex_animations(),
            large_fonts: self.count_large_fonts(),
            performance_score: self.calculate_performance_score(),
        }
    }

    /// Run linting analysis
    fn run_linting(&mut self) -> Result<()> {
        let lint_config =
            LintConfig::default().with_strict_compliance(self.config.strict_compliance);

        let mut issues = Vec::new();
        let rules = linting::rules::BuiltinRules::all_rules();

        for rule in rules {
            if !lint_config.is_rule_enabled(rule.id()) {
                continue;
            }

            let mut rule_issues = rule.check_script(self);
            rule_issues.retain(|issue| lint_config.should_report_severity(issue.severity()));

            issues.extend(rule_issues);

            if lint_config.max_issues > 0 && issues.len() >= lint_config.max_issues {
                issues.truncate(lint_config.max_issues);
                break;
            }
        }

        self.lint_issues = issues;
        Ok(())
    }

    /// Resolve all styles with inheritance and overrides
    fn resolve_all_styles(&mut self) -> Result<()> {
        let analyzer = StyleAnalyzer::new(self.script);
        self.resolved_styles = analyzer.resolved_styles().values().cloned().collect();
        Ok(())
    }

    /// Analyze events for timing, overlaps, and performance
    fn analyze_events(&mut self) -> Result<()> {
        if let Some(Section::Events(events)) = self
            .script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            for event in events {
                match DialogueInfo::analyze(event) {
                    Ok(info) => self.dialogue_info.push(info),
                    Err(_) => {} // Skip invalid events
                }
            }
        }
        Ok(())
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

/// Performance analysis summary
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    /// Total number of dialogue events
    pub total_events: usize,

    /// Number of overlapping events
    pub overlapping_events: usize,

    /// Number of complex animations
    pub complex_animations: usize,

    /// Number of oversized fonts
    pub large_fonts: usize,

    /// Overall performance score (0-100, higher is better)
    pub performance_score: u8,
}

impl PerformanceSummary {
    /// Check if script has performance concerns
    #[must_use] pub const fn has_performance_issues(&self) -> bool {
        self.performance_score < 80
    }

    /// Get performance recommendation
    #[must_use] pub const fn recommendation(&self) -> Option<&'static str> {
        if self.overlapping_events > 10 {
            Some("Consider reducing overlapping events for better performance")
        } else if self.complex_animations > 20 {
            Some("Many complex animations may impact rendering performance")
        } else if self.large_fonts > 5 {
            Some("Large font sizes may cause memory issues")
        } else if self.total_events > 1000 {
            Some("Very large script - consider splitting into multiple files")
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_config_default() {
        let config = AnalysisConfig::default();
        assert!(config.unicode_linebreaks);
        assert!(config.performance_hints);
        assert!(!config.strict_compliance);
        assert_eq!(config.max_events_threshold, 1000);
    }

    #[test]
    fn script_analysis_basic() {
        let script_text = r"
[Script Info]
Title: Test Script

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Second line
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analysis = ScriptAnalysis::analyze(&script).unwrap();

        assert_eq!(analysis.lint_issues().len(), 0);
        assert!(!analysis.has_critical_issues());

        let perf = analysis.performance_summary();
        assert!(perf.performance_score > 0);
    }

    #[test]
    fn performance_summary_recommendations() {
        let summary = PerformanceSummary {
            total_events: 100,
            overlapping_events: 15,
            complex_animations: 5,
            large_fonts: 2,
            performance_score: 75,
        };

        assert!(summary.has_performance_issues());
        assert!(summary.recommendation().is_some());
        assert!(summary
            .recommendation()
            .unwrap()
            .contains("overlapping events"));
    }
}
