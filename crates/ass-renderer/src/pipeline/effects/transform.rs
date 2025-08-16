//! Transform effect for geometric transformations

#[cfg(feature = "nostd")]
use alloc::vec;

use crate::pipeline::effects::Effect;
use crate::utils::{Matrix3x3, RenderError};

/// Transform effect for applying geometric transformations
pub struct TransformEffect {
    matrix: Matrix3x3,
}

impl TransformEffect {
    /// Create a new transform effect with identity matrix
    pub fn new() -> Self {
        Self {
            matrix: Matrix3x3::identity(),
        }
    }

    /// Create transform from matrix
    pub fn from_matrix(matrix: Matrix3x3) -> Self {
        Self { matrix }
    }

    /// Set translation
    pub fn translate(mut self, x: f32, y: f32) -> Self {
        self.matrix = self.matrix.multiply(&Matrix3x3::translate(x, y));
        self
    }

    /// Set scale
    pub fn scale(mut self, sx: f32, sy: f32) -> Self {
        self.matrix = self.matrix.multiply(&Matrix3x3::scale(sx, sy));
        self
    }

    /// Set rotation (angle in radians)
    pub fn rotate(mut self, angle: f32) -> Self {
        self.matrix = self.matrix.multiply(&Matrix3x3::rotate(angle));
        self
    }

    /// Apply bilinear interpolation for pixel sampling
    fn sample_bilinear(pixels: &[u8], x: f32, y: f32, width: u32, height: u32) -> [u8; 4] {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;

        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        let mut result = [0u8; 4];

        // Sample four neighboring pixels
        let p00 = Self::get_pixel(pixels, x0, y0, width, height);
        let p10 = Self::get_pixel(pixels, x1, y0, width, height);
        let p01 = Self::get_pixel(pixels, x0, y1, width, height);
        let p11 = Self::get_pixel(pixels, x1, y1, width, height);

        // Bilinear interpolation
        for i in 0..4 {
            let v0 = p00[i] as f32 * (1.0 - fx) + p10[i] as f32 * fx;
            let v1 = p01[i] as f32 * (1.0 - fx) + p11[i] as f32 * fx;
            result[i] = (v0 * (1.0 - fy) + v1 * fy) as u8;
        }

        result
    }

    /// Get pixel value with bounds checking
    fn get_pixel(pixels: &[u8], x: i32, y: i32, width: u32, height: u32) -> [u8; 4] {
        if x < 0 || x >= width as i32 || y < 0 || y >= height as i32 {
            return [0, 0, 0, 0];
        }

        let idx = (y as u32 * width * 4 + x as u32 * 4) as usize;
        [
            pixels.get(idx).copied().unwrap_or(0),
            pixels.get(idx + 1).copied().unwrap_or(0),
            pixels.get(idx + 2).copied().unwrap_or(0),
            pixels.get(idx + 3).copied().unwrap_or(0),
        ]
    }
}

impl Effect for TransformEffect {
    fn apply(&self, pixels: &mut [u8], width: u32, height: u32) -> Result<(), RenderError> {
        if pixels.len() != (width * height * 4) as usize {
            return Err(RenderError::InvalidBufferSize {
                expected: (width * height * 4) as usize,
                actual: pixels.len(),
            });
        }

        let mut output = vec![0u8; pixels.len()];

        // Apply inverse transform to sample from source
        // This would need proper matrix inversion in production
        for y in 0..height {
            for x in 0..width {
                // Transform destination coordinates to source coordinates
                let (src_x, src_y) = self.matrix.transform_point(x as f32, y as f32);

                // Sample from source with bilinear interpolation
                let color = Self::sample_bilinear(pixels, src_x, src_y, width, height);

                let idx = (y * width * 4 + x * 4) as usize;
                output[idx..idx + 4].copy_from_slice(&color);
            }
        }

        pixels.copy_from_slice(&output);
        Ok(())
    }

    fn name(&self) -> &str {
        "Transform"
    }
}

impl Default for TransformEffect {
    fn default() -> Self {
        Self::new()
    }
}
