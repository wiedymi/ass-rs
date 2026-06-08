//! Fluent setter methods for [`StyleBuilder`].

use super::StyleBuilder;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

impl StyleBuilder {
    /// Set style name
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Set font name
    pub fn font(mut self, font: &str) -> Self {
        self.fontname = Some(font.to_string());
        self
    }

    /// Set font size
    pub fn size(mut self, size: u32) -> Self {
        self.fontsize = Some(size);
        self
    }

    /// Set primary text color (in ASS color format)
    pub fn color(mut self, color: &str) -> Self {
        self.primary_colour = Some(color.to_string());
        self
    }

    /// Set bold formatting
    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = Some(bold);
        self
    }

    /// Set italic formatting
    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = Some(italic);
        self
    }

    /// Set alignment (1-9, numpad style)
    pub fn align(mut self, alignment: u32) -> Self {
        self.alignment = Some(alignment);
        self
    }

    /// Set secondary color (for collision effects)
    pub fn secondary_color(mut self, color: &str) -> Self {
        self.secondary_colour = Some(color.to_string());
        self
    }

    /// Set outline color
    pub fn outline_color(mut self, color: &str) -> Self {
        self.outline_colour = Some(color.to_string());
        self
    }

    /// Set shadow/background color
    pub fn back_color(mut self, color: &str) -> Self {
        self.back_colour = Some(color.to_string());
        self
    }

    /// Set underline formatting
    pub fn underline(mut self, underline: bool) -> Self {
        self.underline = Some(underline);
        self
    }

    /// Set strikeout formatting
    pub fn strikeout(mut self, strikeout: bool) -> Self {
        self.strikeout = Some(strikeout);
        self
    }

    /// Set horizontal scale percentage
    pub fn scale_x(mut self, scale: f32) -> Self {
        self.scale_x = Some(scale);
        self
    }

    /// Set vertical scale percentage
    pub fn scale_y(mut self, scale: f32) -> Self {
        self.scale_y = Some(scale);
        self
    }

    /// Set character spacing in pixels
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = Some(spacing);
        self
    }

    /// Set rotation angle in degrees
    pub fn angle(mut self, angle: f32) -> Self {
        self.angle = Some(angle);
        self
    }

    /// Set border style (1=outline+shadow, 3=opaque box)
    pub fn border_style(mut self, style: u32) -> Self {
        self.border_style = Some(style);
        self
    }

    /// Set outline width in pixels
    pub fn outline(mut self, width: f32) -> Self {
        self.outline = Some(width);
        self
    }

    /// Set shadow depth in pixels
    pub fn shadow(mut self, depth: f32) -> Self {
        self.shadow = Some(depth);
        self
    }

    /// Set left margin in pixels
    pub fn margin_left(mut self, margin: u32) -> Self {
        self.margin_l = Some(margin);
        self
    }

    /// Set right margin in pixels
    pub fn margin_right(mut self, margin: u32) -> Self {
        self.margin_r = Some(margin);
        self
    }

    /// Set vertical margin in pixels
    pub fn margin_vertical(mut self, margin: u32) -> Self {
        self.margin_v = Some(margin);
        self
    }

    /// Set top margin in pixels (V4++)
    pub fn margin_top(mut self, margin: u32) -> Self {
        self.margin_t = Some(margin);
        self
    }

    /// Set bottom margin in pixels (V4++)
    pub fn margin_bottom(mut self, margin: u32) -> Self {
        self.margin_b = Some(margin);
        self
    }

    /// Set font encoding identifier
    pub fn encoding(mut self, encoding: u32) -> Self {
        self.encoding = Some(encoding);
        self
    }

    /// Set alpha level (SSA v4) - transparency from 0-255 (0=opaque, 255=transparent)
    pub fn alpha_level(mut self, alpha: u32) -> Self {
        self.alpha_level = Some(alpha);
        self
    }

    /// Set positioning context (V4++)
    pub fn relative_to(mut self, relative: &str) -> Self {
        self.relative_to = Some(relative.to_string());
        self
    }
}
