//! Layout context for positioning calculations

use super::{Alignment, TextMetrics};

/// Layout context for positioning calculations
pub struct LayoutContext {
    pub screen_width: f32,
    pub screen_height: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub margin_vertical: f32,
    pub play_res_x: f32,
    pub play_res_y: f32,
}

impl LayoutContext {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_width,
            screen_height,
            margin_left: 0.0,
            margin_right: 0.0,
            margin_vertical: 0.0,
            play_res_x: screen_width,
            play_res_y: screen_height,
        }
    }

    /// Calculate position for text with given alignment and metrics
    pub fn calculate_position(&self, alignment: Alignment, metrics: &TextMetrics) -> (f32, f32) {
        // In ASS/SSA, the position is where the text glyph paths start
        // For proper centering, we need to account for text dimensions

        // Calculate horizontal position (left edge of text bounding box)
        let x = match alignment {
            Alignment::BottomLeft | Alignment::MiddleLeft | Alignment::TopLeft => {
                // Left alignment - text starts at left margin
                self.margin_left
            }
            Alignment::BottomCenter | Alignment::Center | Alignment::TopCenter => {
                // Center alignment - center the text horizontally
                // This should put the left edge at center minus half width
                (self.screen_width / 2.0) - (metrics.width / 2.0)
            }
            Alignment::BottomRight | Alignment::MiddleRight | Alignment::TopRight => {
                // Right alignment - text ends at right margin
                self.screen_width - self.margin_right - metrics.width
            }
        };

        // Calculate vertical position (top edge of text bounding box)
        // The glyphs are drawn from their top-left corner typically
        let y = match alignment {
            Alignment::TopLeft | Alignment::TopCenter | Alignment::TopRight => {
                // Top alignment - text at top margin
                self.margin_vertical
            }
            Alignment::MiddleLeft | Alignment::Center | Alignment::MiddleRight => {
                // Middle alignment - center vertically
                (self.screen_height / 2.0) - (metrics.height / 2.0)
            }
            Alignment::BottomLeft | Alignment::BottomCenter | Alignment::BottomRight => {
                // Bottom alignment - text bottom at bottom margin
                self.screen_height - self.margin_vertical - metrics.height
            }
        };

        (x, y)
    }

    /// Apply PlayResX/PlayResY scaling
    pub fn apply_play_res_scaling(&self, x: f32, y: f32) -> (f32, f32) {
        let scale_x = self.screen_width / self.play_res_x;
        let scale_y = self.screen_height / self.play_res_y;
        (x * scale_x, y * scale_y)
    }
}
