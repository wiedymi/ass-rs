//! Core type definitions for the style analyzer.
//!
//! Defines [`StyleAnalyzer`] together with its configuration types
//! [`StyleAnalysisConfig`] and [`PerformanceThresholds`], plus the
//! [`AnalysisOptions`] flag set that drives analysis behavior.

use crate::{
    analysis::styles::{
        resolved_style::ResolvedStyle,
        validation::{StyleConflict, StyleInheritance},
    },
    parser::Script,
};
use alloc::{collections::BTreeMap, vec::Vec};

bitflags::bitflags! {
    /// Analysis options for style analyzer
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AnalysisOptions: u8 {
        /// Enable inheritance analysis
        const INHERITANCE = 1 << 0;
        /// Enable conflict detection
        const CONFLICTS = 1 << 1;
        /// Enable performance analysis
        const PERFORMANCE = 1 << 2;
        /// Enable value validation
        const VALIDATION = 1 << 3;
        /// Use strict validation rules
        const STRICT_VALIDATION = 1 << 4;
    }
}

/// Comprehensive style analyzer for ASS scripts
///
/// Orchestrates style analysis including resolution, validation, and conflict
/// detection. Maintains efficient caches for resolved styles and analysis results.
#[derive(Debug)]
pub struct StyleAnalyzer<'a> {
    /// Reference to script being analyzed
    pub(super) script: &'a Script<'a>,
    /// Cached resolved styles
    pub(super) resolved_styles: BTreeMap<&'a str, ResolvedStyle<'a>>,
    /// Style inheritance tracking
    pub(super) inheritance_info: BTreeMap<&'a str, StyleInheritance<'a>>,
    /// Detected style conflicts
    pub(super) conflicts: Vec<StyleConflict<'a>>,
    /// Analysis configuration
    pub(super) config: StyleAnalysisConfig,
    /// Resolution scaling factor (layout to play resolution)
    pub(super) resolution_scaling: Option<(f32, f32)>,
}

/// Configuration for style analysis behavior
#[derive(Debug, Clone)]
pub struct StyleAnalysisConfig {
    /// Analysis options flags
    pub options: AnalysisOptions,
    /// Performance analysis thresholds
    pub performance_thresholds: PerformanceThresholds,
}

/// Performance analysis threshold configuration
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// Font size threshold for performance warnings
    pub large_font_threshold: f32,
    /// Outline thickness threshold
    pub large_outline_threshold: f32,
    /// Shadow distance threshold
    pub large_shadow_threshold: f32,
    /// Scaling factor threshold
    pub scaling_threshold: f32,
}

impl Default for StyleAnalysisConfig {
    fn default() -> Self {
        Self {
            options: AnalysisOptions::INHERITANCE
                | AnalysisOptions::CONFLICTS
                | AnalysisOptions::PERFORMANCE
                | AnalysisOptions::VALIDATION,
            performance_thresholds: PerformanceThresholds::default(),
        }
    }
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            large_font_threshold: 50.0,
            large_outline_threshold: 4.0,
            large_shadow_threshold: 4.0,
            scaling_threshold: 200.0,
        }
    }
}
