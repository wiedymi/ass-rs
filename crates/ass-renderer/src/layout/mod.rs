//! Layout system for proper subtitle positioning

pub mod positioning;

#[cfg(feature = "nostd")]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
#[cfg(not(feature = "nostd"))]
use std::{string::String, vec::Vec};

pub use positioning::{convert_ssa_alignment, scale_coordinates, BoundingBox, PositionInfo};

/// Alignment types for subtitle positioning
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    // ASS alignments (numpad layout)
    /// Bottom left alignment
    BottomLeft = 1,
    /// Bottom center alignment
    BottomCenter = 2,
    /// Bottom right alignment
    BottomRight = 3,
    /// Middle left alignment
    MiddleLeft = 4,
    /// Center alignment
    Center = 5,
    /// Middle right alignment
    MiddleRight = 6,
    /// Top left alignment
    TopLeft = 7,
    /// Top center alignment
    TopCenter = 8,
    /// Top right alignment
    TopRight = 9,
}

impl Alignment {
    /// Convert from numeric alignment value
    pub fn from_value(value: u8) -> Self {
        match value {
            1 => Self::BottomLeft,
            2 => Self::BottomCenter,
            3 => Self::BottomRight,
            4 => Self::MiddleLeft,
            5 => Self::Center,
            6 => Self::MiddleRight,
            7 => Self::TopLeft,
            8 => Self::TopCenter,
            9 => Self::TopRight,
            _ => Self::BottomCenter, // Default
        }
    }

    /// Check if this is a legacy SSA alignment value
    pub fn from_ssa_legacy(value: u8) -> Self {
        // SSA legacy: 1=left, 2=center, 3=right
        // +0=sub(bottom), +4=title(middle), +8=top, +128=mid-title
        match value & 3 {
            1 => {
                // Left alignment
                match value & 12 {
                    0 => Self::BottomLeft,
                    4 => Self::MiddleLeft,
                    8 | 12 => Self::TopLeft,
                    _ => Self::BottomLeft,
                }
            }
            2 => {
                // Center alignment
                match value & 12 {
                    0 => Self::BottomCenter,
                    4 => Self::Center,
                    8 | 12 => Self::TopCenter,
                    _ => Self::BottomCenter,
                }
            }
            3 => {
                // Right alignment
                match value & 12 {
                    0 => Self::BottomRight,
                    4 => Self::MiddleRight,
                    8 | 12 => Self::TopRight,
                    _ => Self::BottomRight,
                }
            }
            _ => Self::BottomCenter,
        }
    }
}

/// Text metrics for layout calculations
#[derive(Debug, Clone)]
pub struct TextMetrics {
    pub width: f32,
    pub height: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub baseline: f32,
}

impl TextMetrics {
    /// Create from shaped text
    pub fn from_shaped(shaped: &crate::pipeline::shaping::ShapedText) -> Self {
        // ShapedText has width, height, and baseline
        // We'll estimate ascent/descent from height and baseline
        let ascent = shaped.baseline;
        let descent = shaped.height - shaped.baseline;

        Self {
            width: shaped.width,
            height: shaped.height,
            ascent,
            descent,
            line_gap: 0.0, // Not available in ShapedText
            baseline: shaped.baseline,
        }
    }

    /// Create with estimated values
    pub fn estimated(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            ascent: height * 0.8,
            descent: height * 0.2,
            line_gap: height * 0.1,
            baseline: height * 0.8,
        }
    }
}

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

/// Multi-line text layout
pub struct MultiLineLayout {
    pub lines: Vec<LineLayout>,
    pub total_width: f32,
    pub total_height: f32,
}

/// Single line layout information
pub struct LineLayout {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
}

impl MultiLineLayout {
    /// Create layout for multi-line text
    pub fn new(
        text: &str,
        _context: &LayoutContext,
        alignment: Alignment,
        line_spacing: f32,
    ) -> Self {
        // Text has already been processed by ass-core, so \N is now \n
        let lines: Vec<&str> = text.split('\n').collect();
        let mut line_layouts = Vec::new();
        let mut total_height = 0.0;
        let mut max_width = 0.0;

        // TODO: Actual text shaping would go here
        // For now, use estimates
        let line_height = 50.0; // Placeholder

        for (i, line) in lines.iter().enumerate() {
            let line_width = line.len() as f32 * 20.0; // Placeholder
            max_width = f32::max(max_width, line_width);

            let y_offset = i as f32 * (line_height + line_spacing);

            line_layouts.push(LineLayout {
                text: line.to_string(),
                x: 0.0, // Will be adjusted based on alignment
                y: y_offset,
                width: line_width,
                height: line_height,
                baseline: line_height * 0.8,
            });

            total_height = y_offset + line_height;
        }

        // Adjust line positions based on alignment
        for line in &mut line_layouts {
            line.x = match alignment {
                Alignment::BottomLeft | Alignment::MiddleLeft | Alignment::TopLeft => 0.0,
                Alignment::BottomCenter | Alignment::Center | Alignment::TopCenter => {
                    (max_width - line.width) / 2.0
                }
                Alignment::BottomRight | Alignment::MiddleRight | Alignment::TopRight => {
                    max_width - line.width
                }
            };
        }

        Self {
            lines: line_layouts,
            total_width: max_width,
            total_height,
        }
    }
}
