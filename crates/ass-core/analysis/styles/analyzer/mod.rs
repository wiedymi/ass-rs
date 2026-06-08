//! Style analyzer for comprehensive ASS script style analysis
//!
//! Provides the main `StyleAnalyzer` interface for resolving styles, detecting
//! conflicts, and performing validation. Orchestrates analysis across multiple
//! sub-modules with efficient caching and zero-copy design.
//!
//! # Features
//!
//! - Comprehensive style resolution with inheritance support
//! - Conflict detection including circular inheritance and duplicates
//! - Performance analysis with configurable thresholds
//! - Validation with multiple severity levels
//! - Zero-copy analysis with lifetime-generic references
//!
//! # Performance
//!
//! - Target: <2ms for complete script style analysis
//! - Memory: Efficient caching with zero-copy style references
//! - Lazy evaluation: Analysis performed only when requested

mod accessors;
mod analysis;
mod checks;
mod dependency;
mod types;

#[cfg(test)]
mod basic_tests;
#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod extract_tests;
#[cfg(test)]
mod inheritance_conflict_tests;
#[cfg(test)]
mod inheritance_tests;
#[cfg(test)]
mod scaling_inheritance_tests;
#[cfg(test)]
mod scaling_tests;
#[cfg(test)]
mod validation_tests;

pub use types::{AnalysisOptions, PerformanceThresholds, StyleAnalysisConfig, StyleAnalyzer};
