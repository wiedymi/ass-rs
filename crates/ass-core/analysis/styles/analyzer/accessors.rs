//! Construction and accessor methods for [`StyleAnalyzer`].
//!
//! Provides analyzer creation entry points and read-only accessors for the
//! cached resolution, inheritance, and conflict results, plus the top-level
//! [`StyleAnalyzer::validate_styles`] driver.

use super::{AnalysisOptions, StyleAnalysisConfig, StyleAnalyzer};
use crate::{
    analysis::styles::{
        resolved_style::ResolvedStyle,
        validation::{StyleConflict, StyleInheritance, StyleValidationIssue},
    },
    parser::Script,
};
use alloc::{collections::BTreeMap, vec::Vec};

impl<'a> StyleAnalyzer<'a> {
    /// Create analyzer with default configuration
    #[must_use]
    pub fn new(script: &'a Script<'a>) -> Self {
        Self::new_with_config(script, StyleAnalysisConfig::default())
    }

    /// Create analyzer with custom configuration
    #[must_use]
    pub fn new_with_config(script: &'a Script<'a>, config: StyleAnalysisConfig) -> Self {
        let mut analyzer = Self {
            script,
            resolved_styles: BTreeMap::new(),
            inheritance_info: BTreeMap::new(),
            conflicts: Vec::new(),
            config,
            resolution_scaling: None,
        };

        analyzer.calculate_resolution_scaling();
        analyzer.analyze_all_styles();
        analyzer
    }

    /// Get resolved style by name
    #[must_use]
    pub fn resolve_style(&self, name: &str) -> Option<&ResolvedStyle<'a>> {
        self.resolved_styles.get(name)
    }

    /// Get all resolved styles
    #[must_use]
    pub const fn resolved_styles(&self) -> &BTreeMap<&'a str, ResolvedStyle<'a>> {
        &self.resolved_styles
    }

    /// Get detected conflicts
    #[must_use]
    pub fn conflicts(&self) -> &[StyleConflict<'a>] {
        &self.conflicts
    }

    /// Get inheritance information
    #[must_use]
    pub const fn inheritance_info(&self) -> &BTreeMap<&'a str, StyleInheritance<'a>> {
        &self.inheritance_info
    }

    /// Validate all styles and return issues
    #[must_use]
    pub fn validate_styles(&self) -> Vec<StyleValidationIssue> {
        let mut issues = Vec::new();

        for resolved in self.resolved_styles.values() {
            if self.config.options.contains(AnalysisOptions::VALIDATION) {
                issues.extend(self.validate_style_properties(resolved));
            }

            if self.config.options.contains(AnalysisOptions::PERFORMANCE) {
                issues.extend(self.analyze_style_performance(resolved));
            }
        }

        issues
    }
}
