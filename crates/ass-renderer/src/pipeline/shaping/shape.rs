//! Core text shaping into glyphs using rustybuzz.

#[cfg(feature = "nostd")]
use alloc::{string::ToString, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::ToString, vec::Vec};

use crate::utils::RenderError;
use fontdb::Database as FontDatabase;
use rustybuzz::{Face, Feature, UnicodeBuffer, Variation};

use super::font_select::find_font_for_text;
use super::{FontMetrics, ShapedGlyph, ShapedText};

/// Shape text into glyphs
pub fn shape_text(
    text: &str,
    font_family: &str,
    font_size: f32,
    font_database: &FontDatabase,
) -> Result<ShapedText, RenderError> {
    shape_text_with_style(text, font_family, font_size, false, false, font_database)
}

/// Shape text with style, caching the result per thread keyed by
/// (text, font, size, bold, italic).
///
/// Avoids re-shaping the same run within a frame — the pipeline shapes for layout
/// and the backend re-shapes for rasterization — and across frames for content
/// whose glyphs do not change (e.g. transform-animated or karaoke text). The
/// shaped result depends only on the font outlines, so it is valid across the
/// pipeline's and backend's (equivalent) system-font databases.
pub fn shape_text_cached(
    text: &str,
    font_family: &str,
    font_size: f32,
    bold: bool,
    italic: bool,
    font_database: &FontDatabase,
) -> Result<ShapedText, RenderError> {
    #[cfg(not(feature = "nostd"))]
    {
        use std::cell::RefCell;
        use std::collections::HashMap;
        // (text, font family, font size bits, bold, italic)
        type ShapeCacheKey = (String, String, u32, bool, bool);
        thread_local! {
            static CACHE: RefCell<HashMap<ShapeCacheKey, ShapedText>> = RefCell::new(HashMap::new());
        }
        let key = (
            text.to_string(),
            font_family.to_string(),
            font_size.to_bits(),
            bold,
            italic,
        );
        if let Some(shaped) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
            return Ok(shaped);
        }
        let shaped =
            shape_text_with_style(text, font_family, font_size, bold, italic, font_database)?;
        CACHE.with(|c| {
            let mut map = c.borrow_mut();
            // Bound memory: drop the whole cache if it grows large (re-shaping a
            // cold entry is cheap next to the hot-path savings).
            if map.len() >= 8192 {
                map.clear();
            }
            map.insert(key, shaped.clone());
        });
        Ok(shaped)
    }
    #[cfg(feature = "nostd")]
    {
        shape_text_with_style(text, font_family, font_size, bold, italic, font_database)
    }
}

/// Read a font file once and cache its bytes (per thread), so repeated shaping of
/// the same font does not re-read it from disk on every call.
#[cfg(not(feature = "nostd"))]
fn cached_font_file(
    path: &std::path::Path,
) -> Result<std::sync::Arc<dyn AsRef<[u8]> + Send + Sync>, RenderError> {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;

    thread_local! {
        static CACHE: RefCell<HashMap<PathBuf, Arc<Vec<u8>>>> = RefCell::new(HashMap::new());
    }

    CACHE.with(|cache| {
        // Clone-and-drop the borrow before any borrow_mut below.
        let cached = cache.borrow().get(path).cloned();
        let data = if let Some(data) = cached {
            data
        } else {
            let bytes = std::fs::read(path)
                .map_err(|e| RenderError::FontError(format!("Failed to read font file: {e}")))?;
            let data = Arc::new(bytes);
            cache.borrow_mut().insert(path.to_path_buf(), data.clone());
            data
        };
        let shared: Arc<dyn AsRef<[u8]> + Send + Sync> = data;
        Ok(shared)
    })
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

    // Extract data based on source type. File sources are read from disk once and
    // cached (per thread) — re-reading the font file on every shape call was the
    // dominant per-frame cost.
    let font_data = match source {
        fontdb::Source::Binary(data) | fontdb::Source::SharedFile(_, data) => data,
        fontdb::Source::File(path) => {
            #[cfg(not(feature = "nostd"))]
            {
                cached_font_file(&path)?
            }
            #[cfg(feature = "nostd")]
            {
                let _ = path;
                return Err(RenderError::FontError(
                    "File reading not supported in no_std mode".into(),
                ));
            }
        }
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

    // Calculate metrics using VSFilter-compatible values. `height` is the font's
    // ascent+descent box (Windows metrics) used for vertical centering, matching
    // libass; the line-to-line ADVANCE is computed separately in the pipeline.
    let baseline = metrics.baseline(font_size);
    let ascent = metrics.ascender * scale;
    let descent = metrics.descender * scale;
    let height = ascent - descent;

    // Ink extents: the union of each glyph's outline bbox placed at its pen
    // position. libass wraps on ink width (x_max - x_min), which is narrower than
    // the advance sum by the leading/trailing side bearings — enough to flip a
    // borderline line's break count.
    let (mut ink_min, mut ink_max) = (f32::INFINITY, f32::NEG_INFINITY);
    for glyph in &glyphs {
        if let Some(bbox) = ttf_face.glyph_bounding_box(ttf_parser::GlyphId(glyph.glyph_id as u16))
        {
            ink_min = ink_min.min(glyph.x_position + f32::from(bbox.x_min) * scale);
            ink_max = ink_max.max(glyph.x_position + f32::from(bbox.x_max) * scale);
        }
    }
    if ink_min > ink_max {
        // No inked glyphs (e.g. all spaces): fall back to the advance box.
        ink_min = 0.0;
        ink_max = cursor_x;
    }

    Ok(ShapedText {
        width: cursor_x,
        height,
        baseline,
        font_size,
        glyphs,
        ascent,
        descent,
        ink_min,
        ink_max,
    })
}
