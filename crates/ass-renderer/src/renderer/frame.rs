//! Rendered frame representation

#[cfg(feature = "nostd")]
use alloc::{sync::Arc, vec, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{sync::Arc, vec::Vec};

/// Rendered frame containing pixel data.
///
/// The pixel buffer is reference-counted (`Arc`) so that cloning a frame — e.g.
/// returning a cached static frame for a new timestamp — is O(1) and shares the
/// pixels. Mutating accessors use copy-on-write, so an exclusively-owned frame is
/// still mutated in place.
#[derive(Clone)]
pub struct Frame {
    buffer: Arc<Vec<u8>>,
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
            buffer: Arc::new(buffer),
            width,
            height,
            timestamp,
            format: PixelFormat::Rgba8,
        }
    }

    /// Create a frame from RGBA data
    pub fn from_rgba(buffer: Vec<u8>, width: u32, height: u32) -> Self {
        Self::new(buffer, width, height, 0)
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
            buffer: Arc::new(buffer),
            width,
            height,
            timestamp,
            format,
        }
    }

    /// Create an empty frame (transparent)
    pub fn empty(width: u32, height: u32, timestamp: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self::new(vec![0; size], width, height, timestamp)
    }

    /// Clone this frame sharing its pixel buffer (O(1)) but with a new timestamp.
    /// Used to serve a cached static frame for the current time without copying.
    pub fn with_timestamp(&self, timestamp: u32) -> Self {
        Self {
            buffer: Arc::clone(&self.buffer),
            timestamp,
            ..*self
        }
    }

    /// Get frame buffer data
    pub fn data(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    /// Get frame pixels (alias for data())
    pub fn pixels(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    /// Get mutable frame buffer data (copy-on-write if the buffer is shared)
    pub fn data_mut(&mut self) -> &mut [u8] {
        Arc::make_mut(&mut self.buffer).as_mut_slice()
    }

    /// Take ownership of the buffer (clones only if the buffer is still shared)
    pub fn into_buffer(self) -> Vec<u8> {
        Arc::try_unwrap(self.buffer).unwrap_or_else(|arc| (*arc).clone())
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
                for chunk in Arc::make_mut(&mut self.buffer).chunks_exact_mut(4) {
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
                self.buffer = Arc::new(rgba);
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
