//! Font metrics handling for libass/VSFilter compatibility

use crate::utils::RenderError;
use ttf_parser::Face;

/// Font metrics with VSFilter compatibility
#[derive(Debug, Clone)]
pub struct FontMetrics {
    /// Ascender value
    pub ascender: f32,
    /// Descender value (negative)
    pub descender: f32,
    /// Line gap
    pub line_gap: f32,
    /// Units per em
    pub units_per_em: f32,
    /// Whether metrics are from OS/2 table
    pub uses_os2: bool,
}

impl FontMetrics {
    /// Get font metrics with VSFilter compatibility
    ///
    /// Libass uses OS/2 table's usWinAscent/usWinDescent for compatibility with VSFilter.
    /// If OS/2 table is not available, falls back to hhea table.
    pub fn from_face(face: &Face) -> Self {
        let units_per_em = face.units_per_em() as f32;

        // Try to get OS/2 table metrics first (for VSFilter compatibility)
        if let Some(os2) = face.tables().os2 {
            // Use typographic metrics from OS/2 table if available
            // Note: ttf-parser's typographic_ascender/descender return i16 directly
            // We'll use these if they're non-zero, otherwise fall back to regular metrics
            let typographic_ascender = os2.typographic_ascender();
            let typographic_descender = os2.typographic_descender();

            let ascender = if typographic_ascender != 0 {
                typographic_ascender as f32
            } else {
                face.ascender() as f32
            };

            let descender = if typographic_descender != 0 {
                typographic_descender as f32
            } else {
                face.descender() as f32
            };

            return FontMetrics {
                ascender,
                descender,
                line_gap: 0.0, // OS/2 doesn't have line gap in Windows metrics
                units_per_em,
                uses_os2: true,
            };
        }

        // Fall back to hhea table metrics
        let ascender = face.ascender() as f32;
        let descender = face.descender() as f32;
        let line_gap = face.line_gap() as f32;

        FontMetrics {
            ascender,
            descender,
            line_gap,
            units_per_em,
            uses_os2: false,
        }
    }

    /// Calculate line height at given font size
    /// Match libass behavior: use font size directly as the line height
    pub fn line_height(&self, font_size: f32) -> f32 {
        // Libass uses the font size directly for line height calculations
        // rather than calculating from metrics
        font_size
    }

    /// Calculate baseline offset at given font size
    pub fn baseline(&self, font_size: f32) -> f32 {
        // Calculate baseline as a proportion of font size
        // Most fonts have baseline at about 80% of font size from top
        let scale = font_size / self.units_per_em;
        self.ascender * scale
    }

    /// Apply font spacing (letter spacing) to advance width
    pub fn apply_spacing(advance: f32, spacing: f32, font_size: f32) -> f32 {
        // ASS font spacing is in pixels, added to each character's advance
        advance + spacing
    }
}
