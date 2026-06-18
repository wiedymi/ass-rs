//! Text shaping module using rustybuzz

mod font_metrics;
mod font_select;
mod glyph_renderer;
mod shape;

pub use font_metrics::FontMetrics;
pub use font_select::find_font_for_text;
pub use glyph_renderer::GlyphRenderer;
pub use shape::{shape_text, shape_text_cached, shape_text_with_style};

#[cfg(feature = "nostd")]
use alloc::vec::Vec;
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

/// Shaped glyph representation
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    /// Glyph ID in the font
    pub glyph_id: u32,
    /// X position
    pub x_position: f32,
    /// Y position
    pub y_position: f32,
    /// X offset
    pub x_offset: f32,
    /// Y offset
    pub y_offset: f32,
    /// Horizontal advance
    pub x_advance: f32,
    /// Vertical advance
    pub y_advance: f32,
    /// Cluster index in original text
    pub cluster: u32,
}

/// Shaped text result
#[derive(Debug, Clone)]
pub struct ShapedText {
    /// Shaped glyphs
    pub glyphs: Vec<ShapedGlyph>,
    /// Total width
    pub width: f32,
    /// Total height (line height)
    pub height: f32,
    /// Baseline position
    pub baseline: f32,
    /// Font size used for shaping
    pub font_size: f32,
    /// Font ascent (for underline/strikeout positioning)
    pub ascent: f32,
    /// Font descent (for underline/strikeout positioning)
    pub descent: f32,
    /// Left edge of the inked glyph outlines (the first glyph's left side bearing).
    /// libass measures line widths from ink extents, not advances, so wrapping uses
    /// `width - ink_min - (width - ink_max)` to match its break points.
    pub ink_min: f32,
    /// Right edge of the inked glyph outlines.
    pub ink_max: f32,
}

impl ShapedText {
    /// Get the total horizontal advance of the shaped text
    pub fn total_advance(&self) -> Option<f32> {
        if self.glyphs.is_empty() {
            return Some(0.0);
        }

        // Calculate the total advance by summing up all glyph advances
        let mut total = 0.0;
        for glyph in &self.glyphs {
            total += glyph.x_advance;
        }
        Some(total)
    }
}
