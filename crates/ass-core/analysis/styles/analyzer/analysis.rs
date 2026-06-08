//! Resolution scaling and inheritance-aware style resolution.
//!
//! Computes the layout-to-play resolution scaling factor and resolves every
//! style in the script, applying inheritance and scaling as configured.

use super::{AnalysisOptions, StyleAnalyzer};
use crate::{
    analysis::styles::resolved_style::ResolvedStyle,
    parser::{Section, Style},
};

impl<'a> StyleAnalyzer<'a> {
    /// Calculate resolution scaling factor from script info
    pub(super) fn calculate_resolution_scaling(&mut self) {
        // Find ScriptInfo section
        for section in self.script.sections() {
            if let Section::ScriptInfo(script_info) = section {
                let layout_res = script_info.layout_resolution();
                let play_res = script_info.play_resolution();

                if let (Some((layout_x, layout_y)), Some((play_x, play_y))) = (layout_res, play_res)
                {
                    // Only apply scaling if resolutions differ
                    if layout_x != play_x || layout_y != play_y {
                        // Scale FROM layout TO play resolution
                        // If LayoutRes > PlayRes, we scale down (e.g., 1920→640 = 0.333)
                        // If LayoutRes < PlayRes, we scale up (e.g., 640→1920 = 3.0)
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
    pub(super) fn analyze_all_styles(&mut self) {
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
}
