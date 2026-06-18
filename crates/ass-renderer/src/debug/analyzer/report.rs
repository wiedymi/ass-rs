use crate::Frame;

#[cfg(feature = "nostd")]
use alloc::vec::Vec;

use super::TextArea;

/// Detailed analysis report for a rendered frame
#[derive(Debug, Clone)]
pub struct AnalysisReport {
    /// Frame width in pixels
    pub width: u32,
    /// Frame height in pixels
    pub height: u32,
    /// Total number of pixels in the frame
    pub total_pixels: usize,
    /// Number of non-transparent pixels
    pub non_transparent_pixels: usize,
    /// Percentage of transparent pixels
    pub transparency_percentage: f32,
    /// Histogram of pixel color distribution
    pub pixel_histogram: PixelHistogram,
    /// Detected regions in the frame
    pub regions: Vec<Region>,
    /// Detected text areas in the frame
    pub text_areas: Vec<TextArea>,
    /// Dominant color in the frame [R, G, B, A]
    pub dominant_color: [u8; 4],
    /// Estimated contrast ratio
    pub contrast_ratio: f32,
}

impl AnalysisReport {
    pub(super) fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            total_pixels: (width * height) as usize,
            non_transparent_pixels: 0,
            transparency_percentage: 0.0,
            pixel_histogram: PixelHistogram::default(),
            regions: Vec::new(),
            text_areas: Vec::new(),
            dominant_color: [0, 0, 0, 0],
            contrast_ratio: 0.0,
        }
    }

    pub(super) fn calculate_statistics(&mut self, frame: &Frame) {
        self.non_transparent_pixels = self.pixel_histogram.non_transparent_count;
        self.transparency_percentage = ((self.total_pixels - self.non_transparent_pixels) as f32
            / self.total_pixels as f32)
            * 100.0;

        // Calculate dominant color
        self.dominant_color = self.calculate_dominant_color();

        // Calculate contrast ratio (simplified)
        self.contrast_ratio = self.calculate_contrast_ratio(frame);
    }

    fn calculate_dominant_color(&self) -> [u8; 4] {
        let hist = &self.pixel_histogram;

        // Find the peak in each channel
        let mut dominant = [0u8; 4];

        for (i, channel) in [&hist.red, &hist.green, &hist.blue, &hist.alpha]
            .iter()
            .enumerate()
        {
            let mut max_count = 0;
            let mut max_value = 0;

            for (value, &count) in channel.iter().enumerate() {
                if count > max_count {
                    max_count = count;
                    max_value = value;
                }
            }

            dominant[i] = max_value as u8;
        }

        dominant
    }

    fn calculate_contrast_ratio(&self, _frame: &Frame) -> f32 {
        // Simplified contrast calculation
        // In a real implementation, this would calculate Weber contrast or Michelson contrast
        if self.pixel_histogram.white_pixels > 0 && self.pixel_histogram.black_pixels > 0 {
            let white_ratio =
                self.pixel_histogram.white_pixels as f32 / self.non_transparent_pixels as f32;
            let black_ratio =
                self.pixel_histogram.black_pixels as f32 / self.non_transparent_pixels as f32;

            // High contrast if we have both bright and dark pixels
            (white_ratio * black_ratio * 100.0).min(1.0)
        } else {
            0.0
        }
    }

    pub fn print_summary(&self) {
        println!("\n╔════════════════════════════════════════╗");
        println!("║         Frame Analysis Report          ║");
        println!("╚════════════════════════════════════════╝");

        println!("\n📊 Basic Statistics:");
        println!(
            "  • Resolution: {width}x{height}",
            width = self.width,
            height = self.height
        );
        println!(
            "  • Total Pixels: {total_pixels}",
            total_pixels = self.total_pixels
        );
        println!(
            "  • Non-transparent: {} ({:.1}%)",
            self.non_transparent_pixels,
            (self.non_transparent_pixels as f32 / self.total_pixels as f32) * 100.0
        );
        println!(
            "  • Transparency: {transparency:.1}%",
            transparency = self.transparency_percentage
        );

        println!("\n🎨 Color Distribution:");
        println!(
            "  • White pixels: {white_pixels}",
            white_pixels = self.pixel_histogram.white_pixels
        );
        println!(
            "  • Black pixels: {black_pixels}",
            black_pixels = self.pixel_histogram.black_pixels
        );
        println!(
            "  • Gray pixels: {gray_pixels}",
            gray_pixels = self.pixel_histogram.gray_pixels
        );
        println!(
            "  • Colored pixels: {colored_pixels}",
            colored_pixels = self.pixel_histogram.colored_pixels
        );
        println!(
            "  • Dominant color: RGBA({r}, {g}, {b}, {a})",
            r = self.dominant_color[0],
            g = self.dominant_color[1],
            b = self.dominant_color[2],
            a = self.dominant_color[3]
        );

        println!("\n🔍 Region Detection:");
        println!(
            "  • Detected regions: {regions_count}",
            regions_count = self.regions.len()
        );
        for (i, region) in self.regions.iter().enumerate() {
            println!(
                "    {}. Box: ({}, {}) to ({}, {}) | Pixels: {} | Color: RGBA({}, {}, {}, {})",
                i + 1,
                region.min_x,
                region.min_y,
                region.max_x,
                region.max_y,
                region.pixel_count,
                region.avg_color[0],
                region.avg_color[1],
                region.avg_color[2],
                region.avg_color[3]
            );
        }

        println!("\n📝 Text Detection:");
        if self.text_areas.is_empty() {
            println!("  • No text areas detected");
        } else {
            println!(
                "  • Detected text areas: {text_areas_count}",
                text_areas_count = self.text_areas.len()
            );
            for (i, area) in self.text_areas.iter().enumerate() {
                println!(
                    "    {}. Position: ({}, {}) | Size: {}x{} | Font: ~{}px | Confidence: {:.1}%",
                    i + 1,
                    area.x,
                    area.y,
                    area.width,
                    area.height,
                    area.estimated_font_size,
                    area.confidence * 100.0
                );
            }
        }

        println!("\n⚡ Performance Indicators:");
        println!(
            "  • Contrast ratio: {contrast_ratio:.2}",
            contrast_ratio = self.contrast_ratio
        );
        println!(
            "  • Memory usage: {memory_kb} KB",
            memory_kb = (self.total_pixels * 4) / 1024
        );

        println!("\n═══════════════════════════════════════");
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{
    "resolution": {{ "width": {}, "height": {} }},
    "pixels": {{
        "total": {},
        "non_transparent": {},
        "transparency_percentage": {:.2}
    }},
    "color_distribution": {{
        "white": {},
        "black": {},
        "gray": {},
        "colored": {}
    }},
    "regions": {},
    "text_areas": {},
    "dominant_color": [{}, {}, {}, {}],
    "contrast_ratio": {:.2}
}}"#,
            self.width,
            self.height,
            self.total_pixels,
            self.non_transparent_pixels,
            self.transparency_percentage,
            self.pixel_histogram.white_pixels,
            self.pixel_histogram.black_pixels,
            self.pixel_histogram.gray_pixels,
            self.pixel_histogram.colored_pixels,
            self.regions.len(),
            self.text_areas.len(),
            self.dominant_color[0],
            self.dominant_color[1],
            self.dominant_color[2],
            self.dominant_color[3],
            self.contrast_ratio
        )
    }
}

#[derive(Debug, Clone)]
pub struct PixelHistogram {
    pub red: [usize; 256],
    pub green: [usize; 256],
    pub blue: [usize; 256],
    pub alpha: [usize; 256],
    pub non_transparent_count: usize,
    pub white_pixels: usize,
    pub black_pixels: usize,
    pub gray_pixels: usize,
    pub colored_pixels: usize,
}

impl Default for PixelHistogram {
    fn default() -> Self {
        Self {
            red: [0; 256],
            green: [0; 256],
            blue: [0; 256],
            alpha: [0; 256],
            non_transparent_count: 0,
            white_pixels: 0,
            black_pixels: 0,
            gray_pixels: 0,
            colored_pixels: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Region {
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
    pub pixel_count: usize,
    pub avg_color: [u8; 4],
}
