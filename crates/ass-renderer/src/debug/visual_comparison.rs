//! Visual comparison and debugging tools for ASS rendering

use crate::utils::RenderError;
use ass_core::Script;
use tiny_skia::{Color, Paint, Pixmap, Stroke, Transform};

#[cfg(not(feature = "nostd"))]
use std::{fs, path::Path};

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

/// Visual comparison renderer
pub struct VisualComparison {
    width: u32,
    height: u32,
    debug_enabled: bool,
    debug_info: Vec<RenderDebugInfo>,
}

impl VisualComparison {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            debug_enabled: true,
            debug_info: Vec::new(),
        }
    }

    /// Enable/disable debug mode
    pub fn set_debug(&mut self, enabled: bool) {
        self.debug_enabled = enabled;
    }

    /// Render with debug overlay
    pub fn render_with_debug(
        &mut self,
        script: &Script,
        _time_ms: u32,
    ) -> Result<Pixmap, RenderError> {
        // Clear debug info
        self.debug_info.clear();

        // Get script info
        use ass_core::parser::ast::SectionType;
        let play_res_x = if let Some(ass_core::parser::ast::Section::ScriptInfo(info)) =
            script.find_section(SectionType::ScriptInfo)
        {
            info.get_field("PlayResX")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(384)
        } else {
            384
        };
        let play_res_y = if let Some(ass_core::parser::ast::Section::ScriptInfo(info)) =
            script.find_section(SectionType::ScriptInfo)
        {
            info.get_field("PlayResY")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(288)
        } else {
            288
        };

        // Create a basic pixmap for now - full rendering would require more setup
        let mut pixmap = Pixmap::new(self.width, self.height).ok_or(RenderError::InvalidPixmap)?;

        if self.debug_enabled {
            // Add debug overlay
            self.draw_debug_overlay(&mut pixmap, &self.debug_info)?;

            // Add grid for alignment reference
            self.draw_alignment_grid(&mut pixmap, play_res_x, play_res_y)?;

            // Add color reference
            self.draw_color_reference(&mut pixmap)?;
        }

        Ok(pixmap)
    }

    /// Draw debug overlay with rendering info
    fn draw_debug_overlay(
        &self,
        pixmap: &mut Pixmap,
        debug_info: &[RenderDebugInfo],
    ) -> Result<(), RenderError> {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(255, 255, 0, 180)); // Yellow for debug text

        // Draw debug info in top-left corner
        let mut y_offset = 20.0;
        for (i, info) in debug_info.iter().enumerate() {
            let text = format!(
                "Event {}: Font: {:.1}pt (scaled: {:.1}pt), Color: {} -> RGBA({},{},{},{})",
                i,
                info.calculated_font_size,
                info.scaled_font_size,
                info.color_bbggrr,
                info.color_rgba[0],
                info.color_rgba[1],
                info.color_rgba[2],
                info.color_rgba[3],
            );

            // Draw text background for readability
            let mut bg_paint = Paint::default();
            bg_paint.set_color(Color::from_rgba8(0, 0, 0, 180));
            pixmap.fill_rect(
                tiny_skia::Rect::from_xywh(5.0, y_offset - 15.0, text.len() as f32 * 7.0, 20.0)
                    .unwrap(),
                &bg_paint,
                Transform::identity(),
                None,
            );

            // Would draw actual text here with a proper text renderer
            // For now, just indicate where it would be
            y_offset += 25.0;
        }

        Ok(())
    }

    /// Draw alignment grid for reference
    fn draw_alignment_grid(
        &self,
        pixmap: &mut Pixmap,
        _play_res_x: u32,
        _play_res_y: u32,
    ) -> Result<(), RenderError> {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(100, 100, 100, 50)); // Semi-transparent gray

        let width = pixmap.width() as f32;
        let height = pixmap.height() as f32;

        // Draw 3x3 grid for alignment positions
        let h_third = width / 3.0;
        let v_third = height / 3.0;

        let stroke = Stroke {
            width: 1.0,
            ..Default::default()
        };

        // Vertical lines
        for i in 1..3 {
            let x = h_third * i as f32;
            if let Some(rect) = tiny_skia::Rect::from_xywh(x, 0.0, 1.0, height) {
                let path = tiny_skia::PathBuilder::from_rect(rect);
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }

        // Horizontal lines
        for i in 1..3 {
            let y = v_third * i as f32;
            if let Some(rect) = tiny_skia::Rect::from_xywh(0.0, y, width, 1.0) {
                let path = tiny_skia::PathBuilder::from_rect(rect);
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }

        Ok(())
    }

    /// Draw color reference swatches
    fn draw_color_reference(&self, pixmap: &mut Pixmap) -> Result<(), RenderError> {
        // Draw common ASS colors for reference
        let colors = [
            ("White", [255, 255, 255, 255]),
            ("Cyan", [255, 255, 0, 255]),   // In BBGGRR: &H00FFFF&
            ("Yellow", [0, 255, 255, 255]), // In BBGGRR: &H00FFFF&
            ("Red", [0, 0, 255, 255]),      // In BBGGRR: &H0000FF&
            ("Blue", [255, 0, 0, 255]),     // In BBGGRR: &HFF0000&
        ];

        let mut x_offset = pixmap.width() as f32 - 250.0;
        let y_offset = 10.0;

        for (_name, rgba) in colors.iter() {
            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]));

            // Draw color swatch
            pixmap.fill_rect(
                tiny_skia::Rect::from_xywh(x_offset, y_offset, 40.0, 20.0).unwrap(),
                &paint,
                Transform::identity(),
                None,
            );

            x_offset += 45.0;
        }

        Ok(())
    }

    /// Export debug info to file
    #[cfg(all(not(feature = "nostd"), feature = "serde"))]
    pub fn export_debug_info(&self, path: &Path) -> Result<(), RenderError> {
        let json = serde_json::to_string_pretty(&self.debug_info)
            .map_err(|e| RenderError::BackendError(format!("Serialization failed: {}", e)))?;
        fs::write(path, json).map_err(|e| RenderError::IOError(e.to_string()))?;
        Ok(())
    }

    /// Compare with libass output
    pub fn compare_with_libass(
        &self,
        our_output: &Pixmap,
        libass_output: &Pixmap,
    ) -> ComparisonResult {
        let mut differences = Vec::new();
        let mut total_diff = 0.0;
        let mut max_diff = 0.0;

        // Compare pixel by pixel
        for y in 0..our_output.height().min(libass_output.height()) {
            for x in 0..our_output.width().min(libass_output.width()) {
                let our_pixel = our_output.pixel(x, y).unwrap();
                let lib_pixel = libass_output.pixel(x, y).unwrap();

                let r_diff = (our_pixel.red() as i32 - lib_pixel.red() as i32).abs() as f32;
                let g_diff = (our_pixel.green() as i32 - lib_pixel.green() as i32).abs() as f32;
                let b_diff = (our_pixel.blue() as i32 - lib_pixel.blue() as i32).abs() as f32;
                let a_diff = (our_pixel.alpha() as i32 - lib_pixel.alpha() as i32).abs() as f32;

                let pixel_diff = (r_diff + g_diff + b_diff + a_diff) / 4.0;

                if pixel_diff > 10.0 {
                    differences.push(PixelDifference {
                        x,
                        y,
                        our_color: [
                            our_pixel.red(),
                            our_pixel.green(),
                            our_pixel.blue(),
                            our_pixel.alpha(),
                        ],
                        libass_color: [
                            lib_pixel.red(),
                            lib_pixel.green(),
                            lib_pixel.blue(),
                            lib_pixel.alpha(),
                        ],
                        difference: pixel_diff,
                    });
                }

                total_diff += pixel_diff;
                max_diff = if pixel_diff > max_diff {
                    pixel_diff
                } else {
                    max_diff
                };
            }
        }

        let pixel_count = (our_output.width() * our_output.height()) as f32;

        ComparisonResult {
            average_difference: total_diff / pixel_count,
            max_difference: max_diff,
            different_pixels: differences.len(),
            total_pixels: pixel_count as usize,
            pixel_differences: differences,
        }
    }
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

/// Create side-by-side comparison image
pub fn create_comparison_image(
    our_output: &Pixmap,
    libass_output: &Pixmap,
    diff_map: Option<&Pixmap>,
) -> Result<Pixmap, RenderError> {
    let width = our_output.width() + libass_output.width() + diff_map.map_or(0, |d| d.width());
    let height = our_output.height().max(libass_output.height());

    let mut comparison = Pixmap::new(width, height).ok_or_else(|| RenderError::InvalidPixmap)?;

    // Copy our output to left side
    comparison.draw_pixmap(
        0,
        0,
        our_output.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    // Copy libass output to middle
    comparison.draw_pixmap(
        our_output.width() as i32,
        0,
        libass_output.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    // Copy diff map to right side if provided
    if let Some(diff) = diff_map {
        comparison.draw_pixmap(
            (our_output.width() + libass_output.width()) as i32,
            0,
            diff.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            Transform::identity(),
            None,
        );
    }

    Ok(comparison)
}
