//! ASS script style analysis and resolution
//!
//! Provides comprehensive style analysis capabilities including inheritance resolution,
//! conflict detection, performance analysis, and validation. Designed for zero-copy
//! efficiency with lifetime-generic references to original style definitions.
//!
//! # Features
//!
//! - **Style Resolution**: Compute effective styles from base definitions and overrides
//! - **Inheritance Analysis**: Track style inheritance chains and detect circular references
//! - **Conflict Detection**: Identify duplicate names, missing references, and conflicts
//! - **Performance Assessment**: Analyze rendering complexity and optimization opportunities
//! - **Validation**: Check property values, font availability, and spec compliance
//! - **Zero-Copy Design**: Minimal allocations via lifetime-generic spans
//!
//! # Performance Targets
//!
//! - Style resolution: <1ms for typical scripts
//! - Memory usage: ~200 bytes per resolved style
//! - Analysis: <2ms for complete script style analysis
//! - Caching: Efficient resolution with zero-copy references
//!
//! # Quick Start
//!
//! ```rust
//! use ass_core::analysis::styles::{StyleAnalyzer, ResolvedStyle};
//! use ass_core::parser::Script;
//!
//! let script_text = r#"
//! [V4+ Styles]
//! Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
//! Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
//! "#;
//!
//! let script = Script::parse(script_text)?;
//! let analyzer = StyleAnalyzer::new(&script);
//!
//! if let Some(resolved) = analyzer.resolve_style("Default") {
//!     println!("Font: {}, Size: {}", resolved.font_name(), resolved.font_size());
//!     println!("Complexity: {}/100", resolved.complexity_score());
//!     println!("Performance issues: {}", resolved.has_performance_issues());
//! }
//!
//! // Validate all styles
//! let issues = analyzer.validate_styles();
//! for issue in issues {
//!     println!("{}: {} in {}", issue.severity, issue.message, issue.field);
//! }
//!
//! // Check for conflicts
//! for conflict in analyzer.conflicts() {
//!     println!("Conflict: {}", conflict.description);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Module Organization
//!
//! - [`resolved_style`] - Fully resolved style representation with computed values
//! - [`validation`] - Style validation, conflict detection, and issue reporting
//! - [`analyzer`] - Main analysis interface orchestrating all style operations

pub mod analyzer;
pub mod resolved_style;
pub mod validation;

pub use analyzer::{PerformanceThresholds, StyleAnalysisConfig, StyleAnalyzer};
pub use resolved_style::ResolvedStyle;
pub use validation::{
    ConflictType, StyleConflict, StyleInheritance, StyleValidationIssue, ValidationSeverity,
};
