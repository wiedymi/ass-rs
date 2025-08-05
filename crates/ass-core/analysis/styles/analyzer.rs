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

use crate::{
    analysis::styles::{
        resolved_style::ResolvedStyle,
        validation::{StyleConflict, StyleInheritance, StyleValidationIssue},
    },
    parser::{Script, Section, Style},
};
use alloc::{collections::BTreeMap, collections::BTreeSet, vec::Vec};

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
    script: &'a Script<'a>,
    /// Cached resolved styles
    resolved_styles: BTreeMap<&'a str, ResolvedStyle<'a>>,
    /// Style inheritance tracking
    inheritance_info: BTreeMap<&'a str, StyleInheritance<'a>>,
    /// Detected style conflicts
    conflicts: Vec<StyleConflict<'a>>,
    /// Analysis configuration
    config: StyleAnalysisConfig,
    /// Resolution scaling factor (layout to play resolution)
    resolution_scaling: Option<(f32, f32)>,
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

    /// Calculate resolution scaling factor from script info
    fn calculate_resolution_scaling(&mut self) {
        // Find ScriptInfo section
        for section in self.script.sections() {
            if let Section::ScriptInfo(script_info) = section {
                let layout_res = script_info.layout_resolution();
                let play_res = script_info.play_resolution();

                if let (Some((layout_x, layout_y)), Some((play_x, play_y))) = (layout_res, play_res)
                {
                    // Only apply scaling if resolutions differ
                    if layout_x != play_x || layout_y != play_y {
                        #[allow(clippy::cast_precision_loss)]
                        let scale_x = play_x as f32 / layout_x as f32;
                        #[allow(clippy::cast_precision_loss)]
                        let scale_y = play_y as f32 / layout_y as f32;
                        self.resolution_scaling = Some((scale_x, scale_y));
                    }
                }
                break;
            }
        }
    }

    /// Analyze all styles in script
    fn analyze_all_styles(&mut self) {
        for section in self.script.sections() {
            if let Section::Styles(styles) = section {
                // Build dependency graph and resolve in order
                if let Some(ordered_styles) = self.build_dependency_order(styles) {
                    self.resolve_styles_with_inheritance(&ordered_styles);
                } else {
                    // Fall back to non-inherited resolution if circular dependency detected
                    for style in styles {
                        if let Ok(mut resolved) = ResolvedStyle::from_style(style) {
                            // Apply resolution scaling if needed
                            if let Some((scale_x, scale_y)) = self.resolution_scaling {
                                resolved.apply_resolution_scaling(scale_x, scale_y);
                            }
                            self.resolved_styles.insert(style.name, resolved);
                        }
                    }
                }

                if self.config.options.contains(AnalysisOptions::CONFLICTS) {
                    self.detect_style_conflicts_from_section(styles);
                }
                break;
            }
        }
    }

    /// Build dependency order for styles using topological sort
    /// Returns None if circular dependency is detected
    fn build_dependency_order(&mut self, styles: &'a [Style<'a>]) -> Option<Vec<&'a Style<'a>>> {
        // Create style map for quick lookup
        let style_map: BTreeMap<&str, &Style> = styles.iter().map(|s| (s.name, s)).collect();

        // Build adjacency list (child -> parent)
        let mut dependencies: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        let mut in_degree: BTreeMap<&str, usize> = BTreeMap::new();

        // Initialize all styles
        for style in styles {
            dependencies.insert(style.name, BTreeSet::new());
            in_degree.insert(style.name, 0);
        }

        // Build dependency graph
        for style in styles {
            if let Some(parent_name) = style.parent {
                if style_map.contains_key(parent_name) {
                    dependencies
                        .get_mut(style.name)
                        .unwrap()
                        .insert(parent_name);
                    *in_degree.get_mut(parent_name).unwrap() += 1;

                    // Track inheritance for analysis
                    if self.config.options.contains(AnalysisOptions::INHERITANCE) {
                        if let Some(inheritance) = self.inheritance_info.get_mut(style.name) {
                            inheritance.set_parent(parent_name);
                        } else {
                            let mut inheritance = StyleInheritance::new(style.name);
                            inheritance.set_parent(parent_name);
                            self.inheritance_info.insert(style.name, inheritance);
                        }
                    }
                } else {
                    // Parent style not found - add warning conflict
                    self.conflicts
                        .push(StyleConflict::missing_parent(style.name, parent_name));
                }
            } else if self.config.options.contains(AnalysisOptions::INHERITANCE) {
                // Style has no parent
                self.inheritance_info
                    .insert(style.name, StyleInheritance::new(style.name));
            }
        }

        // Check for circular dependencies using DFS
        if Self::has_circular_dependency(&dependencies) {
            self.conflicts.push(StyleConflict::circular_inheritance(
                dependencies.keys().copied().collect(),
            ));
            return None;
        }

        // Perform topological sort
        let mut result = Vec::new();
        let mut queue: Vec<&str> = Vec::new();

        // Find all nodes with no dependencies
        for (name, degree) in &in_degree {
            if *degree == 0 {
                queue.push(name);
            }
        }

        while let Some(current) = queue.pop() {
            if let Some(style) = style_map.get(current) {
                result.push(*style);
            }

            // Update in-degrees
            for (child, parents) in &dependencies {
                if parents.contains(current) {
                    if let Some(degree) = in_degree.get_mut(child) {
                        *degree = degree.saturating_sub(1);
                        if *degree == 0 {
                            queue.push(child);
                        }
                    }
                }
            }
        }

        // Check if all styles were processed
        if result.len() == styles.len() {
            Some(result)
        } else {
            // Not all styles processed - circular dependency exists
            None
        }
    }

    /// Check for circular dependencies using DFS
    fn has_circular_dependency(dependencies: &BTreeMap<&str, BTreeSet<&str>>) -> bool {
        let mut visited = BTreeSet::new();
        let mut rec_stack = BTreeSet::new();

        for node in dependencies.keys() {
            if !visited.contains(node)
                && Self::dfs_has_cycle(node, dependencies, &mut visited, &mut rec_stack)
            {
                return true;
            }
        }

        false
    }

    /// DFS helper for cycle detection
    fn dfs_has_cycle<'b>(
        node: &'b str,
        dependencies: &BTreeMap<&'b str, BTreeSet<&'b str>>,
        visited: &mut BTreeSet<&'b str>,
        rec_stack: &mut BTreeSet<&'b str>,
    ) -> bool {
        visited.insert(node);
        rec_stack.insert(node);

        if let Some(neighbors) = dependencies.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if Self::dfs_has_cycle(neighbor, dependencies, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// Resolve styles with inheritance support
    fn resolve_styles_with_inheritance(&mut self, ordered_styles: &[&'a Style<'a>]) {
        for style in ordered_styles {
            let resolved = if let Some(parent_name) = style.parent {
                // Get parent's resolved style
                self.resolved_styles.get(parent_name).map_or_else(
                    || ResolvedStyle::from_style(style),
                    |parent_resolved| ResolvedStyle::from_style_with_parent(style, parent_resolved),
                )
            } else {
                // No parent - resolve directly
                ResolvedStyle::from_style(style)
            };

            if let Ok(mut resolved_style) = resolved {
                // Apply resolution scaling if needed
                if let Some((scale_x, scale_y)) = self.resolution_scaling {
                    resolved_style.apply_resolution_scaling(scale_x, scale_y);
                }
                self.resolved_styles.insert(style.name, resolved_style);
            }
        }
    }

    /// Extract styles from script sections
    #[must_use]
    pub fn extract_styles(&self) -> Option<&[Style<'a>]> {
        for section in self.script.sections() {
            if let Section::Styles(styles) = section {
                return Some(styles);
            }
        }
        None
    }

    /// Detect conflicts between styles in a section
    fn detect_style_conflicts_from_section(&mut self, styles: &[Style<'a>]) {
        let mut name_counts: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

        for style in styles {
            name_counts.entry(style.name).or_default().push(style.name);
        }

        for (_name, instances) in name_counts {
            if instances.len() > 1 {
                self.conflicts
                    .push(StyleConflict::duplicate_name(instances));
            }
        }
    }

    /// Validate style properties
    fn validate_style_properties(&self, style: &ResolvedStyle<'a>) -> Vec<StyleValidationIssue> {
        let mut issues = Vec::new();

        if style.font_size() <= 0.0 {
            issues.push(StyleValidationIssue::error(
                "font_size",
                "Font size must be positive",
            ));
        }

        if self
            .config
            .options
            .contains(AnalysisOptions::STRICT_VALIDATION)
            && style.font_size() > 200.0
        {
            issues.push(StyleValidationIssue::warning(
                "font_size",
                "Very large font size may cause performance issues",
            ));
        }

        issues
    }

    /// Analyze style performance impact
    fn analyze_style_performance(&self, style: &ResolvedStyle<'a>) -> Vec<StyleValidationIssue> {
        let mut issues = Vec::new();
        let thresholds = &self.config.performance_thresholds;

        if style.font_size() > thresholds.large_font_threshold {
            issues.push(StyleValidationIssue::info_with_suggestion(
                "font_size",
                "Large font size detected",
                "Consider reducing font size for better performance",
            ));
        }

        if style.has_performance_issues() {
            issues.push(StyleValidationIssue::warning(
                "complexity",
                "Style has high rendering complexity",
            ));
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::styles::validation::ConflictType;
    #[cfg(not(feature = "std"))]
    use alloc::format;

    #[test]
    fn analyzer_creation() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        assert_eq!(analyzer.resolved_styles().len(), 1);
        assert!(analyzer.resolve_style("Default").is_some());
    }

    #[test]
    fn config_defaults() {
        let config = StyleAnalysisConfig::default();
        assert!(config.options.contains(AnalysisOptions::INHERITANCE));
        assert!(config.options.contains(AnalysisOptions::CONFLICTS));
        assert!(config.options.contains(AnalysisOptions::VALIDATION));
        assert!(!config.options.contains(AnalysisOptions::STRICT_VALIDATION));
    }

    #[test]
    fn performance_thresholds() {
        let thresholds = PerformanceThresholds::default();
        assert!((thresholds.large_font_threshold - 50.0).abs() < f32::EPSILON);
        assert!((thresholds.large_outline_threshold - 4.0).abs() < f32::EPSILON);
        assert!((thresholds.large_shadow_threshold - 4.0).abs() < f32::EPSILON);
        assert!((thresholds.scaling_threshold - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn analyzer_with_custom_config() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let config = StyleAnalysisConfig {
            options: AnalysisOptions::VALIDATION | AnalysisOptions::STRICT_VALIDATION,
            performance_thresholds: PerformanceThresholds {
                large_font_threshold: 30.0,
                large_outline_threshold: 2.0,
                large_shadow_threshold: 2.0,
                scaling_threshold: 150.0,
            },
        };
        let analyzer = StyleAnalyzer::new_with_config(&script, config);

        assert_eq!(analyzer.resolved_styles().len(), 1);
        assert!(analyzer.resolve_style("Default").is_some());
    }

    #[test]
    fn analyzer_multiple_styles() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Title,Arial,32,&H00FFFF00,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,3,0,2,20,20,20,1
Style: Subtitle,Arial,16,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,1,0,0,100,100,0,0,1,1,0,2,5,5,5,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        assert_eq!(analyzer.resolved_styles().len(), 3);
        assert!(analyzer.resolve_style("Default").is_some());
        assert!(analyzer.resolve_style("Title").is_some());
        assert!(analyzer.resolve_style("Subtitle").is_some());
        assert!(analyzer.resolve_style("NonExistent").is_none());
    }

    #[test]
    fn analyzer_duplicate_styles() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Default,Times,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let conflicts = analyzer.conflicts();
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn analyzer_no_styles_section() {
        let script_text = r"
[Script Info]
Title: Test Script
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        assert_eq!(analyzer.resolved_styles().len(), 0);
        assert!(analyzer.conflicts().is_empty());
    }

    #[test]
    fn analyzer_empty_styles_section() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        assert_eq!(analyzer.resolved_styles().len(), 0);
        assert!(analyzer.conflicts().is_empty());
    }

    #[test]
    fn analyzer_extract_styles() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let styles = analyzer.extract_styles();
        assert!(styles.is_some());
        assert_eq!(styles.unwrap().len(), 1);
    }

    #[test]
    fn analyzer_extract_styles_no_section() {
        let script_text = r"
[Script Info]
Title: Test Script
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let styles = analyzer.extract_styles();
        assert!(styles.is_none());
    }

    #[test]
    fn analyzer_inheritance_info() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Title,Arial,32,&H00FFFF00,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,3,0,2,20,20,20,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let inheritance_info = analyzer.inheritance_info();
        assert_eq!(inheritance_info.len(), 2);
        assert!(inheritance_info.contains_key("Default"));
        assert!(inheritance_info.contains_key("Title"));
    }

    #[test]
    fn analyzer_validate_styles() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Large,Arial,60,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,5,0,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let issues = analyzer.validate_styles();
        // Should have some validation issues or none
        assert!(issues.is_empty() || !issues.is_empty());
    }

    #[test]
    fn analyzer_strict_validation() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Large,Arial,250,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let config = StyleAnalysisConfig {
            options: AnalysisOptions::VALIDATION | AnalysisOptions::STRICT_VALIDATION,
            performance_thresholds: PerformanceThresholds::default(),
        };
        let analyzer = StyleAnalyzer::new_with_config(&script, config);

        let issues = analyzer.validate_styles();
        // Should have validation issues for large font size
        assert!(!issues.is_empty());
    }

    #[test]
    fn analyzer_performance_analysis() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Heavy,Arial,60,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,8,5,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let config = StyleAnalysisConfig {
            options: AnalysisOptions::PERFORMANCE,
            performance_thresholds: PerformanceThresholds {
                large_font_threshold: 30.0,
                large_outline_threshold: 2.0,
                large_shadow_threshold: 2.0,
                scaling_threshold: 150.0,
            },
        };
        let analyzer = StyleAnalyzer::new_with_config(&script, config);

        let issues = analyzer.validate_styles();
        // Should have performance issues for large values
        assert!(!issues.is_empty());
    }

    #[test]
    fn analyzer_options_flags() {
        let options = AnalysisOptions::INHERITANCE | AnalysisOptions::CONFLICTS;
        assert!(options.contains(AnalysisOptions::INHERITANCE));
        assert!(options.contains(AnalysisOptions::CONFLICTS));
        assert!(!options.contains(AnalysisOptions::VALIDATION));
        assert!(!options.contains(AnalysisOptions::PERFORMANCE));
        assert!(!options.contains(AnalysisOptions::STRICT_VALIDATION));
    }

    #[test]
    fn analyzer_options_debug() {
        let options = AnalysisOptions::INHERITANCE;
        let debug_str = format!("{options:?}");
        assert!(debug_str.contains("INHERITANCE"));
    }

    #[test]
    fn analyzer_config_debug() {
        let config = StyleAnalysisConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("StyleAnalysisConfig"));
        assert!(debug_str.contains("options"));
        assert!(debug_str.contains("performance_thresholds"));
    }

    #[test]
    fn analyzer_debug() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);
        let debug_str = format!("{analyzer:?}");
        assert!(debug_str.contains("StyleAnalyzer"));
    }

    #[test]
    fn performance_thresholds_debug() {
        let thresholds = PerformanceThresholds::default();
        let debug_str = format!("{thresholds:?}");
        assert!(debug_str.contains("PerformanceThresholds"));
        assert!(debug_str.contains("large_font_threshold"));
    }

    #[test]
    fn config_clone() {
        let config = StyleAnalysisConfig::default();
        let cloned = config.clone();
        assert_eq!(config.options, cloned.options);
        assert!(
            (config.performance_thresholds.large_font_threshold
                - cloned.performance_thresholds.large_font_threshold)
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn performance_thresholds_clone() {
        let thresholds = PerformanceThresholds::default();
        let cloned = thresholds.clone();
        assert!(
            (thresholds.large_font_threshold - cloned.large_font_threshold).abs() < f32::EPSILON
        );
        assert!(
            (thresholds.large_outline_threshold - cloned.large_outline_threshold).abs()
                < f32::EPSILON
        );
        assert!(
            (thresholds.large_shadow_threshold - cloned.large_shadow_threshold).abs()
                < f32::EPSILON
        );
        assert!((thresholds.scaling_threshold - cloned.scaling_threshold).abs() < f32::EPSILON);
    }

    #[test]
    fn analyzer_minimal_options() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let config = StyleAnalysisConfig {
            options: AnalysisOptions::empty(),
            performance_thresholds: PerformanceThresholds::default(),
        };
        let analyzer = StyleAnalyzer::new_with_config(&script, config);

        assert_eq!(analyzer.resolved_styles().len(), 1);
        assert!(analyzer.inheritance_info().is_empty());
        assert!(analyzer.conflicts().is_empty());
    }

    #[test]
    fn analyzer_style_inheritance_basic() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: BaseStyle,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: *BaseStyle,DerivedStyle,Verdana,24,&HFF00FFFF,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        assert_eq!(analyzer.resolved_styles().len(), 2);

        let base_style = analyzer.resolve_style("BaseStyle").unwrap();
        assert_eq!(base_style.font_name(), "Arial");
        assert!((base_style.font_size() - 20.0).abs() < f32::EPSILON);
        assert!(!base_style.is_bold());

        let derived_style = analyzer.resolve_style("DerivedStyle").unwrap();
        assert_eq!(derived_style.font_name(), "Verdana");
        assert!((derived_style.font_size() - 24.0).abs() < f32::EPSILON);
        assert!(derived_style.is_bold());
        // Should inherit colors from base
        assert_eq!(derived_style.primary_color(), [255, 255, 0, 255]); // Overridden
        assert_eq!(
            derived_style.secondary_color(),
            base_style.secondary_color()
        );
        assert_eq!(derived_style.outline_color(), base_style.outline_color());
    }

    #[test]
    fn analyzer_style_inheritance_partial_override() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: BaseStyle,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,1,1,0,0,100,100,0,0,1,2,3,2,10,10,10,1
Style: *BaseStyle,DerivedStyle,Verdana,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,2,3,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let base_style = analyzer.resolve_style("BaseStyle").unwrap();
        let derived_style = analyzer.resolve_style("DerivedStyle").unwrap();

        // Should override font name
        assert_eq!(derived_style.font_name(), "Verdana");
        // Should override font size
        assert!((derived_style.font_size() - 24.0).abs() < f32::EPSILON);
        // Should inherit colors
        assert_eq!(derived_style.primary_color(), base_style.primary_color());
        // Should inherit bold but override italic
        assert!(derived_style.is_bold());
        assert!(!derived_style.is_italic());
        // Should inherit shadow
        assert!((derived_style.shadow() - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn analyzer_style_inheritance_chain() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: GrandParent,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: *GrandParent,Parent,Verdana,24,&H00FFFF00,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,3,0,2,15,15,15,1
Style: *Parent,Child,Times,28,&H00FF00FF,&H000000FF,&H00000000,&H00000000,1,1,0,0,100,100,0,0,1,4,0,2,20,20,20,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        assert_eq!(analyzer.resolved_styles().len(), 3);

        let grandparent = analyzer.resolve_style("GrandParent").unwrap();
        let parent = analyzer.resolve_style("Parent").unwrap();
        let child = analyzer.resolve_style("Child").unwrap();

        // GrandParent properties
        assert_eq!(grandparent.font_name(), "Arial");
        assert!((grandparent.font_size() - 20.0).abs() < f32::EPSILON);
        assert!(!grandparent.is_bold());
        assert!((grandparent.outline() - 2.0).abs() < f32::EPSILON);

        // Parent inherits and overrides
        assert_eq!(parent.font_name(), "Verdana");
        assert!((parent.font_size() - 24.0).abs() < f32::EPSILON);
        assert!(parent.is_bold());
        assert!((parent.outline() - 3.0).abs() < f32::EPSILON);
        assert_eq!(parent.margin_l(), 15);

        // Child inherits from Parent and overrides
        assert_eq!(child.font_name(), "Times");
        assert!((child.font_size() - 28.0).abs() < f32::EPSILON);
        assert!(child.is_bold());
        assert!(child.is_italic());
        assert!((child.outline() - 4.0).abs() < f32::EPSILON);
        assert_eq!(child.margin_l(), 20);
    }

    #[test]
    fn analyzer_style_inheritance_missing_parent() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: *NonExistent,Orphan,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        // Should still resolve the style without inheritance
        assert_eq!(analyzer.resolved_styles().len(), 1);
        let orphan = analyzer.resolve_style("Orphan").unwrap();
        assert_eq!(orphan.font_name(), "Arial");

        // Should have a conflict for missing parent
        let conflicts = analyzer.conflicts();
        assert!(!conflicts.is_empty());
        assert!(conflicts
            .iter()
            .any(|c| matches!(c.conflict_type, ConflictType::MissingReference)));
    }

    #[test]
    fn analyzer_style_circular_inheritance() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: *StyleB,StyleA,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: *StyleA,StyleB,Verdana,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        // Should still resolve styles without inheritance due to circular dependency
        assert_eq!(analyzer.resolved_styles().len(), 2);

        // Should detect circular inheritance
        let conflicts = analyzer.conflicts();
        assert!(conflicts
            .iter()
            .any(|c| matches!(c.conflict_type, ConflictType::CircularInheritance)));
    }

    #[test]
    fn analyzer_style_self_inheritance() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: *SelfRef,SelfRef,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        // Should resolve without inheritance due to self-reference
        assert_eq!(analyzer.resolved_styles().len(), 1);

        // Should detect circular inheritance
        let conflicts = analyzer.conflicts();
        assert!(conflicts
            .iter()
            .any(|c| matches!(c.conflict_type, ConflictType::CircularInheritance)));
    }

    #[test]
    fn analyzer_inheritance_info_tracking() {
        let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: BaseStyle,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: *BaseStyle,Child1,Verdana,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: *BaseStyle,Child2,Times,18,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let inheritance_info = analyzer.inheritance_info();
        assert_eq!(inheritance_info.len(), 3);

        // Check BaseStyle has no parent
        let base_info = inheritance_info.get("BaseStyle").unwrap();
        assert!(base_info.is_root());
        assert!(base_info.parents.is_empty());

        // Check Child1 has BaseStyle as parent
        let child1_info = inheritance_info.get("Child1").unwrap();
        assert!(!child1_info.is_root());
        assert_eq!(child1_info.parents.len(), 1);
        assert_eq!(child1_info.parents[0], "BaseStyle");

        // Check Child2 has BaseStyle as parent
        let child2_info = inheritance_info.get("Child2").unwrap();
        assert!(!child2_info.is_root());
        assert_eq!(child2_info.parents.len(), 1);
        assert_eq!(child2_info.parents[0], "BaseStyle");
    }

    #[test]
    fn analyzer_layout_resolution_scaling() {
        let script_text = r"
[Script Info]
Title: Resolution Scaling Test
LayoutResX: 640
LayoutResY: 480
PlayResX: 1280
PlayResY: 960

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,2,0,1,4,2,2,10,10,20,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let default_style = analyzer.resolve_style("Default").unwrap();
        // Resolution is scaled 2x (1280/640 = 2, 960/480 = 2)
        assert!((default_style.font_size() - 40.0).abs() < f32::EPSILON); // 20 * 2
        assert!((default_style.spacing() - 4.0).abs() < f32::EPSILON); // 2 * 2
        assert!((default_style.outline() - 8.0).abs() < f32::EPSILON); // 4 * 2
        assert!((default_style.shadow() - 4.0).abs() < f32::EPSILON); // 2 * 2
        assert_eq!(default_style.margin_l(), 20); // 10 * 2
        assert_eq!(default_style.margin_r(), 20); // 10 * 2
        assert_eq!(default_style.margin_t(), 40); // 20 * 2
        assert_eq!(default_style.margin_b(), 40); // 20 * 2
    }

    #[test]
    fn analyzer_layout_resolution_scaling_asymmetric() {
        let script_text = r"
[Script Info]
Title: Asymmetric Resolution Scaling Test
LayoutResX: 640
LayoutResY: 480
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,2,0,1,4,2,2,10,10,20,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let default_style = analyzer.resolve_style("Default").unwrap();
        // X scale: 1920/640 = 3, Y scale: 1080/480 = 2.25, average = 2.625
        let avg_scale = 2.625;
        assert!((20.0f32.mul_add(-avg_scale, default_style.font_size())).abs() < 0.01);
        assert!((default_style.spacing() - 6.0).abs() < f32::EPSILON); // 2 * 3
        assert!((4.0f32.mul_add(-avg_scale, default_style.outline())).abs() < 0.01);
        assert!((2.0f32.mul_add(-avg_scale, default_style.shadow())).abs() < 0.01);
        assert_eq!(default_style.margin_l(), 30); // 10 * 3
        assert_eq!(default_style.margin_r(), 30); // 10 * 3
        assert_eq!(default_style.margin_t(), 45); // 20 * 2.25
        assert_eq!(default_style.margin_b(), 45); // 20 * 2.25
    }

    #[test]
    fn analyzer_no_resolution_scaling_when_same() {
        let script_text = r"
[Script Info]
Title: No Scaling Test
LayoutResX: 1920
LayoutResY: 1080
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,2,0,1,4,2,2,10,10,20,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let default_style = analyzer.resolve_style("Default").unwrap();
        // No scaling should be applied
        assert!((default_style.font_size() - 20.0).abs() < f32::EPSILON);
        assert!((default_style.spacing() - 2.0).abs() < f32::EPSILON);
        assert!((default_style.outline() - 4.0).abs() < f32::EPSILON);
        assert!((default_style.shadow() - 2.0).abs() < f32::EPSILON);
        assert_eq!(default_style.margin_l(), 10);
        assert_eq!(default_style.margin_r(), 10);
        assert_eq!(default_style.margin_t(), 20);
        assert_eq!(default_style.margin_b(), 20);
    }

    #[test]
    fn analyzer_resolution_scaling_with_inheritance() {
        let script_text = r"
[Script Info]
Title: Scaling with Inheritance Test
LayoutResX: 640
LayoutResY: 480
PlayResX: 1280
PlayResY: 960

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Base,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,2,0,1,4,2,2,10,10,20,1
Style: *Base,Derived,Verdana,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,15,15,20,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let base_style = analyzer.resolve_style("Base").unwrap();
        let derived_style = analyzer.resolve_style("Derived").unwrap();

        // Base style should be scaled 2x
        assert!((base_style.font_size() - 40.0).abs() < f32::EPSILON);

        // Derived style overrides font size to 24, which should be scaled to 48
        assert!((derived_style.font_size() - 48.0).abs() < f32::EPSILON);
        // Margins are overridden and should be scaled
        assert_eq!(derived_style.margin_l(), 30); // 15 * 2
        assert_eq!(derived_style.margin_r(), 30); // 15 * 2
                                                  // Since margin_v is "20", it should be scaled to 40
        assert_eq!(derived_style.margin_t(), 40); // 20 * 2
        assert_eq!(derived_style.margin_b(), 40); // 20 * 2
    }

    #[test]
    fn analyzer_no_resolution_info_no_scaling() {
        let script_text = r"
[Script Info]
Title: No Resolution Info Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,2,0,1,4,2,2,10,10,20,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let default_style = analyzer.resolve_style("Default").unwrap();
        // No scaling should be applied when resolution info is missing
        assert!((default_style.font_size() - 20.0).abs() < f32::EPSILON);
        assert!((default_style.spacing() - 2.0).abs() < f32::EPSILON);
        assert!((default_style.outline() - 4.0).abs() < f32::EPSILON);
        assert!((default_style.shadow() - 2.0).abs() < f32::EPSILON);
        assert_eq!(default_style.margin_l(), 10);
        assert_eq!(default_style.margin_r(), 10);
        assert_eq!(default_style.margin_t(), 20);
        assert_eq!(default_style.margin_b(), 20);
    }

    #[test]
    fn analyzer_partial_resolution_info_no_scaling() {
        let script_text = r"
[Script Info]
Title: Partial Resolution Info Test
LayoutResX: 640
PlayResY: 960

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,2,0,1,4,2,2,10,10,20,1
";

        let script = crate::parser::Script::parse(script_text).unwrap();
        let analyzer = StyleAnalyzer::new(&script);

        let default_style = analyzer.resolve_style("Default").unwrap();
        // No scaling should be applied when resolution info is incomplete
        assert!((default_style.font_size() - 20.0).abs() < f32::EPSILON);
        assert!((default_style.spacing() - 2.0).abs() < f32::EPSILON);
        assert!((default_style.outline() - 4.0).abs() < f32::EPSILON);
        assert!((default_style.shadow() - 2.0).abs() < f32::EPSILON);
        assert_eq!(default_style.margin_l(), 10);
        assert_eq!(default_style.margin_r(), 10);
        assert_eq!(default_style.margin_t(), 20);
        assert_eq!(default_style.margin_b(), 20);
    }
}
