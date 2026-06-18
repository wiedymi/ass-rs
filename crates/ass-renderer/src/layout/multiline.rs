//! Multi-line text layout

#[cfg(feature = "nostd")]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
#[cfg(not(feature = "nostd"))]
use std::{string::String, vec::Vec};

use super::{Alignment, LayoutContext};

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
