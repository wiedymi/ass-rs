//! Fixed software pipeline implementation with proper style resolution

#[cfg(feature = "nostd")]
use alloc::string::String;
#[cfg(not(feature = "nostd"))]
use std::string::String;

use ahash::AHashMap;
use fontdb::Database as FontDatabase;

use crate::pipeline::shaping::GlyphRenderer;

mod animation;
mod drawing;
mod position;
mod run;
mod style;
mod text;
mod wrap;
use style::OwnedStyle;

/// Software rendering pipeline with proper style inheritance
pub struct SoftwarePipeline {
    /// Font database for text rendering
    font_database: FontDatabase,
    /// Glyph renderer
    #[allow(dead_code)] // Glyph rendering component - used in future rendering features
    glyph_renderer: GlyphRenderer,
    /// Collision resolver for subtitle positioning
    collision_resolver: crate::collision::CollisionResolver,
    /// Render cache for performance
    cache: crate::cache::RenderCache,
    /// Current script styles map for quick lookup
    styles_map: AHashMap<String, OwnedStyle>,
    /// Default style for fallback
    default_style: Option<OwnedStyle>,
    /// Script playback resolution from PlayResX/PlayResY
    play_res_x: f32,
    play_res_y: f32,
    /// Script layout resolution from LayoutResX/LayoutResY (if present)
    layout_res_x: Option<f32>,
    layout_res_y: Option<f32>,
    /// Whether to scale border and shadow with video resolution
    scaled_border_and_shadow: bool,
    /// DPI scale factor for font rendering (default: 0.9)
    /// libass uses 72 DPI, some systems use 96 DPI
    /// Empirically adjusted to 0.9 for better libass visual match
    dpi_scale: f32,
    /// Script `WrapStyle` header (0 smart, 1 greedy, 2 none, 3 smart); the `\q`
    /// override takes precedence per event. Defaults to 0.
    wrap_style: u8,
}

impl Default for SoftwarePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl SoftwarePipeline {
    /// Create a new fixed software pipeline
    pub fn new() -> Self {
        let mut font_database = FontDatabase::new();
        font_database.load_system_fonts();

        Self {
            font_database,
            glyph_renderer: GlyphRenderer::new(),
            collision_resolver: crate::collision::CollisionResolver::new(1920.0, 1080.0),
            cache: crate::cache::RenderCache::with_limits(5000, 2000),
            styles_map: AHashMap::new(),
            default_style: None,
            play_res_x: 1920.0, // Default resolution
            play_res_y: 1080.0, // Default resolution
            layout_res_x: None,
            layout_res_y: None,
            scaled_border_and_shadow: true, // Default to true per ASS spec
            dpi_scale: 0.9,                 // Adjusted for better libass compatibility (was 0.75)
            wrap_style: 0,
        }
    }

    /// Create with specific dimensions
    pub fn with_dimensions(width: f32, height: f32) -> Self {
        let mut font_database = FontDatabase::new();
        font_database.load_system_fonts();

        Self {
            font_database,
            glyph_renderer: GlyphRenderer::new(),
            collision_resolver: crate::collision::CollisionResolver::new(width, height),
            cache: crate::cache::RenderCache::with_limits(5000, 2000),
            styles_map: AHashMap::new(),
            default_style: None,
            play_res_x: width,  // Use provided dimensions as default
            play_res_y: height, // Use provided dimensions as default
            layout_res_x: None,
            layout_res_y: None,
            scaled_border_and_shadow: true, // Default to true per ASS spec
            dpi_scale: 0.9,                 // Adjusted for better libass compatibility (was 0.75)
            wrap_style: 0,
        }
    }

    /// Set DPI scale factor (default is 0.9 for libass compatibility)
    /// Use 1.0 for 96 DPI, 0.9 for empirically matched libass rendering
    pub fn set_dpi_scale(&mut self, scale: f32) {
        self.dpi_scale = scale;
    }

    /// Get current DPI scale factor
    pub fn dpi_scale(&self) -> f32 {
        self.dpi_scale
    }
}
