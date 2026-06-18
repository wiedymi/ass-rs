//! Owned style storage plus ASS colour/alpha parsing for the software pipeline.

#[cfg(feature = "nostd")]
use alloc::string::{String, ToString};
#[cfg(not(feature = "nostd"))]
use std::string::{String, ToString};

use ass_core::parser::Style;

/// Owned style for storing in pipeline
#[derive(Clone)]
pub(super) struct OwnedStyle {
    #[allow(dead_code)] // Style identifier - stored for completeness
    name: String,
    pub(super) fontname: String,
    pub(super) fontsize: String,
    pub(super) primary_colour: String,
    pub(super) secondary_colour: String,
    pub(super) outline_colour: String,
    pub(super) back_colour: String,
    pub(super) bold: String,
    pub(super) italic: String,
    pub(super) underline: String,
    pub(super) strikeout: String,
    pub(super) scale_x: String,
    pub(super) scale_y: String,
    pub(super) spacing: String,
    #[allow(dead_code)] // Text rotation angle - stored for completeness
    angle: String,
    #[allow(dead_code)] // Border rendering style - stored for completeness
    pub(super) border_style: String,
    pub(super) outline: String,
    pub(super) shadow: String,
    pub(super) alignment: String,
    pub(super) margin_l: String,
    pub(super) margin_r: String,
    pub(super) margin_v: String,
    #[allow(dead_code)] // Text encoding specification - stored for completeness
    encoding: String,
}

impl OwnedStyle {
    pub(super) fn from_style(style: &Style) -> Self {
        Self {
            name: style.name.to_string(),
            fontname: style.fontname.to_string(),
            fontsize: style.fontsize.to_string(),
            primary_colour: style.primary_colour.to_string(),
            secondary_colour: style.secondary_colour.to_string(),
            outline_colour: style.outline_colour.to_string(),
            back_colour: style.back_colour.to_string(),
            bold: style.bold.to_string(),
            italic: style.italic.to_string(),
            underline: style.underline.to_string(),
            strikeout: style.strikeout.to_string(),
            scale_x: style.scale_x.to_string(),
            scale_y: style.scale_y.to_string(),
            spacing: style.spacing.to_string(),
            angle: style.angle.to_string(),
            border_style: style.border_style.to_string(),
            outline: style.outline.to_string(),
            shadow: style.shadow.to_string(),
            alignment: style.alignment.to_string(),
            margin_l: style.margin_l.to_string(),
            margin_r: style.margin_r.to_string(),
            margin_v: style.margin_v.to_string(),
            encoding: style.encoding.to_string(),
        }
    }
}

impl super::SoftwarePipeline {
    /// Get style by name from the stored styles
    #[allow(dead_code)] // Utility method for style lookup
    fn get_style(&self, style_name: &str) -> Option<&OwnedStyle> {
        self.styles_map
            .get(style_name)
            .or(self.default_style.as_ref())
    }

    /// Parse color from ASS format
    pub(super) fn parse_ass_color(color: &str) -> [u8; 4] {
        // ASS colors are in &HAABBGGRR format where:
        // AA = Alpha (00 = opaque, FF = transparent)
        // BB = Blue, GG = Green, RR = Red
        let color_trimmed = color.trim_end_matches('&');
        if let Some(hex) = color_trimmed.strip_prefix("&H") {
            if let Ok(value) = u32::from_str_radix(hex, 16) {
                // Check if this is AABBGGRR (8 hex digits) or BBGGRR (6 hex digits)
                let (alpha, bgr_value) = if hex.len() >= 8 {
                    // Full AABBGGRR format
                    let alpha = ((value >> 24) & 0xFF) as u8;
                    // Convert ASS alpha (00=opaque, FF=transparent) to RGBA (00=transparent, FF=opaque)
                    let rgba_alpha = 255 - alpha;
                    (rgba_alpha, value & 0xFFFFFF)
                } else {
                    // Legacy BBGGRR format without alpha, assume opaque
                    (255u8, value)
                };

                // Extract colors in BGR order
                let r = (bgr_value & 0xFF) as u8; // Last 2 hex digits = Red
                let g = ((bgr_value >> 8) & 0xFF) as u8; // Middle 2 hex digits = Green
                let b = ((bgr_value >> 16) & 0xFF) as u8; // First 2 hex digits = Blue

                // Return in RGBA order for rendering
                return [r, g, b, alpha];
            }
        }
        [255, 255, 255, 255] // Default white, opaque
    }

    /// Parse alpha from ASS format
    #[allow(dead_code)] // Utility for parsing ASS alpha values
    fn parse_ass_alpha(alpha: &str) -> u8 {
        if let Some(hex) = alpha.strip_prefix("&H") {
            if let Ok(value) = u8::from_str_radix(hex, 16) {
                // ASS alpha is inverted (00 = opaque, FF = transparent)
                return 255 - value;
            }
        }
        255 // Default opaque
    }
}
