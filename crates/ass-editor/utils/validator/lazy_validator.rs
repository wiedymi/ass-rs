//! Lazy validator type and its construction/configuration.
//!
//! Defines `LazyValidator`, holding validation configuration, cached results,
//! and the ass-core analysis configuration used during validation.

use super::{ValidationResult, ValidatorConfig};

#[cfg(feature = "analysis")]
use ass_core::analysis::{AnalysisConfig, ScriptAnalysisOptions};

#[cfg(feature = "std")]
use std::time::Instant;

/// Lazy validator that wraps ass-core's ScriptAnalysis
///
/// Provides on-demand validation with caching and incremental updates
/// as specified in the architecture (line 164).
#[derive(Debug)]
pub struct LazyValidator {
    /// Configuration for validation behavior
    pub(super) config: ValidatorConfig,

    /// Cached validation result
    pub(super) cached_result: Option<ValidationResult>,

    /// Hash of last validated content
    pub(super) content_hash: u64,

    /// Last validation timestamp
    #[cfg(feature = "std")]
    pub(super) last_validation: Option<Instant>,

    /// Core analysis configuration
    #[cfg(feature = "analysis")]
    pub(super) analysis_config: AnalysisConfig,
}

impl LazyValidator {
    /// Create a new lazy validator with default configuration
    pub fn new() -> Self {
        Self::with_config(ValidatorConfig::default())
    }

    /// Create a new lazy validator with custom configuration
    pub fn with_config(config: ValidatorConfig) -> Self {
        Self {
            #[cfg(feature = "analysis")]
            analysis_config: AnalysisConfig {
                options: {
                    let mut options = ScriptAnalysisOptions::empty();
                    if config.enable_unicode_checks {
                        options |= ScriptAnalysisOptions::UNICODE_LINEBREAKS;
                    }
                    if config.enable_performance_hints {
                        options |= ScriptAnalysisOptions::PERFORMANCE_HINTS;
                    }
                    if config.enable_spec_compliance {
                        options |= ScriptAnalysisOptions::STRICT_COMPLIANCE;
                    }
                    if config.enable_accessibility_checks {
                        options |= ScriptAnalysisOptions::BIDI_ANALYSIS;
                    }
                    options
                },
                max_events_threshold: 1000,
            },
            config,
            cached_result: None,
            content_hash: 0,
            #[cfg(feature = "std")]
            last_validation: None,
        }
    }

    /// Update configuration
    pub fn set_config(&mut self, config: ValidatorConfig) {
        self.config = config;
        self.clear_cache(); // Config change invalidates cache

        #[cfg(feature = "analysis")]
        {
            self.analysis_config = AnalysisConfig {
                options: {
                    let mut options = ScriptAnalysisOptions::empty();
                    if self.config.enable_unicode_checks {
                        options |= ScriptAnalysisOptions::UNICODE_LINEBREAKS;
                    }
                    if self.config.enable_performance_hints {
                        options |= ScriptAnalysisOptions::PERFORMANCE_HINTS;
                    }
                    if self.config.enable_spec_compliance {
                        options |= ScriptAnalysisOptions::STRICT_COMPLIANCE;
                    }
                    if self.config.enable_accessibility_checks {
                        options |= ScriptAnalysisOptions::BIDI_ANALYSIS;
                    }
                    options
                },
                max_events_threshold: 1000,
            };
        }
    }
}

impl Default for LazyValidator {
    fn default() -> Self {
        Self::new()
    }
}
