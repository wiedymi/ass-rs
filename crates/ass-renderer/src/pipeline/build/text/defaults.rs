//! Style-derived defaults (font, size, colours, formatting) resolved once per
//! event for the software pipeline.

use super::super::OwnedStyle;
use super::types::TextDefaults;

impl super::super::SoftwarePipeline {
    /// Resolve the style-derived defaults (font, size, colours, and formatting)
    /// shared by every segment of the event.
    pub(super) fn resolve_text_defaults<'a>(
        &self,
        style: Option<&'a OwnedStyle>,
        scale_y: f32,
    ) -> TextDefaults<'a> {
        // Get base style properties
        let default_font_name = style.map(|s| s.fontname.as_ref()).unwrap_or("Arial");
        let default_font_size_base = style
            .map(|s| s.fontsize.parse::<f32>().unwrap_or(48.0))
            .unwrap_or(48.0);
        // Font sizes in ASS are already in the script resolution coordinate system
        // They need to be scaled according to the PlayResY to output resolution ratio
        // This matches libass behavior
        // Also apply DPI scale to match libass (72 DPI vs 96 DPI)
        let _default_font_size = default_font_size_base * scale_y * self.dpi_scale;
        // In ASS format: -1 = true (bold/italic), 0 = false
        let default_bold = style.map(|s| s.bold == "-1").unwrap_or(false);
        let default_italic = style.map(|s| s.italic == "-1").unwrap_or(false);
        let default_underline = style.map(|s| s.underline == "-1").unwrap_or(false);
        let default_strikeout = style.map(|s| s.strikeout == "-1").unwrap_or(false);

        // Parse style colors
        let default_primary_color = style
            .map(|s| Self::parse_ass_color(&s.primary_colour))
            .unwrap_or([255, 255, 255, 255]);
        let default_secondary_color = style
            .map(|s| Self::parse_ass_color(&s.secondary_colour))
            .unwrap_or([255, 0, 0, 255]);
        let default_outline_color = style
            .map(|s| Self::parse_ass_color(&s.outline_colour))
            .unwrap_or([0, 0, 0, 255]);
        let default_back_color = style
            .map(|s| Self::parse_ass_color(&s.back_colour))
            .unwrap_or([0, 0, 0, 128]);

        // Parse style formatting values and scale them only if ScaledBorderAndShadow is enabled
        let default_outline_base = style
            .map(|s| s.outline.parse::<f32>().unwrap_or(2.0))
            .unwrap_or(2.0);
        let default_shadow_base = style
            .map(|s| s.shadow.parse::<f32>().unwrap_or(2.0))
            .unwrap_or(2.0);
        // Scale outline and shadow from script coordinates only if ScaledBorderAndShadow is true
        let default_outline = if self.scaled_border_and_shadow {
            default_outline_base * scale_y
        } else {
            default_outline_base
        };
        let default_shadow = if self.scaled_border_and_shadow {
            default_shadow_base * scale_y
        } else {
            default_shadow_base
        };
        let default_scale_x = style
            .map(|s| s.scale_x.parse::<f32>().unwrap_or(100.0))
            .unwrap_or(100.0);
        let default_scale_y = style
            .map(|s| s.scale_y.parse::<f32>().unwrap_or(100.0))
            .unwrap_or(100.0);
        let default_spacing = style
            .map(|s| s.spacing.parse::<f32>().unwrap_or(0.0))
            .unwrap_or(0.0);
        let default_alignment = style
            .map(|s| s.alignment.parse::<u8>().unwrap_or(2))
            .unwrap_or(2);

        TextDefaults {
            font_name: default_font_name,
            font_size_base: default_font_size_base,
            bold: default_bold,
            italic: default_italic,
            underline: default_underline,
            strikeout: default_strikeout,
            primary_color: default_primary_color,
            secondary_color: default_secondary_color,
            outline_color: default_outline_color,
            back_color: default_back_color,
            outline: default_outline,
            shadow: default_shadow,
            scale_x: default_scale_x,
            scale_y: default_scale_y,
            spacing: default_spacing,
            alignment: default_alignment,
        }
    }
}
