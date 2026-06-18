//! Data structures for visual comparison and rendering debug info

/// Visual debug info for rendering comparison
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RenderDebugInfo {
    /// Font size as calculated
    pub calculated_font_size: f32,
    /// Font size after PlayRes scaling
    pub scaled_font_size: f32,
    /// PlayResX value
    pub play_res_x: u32,
    /// PlayResY value
    pub play_res_y: u32,
    /// Render resolution
    pub render_width: u32,
    pub render_height: u32,
    /// Color in BBGGRR format (as hex string)
    pub color_bbggrr: String,
    /// Converted RGBA values
    pub color_rgba: [u8; 4],
    /// Font metrics
    pub font_metrics: FontMetricsDebug,
    /// Text bounding box
    pub text_bbox: BoundingBoxDebug,
    /// Animation progress
    pub animation_progress: Option<f32>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FontMetricsDebug {
    pub ascender: f32,
    pub descender: f32,
    pub line_gap: f32,
    pub units_per_em: f32,
    pub scale_factor: f32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct BoundingBoxDebug {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug)]
pub struct ComparisonResult {
    pub average_difference: f32,
    pub max_difference: f32,
    pub different_pixels: usize,
    pub total_pixels: usize,
    pub pixel_differences: Vec<PixelDifference>,
}

#[derive(Debug)]
pub struct PixelDifference {
    pub x: u32,
    pub y: u32,
    pub our_color: [u8; 4],
    pub libass_color: [u8; 4],
    pub difference: f32,
}
