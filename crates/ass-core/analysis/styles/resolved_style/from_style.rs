//! Construction of `ResolvedStyle` values from base style definitions.
//!
//! Houses the `ResolvedStyle::from_style` constructor that resolves all style
//! properties, validates values, and computes performance metrics.

use super::parsing::{
    parse_bool_flag, parse_color_with_default, parse_float, parse_font_size, parse_percentage,
    parse_u16, parse_u8,
};
use super::{ResolvedStyle, TextFormatting};
use crate::{parser::Style, Result};
use alloc::string::ToString;

impl<'a> ResolvedStyle<'a> {
    /// Create `ResolvedStyle` from base Style definition
    ///
    /// Resolves all style properties, validates values, and computes
    /// performance metrics. Invalid values are replaced with defaults.
    ///
    /// # Arguments
    ///
    /// * `style` - Base style definition to resolve
    ///
    /// # Returns
    ///
    /// Fully resolved style with computed properties.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ass_core::analysis::styles::resolved_style::ResolvedStyle;
    /// # use ass_core::parser::Style;
    /// let style = Style { name: "Default", fontname: "Arial", fontsize: "20", ..Default::default() };
    /// let resolved = ResolvedStyle::from_style(&style)?;
    /// assert_eq!(resolved.font_name(), "Arial");
    /// assert_eq!(resolved.font_size(), 20.0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if style parsing fails or contains invalid values.
    pub fn from_style(style: &'a Style<'a>) -> Result<Self> {
        let font_name = if style.fontname.is_empty() {
            "Arial".to_string()
        } else {
            style.fontname.to_string()
        };

        let font_size = parse_font_size(style.fontsize)?;
        let primary_color = parse_color_with_default(style.primary_colour)?;
        let secondary_color = parse_color_with_default(style.secondary_colour)?;
        let outline_color = parse_color_with_default(style.outline_colour)?;
        let back_color = parse_color_with_default(style.back_colour)?;

        let mut formatting = TextFormatting::empty();
        if parse_bool_flag(style.bold)? {
            formatting |= TextFormatting::BOLD;
        }
        if parse_bool_flag(style.italic)? {
            formatting |= TextFormatting::ITALIC;
        }
        if parse_bool_flag(style.underline)? {
            formatting |= TextFormatting::UNDERLINE;
        }
        if parse_bool_flag(style.strikeout)? {
            formatting |= TextFormatting::STRIKE_OUT;
        }

        let scale_x = parse_percentage(style.scale_x)?;
        let scale_y = parse_percentage(style.scale_y)?;
        let spacing = parse_float(style.spacing)?;
        let angle = parse_float(style.angle)?;

        let border_style = parse_u8(style.border_style)?;
        let outline = parse_float(style.outline)?;
        let shadow = parse_float(style.shadow)?;

        let alignment = parse_u8(style.alignment)?;
        let margin_l = parse_u16(style.margin_l)?;
        let margin_r = parse_u16(style.margin_r)?;

        // Handle v4+ vs v4++ margin formats
        let (margin_t, margin_b) = if let (Some(t), Some(b)) = (style.margin_t, style.margin_b) {
            // v4++ format with separate top/bottom margins
            (parse_u16(t)?, parse_u16(b)?)
        } else {
            // v4+ format with single vertical margin
            let margin_v = parse_u16(style.margin_v)?;
            (margin_v, margin_v)
        };

        let encoding = parse_u8(style.encoding)?;

        let resolved = Self {
            name: style.name,
            font_name,
            font_size,
            primary_color,
            secondary_color,
            outline_color,
            back_color,
            formatting,
            scale_x,
            scale_y,
            spacing,
            angle,
            border_style,
            outline,
            shadow,
            alignment,
            margin_l,
            margin_r,
            margin_t,
            margin_b,
            encoding,
            complexity_score: 0, // Will be computed
        };

        Ok(Self {
            complexity_score: Self::calculate_complexity(&resolved),
            ..resolved
        })
    }
}
