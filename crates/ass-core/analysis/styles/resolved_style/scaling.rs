//! Resolution scaling and rendering-complexity computation for `ResolvedStyle`.
//!
//! Implements `apply_resolution_scaling` for adjusting coordinate-based
//! properties and the `calculate_complexity` heuristic used to score rendering
//! cost.

use super::{ResolvedStyle, TextFormatting};

impl ResolvedStyle<'_> {
    /// Apply resolution scaling to coordinate-based properties
    ///
    /// Scales font size, spacing, outline, shadow, and margins based on the
    /// resolution difference between layout and play resolutions.
    ///
    /// # Arguments
    ///
    /// * `scale_x` - Horizontal scaling factor (`PlayResX` / `LayoutResX`)
    /// * `scale_y` - Vertical scaling factor (`PlayResY` / `LayoutResY`)
    pub fn apply_resolution_scaling(&mut self, scale_x: f32, scale_y: f32) {
        // Scale font size (use average of X/Y scaling to maintain aspect ratio)
        let avg_scale = (scale_x + scale_y) / 2.0;
        self.font_size *= avg_scale;

        // Scale spacing (horizontal)
        self.spacing *= scale_x;

        // Scale outline and shadow (use average scaling)
        self.outline *= avg_scale;
        self.shadow *= avg_scale;

        // Scale margins
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            self.margin_l = (f32::from(self.margin_l) * scale_x) as u16;
            self.margin_r = (f32::from(self.margin_r) * scale_x) as u16;
            self.margin_t = (f32::from(self.margin_t) * scale_y) as u16;
            self.margin_b = (f32::from(self.margin_b) * scale_y) as u16;
        }

        // Recalculate complexity score after scaling
        self.complexity_score = Self::calculate_complexity(self);
    }

    /// Calculate rendering complexity score
    pub(super) fn calculate_complexity(style: &Self) -> u8 {
        const EPSILON: f32 = 0.001;
        let mut score = 0u8;

        if style.font_size > 72.0 {
            score += 20;
        } else if style.font_size > 48.0 {
            score += 10;
        }

        if style.outline > 4.0 {
            score += 15;
        } else if style.outline > 2.0 {
            score += 8;
        }

        if style.shadow > 3.0 {
            score += 10;
        } else if style.shadow > 1.0 {
            score += 5;
        }

        if (style.scale_x - 100.0).abs() > EPSILON || (style.scale_y - 100.0).abs() > EPSILON {
            score += 10;
        }

        if style.angle.abs() > EPSILON {
            score += 15;
        }

        if style.formatting.contains(TextFormatting::BOLD) {
            score += 2;
        }
        if style.formatting.contains(TextFormatting::ITALIC) {
            score += 2;
        }
        if style
            .formatting
            .intersects(TextFormatting::UNDERLINE | TextFormatting::STRIKE_OUT)
        {
            score += 5;
        }

        score.min(100)
    }
}
