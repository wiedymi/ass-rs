//! Resolved style representation with computed values and metrics
//!
//! Provides the `ResolvedStyle` struct containing fully computed style properties
//! after applying inheritance, overrides, and default fallbacks. Includes
//! performance analysis and rendering complexity assessment.
//!
//! # Features
//!
//! - Zero-copy style name references to original definitions
//! - Computed RGBA color values for efficient rendering
//! - Performance complexity scoring (0-100 scale)
//! - Font and layout property validation
//! - Memory-efficient representation via packed fields
//!
//! # Performance
//!
//! - Target: <0.1ms per style resolution
//! - Memory: ~200 bytes per resolved style
//! - Zero allocations for style name references

use alloc::string::String;

mod accessors;
mod from_style;
mod inheritance;
mod parsing;
mod scaling;

#[cfg(test)]
mod formatting_tests;
#[cfg(test)]
mod parse_tests;
#[cfg(test)]
mod resolution_tests;
#[cfg(test)]
mod scaling_tests;
#[cfg(test)]
mod test_support;

bitflags::bitflags! {
    /// Text formatting options for resolved styles
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TextFormatting: u8 {
        /// Bold text formatting
        const BOLD = 1 << 0;
        /// Italic text formatting
        const ITALIC = 1 << 1;
        /// Underline text formatting
        const UNDERLINE = 1 << 2;
        /// Strike-through text formatting
        const STRIKE_OUT = 1 << 3;
    }
}

/// Fully resolved style with computed values and performance metrics
///
/// Contains effective style values after applying inheritance, overrides,
/// and defaults. Optimized for rendering with pre-computed color values
/// and complexity scoring for performance assessment.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedStyle<'a> {
    /// Original style name (zero-copy reference)
    pub name: &'a str,
    /// Resolved font family name
    font_name: String,
    /// Font size in points
    font_size: f32,
    /// Primary text color (RGBA)
    primary_color: [u8; 4],
    /// Secondary text color (RGBA)
    secondary_color: [u8; 4],
    /// Outline color (RGBA)
    outline_color: [u8; 4],
    /// Background color (RGBA)
    back_color: [u8; 4],
    /// Text formatting flags
    formatting: TextFormatting,
    /// Scaling factors (percentage)
    /// Horizontal scaling factor
    scale_x: f32,
    /// Vertical scaling factor
    scale_y: f32,
    /// Character spacing
    spacing: f32,
    /// Text rotation angle
    angle: f32,
    /// Border style (`0=outline+drop_shadow`, `1=opaque_box`)
    border_style: u8,
    /// Outline thickness
    outline: f32,
    /// Shadow distance
    shadow: f32,
    /// Text alignment (1-9, numpad layout)
    alignment: u8,
    /// Margins in pixels
    /// Left margin in pixels
    margin_l: u16,
    /// Right margin in pixels
    margin_r: u16,
    /// Top margin in pixels
    margin_t: u16,
    /// Bottom margin in pixels
    margin_b: u16,
    /// Text encoding
    encoding: u8,
    /// Rendering complexity score (0-100)
    complexity_score: u8,
}
