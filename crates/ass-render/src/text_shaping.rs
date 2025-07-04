//! Text shaping module using rustybuzz for proper complex text layout
//! 
//! This module provides comprehensive text shaping capabilities including:
//! - Complex script support (Arabic, Hebrew, Indic, etc.)
//! - Proper bidirectional text handling
//! - Advanced typography features
//! - Font fallback mechanism
//! - Multi-language text analysis

#[cfg(feature = "hardware")]
use std::collections::HashMap;

#[cfg(feature = "hardware")]
use rustybuzz::{BufferFlags, Direction, Face, Language, Script, UnicodeBuffer, Feature};
#[cfg(feature = "hardware")]
use ttf_parser::Face as TtfFace;

/// Shaped glyph information
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    pub glyph_id: u32,
    pub codepoint: u32,
    pub cluster: u32,
    pub x_advance: i32,
    pub y_advance: i32,
    pub x_offset: i32,
    pub y_offset: i32,
}

/// Text shaping result
#[derive(Debug, Clone)]
pub struct ShapedText {
    pub glyphs: Vec<ShapedGlyph>,
    pub total_advance: f32,
    pub line_height: f32,
}

/// Text shaping engine with font management
pub struct TextShaper {
    #[cfg(feature = "hardware")]
    faces: HashMap<String, Face<'static>>,
    #[cfg(feature = "hardware")]
    fallback_fonts: Vec<String>,
    #[cfg(feature = "hardware")]
    script_font_mapping: HashMap<Script, String>,
    font_size: f32,
    dpi: f32,
    #[cfg(feature = "hardware")]
    features: Vec<Feature>,
}

impl TextShaper {
    /// Create a new text shaper
    pub fn new() -> Self {
        #[cfg(feature = "hardware")]
        let mut features = Vec::new();
        #[cfg(feature = "hardware")]
        {
            // Enable common OpenType features
            features.push(Feature::new(
                rustybuzz::Tag::from_bytes(b"kern"),
                1,
                ..
            ));
            features.push(Feature::new(
                rustybuzz::Tag::from_bytes(b"liga"),
                1,
                ..
            ));
        }

        Self {
            #[cfg(feature = "hardware")]
            faces: HashMap::new(),
            #[cfg(feature = "hardware")]
            fallback_fonts: Vec::new(),
            #[cfg(feature = "hardware")]
            script_font_mapping: HashMap::new(),
            font_size: 16.0,
            dpi: 96.0,
            #[cfg(feature = "hardware")]
            features,
        }
    }

    /// Add a font to the shaper
    #[cfg(feature = "hardware")]
    pub fn add_font(&mut self, name: String, font_data: Vec<u8>) -> Result<(), TextShapingError> {
        let font_data: &'static [u8] = Box::leak(font_data.into_boxed_slice());
        let face = Face::from_slice(font_data, 0).ok_or(TextShapingError::InvalidFont)?;
        self.faces.insert(name.clone(), face);
        
        // Auto-configure font for specific scripts if it supports them
        self.auto_configure_font_scripts(&name);
        
        Ok(())
    }

    /// Add a fallback font chain
    #[cfg(feature = "hardware")]
    pub fn add_fallback_font(&mut self, font_name: String) {
        if !self.fallback_fonts.contains(&font_name) {
            self.fallback_fonts.push(font_name);
        }
    }

    /// Map a script to a specific font
    #[cfg(feature = "hardware")]
    pub fn set_script_font(&mut self, script: Script, font_name: String) {
        self.script_font_mapping.insert(script, font_name);
    }

    /// Auto-configure font for scripts based on available character coverage
    #[cfg(feature = "hardware")]
    fn auto_configure_font_scripts(&mut self, font_name: &str) {
        if let Some(face) = self.faces.get(font_name) {
            // Check if font supports common script ranges
            let scripts_to_check = [
                (Script::Latin, 0x0041..=0x007A),     // A-Z range
                (Script::Arabic, 0x0600..=0x06FF),    // Arabic block
                (Script::Hebrew, 0x0590..=0x05FF),    // Hebrew block
                (Script::Cyrillic, 0x0400..=0x04FF),  // Cyrillic block
                (Script::Hiragana, 0x3040..=0x309F),  // Hiragana block
                (Script::Katakana, 0x30A0..=0x30FF),  // Katakana block
                (Script::HangulSyllables, 0xAC00..=0xD7AF), // Hangul syllables
            ];

            for (script, range) in scripts_to_check {
                // Sample a few characters from the range to test coverage
                let test_chars: Vec<u32> = range.step_by(100).take(5).collect();
                let coverage = test_chars.iter()
                    .filter(|&&codepoint| face.glyph_index(char::from_u32(codepoint).unwrap_or('\0')).is_some())
                    .count();
                
                // If font covers most test characters, map it for this script
                if coverage >= test_chars.len() / 2 {
                    self.script_font_mapping.insert(script, font_name.to_string());
                }
            }
        }
    }

    /// Set font size
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }

    /// Set DPI
    pub fn set_dpi(&mut self, dpi: f32) {
        self.dpi = dpi;
    }

    /// Shape text using the specified font with automatic script detection and fallback
    #[cfg(feature = "hardware")]
    pub fn shape_text(
        &self,
        text: &str,
        font_name: &str,
        direction: TextDirection,
    ) -> Result<ShapedText, TextShapingError> {
        // Detect script and adjust font if needed
        let (detected_script, detected_direction) = self.detect_text_properties(text);
        
        // Choose the best font for this script
        let chosen_font = self.choose_font_for_script(font_name, detected_script)?;
        
        let face = self
            .faces
            .get(&chosen_font)
            .ok_or(TextShapingError::FontNotFound)?;

        // Create a buffer and add text
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(text);

        // Set text properties based on detection or user preference
        let final_direction = match direction {
            TextDirection::LeftToRight => Direction::Ltr,
            TextDirection::RightToLeft => Direction::Rtl,
            TextDirection::TopToBottom => Direction::Ttb,
            TextDirection::BottomToTop => Direction::Btt,
        };

        buffer.set_direction(final_direction);
        buffer.set_script(detected_script);
        
        // Set language based on script
        let language = self.get_language_for_script(detected_script);
        buffer.set_language(language);
        buffer.set_flags(BufferFlags::DEFAULT);

        // Perform shaping with OpenType features
        let glyph_buffer = rustybuzz::shape(face, &self.features, buffer);

        // Extract glyph information
        let glyph_infos = glyph_buffer.glyph_infos();
        let glyph_positions = glyph_buffer.glyph_positions();

        let mut glyphs = Vec::new();
        let mut total_advance = 0.0;

        for (info, pos) in glyph_infos.iter().zip(glyph_positions.iter()) {
            glyphs.push(ShapedGlyph {
                glyph_id: info.glyph_id,
                codepoint: info.codepoint,
                cluster: info.cluster,
                x_advance: pos.x_advance,
                y_advance: pos.y_advance,
                x_offset: pos.x_offset,
                y_offset: pos.y_offset,
            });

            total_advance += pos.x_advance as f32;
        }

        // Convert advances from font units to pixels
        let units_per_em = face.units_per_em() as f32;
        let scale = self.font_size * self.dpi / (72.0 * units_per_em);

        for glyph in &mut glyphs {
            glyph.x_advance = (glyph.x_advance as f32 * scale) as i32;
            glyph.y_advance = (glyph.y_advance as f32 * scale) as i32;
            glyph.x_offset = (glyph.x_offset as f32 * scale) as i32;
            glyph.y_offset = (glyph.y_offset as f32 * scale) as i32;
        }

        total_advance *= scale;

        // Calculate line height from font metrics
        let line_height = self.calculate_line_height(face);

        Ok(ShapedText {
            glyphs,
            total_advance,
            line_height,
        })
    }

    /// Choose the best font for a given script
    #[cfg(feature = "hardware")]
    fn choose_font_for_script(&self, preferred_font: &str, script: Script) -> Result<String, TextShapingError> {
        // First try the preferred font
        if self.faces.contains_key(preferred_font) {
            return Ok(preferred_font.to_string());
        }
        
        // Try script-specific mapping
        if let Some(script_font) = self.script_font_mapping.get(&script) {
            if self.faces.contains_key(script_font) {
                return Ok(script_font.clone());
            }
        }
        
        // Try fallback fonts
        for fallback in &self.fallback_fonts {
            if self.faces.contains_key(fallback) {
                return Ok(fallback.clone());
            }
        }
        
        // Use first available font as last resort
        if let Some(first_font) = self.faces.keys().next() {
            return Ok(first_font.clone());
        }
        
        Err(TextShapingError::FontNotFound)
    }

    /// Get appropriate language for script
    #[cfg(feature = "hardware")]
    fn get_language_for_script(&self, script: Script) -> Language {
        match script {
            Script::Arabic => Language::from_string("ar"),
            Script::Hebrew => Language::from_string("he"),
            Script::Cyrillic => Language::from_string("ru"),
            Script::Hiragana | Script::Katakana => Language::from_string("ja"),
            Script::HangulSyllables => Language::from_string("ko"),
            Script::Devanagari => Language::from_string("hi"),
            Script::Bengali => Language::from_string("bn"),
            Script::Tamil => Language::from_string("ta"),
            Script::Thai => Language::from_string("th"),
            _ => Language::from_string("en"),
        }
    }

    /// Calculate line height from font metrics
    #[cfg(feature = "hardware")]
    fn calculate_line_height(&self, face: &Face) -> f32 {
        let units_per_em = face.units_per_em() as f32;
        let scale = self.font_size * self.dpi / (72.0 * units_per_em);
        
        let ascender = face.ascender() as f32 * scale;
        let descender = face.descender() as f32 * scale;
        let line_gap = face.line_gap() as f32 * scale;
        
        ascender - descender + line_gap
    }

    /// Fallback text shaping without rustybuzz (for software renderer)
    #[cfg(not(feature = "hardware"))]
    pub fn shape_text(
        &self,
        text: &str,
        _font_name: &str,
        _direction: TextDirection,
    ) -> Result<ShapedText, TextShapingError> {
        // Simple fallback implementation for when rustybuzz is not available
        let glyphs: Vec<ShapedGlyph> = text
            .chars()
            .enumerate()
            .map(|(i, ch)| {
                ShapedGlyph {
                    glyph_id: ch as u32,
                    codepoint: ch as u32,
                    cluster: i as u32,
                    x_advance: (self.font_size * 0.6) as i32, // Rough character width
                    y_advance: 0,
                    x_offset: 0,
                    y_offset: 0,
                }
            })
            .collect();

        let total_advance = glyphs.len() as f32 * self.font_size * 0.6;

        Ok(ShapedText {
            glyphs,
            total_advance,
            line_height: self.font_size * 1.2,
        })
    }

    /// Get font metrics
    #[cfg(feature = "hardware")]
    pub fn get_font_metrics(&self, font_name: &str) -> Result<FontMetrics, TextShapingError> {
        let face = self
            .faces
            .get(font_name)
            .ok_or(TextShapingError::FontNotFound)?;

        let units_per_em = face.units_per_em() as f32;
        let scale = self.font_size * self.dpi / (72.0 * units_per_em);

        // Try to get OS/2 table for better metrics
        let ascender = face.ascender() as f32 * scale;
        let descender = face.descender() as f32 * scale;
        let line_gap = face.line_gap() as f32 * scale;

        Ok(FontMetrics {
            ascender,
            descender,
            line_gap,
            line_height: ascender - descender + line_gap,
            units_per_em: units_per_em as u16,
        })
    }

    /// Fallback font metrics
    #[cfg(not(feature = "hardware"))]
    pub fn get_font_metrics(&self, _font_name: &str) -> Result<FontMetrics, TextShapingError> {
        Ok(FontMetrics {
            ascender: self.font_size * 0.8,
            descender: -self.font_size * 0.2,
            line_gap: self.font_size * 0.2,
            line_height: self.font_size * 1.2,
            units_per_em: 1000,
        })
    }

    /// Detect script and direction from text with sophisticated analysis
    #[cfg(feature = "hardware")]
    pub fn detect_text_properties(&self, text: &str) -> (Script, Direction) {
        let mut script_counts = HashMap::new();
        let mut has_rtl = false;
        let mut has_ltr = false;

        for ch in text.chars() {
            let script = self.detect_char_script(ch);
            *script_counts.entry(script).or_insert(0) += 1;

            // Check directionality
            match self.get_char_direction(ch) {
                Direction::Rtl => has_rtl = true,
                Direction::Ltr => has_ltr = true,
                _ => {}
            }
        }

        // Find the most common script (excluding Common and Latin for mixed text)
        let dominant_script = script_counts
            .iter()
            .filter(|(&script, _)| script != Script::Common && script != Script::Latin)
            .max_by_key(|(_, &count)| count)
            .map(|(&script, _)| script)
            .unwrap_or_else(|| {
                // Fallback to most common script including Latin
                script_counts
                    .iter()
                    .max_by_key(|(_, &count)| count)
                    .map(|(&script, _)| script)
                    .unwrap_or(Script::Latin)
            });

        // Determine direction based on script and detected directionality
        let direction = match dominant_script {
            Script::Arabic | Script::Hebrew => Direction::Rtl,
            Script::Mongolian => Direction::Ttb,
            _ => {
                if has_rtl && !has_ltr {
                    Direction::Rtl
                } else {
                    Direction::Ltr
                }
            }
        };

        (dominant_script, direction)
    }

    /// Detect the script of a single character
    #[cfg(feature = "hardware")]
    fn detect_char_script(&self, ch: char) -> Script {
        match ch as u32 {
            // Latin
            0x0041..=0x007A | 0x00C0..=0x024F => Script::Latin,
            // Arabic
            0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF => Script::Arabic,
            // Hebrew
            0x0590..=0x05FF => Script::Hebrew,
            // Cyrillic
            0x0400..=0x04FF | 0x0500..=0x052F => Script::Cyrillic,
            // Greek
            0x0370..=0x03FF | 0x1F00..=0x1FFF => Script::Greek,
            // Hiragana
            0x3040..=0x309F => Script::Hiragana,
            // Katakana
            0x30A0..=0x30FF => Script::Katakana,
            // CJK Unified Ideographs
            0x4E00..=0x9FFF => Script::Han,
            // Hangul
            0xAC00..=0xD7AF => Script::HangulSyllables,
            // Devanagari
            0x0900..=0x097F => Script::Devanagari,
            // Bengali
            0x0980..=0x09FF => Script::Bengali,
            // Tamil
            0x0B80..=0x0BFF => Script::Tamil,
            // Thai
            0x0E00..=0x0E7F => Script::Thai,
            // Myanmar
            0x1000..=0x109F => Script::Myanmar,
            // Ethiopic
            0x1200..=0x137F => Script::Ethiopic,
            // Cherokee
            0x13A0..=0x13FF => Script::Cherokee,
            // Mongolian
            0x1800..=0x18AF => Script::Mongolian,
            // Default to Common for punctuation, numbers, etc.
            _ => Script::Common,
        }
    }

    /// Get the directionality of a character
    #[cfg(feature = "hardware")]
    fn get_char_direction(&self, ch: char) -> Direction {
        match ch as u32 {
            // Arabic, Hebrew (RTL)
            0x0590..=0x05FF | 0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF => Direction::Rtl,
            // Mongolian (TTB)
            0x1800..=0x18AF => Direction::Ttb,
            // Most other scripts (LTR)
            _ => Direction::Ltr,
        }
    }
}

impl Default for TextShaper {
    fn default() -> Self {
        Self::new()
    }
}

/// Text direction for shaping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

/// Font metrics information
#[derive(Debug, Clone)]
pub struct FontMetrics {
    pub ascender: f32,
    pub descender: f32,
    pub line_gap: f32,
    pub line_height: f32,
    pub units_per_em: u16,
}

/// Text shaping errors
#[derive(Debug, Clone, PartialEq)]
pub enum TextShapingError {
    FontNotFound,
    InvalidFont,
    ShapingFailed,
    UnsupportedScript,
}

impl std::fmt::Display for TextShapingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextShapingError::FontNotFound => write!(f, "Font not found"),
            TextShapingError::InvalidFont => write!(f, "Invalid font data"),
            TextShapingError::ShapingFailed => write!(f, "Text shaping failed"),
            TextShapingError::UnsupportedScript => write!(f, "Unsupported script"),
        }
    }
}

impl std::error::Error for TextShapingError {}

/// Layout engine for multi-line text with proper line breaking
pub struct TextLayout {
    shaper: TextShaper,
    max_width: f32,
    line_spacing: f32,
}

impl TextLayout {
    pub fn new(shaper: TextShaper) -> Self {
        Self {
            shaper,
            max_width: f32::INFINITY,
            line_spacing: 1.0,
        }
    }

    pub fn set_max_width(&mut self, width: f32) {
        self.max_width = width;
    }

    pub fn set_line_spacing(&mut self, spacing: f32) {
        self.line_spacing = spacing;
    }

    /// Layout text into multiple lines with proper word wrapping
    pub fn layout_text(
        &self,
        text: &str,
        font_name: &str,
        direction: TextDirection,
    ) -> Result<Vec<ShapedText>, TextShapingError> {
        if self.max_width.is_infinite() {
            // Single line layout
            return Ok(vec![self.shaper.shape_text(text, font_name, direction)?]);
        }

        let mut lines = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line = String::new();

        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            let shaped = self.shaper.shape_text(&test_line, font_name, direction)?;

            if shaped.total_advance <= self.max_width {
                current_line = test_line;
            } else {
                // Finish current line and start new one
                if !current_line.is_empty() {
                    lines.push(
                        self.shaper
                            .shape_text(&current_line, font_name, direction)?,
                    );
                }
                current_line = word.to_string();
            }
        }

        // Add the last line
        if !current_line.is_empty() {
            lines.push(
                self.shaper
                    .shape_text(&current_line, font_name, direction)?,
            );
        }

        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "hardware")]
    fn test_text_direction_detection() {
        let shaper = TextShaper::new();

        let (script, direction) = shaper.detect_text_properties("Hello World");
        assert_eq!(script, Script::Latin);
        assert_eq!(direction, Direction::Ltr);

        let (script, direction) = shaper.detect_text_properties("مرحبا بالعالم");
        assert_eq!(script, Script::Arabic);
        assert_eq!(direction, Direction::Rtl);
    }

    #[test]
    fn test_fallback_shaping() {
        let shaper = TextShaper::new();
        let result = shaper.shape_text("Hello", "Arial", TextDirection::LeftToRight);
        assert!(result.is_ok());

        let shaped = result.unwrap();
        assert_eq!(shaped.glyphs.len(), 5);
        assert!(shaped.total_advance > 0.0);
    }

    #[test]
    fn test_text_layout() {
        let shaper = TextShaper::new();
        let mut layout = TextLayout::new(shaper);
        layout.set_max_width(100.0);

        let result = layout.layout_text(
            "This is a long text that should wrap",
            "Arial",
            TextDirection::LeftToRight,
        );
        assert!(result.is_ok());

        let lines = result.unwrap();
        assert!(lines.len() > 1); // Should wrap into multiple lines
    }
}
