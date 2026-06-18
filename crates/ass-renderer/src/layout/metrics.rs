//! Text metrics for layout calculations

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
