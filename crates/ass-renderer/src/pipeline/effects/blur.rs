//! Blur effect implementations

#[cfg(feature = "nostd")]
use alloc::{vec, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

use crate::pipeline::effects::Effect;
use crate::utils::RenderError;

/// Gaussian blur effect
pub struct GaussianBlur {
    radius: f32,
    sigma: f32,
}

impl GaussianBlur {
    /// Create a new Gaussian blur effect
    pub fn new(radius: f32) -> Self {
        let sigma = radius / 3.0;
        Self { radius, sigma }
    }

    /// Create kernel for Gaussian blur
    fn create_kernel(&self) -> Vec<f32> {
        let size = (self.radius * 2.0) as usize + 1;
        let mut kernel = Vec::with_capacity(size);

        let sigma2 = self.sigma * self.sigma;
        let norm = 1.0 / (2.0 * core::f32::consts::PI * sigma2).sqrt();

        let half = size / 2;
        let mut sum = 0.0;

        for i in 0..size {
            let x = i as f32 - half as f32;
            let weight = norm * (-x * x / (2.0 * sigma2)).exp();
            kernel.push(weight);
            sum += weight;
        }

        // Normalize kernel
        for weight in &mut kernel {
            *weight /= sum;
        }

        kernel
    }

    /// Apply horizontal blur pass
    fn blur_horizontal(&self, pixels: &mut [u8], width: u32, height: u32, kernel: &[f32]) {
        let stride = width * 4;
        let half_kernel = kernel.len() / 2;
        let mut temp = vec![0u8; pixels.len()];

        for y in 0..height {
            for x in 0..width {
                let mut r = 0.0;
                let mut g = 0.0;
                let mut b = 0.0;
                let mut a = 0.0;

                for (i, &weight) in kernel.iter().enumerate() {
                    let sx = (x as i32 + i as i32 - half_kernel as i32)
                        .max(0)
                        .min(width as i32 - 1) as u32;

                    let idx = (y * stride + sx * 4) as usize;
                    r += pixels[idx] as f32 * weight;
                    g += pixels[idx + 1] as f32 * weight;
                    b += pixels[idx + 2] as f32 * weight;
                    a += pixels[idx + 3] as f32 * weight;
                }

                let idx = (y * stride + x * 4) as usize;
                temp[idx] = r.round() as u8;
                temp[idx + 1] = g.round() as u8;
                temp[idx + 2] = b.round() as u8;
                temp[idx + 3] = a.round() as u8;
            }
        }

        pixels.copy_from_slice(&temp);
    }

    /// Apply vertical blur pass
    fn blur_vertical(&self, pixels: &mut [u8], width: u32, height: u32, kernel: &[f32]) {
        let stride = width * 4;
        let half_kernel = kernel.len() / 2;
        let mut temp = vec![0u8; pixels.len()];

        for y in 0..height {
            for x in 0..width {
                let mut r = 0.0;
                let mut g = 0.0;
                let mut b = 0.0;
                let mut a = 0.0;

                for (i, &weight) in kernel.iter().enumerate() {
                    let sy = (y as i32 + i as i32 - half_kernel as i32)
                        .max(0)
                        .min(height as i32 - 1) as u32;

                    let idx = (sy * stride + x * 4) as usize;
                    r += pixels[idx] as f32 * weight;
                    g += pixels[idx + 1] as f32 * weight;
                    b += pixels[idx + 2] as f32 * weight;
                    a += pixels[idx + 3] as f32 * weight;
                }

                let idx = (y * stride + x * 4) as usize;
                temp[idx] = r.round() as u8;
                temp[idx + 1] = g.round() as u8;
                temp[idx + 2] = b.round() as u8;
                temp[idx + 3] = a.round() as u8;
            }
        }

        pixels.copy_from_slice(&temp);
    }
}

impl Effect for GaussianBlur {
    fn apply(&self, pixels: &mut [u8], width: u32, height: u32) -> Result<(), RenderError> {
        if pixels.len() != (width * height * 4) as usize {
            return Err(RenderError::InvalidBufferSize {
                expected: (width * height * 4) as usize,
                actual: pixels.len(),
            });
        }

        let kernel = self.create_kernel();

        // Two-pass separable blur
        self.blur_horizontal(pixels, width, height, &kernel);
        self.blur_vertical(pixels, width, height, &kernel);

        Ok(())
    }

    fn name(&self) -> &str {
        "GaussianBlur"
    }
}

/// Box blur effect (faster but lower quality)
pub struct BoxBlur {
    radius: u32,
}

impl BoxBlur {
    /// Create a new box blur effect
    pub fn new(radius: u32) -> Self {
        Self { radius }
    }
}

impl Effect for BoxBlur {
    fn apply(&self, pixels: &mut [u8], width: u32, height: u32) -> Result<(), RenderError> {
        if pixels.len() != (width * height * 4) as usize {
            return Err(RenderError::InvalidBufferSize {
                expected: (width * height * 4) as usize,
                actual: pixels.len(),
            });
        }

        let stride = width * 4;
        let mut temp = vec![0u8; pixels.len()];

        // Simple box blur implementation
        for y in 0..height {
            for x in 0..width {
                let mut r = 0u32;
                let mut g = 0u32;
                let mut b = 0u32;
                let mut a = 0u32;
                let mut count = 0u32;

                for dy in 0..=self.radius * 2 {
                    for dx in 0..=self.radius * 2 {
                        let sy = (y as i32 + dy as i32 - self.radius as i32)
                            .max(0)
                            .min(height as i32 - 1) as u32;
                        let sx = (x as i32 + dx as i32 - self.radius as i32)
                            .max(0)
                            .min(width as i32 - 1) as u32;

                        let idx = (sy * stride + sx * 4) as usize;
                        r += pixels[idx] as u32;
                        g += pixels[idx + 1] as u32;
                        b += pixels[idx + 2] as u32;
                        a += pixels[idx + 3] as u32;
                        count += 1;
                    }
                }

                let idx = (y * stride + x * 4) as usize;
                temp[idx] = (r / count) as u8;
                temp[idx + 1] = (g / count) as u8;
                temp[idx + 2] = (b / count) as u8;
                temp[idx + 3] = (a / count) as u8;
            }
        }

        pixels.copy_from_slice(&temp);
        Ok(())
    }

    fn name(&self) -> &str {
        "BoxBlur"
    }
}
