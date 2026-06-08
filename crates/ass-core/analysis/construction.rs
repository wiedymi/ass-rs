//! Construction routines for [`ScriptAnalysis`].
//!
//! Provides the public `analyze*` constructors along with the private helpers
//! that resolve styles, analyze events, and run linting while building the
//! cached analysis result.

use super::{
    linting, AnalysisConfig, DialogueInfo, LintConfig, ScriptAnalysis, ScriptAnalysisOptions,
    StyleAnalyzer,
};
#[cfg(feature = "plugins")]
use crate::plugin::ExtensionRegistry;
use crate::{
    parser::{Script, Section},
    Result,
};
use alloc::vec::Vec;

impl<'a> ScriptAnalysis<'a> {
    /// Analyze script with default configuration
    ///
    /// Performs comprehensive analysis including linting, style resolution,
    /// and event analysis. Results are cached for efficient access.
    /// Analyze ASS script for issues, styles, and content
    ///
    /// # Performance
    ///
    /// Target <2ms for typical scripts. Uses lazy evaluation for expensive
    /// operations like Unicode analysis.
    ///
    /// # Errors
    ///
    /// Returns an error if script analysis fails or contains invalid data.
    pub fn analyze(script: &'a Script<'a>) -> Result<Self> {
        #[cfg(feature = "plugins")]
        return Self::analyze_with_registry(script, None, AnalysisConfig::default());
        #[cfg(not(feature = "plugins"))]
        return Self::analyze_with_config(script, AnalysisConfig::default());
    }

    /// Analyze script with extension registry support
    ///
    /// Same as [`analyze`](Self::analyze) but allows custom tag handlers via registry.
    /// Uses default analysis configuration.
    ///
    /// # Arguments
    ///
    /// * `script` - Script to analyze
    /// * `registry` - Optional registry for custom tag handlers
    ///
    /// # Errors
    ///
    /// Returns an error if script analysis fails or contains invalid data.
    #[cfg(feature = "plugins")]
    pub fn analyze_with_registry(
        script: &'a Script<'a>,
        registry: Option<&'a ExtensionRegistry>,
        config: AnalysisConfig,
    ) -> Result<Self> {
        Ok(Self::analyze_impl(script, registry, config))
    }

    /// Analyze script with custom configuration
    ///
    /// Allows fine-tuning analysis behavior for specific use cases.
    ///
    /// # Errors
    ///
    /// Returns an error if script analysis fails or contains invalid data.
    pub fn analyze_with_config(script: &'a Script<'a>, config: AnalysisConfig) -> Result<Self> {
        #[cfg(feature = "plugins")]
        return Ok(Self::analyze_impl(script, None, config));
        #[cfg(not(feature = "plugins"))]
        return Ok(Self::analyze_impl_no_plugins(script, config));
    }

    /// Internal implementation with plugins support
    #[cfg(feature = "plugins")]
    fn analyze_impl(
        script: &'a Script<'a>,
        registry: Option<&'a ExtensionRegistry>,
        config: AnalysisConfig,
    ) -> Self {
        let mut analysis = Self {
            script,
            lint_issues: Vec::new(),
            resolved_styles: Vec::new(),
            dialogue_info: Vec::new(),
            config,
            registry,
        };

        analysis.resolve_all_styles();
        analysis.analyze_events();
        analysis.run_linting();

        analysis
    }

    /// Internal implementation without plugins support
    #[cfg(not(feature = "plugins"))]
    fn analyze_impl_no_plugins(script: &'a Script<'a>, config: AnalysisConfig) -> Self {
        let mut analysis = Self {
            script,
            lint_issues: Vec::new(),
            resolved_styles: Vec::new(),
            dialogue_info: Vec::new(),
            config,
        };

        analysis.resolve_all_styles();
        analysis.analyze_events();
        analysis.run_linting();

        analysis
    }

    /// Run linting analysis
    fn run_linting(&mut self) {
        let lint_config = LintConfig::default().with_strict_compliance(
            self.config
                .options
                .contains(ScriptAnalysisOptions::STRICT_COMPLIANCE),
        );

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
    }

    /// Resolve all styles with inheritance and overrides
    pub(super) fn resolve_all_styles(&mut self) {
        let analyzer = StyleAnalyzer::new(self.script);
        self.resolved_styles = analyzer.resolved_styles().values().cloned().collect();
    }

    /// Analyze events for timing, overlaps, and performance
    pub(super) fn analyze_events(&mut self) {
        if let Some(Section::Events(events)) = self
            .script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Events(_)))
        {
            for event in events {
                #[cfg(feature = "plugins")]
                let info_result = self.registry.map_or_else(
                    || DialogueInfo::analyze(event),
                    |registry| DialogueInfo::analyze_with_registry(event, Some(registry)),
                );

                #[cfg(not(feature = "plugins"))]
                let info_result = DialogueInfo::analyze(event);

                if let Ok(info) = info_result {
                    self.dialogue_info.push(info);
                }
            }
        }
    }
}
