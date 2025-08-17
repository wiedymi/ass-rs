//! Libass renderer for comparison and debugging
//!
//! This module provides a functional libass renderer that can be used
//! to compare output with our ASS renderer implementation.

#![allow(unsafe_code)] // Required for using FFI bindings

#[cfg(feature = "libass-compare")]
use super::libass_ffi::{ASS_Image, LibassLibrary, LibassRenderer as FFIRenderer, LibassTrack};
use crate::{Frame, RenderError};
use std::path::Path;

/// Libass renderer wrapper for debugging and comparison
#[cfg(feature = "libass-compare")]
pub struct LibassRenderer {
    library: LibassLibrary,
    renderer: FFIRenderer,
    track: Option<LibassTrack>,
    width: u32,
    height: u32,
}

#[cfg(feature = "libass-compare")]
impl LibassRenderer {
    /// Create a new libass renderer
    pub fn new(width: u32, height: u32) -> Result<Self, RenderError> {
        let library = LibassLibrary::new().ok_or_else(|| {
            RenderError::InitializationError("Failed to initialize libass library".into())
        })?;

        let mut renderer = FFIRenderer::new(&library).ok_or_else(|| {
            RenderError::InitializationError("Failed to create libass renderer".into())
        })?;

        // Configure renderer
        renderer.set_frame_size(width as i32, height as i32);
        renderer.set_storage_size(width as i32, height as i32);
        renderer.set_use_margins(true);
        renderer.set_pixel_aspect(1.0);
        renderer.set_font_scale(1.0);

        // Load system fonts
        renderer.set_fonts(None, Some("sans-serif"));

        println!("Libass version: 0x{:08x}", LibassLibrary::version());

        Ok(Self {
            library,
            renderer,
            track: None,
            width,
            height,
        })
    }

    /// Load ASS script from string
    pub fn load_script(&mut self, script_content: &str) -> Result<(), RenderError> {
        // Create new track
        let mut track = LibassTrack::new(&self.library)
            .ok_or_else(|| RenderError::ParseError("Failed to create libass track".into()))?;

        // Process the script content
        track.process_data(script_content.as_bytes());

        self.track = Some(track);
        Ok(())
    }

    /// Load ASS script from file
    pub fn load_script_file(&mut self, path: &Path) -> Result<(), RenderError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| RenderError::IOError(e.to_string()))?;
        self.load_script(&content)
    }

    /// Render a frame at the given time (in centiseconds for compatibility)
    pub fn render_frame(
        &mut self,
        script: &ass_core::parser::Script,
        time_cs: u32,
    ) -> Result<Frame, RenderError> {
        // Convert our script to ASS format
        let ass_content = script.to_ass_string();
        self.load_script(&ass_content)?;

        let track = self
            .track
            .as_mut()
            .ok_or_else(|| RenderError::InvalidState("No script loaded".into()))?;

        // Convert centiseconds to milliseconds
        let time_ms = time_cs as i64 * 10;

        // Create RGBA buffer
        let mut buffer = vec![0u8; (self.width * self.height * 4) as usize];

        // Render with libass
        if let Some(images) = self.renderer.render_frame(track, time_ms) {
            // Blend libass images onto buffer
            for image in images {
                self.blend_image(&mut buffer, &image);
            }
        }

        Ok(Frame::new(buffer, self.width, self.height, time_cs))
    }

    /// Render a frame at the given time (in milliseconds) - legacy method
    pub fn render_frame_ms(&mut self, time_ms: i64) -> Result<Vec<u8>, RenderError> {
        let track = self
            .track
            .as_mut()
            .ok_or_else(|| RenderError::InvalidState("No script loaded".into()))?;

        // Create RGBA buffer
        let mut buffer = vec![0u8; (self.width * self.height * 4) as usize];

        // Render with libass
        if let Some(images) = self.renderer.render_frame(track, time_ms) {
            // Blend libass images onto buffer
            for image in images {
                self.blend_image(&mut buffer, &image);
            }
        }

        Ok(buffer)
    }

    /// Render and return as Frame - legacy method
    pub fn render_to_frame(&mut self, time_ms: i64) -> Result<Frame, RenderError> {
        let buffer = self.render_frame_ms(time_ms)?;
        Ok(Frame::new(
            buffer,
            self.width,
            self.height,
            (time_ms / 10) as u32,
        ))
    }

    /// Blend a libass image onto the buffer
    fn blend_image(&self, buffer: &mut [u8], image: &ASS_Image) {
        let stride = self.width * 4;

        // Get color from libass image (RGBA format in little-endian)
        // In libass, color is stored as 0xRRGGBBAA where:
        // - bits 0-7 (0xFF) = Alpha (0 = opaque, 255 = transparent, needs inversion)
        // - bits 8-15 (0xFF00) = Blue
        // - bits 16-23 (0xFF0000) = Green
        // - bits 24-31 (0xFF000000) = Red
        let libass_alpha = (image.color & 0xFF) as u8;
        let a = 255 - libass_alpha; // Invert: libass uses 0=opaque, we need 255=opaque
        let b = ((image.color >> 8) & 0xFF) as u8;
        let g = ((image.color >> 16) & 0xFF) as u8;
        let r = ((image.color >> 24) & 0xFF) as u8;

        // Safety check for bitmap pointer
        if image.bitmap.is_null() {
            return;
        }

        // Get bitmap data
        let bitmap =
            unsafe { std::slice::from_raw_parts(image.bitmap, (image.stride * image.h) as usize) };

        // Blend onto buffer
        for y in 0..image.h {
            let dst_y = (image.dst_y + y) as usize;
            if dst_y >= self.height as usize {
                continue;
            }

            for x in 0..image.w {
                let dst_x = (image.dst_x + x) as usize;
                if dst_x >= self.width as usize {
                    continue;
                }

                let src_idx = (y * image.stride + x) as usize;
                let dst_idx = dst_y * stride as usize + dst_x * 4;

                if src_idx < bitmap.len() && dst_idx + 3 < buffer.len() {
                    // Get alpha from bitmap (libass uses alpha channel for glyph coverage)
                    let coverage = bitmap[src_idx] as f32 / 255.0;

                    // Combine glyph coverage with color alpha
                    let src_alpha = (a as f32 / 255.0) * coverage;
                    let inv_alpha = 1.0 - src_alpha;

                    // Composite over existing pixel using premultiplied alpha
                    buffer[dst_idx] =
                        (buffer[dst_idx] as f32 * inv_alpha + r as f32 * src_alpha) as u8;
                    buffer[dst_idx + 1] =
                        (buffer[dst_idx + 1] as f32 * inv_alpha + g as f32 * src_alpha) as u8;
                    buffer[dst_idx + 2] =
                        (buffer[dst_idx + 2] as f32 * inv_alpha + b as f32 * src_alpha) as u8;
                    buffer[dst_idx + 3] = ((buffer[dst_idx + 3] as f32 * (1.0 - inv_alpha)
                        + src_alpha * 255.0)
                        .min(255.0)) as u8;
                }
            }
        }
    }

    /// Update frame size
    pub fn set_frame_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.renderer.set_frame_size(width as i32, height as i32);
        self.renderer.set_storage_size(width as i32, height as i32);
    }
}

/// Stub implementation when libass-compare feature is disabled
#[cfg(not(feature = "libass-compare"))]
pub struct LibassRenderer;

#[cfg(not(feature = "libass-compare"))]
impl LibassRenderer {
    pub fn new(_width: u32, _height: u32) -> Result<Self, RenderError> {
        Err(RenderError::UnsupportedOperation(
            "libass-compare feature not enabled. Install libass and enable the feature.".into(),
        ))
    }

    pub fn load_script(&mut self, _script_content: &str) -> Result<(), RenderError> {
        Err(RenderError::UnsupportedOperation(
            "libass-compare feature not enabled".into(),
        ))
    }

    pub fn load_script_file(&mut self, _path: &Path) -> Result<(), RenderError> {
        Err(RenderError::UnsupportedOperation(
            "libass-compare feature not enabled".into(),
        ))
    }

    pub fn render_frame(&mut self, _time_ms: i64) -> Result<Vec<u8>, RenderError> {
        Err(RenderError::UnsupportedOperation(
            "libass-compare feature not enabled".into(),
        ))
    }

    pub fn render_to_frame(&mut self, _time_ms: i64) -> Result<Frame, RenderError> {
        Err(RenderError::UnsupportedOperation(
            "libass-compare feature not enabled".into(),
        ))
    }

    pub fn set_frame_size(&mut self, _width: u32, _height: u32) {}
}
