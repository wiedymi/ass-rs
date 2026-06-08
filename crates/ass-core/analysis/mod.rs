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
//! [Events\]
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

use crate::parser::Script;

#[cfg(feature = "plugins")]
use crate::plugin::ExtensionRegistry;
use alloc::vec::Vec;

bitflags::bitflags! {
    /// Script analysis options
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ScriptAnalysisOptions: u8 {
        /// Enable Unicode linebreak analysis (libass 0.17.4+)
        const UNICODE_LINEBREAKS = 1 << 0;
        /// Enable performance warnings
        const PERFORMANCE_HINTS = 1 << 1;
        /// Enable strict spec compliance checking
        const STRICT_COMPLIANCE = 1 << 2;
        /// Enable bidirectional text analysis
        const BIDI_ANALYSIS = 1 << 3;
    }
}

mod construction;
pub mod events;
pub mod linting;
mod queries;
pub mod styles;

#[cfg(test)]
mod analysis_tests;

pub use events::{
    count_overlapping_dialogue_events, count_overlapping_events, find_overlapping_dialogue_events,
    find_overlapping_events, DialogueInfo,
};
pub use linting::{lint_script, LintConfig, LintIssue, LintRule};
pub use styles::{ResolvedStyle, StyleAnalyzer};

/// Comprehensive analysis of an ASS script
///
/// Provides linting, style resolution, and performance analysis.
/// Results are cached for efficient repeated access.
#[derive(Debug, Clone)]
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

    /// Extension registry for custom tag handlers
    #[cfg(feature = "plugins")]
    registry: Option<&'a ExtensionRegistry>,
}

/// Configuration for script analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Analysis options flags
    pub options: ScriptAnalysisOptions,

    /// Maximum allowed events for performance warnings
    pub max_events_threshold: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            options: ScriptAnalysisOptions::UNICODE_LINEBREAKS
                | ScriptAnalysisOptions::PERFORMANCE_HINTS
                | ScriptAnalysisOptions::BIDI_ANALYSIS,
            max_events_threshold: 1000,
        }
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
    #[must_use]
    pub const fn has_performance_issues(&self) -> bool {
        self.performance_score < 80
    }

    /// Get performance recommendation
    #[must_use]
    pub const fn recommendation(&self) -> Option<&'static str> {
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
