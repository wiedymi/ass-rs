//! Glyph outline rendering with per-font and per-glyph path caching.

#[cfg(feature = "nostd")]
use alloc::{string::ToString, sync::Arc, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::ToString, sync::Arc, vec::Vec};

use crate::utils::RenderError;
use ahash::AHashMap;
use fontdb::{Database as FontDatabase, ID as FontId};
use tiny_skia::{Path, PathBuilder};

use super::ShapedText;

/// Glyph rendering context for caching
pub struct GlyphRenderer {
    glyph_cache: AHashMap<GlyphKey, Path>,
    font_cache: AHashMap<FontId, Arc<dyn AsRef<[u8]> + Send + Sync>>,
    // Cache TTC/OTF face index so we render outlines from the correct subface
    font_index_cache: AHashMap<FontId, u32>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct GlyphKey {
    font_id: FontId,
    glyph_id: u32,
    size: u32, // Font size in fixed point (16.16)
}

impl Default for GlyphRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl GlyphRenderer {
    /// Create new glyph renderer
    pub fn new() -> Self {
        Self {
            glyph_cache: AHashMap::new(),
            font_cache: AHashMap::new(),
            font_index_cache: AHashMap::new(),
        }
    }

    /// Render shaped text to paths
    pub fn render_shaped_text(
        &mut self,
        shaped: &ShapedText,
        font_id: FontId,
        font_database: &FontDatabase,
        spacing: f32,
    ) -> Result<Vec<Path>, RenderError> {
        // Get or cache font data and face index
        let (font_data, face_index) = if let Some(data) = self.font_cache.get(&font_id) {
            let idx = *self.font_index_cache.get(&font_id).unwrap_or(&0);
            (data.clone(), idx)
        } else {
            let (source, index) = font_database
                .face_source(font_id)
                .ok_or_else(|| RenderError::FontError("Failed to load font data".to_string()))?;

            let data = match source {
                fontdb::Source::Binary(data) => data,
                fontdb::Source::File(_path) => {
                    #[cfg(not(feature = "nostd"))]
                    {
                        std::sync::Arc::new(std::fs::read(&_path).map_err(|e| {
                            RenderError::FontError(format!("Failed to read font file: {e}"))
                        })?)
                    }
                    #[cfg(feature = "nostd")]
                    {
                        return Err(RenderError::FontError(
                            "File reading not supported in no_std mode".into(),
                        ));
                    }
                }
                fontdb::Source::SharedFile(_, data) => data,
            };

            self.font_cache.insert(font_id, data.clone());
            self.font_index_cache.insert(font_id, index);
            (data, index)
        };

        // Parse font with ttf-parser using the correct subface index
        let font = ttf_parser::Face::parse(font_data.as_ref().as_ref(), face_index)
            .map_err(|_| RenderError::FontError("Failed to parse font".to_string()))?;

        let mut paths = Vec::new();
        let mut accumulated_spacing = 0.0;

        // Render each glyph
        for (i, glyph) in shaped.glyphs.iter().enumerate() {
            let size_fixed = (shaped.font_size * 65536.0) as u32; // Convert to 16.16 fixed point
            let key = GlyphKey {
                font_id,
                glyph_id: glyph.glyph_id,
                size: size_fixed,
            };

            // Apply spacing to x position (accumulated for all previous glyphs)
            let adjusted_x = glyph.x_position + accumulated_spacing;

            // Check cache first
            if let Some(cached_path) = self.glyph_cache.get(&key) {
                // Transform cached path to glyph position
                // y_position is already the baseline position
                let transform = tiny_skia::Transform::from_translate(adjusted_x, glyph.y_position);
                if let Some(transformed) = cached_path.clone().transform(transform) {
                    paths.push(transformed);
                }
                // Add spacing for next glyph (spacing is added after each character)
                if i < shaped.glyphs.len() - 1 {
                    accumulated_spacing += spacing;
                }
                continue;
            }

            // Build glyph path
            let mut builder = PathBuilder::new();
            let glyph_id = ttf_parser::GlyphId(glyph.glyph_id as u16);

            // Get glyph outline
            if let Some(_bbox) = font.glyph_bounding_box(glyph_id) {
                let scale = shaped.font_size / font.units_per_em() as f32;

                // Outline builder to convert ttf-parser outlines to tiny-skia paths
                struct OutlineBuilder {
                    builder: PathBuilder,
                    scale: f32,
                }

                impl ttf_parser::OutlineBuilder for OutlineBuilder {
                    fn move_to(&mut self, x: f32, y: f32) {
                        self.builder.move_to(x * self.scale, -y * self.scale);
                    }

                    fn line_to(&mut self, x: f32, y: f32) {
                        self.builder.line_to(x * self.scale, -y * self.scale);
                    }

                    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                        self.builder.quad_to(
                            x1 * self.scale,
                            -y1 * self.scale,
                            x * self.scale,
                            -y * self.scale,
                        );
                    }

                    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                        self.builder.cubic_to(
                            x1 * self.scale,
                            -y1 * self.scale,
                            x2 * self.scale,
                            -y2 * self.scale,
                            x * self.scale,
                            -y * self.scale,
                        );
                    }

                    fn close(&mut self) {
                        self.builder.close();
                    }
                }

                let mut outline_builder = OutlineBuilder { builder, scale };
                font.outline_glyph(glyph_id, &mut outline_builder);
                builder = outline_builder.builder;
            }

            if let Some(path) = builder.finish() {
                // Cache the base glyph path
                self.glyph_cache.insert(key, path.clone());

                // Transform to position with spacing adjustment
                // y_position is already the baseline position
                let transform = tiny_skia::Transform::from_translate(adjusted_x, glyph.y_position);
                if let Some(transformed) = path.transform(transform) {
                    paths.push(transformed);
                }
            }

            // Add spacing for next glyph (spacing is added after each character)
            if i < shaped.glyphs.len() - 1 {
                accumulated_spacing += spacing;
            }
        }

        Ok(paths)
    }
}
