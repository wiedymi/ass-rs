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
    // Find best matching font, taking the input text into account for coverage (CJK/Hangul, etc.)
    let font_id = find_font_for_text(font_database, font_family, bold, italic, text)?;

    // Get font data using face_source
    let (source, index) = font_database
        .face_source(font_id)
        .ok_or_else(|| RenderError::FontError("Failed to load font data".to_string()))?;

    // Extract data based on source type
    let font_data = match source {
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

    // Create rustybuzz face
    let rb_face = Face::from_slice(font_data.as_ref().as_ref(), index)
        .ok_or_else(|| RenderError::FontError("Failed to create font face".to_string()))?;

    // Parse with ttf-parser for OS/2 table access
    let ttf_face = ttf_parser::Face::parse(font_data.as_ref().as_ref(), index)
        .map_err(|_| RenderError::FontError("Failed to parse font for metrics".to_string()))?;

    // Get font metrics with VSFilter compatibility
    let metrics = FontMetrics::from_face(&ttf_face);

    // Create buffer and add text
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);

    // Shape the text
    let features: Vec<Feature> = Vec::new();
    let _variations: Vec<Variation> = Vec::new();
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

/// Find matching font in database with fallback support for CJK/Chinese/Japanese/Korean text
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
                println!("Using Japanese font fallback: {family} -> {fallback}");
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
            "Font '{family}' not found and no fallback available"
        ))
    })
}

/// Determine if text contains significant CJK/Hangul content
fn text_has_cjk_or_hangul(text: &str) -> bool {
    text.chars().any(|ch| {
        let u = ch as u32;
        // CJK Unified Ideographs
        (0x4E00..=0x9FFF).contains(&u)
            // CJK Unified Ideographs Extension A/B (basic check)
            || (0x3400..=0x4DBF).contains(&u)
            || (0x20000..=0x2A6DF).contains(&u)
            // Hangul Syllables + Jamo
            || (0xAC00..=0xD7AF).contains(&u)
            || (0x1100..=0x11FF).contains(&u)
            // Hiragana, Katakana
            || (0x3040..=0x30FF).contains(&u)
    })
}

/// Check how many characters of `text` are supported by the font face
fn font_support_count(
    font_database: &FontDatabase,
    font_id: FontId,
    text: &str,
) -> Result<usize, RenderError> {
    let (source, index) = font_database
        .face_source(font_id)
        .ok_or_else(|| RenderError::FontError("Failed to load font data".to_string()))?;

    let data: std::sync::Arc<dyn AsRef<[u8]> + Send + Sync> = match source {
        fontdb::Source::Binary(data) => data,
        fontdb::Source::File(path) => {
            #[cfg(not(feature = "nostd"))]
            {
                std::sync::Arc::new(std::fs::read(&path).map_err(|e| {
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

    let face = ttf_parser::Face::parse(data.as_ref().as_ref(), index)
        .map_err(|_| RenderError::FontError("Failed to parse font".to_string()))?;

    // Count support for BMP chars in text
    let mut count = 0usize;
    for ch in text.chars() {
        if face.glyph_index(ch).is_some() {
            count += 1;
        }
    }
    Ok(count)
}

/// Attempt to pick a font that best covers the input text while respecting the requested family if possible
fn find_font_for_text(
    font_database: &FontDatabase,
    family: &str,
    bold: bool,
    italic: bool,
    text: &str,
) -> Result<FontId, RenderError> {
    // First, try the requested font (or its immediate fallback in find_font)
    let primary = find_font(font_database, family, bold, italic)?;
    let primary_support = font_support_count(font_database, primary, text)?;
    let text_len = text.chars().count().max(1);

    if primary_support == text_len {
        return Ok(primary);
    }

    // If the text contains CJK/Hangul, try curated fallback lists first
    if text_has_cjk_or_hangul(text) {
        let curated_lists: &[&[&str]] = &[
            // Simplified Chinese
            &[
                "Noto Sans CJK SC",
                "Noto Sans SC",
                "Source Han Sans CN",
                "PingFang SC",
                "Microsoft YaHei",
                "SimHei",
                "SimSun",
            ],
            // Traditional Chinese
            &[
                "Noto Sans CJK TC",
                "Noto Sans TC",
                "Source Han Sans TW",
                "PingFang TC",
                "Heiti TC",
            ],
            // Japanese
            &[
                "Noto Sans CJK JP",
                "Noto Sans JP",
                "Hiragino Sans",
                "Hiragino Kaku Gothic Pro",
                "Yu Gothic",
                "MS Gothic",
                "Meiryo",
            ],
            // Korean
            &[
                "Noto Sans CJK KR",
                "Noto Sans KR",
                "Apple SD Gothic Neo",
                "Malgun Gothic",
            ],
            // Generic CJK
            &[
                "Source Han Sans",
                "Noto Sans CJK",
            ],
        ];

        let mut best_font = None;
        let mut best_support = primary_support;

        for list in curated_lists {
            for &name in *list {
                let query = fontdb::Query {
                    families: &[fontdb::Family::Name(name)],
                    weight: if bold { fontdb::Weight::BOLD } else { fontdb::Weight::NORMAL },
                    stretch: fontdb::Stretch::Normal,
                    style: if italic { fontdb::Style::Italic } else { fontdb::Style::Normal },
                };
                if let Some(id) = font_database.query(&query) {
                    let support = font_support_count(font_database, id, text)?;
                    if support == text_len {
                        return Ok(id);
                    }
                    if support > best_support {
                        best_support = support;
                        best_font = Some(id);
                    }
                }
            }
        }

        if let Some(id) = best_font {
            return Ok(id);
        }
    }

    // As a last resort, scan all faces to find the best coverage
    let mut best_font = primary;
    let mut best_support = primary_support;
    for face in font_database.faces() {
        // Skip if it's the same font
        if face.id == primary {
            continue;
        }
        let id = face.id;
        let support = font_support_count(font_database, id, text)?;
        if support > best_support {
            best_support = support;
            best_font = id;
            if best_support == text_len {
                break;
            }
        }
    }

    Ok(best_font)
}

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
