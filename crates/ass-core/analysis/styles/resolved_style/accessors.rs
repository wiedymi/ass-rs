//! Accessor methods exposing computed properties of `ResolvedStyle`.
//!
//! Read-only getters for fonts, colors, formatting flags, margins, and the
//! performance metrics computed during style resolution.

use super::{ResolvedStyle, TextFormatting};

impl ResolvedStyle<'_> {
    /// Get font family name
    #[must_use]
    pub fn font_name(&self) -> &str {
        &self.font_name
    }

    /// Get font size in points
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Get primary color as RGBA bytes
    #[must_use]
    pub const fn primary_color(&self) -> [u8; 4] {
        self.primary_color
    }

    /// Get rendering complexity score (0-100)
    #[must_use]
    pub const fn complexity_score(&self) -> u8 {
        self.complexity_score
    }

    /// Check if style has performance concerns
    #[must_use]
    pub const fn has_performance_issues(&self) -> bool {
        self.complexity_score > 70
    }

    /// Get text formatting flags
    #[must_use]
    pub const fn formatting(&self) -> TextFormatting {
        self.formatting
    }

    /// Check if text is bold
    #[must_use]
    pub const fn is_bold(&self) -> bool {
        self.formatting.contains(TextFormatting::BOLD)
    }

    /// Check if text is italic
    #[must_use]
    pub const fn is_italic(&self) -> bool {
        self.formatting.contains(TextFormatting::ITALIC)
    }

    /// Check if text is underlined
    #[must_use]
    pub const fn is_underline(&self) -> bool {
        self.formatting.contains(TextFormatting::UNDERLINE)
    }

    /// Check if text has strike-through
    #[must_use]
    pub const fn is_strike_out(&self) -> bool {
        self.formatting.contains(TextFormatting::STRIKE_OUT)
    }

    /// Get left margin in pixels
    #[must_use]
    pub const fn margin_l(&self) -> u16 {
        self.margin_l
    }

    /// Get right margin in pixels
    #[must_use]
    pub const fn margin_r(&self) -> u16 {
        self.margin_r
    }

    /// Get top margin in pixels
    #[must_use]
    pub const fn margin_t(&self) -> u16 {
        self.margin_t
    }

    /// Get bottom margin in pixels
    #[must_use]
    pub const fn margin_b(&self) -> u16 {
        self.margin_b
    }

    /// Get outline thickness
    #[must_use]
    pub const fn outline(&self) -> f32 {
        self.outline
    }

    /// Get shadow distance
    #[must_use]
    pub const fn shadow(&self) -> f32 {
        self.shadow
    }

    /// Get secondary color as RGBA bytes
    #[must_use]
    pub const fn secondary_color(&self) -> [u8; 4] {
        self.secondary_color
    }

    /// Get outline color as RGBA bytes
    #[must_use]
    pub const fn outline_color(&self) -> [u8; 4] {
        self.outline_color
    }

    /// Get character spacing
    #[must_use]
    pub const fn spacing(&self) -> f32 {
        self.spacing
    }

    /// Get text rotation angle
    #[must_use]
    pub const fn angle(&self) -> f32 {
        self.angle
    }
}
