//! Frame analysis tools for detailed debugging and profiling
//!
//! This module provides in-depth analysis capabilities for rendered frames,
//! including pixel statistics, region detection, and text analysis.

use crate::Frame;

#[cfg(feature = "nostd")]
use alloc::vec::Vec;

mod report;

pub use report::{AnalysisReport, PixelHistogram, Region};

/// Frame analyzer for detailed text-based debugging
pub struct FrameAnalyzer {
    enable_pixel_histogram: bool,
    enable_region_analysis: bool,
    enable_text_detection: bool,
}

impl Default for FrameAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameAnalyzer {
    /// Create a new frame analyzer with all analysis features enabled
    pub fn new() -> Self {
        Self {
            enable_pixel_histogram: true,
            enable_region_analysis: true,
            enable_text_detection: true,
        }
    }

    /// Analyze a frame and generate a detailed report
    pub fn analyze(&self, frame: &Frame) -> AnalysisReport {
        let mut report = AnalysisReport::new(frame.width(), frame.height());

        if self.enable_pixel_histogram {
            report.pixel_histogram = self.calculate_pixel_histogram(frame);
        }

        if self.enable_region_analysis {
            report.regions = self.detect_regions(frame);
        }

        if self.enable_text_detection {
            report.text_areas = self.detect_text_areas(frame);
        }

        report.calculate_statistics(frame);
        report
    }

    fn calculate_pixel_histogram(&self, frame: &Frame) -> PixelHistogram {
        let pixels = frame.pixels();
        let mut histogram = PixelHistogram::default();

        for chunk in pixels.chunks(4) {
            if chunk.len() == 4 {
                let r = chunk[0];
                let g = chunk[1];
                let b = chunk[2];
                let a = chunk[3];

                histogram.red[r as usize] += 1;
                histogram.green[g as usize] += 1;
                histogram.blue[b as usize] += 1;
                histogram.alpha[a as usize] += 1;

                if a > 0 {
                    histogram.non_transparent_count += 1;

                    // Classify pixel
                    if r == g && g == b {
                        if r > 200 {
                            histogram.white_pixels += 1;
                        } else if r < 50 {
                            histogram.black_pixels += 1;
                        } else {
                            histogram.gray_pixels += 1;
                        }
                    } else {
                        histogram.colored_pixels += 1;
                    }
                }
            }
        }

        histogram
    }

    fn detect_regions(&self, frame: &Frame) -> Vec<Region> {
        let mut regions = Vec::new();
        let pixels = frame.pixels();
        let width = frame.width() as usize;
        let height = frame.height() as usize;

        // Simple region detection using connected components
        let mut visited = vec![false; width * height];

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let pixel_idx = idx * 4;

                if !visited[idx] && pixel_idx + 3 < pixels.len() && pixels[pixel_idx + 3] > 0 {
                    // Found a non-transparent, unvisited pixel
                    let region = self.flood_fill(frame, &mut visited, x, y);
                    if region.pixel_count > 10 {
                        // Filter out tiny regions
                        regions.push(region);
                    }
                }
            }
        }

        regions
    }

    fn flood_fill(
        &self,
        frame: &Frame,
        visited: &mut [bool],
        start_x: usize,
        start_y: usize,
    ) -> Region {
        let pixels = frame.pixels();
        let width = frame.width() as usize;
        let height = frame.height() as usize;

        let mut region = Region {
            min_x: start_x as u32,
            min_y: start_y as u32,
            max_x: start_x as u32,
            max_y: start_y as u32,
            pixel_count: 0,
            avg_color: [0, 0, 0, 0],
        };

        let mut stack = vec![(start_x, start_y)];
        let mut color_sum = [0u64; 4];

        while let Some((x, y)) = stack.pop() {
            if x >= width || y >= height {
                continue;
            }

            let idx = y * width + x;
            if visited[idx] {
                continue;
            }

            let pixel_idx = idx * 4;
            if pixel_idx + 3 >= pixels.len() || pixels[pixel_idx + 3] == 0 {
                continue;
            }

            visited[idx] = true;
            region.pixel_count += 1;

            // Update bounds
            region.min_x = region.min_x.min(x as u32);
            region.min_y = region.min_y.min(y as u32);
            region.max_x = region.max_x.max(x as u32);
            region.max_y = region.max_y.max(y as u32);

            // Accumulate color
            for i in 0..4 {
                color_sum[i] += pixels[pixel_idx + i] as u64;
            }

            // Add neighbors
            if x > 0 {
                stack.push((x - 1, y));
            }
            if x + 1 < width {
                stack.push((x + 1, y));
            }
            if y > 0 {
                stack.push((x, y - 1));
            }
            if y + 1 < height {
                stack.push((x, y + 1));
            }
        }

        // Calculate average color
        if region.pixel_count > 0 {
            for (i, sum) in color_sum.iter().enumerate() {
                region.avg_color[i] = (sum / region.pixel_count as u64) as u8;
            }
        }

        region
    }

    fn detect_text_areas(&self, frame: &Frame) -> Vec<TextArea> {
        let regions = self.detect_regions(frame);
        let mut text_areas = Vec::new();

        for region in regions {
            // Heuristics to identify text regions:
            // - Aspect ratio typical of text lines
            // - High contrast colors (usually white/yellow text)
            // - Reasonable size

            let width = region.max_x - region.min_x + 1;
            let height = region.max_y - region.min_y + 1;
            let aspect_ratio = width as f32 / height as f32;

            // Text typically has aspect ratio > 2 for horizontal text
            // and reasonable height (20-200 pixels for typical subtitles)
            if aspect_ratio > 1.5 && (15..200).contains(&height) {
                // Check if it's high contrast (likely text)
                let is_bright = region.avg_color[0] > 200
                    || region.avg_color[1] > 200
                    || region.avg_color[2] > 200;

                if is_bright {
                    text_areas.push(TextArea {
                        x: region.min_x,
                        y: region.min_y,
                        width,
                        height,
                        confidence: calculate_text_confidence(&region),
                        estimated_font_size: estimate_font_size(height),
                    });
                }
            }
        }

        text_areas
    }
}

#[derive(Debug, Clone)]
pub struct TextArea {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub confidence: f32,
    pub estimated_font_size: u32,
}

fn calculate_text_confidence(region: &Region) -> f32 {
    // Simple heuristic for text confidence
    let width = region.max_x - region.min_x + 1;
    let height = region.max_y - region.min_y + 1;
    let aspect = width as f32 / height as f32;

    let mut confidence: f32 = 0.0;

    // Good aspect ratio for text
    if (2.0..20.0).contains(&aspect) {
        confidence += 0.3;
    }

    // High brightness (typical for subtitles)
    let brightness =
        (region.avg_color[0] as f32 + region.avg_color[1] as f32 + region.avg_color[2] as f32)
            / 3.0;
    if brightness > 200.0 {
        confidence += 0.4;
    }

    // Reasonable size
    if (20..150).contains(&height) {
        confidence += 0.3;
    }

    confidence.min(1.0)
}

fn estimate_font_size(height: u32) -> u32 {
    // Rough estimation: font size is about 75% of bounding box height
    (height as f32 * 0.75) as u32
}
