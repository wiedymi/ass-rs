//! Rendering context with fonts, resolution, and backend configuration

use fontdb::Database as FontDatabase;

#[cfg(feature = "nostd")]
use alloc::sync::Arc;
#[cfg(not(feature = "nostd"))]
use std::sync::Arc;

/// Rendering context containing fonts, resolution, and configuration
#[derive(Clone)]
pub struct RenderContext {
    width: u32,
    height: u32,
    font_database: Arc<FontDatabase>,
    playback_res_x: u32,
    playback_res_y: u32,
    storage_res_x: u32,
    storage_res_y: u32,
    frame_rate: f32,
    par: f32,
}

impl RenderContext {
    /// Create a new render context with the given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let mut font_database = FontDatabase::new();
        font_database.load_system_fonts();

        Self {
            width,
            height,
            font_database: Arc::new(font_database),
            playback_res_x: width,
            playback_res_y: height,
            storage_res_x: width,
            storage_res_y: height,
            frame_rate: 24.0,
            par: 1.0,
        }
    }

    /// Create context with custom font database
    pub fn with_font_database(width: u32, height: u32, font_database: FontDatabase) -> Self {
        Self {
            width,
            height,
            font_database: Arc::new(font_database),
            playback_res_x: width,
            playback_res_y: height,
            storage_res_x: width,
            storage_res_y: height,
            frame_rate: 24.0,
            par: 1.0,
        }
    }

    /// Set playback resolution (from script info)
    pub fn set_playback_resolution(&mut self, x: u32, y: u32) {
        self.playback_res_x = x;
        self.playback_res_y = y;
    }

    /// Set storage resolution (original script resolution)
    pub fn set_storage_resolution(&mut self, x: u32, y: u32) {
        self.storage_res_x = x;
        self.storage_res_y = y;
    }

    /// Set frame rate
    pub fn set_frame_rate(&mut self, fps: f32) {
        self.frame_rate = fps;
    }

    /// Set pixel aspect ratio
    pub fn set_pixel_aspect_ratio(&mut self, par: f32) {
        self.par = par;
    }

    /// Get render width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get render height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get font database
    pub fn font_database(&self) -> &FontDatabase {
        &self.font_database
    }

    /// Get mutable font database
    pub fn font_database_mut(&mut self) -> &mut FontDatabase {
        Arc::get_mut(&mut self.font_database).expect("Font database has multiple references")
    }

    /// Get playback resolution X
    pub fn playback_res_x(&self) -> u32 {
        self.playback_res_x
    }

    /// Get playback resolution Y
    pub fn playback_res_y(&self) -> u32 {
        self.playback_res_y
    }

    /// Get storage resolution X
    pub fn storage_res_x(&self) -> u32 {
        self.storage_res_x
    }

    /// Get storage resolution Y
    pub fn storage_res_y(&self) -> u32 {
        self.storage_res_y
    }

    /// Get frame rate
    pub fn frame_rate(&self) -> f32 {
        self.frame_rate
    }

    /// Get pixel aspect ratio
    pub fn pixel_aspect_ratio(&self) -> f32 {
        self.par
    }

    /// Calculate X scale factor from storage to playback resolution
    pub fn scale_x(&self) -> f32 {
        self.playback_res_x as f32 / self.storage_res_x.max(1) as f32
    }

    /// Calculate Y scale factor from storage to playback resolution
    pub fn scale_y(&self) -> f32 {
        self.playback_res_y as f32 / self.storage_res_y.max(1) as f32
    }

    /// Calculate X scale factor from playback to render resolution
    pub fn render_scale_x(&self) -> f32 {
        self.width as f32 / self.playback_res_x.max(1) as f32
    }

    /// Calculate Y scale factor from playback to render resolution
    pub fn render_scale_y(&self) -> f32 {
        self.height as f32 / self.playback_res_y.max(1) as f32
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}
