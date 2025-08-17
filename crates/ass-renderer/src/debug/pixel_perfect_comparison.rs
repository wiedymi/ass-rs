//! Pixel-perfect comparison utilities for frame analysis

use crate::renderer::Frame;

#[cfg(feature = "nostd")]
use alloc::vec::Vec;
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

/// Result of pixel-perfect comparison between two frames
#[derive(Debug, Clone)]
pub struct PixelPerfectComparison {
    /// Percentage of pixels that differ (0.0 = identical, 1.0 = completely different)
    pub pixel_diff_percentage: f64,
    /// Maximum difference in any pixel value (0-255)
    pub max_pixel_diff: u8,
    /// Average difference across all pixels
    pub avg_pixel_diff: f64,
    /// Total number of different pixels
    pub different_pixels: usize,
    /// Optional difference map (per-pixel difference values)
    pub difference_map: Option<Vec<u8>>,
}

impl PixelPerfectComparison {
    /// Compare two frames pixel by pixel
    pub fn compare_frames(frame1_data: &[u8], frame2_data: &[u8], width: u32, height: u32) -> Self {
        let total_pixels = (width * height) as usize;
        let expected_len = total_pixels * 4; // RGBA

        // Ensure both frames have the same size
        if frame1_data.len() != expected_len || frame2_data.len() != expected_len {
            return Self {
                pixel_diff_percentage: 1.0, // Complete difference if sizes don't match
                max_pixel_diff: 255,
                avg_pixel_diff: 255.0,
                different_pixels: total_pixels,
                difference_map: None,
            };
        }

        let mut different_pixels = 0;
        let mut max_diff = 0u8;
        let mut total_diff = 0u64;
        let mut diff_map = Vec::with_capacity(total_pixels);

        // Compare pixels
        for pixel_idx in 0..total_pixels {
            let base_idx = pixel_idx * 4;

            // Calculate difference for this pixel (RGBA)
            let r_diff = (frame1_data[base_idx] as i16 - frame2_data[base_idx] as i16).abs();
            let g_diff =
                (frame1_data[base_idx + 1] as i16 - frame2_data[base_idx + 1] as i16).abs();
            let b_diff =
                (frame1_data[base_idx + 2] as i16 - frame2_data[base_idx + 2] as i16).abs();
            let a_diff =
                (frame1_data[base_idx + 3] as i16 - frame2_data[base_idx + 3] as i16).abs();

            // Use maximum component difference as pixel difference
            let pixel_diff = r_diff.max(g_diff).max(b_diff).max(a_diff) as u8;
            diff_map.push(pixel_diff);

            if pixel_diff > 0 {
                different_pixels += 1;
                max_diff = max_diff.max(pixel_diff);
                total_diff += pixel_diff as u64;
            }
        }

        let pixel_diff_percentage = different_pixels as f64 / total_pixels as f64;
        let avg_pixel_diff = if different_pixels > 0 {
            total_diff as f64 / different_pixels as f64
        } else {
            0.0
        };

        Self {
            pixel_diff_percentage,
            max_pixel_diff: max_diff,
            avg_pixel_diff,
            different_pixels,
            difference_map: Some(diff_map),
        }
    }

    /// Compare frames with tolerance for small differences
    pub fn compare_frames_with_tolerance(
        frame1_data: &[u8],
        frame2_data: &[u8],
        width: u32,
        height: u32,
        tolerance: u8,
    ) -> Self {
        let mut comparison = Self::compare_frames(frame1_data, frame2_data, width, height);

        // Apply tolerance - pixels within tolerance are considered identical
        if let Some(ref mut diff_map) = comparison.difference_map {
            let mut adjusted_different_pixels = 0;
            let mut adjusted_total_diff = 0u64;
            let mut adjusted_max_diff = 0u8;

            for diff in diff_map.iter_mut() {
                if *diff <= tolerance {
                    *diff = 0; // Consider as identical
                } else {
                    adjusted_different_pixels += 1;
                    adjusted_total_diff += *diff as u64;
                    adjusted_max_diff = adjusted_max_diff.max(*diff);
                }
            }

            let total_pixels = (width * height) as usize;
            comparison.pixel_diff_percentage =
                adjusted_different_pixels as f64 / total_pixels as f64;
            comparison.different_pixels = adjusted_different_pixels;
            comparison.max_pixel_diff = adjusted_max_diff;
            comparison.avg_pixel_diff = if adjusted_different_pixels > 0 {
                adjusted_total_diff as f64 / adjusted_different_pixels as f64
            } else {
                0.0
            };
        }

        comparison
    }

    /// Check if frames are considered identical within given thresholds
    pub fn are_frames_compatible(&self, max_diff_percentage: f64, max_pixel_diff: u8) -> bool {
        self.pixel_diff_percentage <= max_diff_percentage && self.max_pixel_diff <= max_pixel_diff
    }

    /// Get a summary string of the comparison results
    pub fn summary(&self) -> String {
        format!(
            "Diff: {:.3}% ({} pixels), Max: {}, Avg: {:.1}",
            self.pixel_diff_percentage * 100.0,
            self.different_pixels,
            self.max_pixel_diff,
            self.avg_pixel_diff
        )
    }
}
