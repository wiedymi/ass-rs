//! Rendered frame representation

#[cfg(feature = "nostd")]
use alloc::{vec, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

/// Rendered frame containing pixel data
#[derive(Clone)]
pub struct Frame {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
    timestamp: u32,
    format: PixelFormat,
}

/// Pixel format for frame data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// RGBA with 8 bits per channel
    Rgba8,
    /// BGRA with 8 bits per channel
    Bgra8,
    /// RGB with 8 bits per channel
    Rgb8,
}

impl Frame {
    /// Create a new frame with the given buffer
    pub fn new(buffer: Vec<u8>, width: u32, height: u32, timestamp: u32) -> Self {
        Self {
            buffer,
            width,
            height,
            timestamp,
            format: PixelFormat::Rgba8,
        }
    }

    /// Create a frame from RGBA data
    pub fn from_rgba(buffer: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            buffer,
            width,
            height,
            timestamp: 0,
            format: PixelFormat::Rgba8,
        }
    }

    /// Create a new frame with specific pixel format
    pub fn with_format(
        buffer: Vec<u8>,
        width: u32,
        height: u32,
        timestamp: u32,
        format: PixelFormat,
    ) -> Self {
        Self {
            buffer,
            width,
            height,
            timestamp,
            format,
        }
    }

    /// Create an empty frame (transparent)
    pub fn empty(width: u32, height: u32, timestamp: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            buffer: vec![0; size],
            width,
            height,
            timestamp,
            format: PixelFormat::Rgba8,
        }
    }

    /// Get frame buffer data
    pub fn data(&self) -> &[u8] {
        &self.buffer
    }

    /// Get frame pixels (alias for data())
    pub fn pixels(&self) -> &[u8] {
        &self.buffer
    }

    /// Get mutable frame buffer data
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    /// Take ownership of the buffer
    pub fn into_buffer(self) -> Vec<u8> {
        self.buffer
    }

    /// Get frame width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get frame height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get frame timestamp in centiseconds
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }

    /// Get pixel format
    pub fn format(&self) -> PixelFormat {
        self.format
    }

    /// Get bytes per pixel
    pub fn bytes_per_pixel(&self) -> usize {
        match self.format {
            PixelFormat::Rgba8 | PixelFormat::Bgra8 => 4,
            PixelFormat::Rgb8 => 3,
        }
    }

    /// Get stride (bytes per row)
    pub fn stride(&self) -> usize {
        self.width as usize * self.bytes_per_pixel()
    }

    /// Convert to RGBA format if not already
    pub fn to_rgba(mut self) -> Self {
        match self.format {
            PixelFormat::Rgba8 => self,
            PixelFormat::Bgra8 => {
                for chunk in self.buffer.chunks_exact_mut(4) {
                    chunk.swap(0, 2);
                }
                self.format = PixelFormat::Rgba8;
                self
            }
            PixelFormat::Rgb8 => {
                let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
                for chunk in self.buffer.chunks_exact(3) {
                    rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
                }
                self.buffer = rgba;
                self.format = PixelFormat::Rgba8;
                self
            }
        }
    }

    /// Check if frame is empty (all transparent)
    pub fn is_empty(&self) -> bool {
        match self.format {
            PixelFormat::Rgba8 | PixelFormat::Bgra8 => {
                self.buffer.chunks_exact(4).all(|pixel| pixel[3] == 0)
            }
            PixelFormat::Rgb8 => false,
        }
    }
}
