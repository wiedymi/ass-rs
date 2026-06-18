//! Frame helper utilities for the debug renderer.
//!
//! Low-level helpers used by [`super::DebugRenderer`] to checksum frame
//! buffers, persist frames as PNG files, and draw debugging overlays.

use crate::{Frame, RenderError};

#[cfg(feature = "nostd")]
extern crate alloc;
#[cfg(feature = "nostd")]
use alloc::{format, vec::Vec};

pub(crate) fn calculate_checksum(pixels: &[u8]) -> u64 {
    #[cfg(not(feature = "nostd"))]
    use std::collections::hash_map::DefaultHasher;
    #[cfg(not(feature = "nostd"))]
    use std::hash::{Hash, Hasher};

    #[cfg(feature = "nostd")]
    use core::hash::{Hash, Hasher};
    #[cfg(feature = "nostd")]
    struct DefaultHasher(u64);

    #[cfg(feature = "nostd")]
    impl DefaultHasher {
        fn new() -> Self {
            DefaultHasher(0)
        }
        fn finish(&self) -> u64 {
            self.0
        }
    }

    #[cfg(feature = "nostd")]
    impl Hasher for DefaultHasher {
        fn write(&mut self, bytes: &[u8]) {
            for &b in bytes {
                self.0 = self.0.wrapping_mul(31).wrapping_add(b as u64);
            }
        }

        fn finish(&self) -> u64 {
            self.0
        }
    }

    let mut hasher = DefaultHasher::new();
    pixels.hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn save_frame_as_png(frame: &Frame, path: &str) -> Result<(), RenderError> {
    #[cfg(feature = "image")]
    {
        use image::{ImageBuffer, Rgba};

        let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
            frame.width(),
            frame.height(),
            frame.pixels().to_vec(),
        )
        .ok_or_else(|| RenderError::BackendError("Failed to create image buffer".into()))?;

        img.save(path)
            .map_err(|e| RenderError::BackendError(format!("Failed to save PNG: {e}")))?;
    }

    #[cfg(not(feature = "image"))]
    {
        let _ = (frame, path);
        // Silent no-op if image feature is not enabled
    }

    Ok(())
}

pub(crate) fn draw_rectangle(
    frame: &mut Frame,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 4],
) -> Result<(), RenderError> {
    // TODO: Implement rectangle drawing
    let _ = (frame, x, y, width, height, color);
    Ok(())
}

pub(crate) fn draw_text_overlay(
    frame: &mut Frame,
    text: &str,
    x: u32,
    y: u32,
) -> Result<(), RenderError> {
    // TODO: Implement text overlay
    let _ = (frame, text, x, y);
    Ok(())
}
