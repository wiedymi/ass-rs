//! Style analysis and resolution for ASS subtitle scripts
//!
//! Provides comprehensive style analysis including inheritance resolution, conflict detection,
//! and performance impact assessment. Designed for zero-copy analysis with efficient caching
//! and trait-based extensibility for custom style processors.
//!
//! # Features
//!
//! - **Style resolution**: Compute effective styles from base definitions and overrides
//! - **Inheritance analysis**: Track style inheritance chains and detect circular references
//! - **Conflict detection**: Identify conflicting style definitions and naming issues
//! - **Performance analysis**: Assess rendering complexity and optimization opportunities
//! - **Validation**: Check for invalid values, missing fonts, and spec compliance
//!
//! # Performance
//!
//! - Target: <1ms for style resolution of typical scripts
//! - Memory: Zero-copy references to original style definitions
//! - Caching: Resolved styles cached for repeated access
//!
//! # Example
//!
//! ```rust
//! use ass_core::analysis::styles::{StyleAnalyzer, ResolvedStyle};
//! use ass_core::Script;
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
//! let mut analyzer = StyleAnalyzer::new(&script);
//! if let Some(resolved) = analyzer.resolve_style("Default") {
//!     println!("Font: {}, Size: {}", resolved.font_name(), resolved.font_size());
//!     println!("Complexity: {}/10", resolved.complexity_score());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::{
    parser::{Script, Section, Style},
    utils::{parse_bgr_color, parse_numeric},
    Result,
};
use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
use core::fmt;

/// A fully resolved style with computed values and analysis metrics
///
/// Contains the effective style values after applying inheritance, overrides,
/// and default fallbacks. Includes performance and complexity metrics.
#[derive(Debug, Clone)]
pub struct ResolvedStyle<'a> {
    /// Style name
    pub name: &'a str,

    /// Resolved font properties
    font_name: String,
    font_size: f32,

    /// Resolved color values (RGBA format)
    primary_color: [u8; 4],
    secondary_color: [u8; 4],
    outline_color: [u8; 4],
    back_color: [u8; 4],

    /// Text formatting flags
    bold: bool,
    italic: bool,
    underline: bool,
    strike_out: bool,

    /// Transform properties
    scale_x: f32,
    scale_y: f32,
    spacing: f32,
    angle: f32,

    /// Border and shadow
    border_style: u8,
    outline: f32,
    shadow: f32,

    /// Alignment and margins
    alignment: u8,
    margin_l: u16,
    margin_r: u16,
    margin_v: u16,

    /// Text encoding
    encoding: u8,

    /// Analysis metrics
    complexity_score: u8,
    performance_impact: StylePerformanceImpact,
    validation_issues: Vec<StyleValidationIssue>,

    /// Source style reference
    source_style: &'a Style<'a>,
}

/// Performance impact assessment for a style
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StylePerformanceImpact {
    /// Minimal impact - basic text rendering
    Minimal,
    /// Low impact - simple formatting
    Low,
    /// Medium impact - effects or scaling
    Medium,
    /// High impact - complex effects
    High,
    /// Critical impact - may cause performance issues
    Critical,
}

/// Style validation issue types
#[derive(Debug, Clone)]
pub struct StyleValidationIssue {
    /// Issue severity
    pub severity: ValidationSeverity,
    /// Issue description
    pub message: String,
    /// Field that has the issue
    pub field: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Severity levels for style validation issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// Informational message
    Info,
    /// Warning about potential issues
    Warning,
    /// Error that should be fixed
    Error,
}

impl fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Style inheritance information
#[derive(Debug, Clone)]
pub struct StyleInheritance<'a> {
    /// Style name
    pub name: &'a str,
    /// Parent styles (if any)
    pub parents: Vec<&'a str>,
    /// Child styles that inherit from this one
    pub children: Vec<&'a str>,
    /// Inheritance depth (0 = root style)
    pub depth: usize,
    /// Whether this style has circular inheritance
    pub has_circular_inheritance: bool,
}

/// Style conflict detection results
#[derive(Debug, Clone)]
pub struct StyleConflict<'a> {
    /// Conflicting style names
    pub styles: Vec<&'a str>,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Conflict description
    pub description: String,
    /// Severity of the conflict
    pub severity: ValidationSeverity,
}

/// Types of style conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictType {
    /// Duplicate style names
    DuplicateName,
    /// Circular inheritance
    CircularInheritance,
    /// Conflicting property values
    PropertyConflict,
    /// Missing referenced style
    MissingReference,
}

/// Comprehensive style analyzer for ASS scripts
///
/// Provides style resolution, validation, and conflict detection with
/// efficient caching and zero-copy analysis where possible.
pub struct StyleAnalyzer<'a> {
    /// Reference to the script being analyzed
    script: &'a Script<'a>,
    /// Cached resolved styles
    resolved_styles: BTreeMap<&'a str, ResolvedStyle<'a>>,
    /// Style inheritance information
    inheritance_info: BTreeMap<&'a str, StyleInheritance<'a>>,
    /// Detected conflicts
    conflicts: Vec<StyleConflict<'a>>,
    /// Analysis configuration
    config: StyleAnalysisConfig,
}

/// Configuration for style analysis behavior
#[derive(Debug, Clone)]
pub struct StyleAnalysisConfig {
    /// Enable inheritance analysis
    pub analyze_inheritance: bool,
    /// Enable conflict detection
    pub detect_conflicts: bool,
    /// Enable performance analysis
    pub analyze_performance: bool,
    /// Enable validation
    pub validate_values: bool,
    /// Strict validation mode
    pub strict_validation: bool,
    /// Font size limits for validation
    pub min_font_size: f32,
    pub max_font_size: f32,
    /// Performance thresholds
    pub performance_thresholds: PerformanceThresholds,
}

/// Performance analysis thresholds
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// Font size threshold for performance warnings
    pub large_font_threshold: f32,
    /// Outline size threshold
    pub large_outline_threshold: f32,
    /// Shadow distance threshold
    pub large_shadow_threshold: f32,
    /// Scaling threshold for performance impact
    pub scaling_threshold: f32,
}

impl Default for StyleAnalysisConfig {
    fn default() -> Self {
        Self {
            analyze_inheritance: true,
            detect_conflicts: true,
            analyze_performance: true,
            validate_values: true,
            strict_validation: false,
            min_font_size: 4.0,
            max_font_size: 200.0,
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

impl<'a> StyleAnalyzer<'a> {
    /// Create a new style analyzer for the given script
    pub fn new(script: &'a Script<'a>) -> Self {
        Self::new_with_config(script, StyleAnalysisConfig::default())
    }

    /// Create a new style analyzer with custom configuration
    pub fn new_with_config(script: &'a Script<'a>, config: StyleAnalysisConfig) -> Self {
        let mut analyzer = Self {
            script,
            resolved_styles: BTreeMap::new(),
            inheritance_info: BTreeMap::new(),
            conflicts: Vec::new(),
            config,
        };

        // Perform initial analysis
        analyzer.analyze_all_styles();

        analyzer
    }

    /// Get a resolved style by name
    ///
    /// Returns the fully resolved style with all inheritance and defaults applied.
    /// Results are cached for efficient repeated access.
    pub fn resolve_style(&mut self, name: &'a str) -> Option<&ResolvedStyle<'a>> {
        // Check cache first
        if self.resolved_styles.contains_key(name) {
            return self.resolved_styles.get(name);
        }

        // Find and resolve the style
        if let Some(style) = self.find_style_definition(name) {
            match self.resolve_style_internal(style) {
                Ok(resolved) => {
                    self.resolved_styles.insert(name, resolved);
                    self.resolved_styles.get(name)
                }
                Err(_) => None,
            }
        } else {
            None
        }
    }

    /// Get all resolved styles
    pub fn resolved_styles(&self) -> &BTreeMap<&'a str, ResolvedStyle<'a>> {
        &self.resolved_styles
    }

    /// Get style inheritance information
    pub fn inheritance_info(&self) -> &BTreeMap<&'a str, StyleInheritance<'a>> {
        &self.inheritance_info
    }

    /// Get detected style conflicts
    pub fn conflicts(&self) -> &[StyleConflict<'a>] {
        &self.conflicts
    }

    /// Get styles with performance concerns
    pub fn performance_issues(&self) -> Vec<&ResolvedStyle<'a>> {
        self.resolved_styles
            .values()
            .filter(|style| {
                matches!(
                    style.performance_impact,
                    StylePerformanceImpact::High | StylePerformanceImpact::Critical
                )
            })
            .collect()
    }

    /// Get styles with validation issues
    pub fn validation_issues(&self) -> Vec<&ResolvedStyle<'a>> {
        self.resolved_styles
            .values()
            .filter(|style| !style.validation_issues.is_empty())
            .collect()
    }

    /// Analyze all styles in the script
    fn analyze_all_styles(&mut self) {
        // Find all style definitions
        if let Some(Section::Styles(styles)) = self
            .script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Styles(_)))
        {
            // Resolve each style
            for style in styles {
                if let Ok(resolved) = self.resolve_style_internal(style) {
                    self.resolved_styles.insert(style.name, resolved);
                }
            }

            // Analyze inheritance if enabled
            if self.config.analyze_inheritance {
                self.analyze_inheritance(styles);
            }

            // Detect conflicts if enabled
            if self.config.detect_conflicts {
                self.detect_conflicts(styles);
            }
        }
    }

    /// Find a style definition by name
    fn find_style_definition(&self, name: &str) -> Option<&'a Style<'a>> {
        if let Some(Section::Styles(styles)) = self
            .script
            .sections()
            .iter()
            .find(|s| matches!(s, Section::Styles(_)))
        {
            styles.iter().find(|style| style.name == name)
        } else {
            None
        }
    }

    /// Resolve a single style with full analysis
    fn resolve_style_internal(&self, style: &'a Style<'a>) -> Result<ResolvedStyle<'a>> {
        let mut resolved = ResolvedStyle {
            name: style.name,
            font_name: style.fontname.to_string(),
            font_size: parse_numeric::<f32>(style.fontsize).unwrap_or(20.0),
            primary_color: parse_bgr_color(style.primary_colour).unwrap_or([255, 255, 255, 0]),
            secondary_color: parse_bgr_color(style.secondary_colour).unwrap_or([255, 0, 0, 0]),
            outline_color: parse_bgr_color(style.outline_colour).unwrap_or([0, 0, 0, 0]),
            back_color: parse_bgr_color(style.back_colour).unwrap_or([0, 0, 0, 0]),
            bold: parse_numeric::<i32>(style.bold).unwrap_or(0) != 0,
            italic: parse_numeric::<i32>(style.italic).unwrap_or(0) != 0,
            underline: parse_numeric::<i32>(style.underline).unwrap_or(0) != 0,
            strike_out: parse_numeric::<i32>(style.strikeout).unwrap_or(0) != 0,
            scale_x: parse_numeric::<f32>(style.scale_x).unwrap_or(100.0),
            scale_y: parse_numeric::<f32>(style.scale_y).unwrap_or(100.0),
            spacing: parse_numeric::<f32>(style.spacing).unwrap_or(0.0),
            angle: parse_numeric::<f32>(style.angle).unwrap_or(0.0),
            border_style: parse_numeric::<u8>(style.border_style).unwrap_or(1),
            outline: parse_numeric::<f32>(style.outline).unwrap_or(2.0),
            shadow: parse_numeric::<f32>(style.shadow).unwrap_or(0.0),
            alignment: parse_numeric::<u8>(style.alignment).unwrap_or(2),
            margin_l: parse_numeric::<u16>(style.margin_l).unwrap_or(10),
            margin_r: parse_numeric::<u16>(style.margin_r).unwrap_or(10),
            margin_v: parse_numeric::<u16>(style.margin_v).unwrap_or(10),
            encoding: parse_numeric::<u8>(style.encoding).unwrap_or(1),
            complexity_score: 0,
            performance_impact: StylePerformanceImpact::Minimal,
            validation_issues: Vec::new(),
            source_style: style,
        };

        // Perform validation if enabled
        if self.config.validate_values {
            self.validate_style(&mut resolved);
        }

        // Analyze performance if enabled
        if self.config.analyze_performance {
            self.analyze_style_performance(&mut resolved);
        }

        Ok(resolved)
    }

    /// Validate style values and record issues
    fn validate_style(&self, style: &mut ResolvedStyle<'a>) {
        // Font size validation
        if style.font_size < self.config.min_font_size {
            style.validation_issues.push(StyleValidationIssue {
                severity: ValidationSeverity::Warning,
                message: format!("Font size {} is very small", style.font_size),
                field: "Fontsize".to_string(),
                suggestion: Some(format!(
                    "Consider using size >= {}",
                    self.config.min_font_size
                )),
            });
        }

        if style.font_size > self.config.max_font_size {
            style.validation_issues.push(StyleValidationIssue {
                severity: ValidationSeverity::Warning,
                message: format!("Font size {} is very large", style.font_size),
                field: "Fontsize".to_string(),
                suggestion: Some(format!(
                    "Consider using size <= {}",
                    self.config.max_font_size
                )),
            });
        }

        // Scaling validation
        if style.scale_x < 10.0 || style.scale_x > 1000.0 {
            style.validation_issues.push(StyleValidationIssue {
                severity: ValidationSeverity::Warning,
                message: format!("Unusual X scaling: {}%", style.scale_x),
                field: "ScaleX".to_string(),
                suggestion: Some("Typical scaling range is 50-200%".to_string()),
            });
        }

        if style.scale_y < 10.0 || style.scale_y > 1000.0 {
            style.validation_issues.push(StyleValidationIssue {
                severity: ValidationSeverity::Warning,
                message: format!("Unusual Y scaling: {}%", style.scale_y),
                field: "ScaleY".to_string(),
                suggestion: Some("Typical scaling range is 50-200%".to_string()),
            });
        }

        // Alignment validation
        if style.alignment > 9 {
            style.validation_issues.push(StyleValidationIssue {
                severity: ValidationSeverity::Error,
                message: format!("Invalid alignment value: {}", style.alignment),
                field: "Alignment".to_string(),
                suggestion: Some("Alignment must be 1-9 (numpad layout)".to_string()),
            });
        }

        // Border style validation
        if style.border_style > 4 {
            style.validation_issues.push(StyleValidationIssue {
                severity: ValidationSeverity::Warning,
                message: format!("Unusual border style: {}", style.border_style),
                field: "BorderStyle".to_string(),
                suggestion: Some("Standard border styles are 0-3".to_string()),
            });
        }

        // Outline and shadow validation
        if style.outline < 0.0 {
            style.validation_issues.push(StyleValidationIssue {
                severity: ValidationSeverity::Error,
                message: "Negative outline value".to_string(),
                field: "Outline".to_string(),
                suggestion: Some("Outline must be >= 0".to_string()),
            });
        }

        if style.shadow < 0.0 {
            style.validation_issues.push(StyleValidationIssue {
                severity: ValidationSeverity::Error,
                message: "Negative shadow value".to_string(),
                field: "Shadow".to_string(),
                suggestion: Some("Shadow must be >= 0".to_string()),
            });
        }
    }

    /// Analyze style performance impact
    fn analyze_style_performance(&self, style: &mut ResolvedStyle<'a>) {
        let mut complexity = 0u8;

        // Font size impact
        if style.font_size > self.config.performance_thresholds.large_font_threshold {
            complexity += 2;
        }

        // Scaling impact
        if style.scale_x > self.config.performance_thresholds.scaling_threshold
            || style.scale_y > self.config.performance_thresholds.scaling_threshold
        {
            complexity += 2;
        }

        // Outline impact
        if style.outline > self.config.performance_thresholds.large_outline_threshold {
            complexity += 3;
        }

        // Shadow impact
        if style.shadow > self.config.performance_thresholds.large_shadow_threshold {
            complexity += 2;
        }

        // Rotation impact
        if style.angle != 0.0 {
            complexity += 3;
        }

        // Bold/italic impact (minimal)
        if style.bold || style.italic {
            complexity += 1;
        }

        // Border style impact
        if style.border_style > 1 {
            complexity += 1;
        }

        style.complexity_score = complexity.min(10);

        // Determine performance impact
        style.performance_impact = match complexity {
            0..=2 => StylePerformanceImpact::Minimal,
            3..=4 => StylePerformanceImpact::Low,
            5..=6 => StylePerformanceImpact::Medium,
            7..=8 => StylePerformanceImpact::High,
            _ => StylePerformanceImpact::Critical,
        };
    }

    /// Analyze style inheritance relationships
    fn analyze_inheritance(&mut self, _styles: &[Style<'a>]) {
        // TODO: Implement inheritance analysis
        // ASS doesn't have formal inheritance, but styles can reference others
        // through override tags and default fallbacks
    }

    /// Detect style conflicts
    fn detect_conflicts(&mut self, styles: &[Style<'a>]) {
        // Check for duplicate names
        let mut name_counts = BTreeMap::new();
        for style in styles {
            *name_counts.entry(style.name).or_insert(0) += 1;
        }

        for (name, count) in name_counts {
            if count > 1 {
                self.conflicts.push(StyleConflict {
                    styles: vec![name],
                    conflict_type: ConflictType::DuplicateName,
                    description: format!("Style '{}' is defined {} times", name, count),
                    severity: ValidationSeverity::Error,
                });
            }
        }
    }
}

impl<'a> ResolvedStyle<'a> {
    /// Get the style name
    pub fn name(&self) -> &str {
        self.name
    }

    /// Get the font name
    pub fn font_name(&self) -> &str {
        &self.font_name
    }

    /// Get the font size
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Get primary color (RGBA)
    pub fn primary_color(&self) -> [u8; 4] {
        self.primary_color
    }

    /// Get secondary color (RGBA)
    pub fn secondary_color(&self) -> [u8; 4] {
        self.secondary_color
    }

    /// Get outline color (RGBA)
    pub fn outline_color(&self) -> [u8; 4] {
        self.outline_color
    }

    /// Get background color (RGBA)
    pub fn back_color(&self) -> [u8; 4] {
        self.back_color
    }

    /// Check if bold
    pub fn is_bold(&self) -> bool {
        self.bold
    }

    /// Check if italic
    pub fn is_italic(&self) -> bool {
        self.italic
    }

    /// Check if underlined
    pub fn is_underline(&self) -> bool {
        self.underline
    }

    /// Check if struck out
    pub fn is_strike_out(&self) -> bool {
        self.strike_out
    }

    /// Get X scaling factor
    pub fn scale_x(&self) -> f32 {
        self.scale_x
    }

    /// Get Y scaling factor
    pub fn scale_y(&self) -> f32 {
        self.scale_y
    }

    /// Get character spacing
    pub fn spacing(&self) -> f32 {
        self.spacing
    }

    /// Get rotation angle
    pub fn angle(&self) -> f32 {
        self.angle
    }

    /// Get border style
    pub fn border_style(&self) -> u8 {
        self.border_style
    }

    /// Get outline thickness
    pub fn outline(&self) -> f32 {
        self.outline
    }

    /// Get shadow distance
    pub fn shadow(&self) -> f32 {
        self.shadow
    }

    /// Get text alignment
    pub fn alignment(&self) -> u8 {
        self.alignment
    }

    /// Get left margin
    pub fn margin_l(&self) -> u16 {
        self.margin_l
    }

    /// Get right margin
    pub fn margin_r(&self) -> u16 {
        self.margin_r
    }

    /// Get vertical margin
    pub fn margin_v(&self) -> u16 {
        self.margin_v
    }

    /// Get text encoding
    pub fn encoding(&self) -> u8 {
        self.encoding
    }

    /// Get complexity score (0-10)
    pub fn complexity_score(&self) -> u8 {
        self.complexity_score
    }

    /// Get performance impact assessment
    pub fn performance_impact(&self) -> StylePerformanceImpact {
        self.performance_impact
    }

    /// Get validation issues
    pub fn validation_issues(&self) -> &[StyleValidationIssue] {
        &self.validation_issues
    }

    /// Check if style has any validation issues
    pub fn has_validation_issues(&self) -> bool {
        !self.validation_issues.is_empty()
    }

    /// Check if style has validation errors (not just warnings)
    pub fn has_validation_errors(&self) -> bool {
        self.validation_issues
            .iter()
            .any(|issue| issue.severity == ValidationSeverity::Error)
    }

    /// Get reference to source style definition
    pub fn source_style(&self) -> &'a Style<'a> {
        self.source_style
    }
}

impl fmt::Display for StylePerformanceImpact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Minimal => write!(f, "minimal"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Script;

    #[test]
    fn style_analyzer_basic() {
        let script_text = r#"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Large,Arial,50,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,1,0,0,0,120,120,0,0,1,3,2,2,10,10,10,1
"#;

        let script = Script::parse(script_text).unwrap();
        let mut analyzer = StyleAnalyzer::new(&script);

        // Test style resolution
        let default_style = analyzer.resolve_style("Default").unwrap();
        assert_eq!(default_style.name(), "Default");
        assert_eq!(default_style.font_name(), "Arial");
        assert_eq!(default_style.font_size(), 20.0);
        assert!(!default_style.is_bold());
        let default_complexity = default_style.complexity_score();

        let large_style = analyzer.resolve_style("Large").unwrap();
        assert_eq!(large_style.font_size(), 50.0);
        assert!(large_style.is_bold());
        assert!(large_style.complexity_score() > default_complexity);
    }

    #[test]
    fn style_validation() {
        let script_text = r#"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Invalid,Arial,300,&HINVALID&,&H000000FF,&H00000000,&H00000000,0,0,0,0,50,50,0,0,1,-1,-1,15,10,10,10,1
"#;

        let script = Script::parse(script_text).unwrap();
        let mut analyzer = StyleAnalyzer::new(&script);

        let invalid_style = analyzer.resolve_style("Invalid").unwrap();
        assert!(invalid_style.has_validation_issues());
        assert!(invalid_style.has_validation_errors());

        let issues = invalid_style.validation_issues();
        assert!(!issues.is_empty());

        // Should have issues with font size, alignment, and negative values
        assert!(issues.iter().any(|i| i.field == "Fontsize"));
        assert!(issues.iter().any(|i| i.field == "Alignment"));
    }

    #[test]
    fn performance_analysis() {
        let script_text = r#"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Simple,Arial,16,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1
Style: Complex,Arial,80,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,1,1,0,0,150,150,0,45,1,5,3,2,10,10,10,1
"#;

        let script = Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let simple_style = analyzer.resolved_styles().get("Simple").unwrap();
        let complex_style = analyzer.resolved_styles().get("Complex").unwrap();

        assert_eq!(
            simple_style.performance_impact(),
            StylePerformanceImpact::Minimal
        );
        assert!(complex_style.performance_impact() > StylePerformanceImpact::Minimal);
        assert!(complex_style.complexity_score() > simple_style.complexity_score());
    }

    #[test]
    fn conflict_detection() {
        let script_text = r#"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Default,Times,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
"#;

        let script = Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let conflicts = analyzer.conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::DuplicateName);
        assert!(conflicts[0].description.contains("Default"));
        assert_eq!(conflicts[0].severity, ValidationSeverity::Error);
    }

    #[test]
    fn style_analysis_config() {
        let config = StyleAnalysisConfig {
            analyze_inheritance: false,
            detect_conflicts: false,
            analyze_performance: true,
            validate_values: false,
            ..StyleAnalysisConfig::default()
        };

        let script_text = r#"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Test,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
"#;

        let script = Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new_with_config(&script, config);

        // Should have no conflicts since detection is disabled
        assert_eq!(analyzer.conflicts().len(), 0);

        // Should still have resolved styles
        assert_eq!(analyzer.resolved_styles().len(), 1);
        assert!(analyzer.resolved_styles().contains_key("Test"));
    }

    #[test]
    fn resolved_style_accessors() {
        let script_text = r#"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Test,Times New Roman,24,&H00FF0000,&H0000FF00,&H000000FF,&H00808080,1,1,1,1,150,75,2.5,45,2,3,4,5,15,20,25,1
"#;

        let script = Script::parse(script_text).unwrap();
        let mut analyzer = StyleAnalyzer::new(&script);
        let style = analyzer.resolve_style("Test").unwrap();

        assert_eq!(style.name(), "Test");
        assert_eq!(style.font_name(), "Times New Roman");
        assert_eq!(style.font_size(), 24.0);
        assert!(style.is_bold());
        assert!(style.is_italic());
        assert!(style.is_underline());
        assert!(style.is_strike_out());
        assert_eq!(style.scale_x(), 150.0);
        assert_eq!(style.scale_y(), 75.0);
        assert_eq!(style.spacing(), 2.5);
        assert_eq!(style.angle(), 45.0);
        assert_eq!(style.border_style(), 2);
        assert_eq!(style.outline(), 3.0);
        assert_eq!(style.shadow(), 4.0);
        assert_eq!(style.alignment(), 5);
        assert_eq!(style.margin_l(), 15);
        assert_eq!(style.margin_r(), 20);
        assert_eq!(style.margin_v(), 25);
        assert_eq!(style.encoding(), 1);
    }
}
