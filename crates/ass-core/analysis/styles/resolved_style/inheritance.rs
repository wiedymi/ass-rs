//! Inheritance-aware construction of `ResolvedStyle` values.
//!
//! Provides `ResolvedStyle::from_style_with_parent`, which resolves a style by
//! inheriting from a parent and applying only the child's overrides.

use super::parsing::{
    parse_color_with_default, parse_float, parse_font_size, parse_percentage, parse_u16, parse_u8,
};
use super::{ResolvedStyle, TextFormatting};
use crate::{parser::Style, Result};
use alloc::string::ToString;

impl<'a> ResolvedStyle<'a> {
    /// Create resolved style with inheritance from parent
    ///
    /// # Arguments
    ///
    /// * `style` - Style definition with possible overrides
    /// * `parent` - Parent style to inherit from
    ///
    /// # Returns
    ///
    /// Resolved style inheriting parent properties with child overrides
    ///
    /// # Errors
    ///
    /// Returns an error if style parsing fails
    #[allow(clippy::cognitive_complexity)]
    pub fn from_style_with_parent(style: &'a Style<'a>, parent: &Self) -> Result<Self> {
        // Start with parent properties
        let mut resolved = parent.clone();

        // Update name to child's name
        resolved.name = style.name;

        // Override properties that are not empty/default in child
        if !style.fontname.is_empty() {
            resolved.font_name = style.fontname.to_string();
        }

        if !style.fontsize.is_empty() && style.fontsize != "0" {
            resolved.font_size = parse_font_size(style.fontsize)?;
        }

        if !style.primary_colour.is_empty() {
            resolved.primary_color = parse_color_with_default(style.primary_colour)?;
        }

        if !style.secondary_colour.is_empty() {
            resolved.secondary_color = parse_color_with_default(style.secondary_colour)?;
        }

        if !style.outline_colour.is_empty() {
            resolved.outline_color = parse_color_with_default(style.outline_colour)?;
        }

        if !style.back_colour.is_empty() {
            resolved.back_color = parse_color_with_default(style.back_colour)?;
        }

        // For formatting flags, only override if value is non-empty
        let mut formatting = resolved.formatting;
        if !style.bold.is_empty() {
            if style.bold == "0" {
                formatting &= !TextFormatting::BOLD;
            } else if style.bold == "1" {
                formatting |= TextFormatting::BOLD;
            }
        }
        if !style.italic.is_empty() {
            if style.italic == "0" {
                formatting &= !TextFormatting::ITALIC;
            } else if style.italic == "1" {
                formatting |= TextFormatting::ITALIC;
            }
        }
        if !style.underline.is_empty() {
            if style.underline == "0" {
                formatting &= !TextFormatting::UNDERLINE;
            } else if style.underline == "1" {
                formatting |= TextFormatting::UNDERLINE;
            }
        }
        if !style.strikeout.is_empty() {
            if style.strikeout == "0" {
                formatting &= !TextFormatting::STRIKE_OUT;
            } else if style.strikeout == "1" {
                formatting |= TextFormatting::STRIKE_OUT;
            }
        }
        resolved.formatting = formatting;

        if !style.scale_x.is_empty() && style.scale_x != "100" {
            resolved.scale_x = parse_percentage(style.scale_x)?;
        }

        if !style.scale_y.is_empty() && style.scale_y != "100" {
            resolved.scale_y = parse_percentage(style.scale_y)?;
        }

        if !style.spacing.is_empty() && style.spacing != "0" {
            resolved.spacing = parse_float(style.spacing)?;
        }

        if !style.angle.is_empty() && style.angle != "0" {
            resolved.angle = parse_float(style.angle)?;
        }

        if !style.border_style.is_empty() {
            resolved.border_style = parse_u8(style.border_style)?;
        }

        if !style.outline.is_empty() && style.outline != "0" {
            resolved.outline = parse_float(style.outline)?;
        }

        if !style.shadow.is_empty() && style.shadow != "0" {
            resolved.shadow = parse_float(style.shadow)?;
        }

        if !style.alignment.is_empty() {
            resolved.alignment = parse_u8(style.alignment)?;
        }

        if !style.margin_l.is_empty() {
            resolved.margin_l = parse_u16(style.margin_l)?;
        }

        if !style.margin_r.is_empty() {
            resolved.margin_r = parse_u16(style.margin_r)?;
        }

        // Handle margin inheritance
        if let (Some(t), Some(b)) = (style.margin_t, style.margin_b) {
            if !t.is_empty() {
                resolved.margin_t = parse_u16(t)?;
            }
            if !b.is_empty() {
                resolved.margin_b = parse_u16(b)?;
            }
        } else if !style.margin_v.is_empty() && style.margin_v != "0" {
            let margin_v = parse_u16(style.margin_v)?;
            resolved.margin_t = margin_v;
            resolved.margin_b = margin_v;
        }
        // If margin_v is empty or "0", keep inherited margins

        if !style.encoding.is_empty() {
            resolved.encoding = parse_u8(style.encoding)?;
        }

        // Recalculate complexity score
        resolved.complexity_score = Self::calculate_complexity(&resolved);

        Ok(resolved)
    }
}
