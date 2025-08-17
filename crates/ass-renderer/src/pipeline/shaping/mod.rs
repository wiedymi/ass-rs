//! Text shaping module using rustybuzz

mod font_metrics;
pub use font_metrics::FontMetrics;

#[cfg(feature = "nostd")]
use alloc::{format, string::ToString, sync::Arc, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::ToString, sync::Arc, vec::Vec};

use crate::utils::RenderError;
use ahash::AHashMap;
use fontdb::{Database as FontDatabase, ID as FontId};
use rustybuzz::{Face, Feature, UnicodeBuffer, Variation};
use tiny_skia::{Path, PathBuilder};

/// Shaped glyph representation
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    /// Glyph ID in the font
    pub glyph_id: u32,
    /// X position
    pub x_position: f32,
    /// Y position
    pub y_position: f32,
    /// X offset
    pub x_offset: f32,
    /// Y offset
    pub y_offset: f32,
    /// Horizontal advance
    pub x_advance: f32,
    /// Vertical advance
    pub y_advance: f32,
    /// Cluster index in original text
    pub cluster: u32,
}

/// Shaped text result
#[derive(Debug, Clone)]
pub struct ShapedText {
    /// Shaped glyphs
    pub glyphs: Vec<ShapedGlyph>,
    /// Total width
    pub width: f32,
    /// Total height (line height)
    pub height: f32,
    /// Baseline position
    pub baseline: f32,
    /// Font size used for shaping
    pub font_size: f32,
    /// Font ascent (for underline/strikeout positioning)
    pub ascent: f32,
    /// Font descent (for underline/strikeout positioning)
    pub descent: f32,
}

impl ShapedText {
    /// Get the total horizontal advance of the shaped text
    pub fn total_advance(&self) -> Option<f32> {
        if self.glyphs.is_empty() {
            return Some(0.0);
        }

        // Calculate the total advance by summing up all glyph advances
        let mut total = 0.0;
        for glyph in &self.glyphs {
            total += glyph.x_advance;
        }
        Some(total)
    }
}

/// Shape text into glyphs
pub fn shape_text(
    text: &str,
    font_family: &str,
    font_size: f32,
    font_database: &FontDatabase,
) -> Result<ShapedText, RenderError> {
    shape_text_with_style(text, font_family, font_size, false, false, font_database)
}

/// Shape text with style options
pub fn shape_text_with_style(
    text: &str,
    font_family: &str,
    font_size: f32,
    bold: bool,
    italic: bool,
    font_database: &FontDatabase,
) -> Result<ShapedText, RenderError> {
    // Find matching font
    let font_id = find_font(font_database, font_family, bold, italic)?;

    // Get font data using face_source
    let (source, index) = font_database
        .face_source(font_id)
        .ok_or_else(|| RenderError::FontError(format!("Failed to load font data")))?;

    // Extract data based on source type
    let font_data = match source {
        fontdb::Source::Binary(data) => data,
        fontdb::Source::File(path) => {
            #[cfg(not(feature = "nostd"))]
            {
                std::sync::Arc::new(std::fs::read(&path).map_err(|e| {
                    RenderError::FontError(format!("Failed to read font file: {}", e))
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

    // Create rustybuzz face
    let rb_face = Face::from_slice(font_data.as_ref().as_ref(), index)
        .ok_or_else(|| RenderError::FontError(format!("Failed to create font face")))?;

    // Parse with ttf-parser for OS/2 table access
    let ttf_face = ttf_parser::Face::parse(font_data.as_ref().as_ref(), index)
        .map_err(|_| RenderError::FontError(format!("Failed to parse font for metrics")))?;

    // Get font metrics with VSFilter compatibility
    let metrics = FontMetrics::from_face(&ttf_face);

    // Create buffer and add text
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);

    // Shape the text
    let features: Vec<Feature> = Vec::new();
    let variations: Vec<Variation> = Vec::new();
    let output = rustybuzz::shape(&rb_face, &features, buffer);

    // Get glyph positions and convert to our format
    let positions = output.glyph_positions();
    let infos = output.glyph_infos();

    let mut glyphs = Vec::new();
    let mut cursor_x = 0.0;
    let mut cursor_y = 0.0;

    let scale = font_size / metrics.units_per_em;

    for (info, pos) in infos.iter().zip(positions.iter()) {
        glyphs.push(ShapedGlyph {
            glyph_id: info.glyph_id,
            x_position: cursor_x + pos.x_offset as f32 * scale,
            y_position: cursor_y + pos.y_offset as f32 * scale,
            x_offset: pos.x_offset as f32 * scale,
            y_offset: pos.y_offset as f32 * scale,
            x_advance: pos.x_advance as f32 * scale,
            y_advance: pos.y_advance as f32 * scale,
            cluster: info.cluster,
        });

        cursor_x += pos.x_advance as f32 * scale;
        cursor_y += pos.y_advance as f32 * scale;
    }

    // Calculate metrics using VSFilter-compatible values
    let height = metrics.line_height(font_size);
    let baseline = metrics.baseline(font_size);
    let ascent = metrics.ascender * scale;
    let descent = metrics.descender * scale;

    Ok(ShapedText {
        width: cursor_x,
        height,
        baseline,
        font_size,
        glyphs,
        ascent,
        descent,
    })
}

/// Find matching font in database with fallback support for CJK text
fn find_font(
    font_database: &FontDatabase,
    family: &str,
    bold: bool,
    italic: bool,
) -> Result<FontId, RenderError> {
    // First try the requested font
    let query = fontdb::Query {
        families: &[fontdb::Family::Name(family), fontdb::Family::SansSerif],
        weight: if bold {
            fontdb::Weight::BOLD
        } else {
            fontdb::Weight::NORMAL
        },
        stretch: fontdb::Stretch::Normal,
        style: if italic {
            fontdb::Style::Italic
        } else {
            fontdb::Style::Normal
        },
    };

    if let Some(id) = font_database.query(&query) {
        return Ok(id);
    }

    // If the requested font contains "CJK" or is a known Japanese font name, try fallbacks
    let is_japanese_font = family.contains("CJK")
        || family.contains("Noto Sans JP")
        || family.contains("Hiragino")
        || family.contains("Yu Gothic")
        || family.contains("MS Gothic")
        || family.contains("Meiryo");

    if is_japanese_font {
        // Try common Japanese font fallbacks
        let japanese_fallbacks = [
            "Hiragino Sans",
            "Hiragino Kaku Gothic Pro",
            "Yu Gothic",
            "MS Gothic",
            "Meiryo",
            "Noto Sans CJK JP",
            "Noto Sans JP",
        ];

        for fallback in &japanese_fallbacks {
            let fallback_query = fontdb::Query {
                families: &[fontdb::Family::Name(fallback)],
                weight: if bold {
                    fontdb::Weight::BOLD
                } else {
                    fontdb::Weight::NORMAL
                },
                stretch: fontdb::Stretch::Normal,
                style: if italic {
                    fontdb::Style::Italic
                } else {
                    fontdb::Style::Normal
                },
            };

            if let Some(id) = font_database.query(&fallback_query) {
                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                println!("Using Japanese font fallback: {} -> {}", family, fallback);
                return Ok(id);
            }
        }
    }

    // Final fallback to any available font
    let final_query = fontdb::Query {
        families: &[fontdb::Family::SansSerif, fontdb::Family::Serif],
        weight: if bold {
            fontdb::Weight::BOLD
        } else {
            fontdb::Weight::NORMAL
        },
        stretch: fontdb::Stretch::Normal,
        style: if italic {
            fontdb::Style::Italic
        } else {
            fontdb::Style::Normal
        },
    };

    font_database.query(&final_query).ok_or_else(|| {
        RenderError::FontError(format!(
            "Font '{}' not found and no fallback available",
            family
        ))
    })
}

/// Glyph rendering context for caching
pub struct GlyphRenderer {
    glyph_cache: AHashMap<GlyphKey, Path>,
    font_cache: AHashMap<FontId, Arc<dyn AsRef<[u8]> + Send + Sync>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct GlyphKey {
    font_id: FontId,
    glyph_id: u32,
    size: u32, // Font size in fixed point (16.16)
}

impl GlyphRenderer {
    /// Create new glyph renderer
    pub fn new() -> Self {
        Self {
            glyph_cache: AHashMap::new(),
            font_cache: AHashMap::new(),
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
        // Get or cache font data
        let font_data = if let Some(data) = self.font_cache.get(&font_id) {
            data.clone()
        } else {
            let (source, _index) = font_database
                .face_source(font_id)
                .ok_or_else(|| RenderError::FontError("Failed to load font data".to_string()))?;

            let data = match source {
                fontdb::Source::Binary(data) => data,
                fontdb::Source::File(path) => {
                    #[cfg(not(feature = "nostd"))]
                    {
                        std::sync::Arc::new(std::fs::read(&path).map_err(|e| {
                            RenderError::FontError(format!("Failed to read font file: {}", e))
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
            data
        };

        // Parse font with ttf-parser
        let font = ttf_parser::Face::parse(font_data.as_ref().as_ref(), 0)
            .map_err(|_| RenderError::FontError("Failed to parse font".to_string()))?;

        let mut paths = Vec::new();
        let mut accumulated_spacing = 0.0;

        // Render each glyph
        for (i, glyph) in shaped.glyphs.iter().enumerate() {
            let size_fixed = (shaped.font_size * 65536.0) as u32; // Convert to 16.16 fixed point
            let key = GlyphKey {
                font_id,
                glyph_id: glyph.glyph_id as u32,
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
            if let Some(bbox) = font.glyph_bounding_box(glyph_id) {
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
