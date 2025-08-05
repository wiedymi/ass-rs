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

use crate::{parser::Style, utils::CoreError, Result};
use alloc::{string::String, string::ToString};

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

    /// Apply resolution scaling to coordinate-based properties
    ///
    /// Scales font size, spacing, outline, shadow, and margins based on the
    /// resolution difference between layout and play resolutions.
    ///
    /// # Arguments
    ///
    /// * `scale_x` - Horizontal scaling factor (`PlayResX` / `LayoutResX`)
    /// * `scale_y` - Vertical scaling factor (`PlayResY` / `LayoutResY`)
    pub fn apply_resolution_scaling(&mut self, scale_x: f32, scale_y: f32) {
        // Scale font size (use average of X/Y scaling to maintain aspect ratio)
        let avg_scale = (scale_x + scale_y) / 2.0;
        self.font_size *= avg_scale;

        // Scale spacing (horizontal)
        self.spacing *= scale_x;

        // Scale outline and shadow (use average scaling)
        self.outline *= avg_scale;
        self.shadow *= avg_scale;

        // Scale margins
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            self.margin_l = (f32::from(self.margin_l) * scale_x) as u16;
            self.margin_r = (f32::from(self.margin_r) * scale_x) as u16;
            self.margin_t = (f32::from(self.margin_t) * scale_y) as u16;
            self.margin_b = (f32::from(self.margin_b) * scale_y) as u16;
        }

        // Recalculate complexity score after scaling
        self.complexity_score = Self::calculate_complexity(self);
    }

    /// Calculate rendering complexity score
    fn calculate_complexity(style: &Self) -> u8 {
        const EPSILON: f32 = 0.001;
        let mut score = 0u8;

        if style.font_size > 72.0 {
            score += 20;
        } else if style.font_size > 48.0 {
            score += 10;
        }

        if style.outline > 4.0 {
            score += 15;
        } else if style.outline > 2.0 {
            score += 8;
        }

        if style.shadow > 3.0 {
            score += 10;
        } else if style.shadow > 1.0 {
            score += 5;
        }

        if (style.scale_x - 100.0).abs() > EPSILON || (style.scale_y - 100.0).abs() > EPSILON {
            score += 10;
        }

        if style.angle.abs() > EPSILON {
            score += 15;
        }

        if style.formatting.contains(TextFormatting::BOLD) {
            score += 2;
        }
        if style.formatting.contains(TextFormatting::ITALIC) {
            score += 2;
        }
        if style
            .formatting
            .intersects(TextFormatting::UNDERLINE | TextFormatting::STRIKE_OUT)
        {
            score += 5;
        }

        score.min(100)
    }
}

/// Parse font size with validation
fn parse_font_size(size_str: &str) -> Result<f32> {
    let size = parse_float(size_str)?;
    if size <= 0.0 || size > 1000.0 {
        Err(CoreError::parse("Invalid font size"))
    } else {
        Ok(size)
    }
}

/// Parse color value with default handling for empty strings
fn parse_color_with_default(color_str: &str) -> Result<[u8; 4]> {
    if color_str.trim().is_empty() {
        Ok([255, 255, 255, 255]) // Default white with full alpha
    } else {
        crate::utils::parse_bgr_color(color_str)
    }
}

/// Parse boolean flag (0 or 1)
fn parse_bool_flag(flag_str: &str) -> Result<bool> {
    match flag_str {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(CoreError::parse("Invalid boolean flag")),
    }
}

/// Parse percentage value
fn parse_percentage(percent_str: &str) -> Result<f32> {
    let value = parse_float(percent_str)?;
    if (0.0..=1000.0).contains(&value) {
        Ok(value)
    } else {
        Err(CoreError::parse("Invalid percentage"))
    }
}

/// Parse float value with validation
fn parse_float(float_str: &str) -> Result<f32> {
    float_str
        .parse::<f32>()
        .map_err(|_| CoreError::parse("Invalid float value"))
}

/// Parse u8 value with validation
fn parse_u8(u8_str: &str) -> Result<u8> {
    u8_str
        .parse::<u8>()
        .map_err(|_| CoreError::parse("Invalid u8 value"))
}

/// Parse u16 value with validation
fn parse_u16(u16_str: &str) -> Result<u16> {
    u16_str
        .parse::<u16>()
        .map_err(|_| CoreError::parse("Invalid u16 value"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::Span;
    #[cfg(not(feature = "std"))]
    fn create_test_style() -> Style<'static> {
        Style {
            name: "Test",
            parent: None,
            fontname: "Arial",
            fontsize: "20",
            primary_colour: "&H00FFFFFF",
            secondary_colour: "&H000000FF",
            outline_colour: "&H00000000",
            back_colour: "&H00000000",
            bold: "0",
            italic: "0",
            underline: "0",
            strikeout: "0",
            scale_x: "100",
            scale_y: "100",
            spacing: "0",
            angle: "0",
            border_style: "1",
            outline: "2",
            shadow: "0",
            alignment: "2",
            margin_l: "10",
            margin_r: "10",
            margin_v: "10",
            margin_t: None,
            margin_b: None,
            encoding: "1",
            relative_to: None,
            span: Span::new(0, 0, 0, 0),
        }
    }

    #[cfg(feature = "std")]
    fn create_test_style() -> Style<'static> {
        Style {
            name: "Test",
            parent: None,
            fontname: "Arial",
            fontsize: "20",
            primary_colour: "&H00FFFFFF",
            secondary_colour: "&H000000FF",
            outline_colour: "&H00000000",
            back_colour: "&H00000000",
            bold: "0",
            italic: "0",
            underline: "0",
            strikeout: "0",
            scale_x: "100",
            scale_y: "100",
            spacing: "0",
            angle: "0",
            border_style: "1",
            outline: "2",
            shadow: "0",
            alignment: "2",
            margin_l: "10",
            margin_r: "10",
            margin_v: "10",
            margin_t: None,
            margin_b: None,
            encoding: "1",
            relative_to: None,
            span: Span::new(0, 0, 0, 0),
        }
    }

    #[test]
    fn resolved_style_creation() {
        let style = create_test_style();
        let resolved = ResolvedStyle::from_style(&style).unwrap();

        assert_eq!(resolved.name, "Test");
        assert_eq!(resolved.font_name(), "Arial");
        assert!((resolved.font_size() - 20.0).abs() < f32::EPSILON);
        assert_eq!(resolved.primary_color(), [255, 255, 255, 0]);
    }

    #[test]
    fn color_parsing() {
        // ASS colors are in BGR format: &HAABBGGRR where AA=alpha, BB=blue, GG=green, RR=red
        assert_eq!(
            crate::utils::parse_bgr_color("&H000000FF").unwrap(),
            [255, 0, 0, 0]
        ); // Red: RR=FF
        assert_eq!(
            crate::utils::parse_bgr_color("&H0000FF00").unwrap(),
            [0, 255, 0, 0]
        ); // Green: GG=FF
        assert_eq!(
            crate::utils::parse_bgr_color("&H00FF0000").unwrap(),
            [0, 0, 255, 0]
        ); // Blue: BB=FF

        // Test case-insensitive prefix
        assert_eq!(
            crate::utils::parse_bgr_color("&h000000FF").unwrap(),
            [255, 0, 0, 0]
        ); // Red with lowercase h

        // Test 6-digit format (no alpha channel)
        assert_eq!(
            crate::utils::parse_bgr_color("&HFF0000").unwrap(),
            [0, 0, 255, 0]
        ); // Blue in 6-digit
        assert_eq!(
            crate::utils::parse_bgr_color("&H00FF00").unwrap(),
            [0, 255, 0, 0]
        ); // Green in 6-digit
        assert_eq!(
            crate::utils::parse_bgr_color("&H0000FF").unwrap(),
            [255, 0, 0, 0]
        ); // Red in 6-digit
    }

    #[test]
    fn complexity_scoring() {
        let mut style = create_test_style();

        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.complexity_score() < 50);

        style.fontsize = "100";
        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.complexity_score() >= 20);
    }

    #[test]
    fn performance_issues_detection() {
        let mut style = create_test_style();

        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(!resolved.has_performance_issues());

        // Create a style with multiple performance-affecting properties
        style.fontsize = "120"; // >72: +20 points
        style.outline = "8"; // >4: +15 points
        style.shadow = "5"; // >3: +10 points
        style.angle = "45"; // !=0: +15 points
        style.scale_x = "150"; // !=100: +10 points
        style.bold = "1"; // +2 points
        style.italic = "1"; // +2 points
        style.underline = "1"; // +5 points
                               // Total: 79 points > 70 threshold

        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.has_performance_issues());
    }

    #[test]
    fn parse_font_size_edge_cases() {
        // Test invalid font sizes
        assert!(parse_font_size("-10").is_err()); // Negative
        assert!(parse_font_size("0").is_err()); // Zero
        assert!(parse_font_size("1001").is_err()); // Too large
        assert!(parse_font_size("abc").is_err()); // Non-numeric
        assert!(parse_font_size("").is_err()); // Empty

        // Test valid font sizes
        assert!(parse_font_size("1").is_ok());
        assert!(parse_font_size("72").is_ok());
        assert!(parse_font_size("1000").is_ok());
    }

    #[test]
    fn parse_color_with_default_invalid_formats() {
        // Test invalid color formats
        assert!(parse_color_with_default("invalid").is_err());
        assert!(parse_color_with_default("&H").is_err());
        assert!(parse_color_with_default("&HZZZZZ").is_err());
        assert!(parse_color_with_default("12345G").is_err()); // Invalid hex character

        // Test empty string returns default
        let default_color = parse_color_with_default("").unwrap();
        assert_eq!(default_color, [255, 255, 255, 255]);

        // Test whitespace only returns default
        let whitespace_color = parse_color_with_default("   ").unwrap();
        assert_eq!(whitespace_color, [255, 255, 255, 255]);
    }

    #[test]
    fn parse_bool_flag_invalid_values() {
        // Test invalid boolean flags
        assert!(parse_bool_flag("2").is_err());
        assert!(parse_bool_flag("-1").is_err());
        assert!(parse_bool_flag("true").is_err());
        assert!(parse_bool_flag("false").is_err());
        assert!(parse_bool_flag("yes").is_err());
        assert!(parse_bool_flag("no").is_err());
        assert!(parse_bool_flag("").is_err());

        // Test valid boolean flags
        assert!(!parse_bool_flag("0").unwrap());
        assert!(parse_bool_flag("1").unwrap());
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn parse_percentage_invalid_values() {
        // Test invalid percentages
        assert!(parse_percentage("-10").is_err()); // Negative
        assert!(parse_percentage("1001").is_err()); // Too large
        assert!(parse_percentage("abc").is_err()); // Non-numeric
        assert!(parse_percentage("").is_err()); // Empty

        // Test valid percentages
        assert_eq!(parse_percentage("0").unwrap(), 0.0);
        assert_eq!(parse_percentage("100").unwrap(), 100.0);
        assert_eq!(parse_percentage("1000").unwrap(), 1000.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn parse_float_invalid_values() {
        assert!(parse_float("abc").is_err());
        assert!(parse_float("").is_err());
        assert!(parse_float("1.2.3").is_err());
        assert!(parse_float("1.2.3.4").is_err());
        assert!(parse_float("not_a_number").is_err());

        // Test valid floats
        assert_eq!(parse_float("0").unwrap(), 0.0);
        assert_eq!(parse_float("-10.5").unwrap(), -10.5);
        assert_eq!(parse_float("123.456").unwrap(), 123.456);
    }

    #[test]
    fn parse_u8_invalid_values() {
        assert!(parse_u8("256").is_err()); // Too large
        assert!(parse_u8("-1").is_err()); // Negative
        assert!(parse_u8("abc").is_err()); // Non-numeric
        assert!(parse_u8("").is_err()); // Empty

        // Test valid u8 values
        assert_eq!(parse_u8("0").unwrap(), 0);
        assert_eq!(parse_u8("255").unwrap(), 255);
    }

    #[test]
    fn parse_u16_invalid_values() {
        assert!(parse_u16("65536").is_err()); // Too large
        assert!(parse_u16("-1").is_err()); // Negative
        assert!(parse_u16("abc").is_err()); // Non-numeric
        assert!(parse_u16("").is_err()); // Empty

        // Test valid u16 values
        assert_eq!(parse_u16("0").unwrap(), 0);
        assert_eq!(parse_u16("65535").unwrap(), 65535);
    }

    #[test]
    fn resolved_style_from_style_with_invalid_values() {
        let mut style = create_test_style();

        // Test with invalid font size - should return error
        style.fontsize = "-10";
        assert!(ResolvedStyle::from_style(&style).is_err());

        style.fontsize = "abc";
        assert!(ResolvedStyle::from_style(&style).is_err());

        // Test with invalid color - should return error
        style.fontsize = "20"; // Reset to valid
        style.primary_colour = "invalid_color";
        assert!(ResolvedStyle::from_style(&style).is_err());

        // Test with invalid boolean flag - should return error
        style.primary_colour = "&HFFFFFF"; // Reset to valid
        style.bold = "2";
        assert!(ResolvedStyle::from_style(&style).is_err());
    }

    #[test]
    fn complexity_calculation_all_branches() {
        let mut style = create_test_style();

        // Test baseline complexity
        let resolved = ResolvedStyle::from_style(&style).unwrap();
        let baseline_score = resolved.complexity_score();

        // Test font size increases complexity
        style.fontsize = "100"; // Large font size
        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.complexity_score() > baseline_score);

        // Test outline increases complexity
        style = create_test_style(); // Reset
        style.outline = "5"; // Large outline
        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.complexity_score() > baseline_score);

        // Test shadow increases complexity
        style = create_test_style(); // Reset
        style.shadow = "5"; // Large shadow
        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.complexity_score() > baseline_score);

        // Test scaling increases complexity
        style = create_test_style(); // Reset
        style.scale_x = "200"; // Non-default scaling
        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.complexity_score() > baseline_score);

        // Test angle increases complexity
        style = create_test_style(); // Reset
        style.angle = "45"; // Rotation
        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.complexity_score() > baseline_score);

        // Test formatting flags increase complexity
        style = create_test_style(); // Reset
        style.bold = "1";
        style.italic = "1";
        style.underline = "1";
        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.complexity_score() > baseline_score);
    }

    #[test]
    fn complexity_score_capped_at_100() {
        let mut style = create_test_style();

        // Set all properties to maximum complexity values
        style.fontsize = "200"; // Large font
        style.outline = "10"; // Large outline
        style.shadow = "10"; // Large shadow
        style.scale_x = "200"; // Large scaling
        style.angle = "180"; // Large rotation
        style.bold = "1";
        style.italic = "1";
        style.underline = "1";
        style.strikeout = "1";

        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.complexity_score() <= 100); // Should be capped at 100
        assert!(resolved.complexity_score() > 50); // Should be high complexity
    }

    #[test]
    fn text_formatting_flags_comprehensive() {
        let mut style = create_test_style();

        // Test all formatting combinations
        style.bold = "1";
        style.italic = "0";
        style.underline = "0";
        style.strikeout = "0";
        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.is_bold());
        assert!(!resolved.is_italic());
        assert!(!resolved.is_underline());
        assert!(!resolved.is_strike_out());
        assert_eq!(resolved.formatting(), TextFormatting::BOLD);

        // Test italic only
        style.bold = "0";
        style.italic = "1";
        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(!resolved.is_bold());
        assert!(resolved.is_italic());
        assert_eq!(resolved.formatting(), TextFormatting::ITALIC);

        // Test underline only
        style.italic = "0";
        style.underline = "1";
        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.is_underline());
        assert_eq!(resolved.formatting(), TextFormatting::UNDERLINE);

        // Test strikeout only
        style.underline = "0";
        style.strikeout = "1";
        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.is_strike_out());
        assert_eq!(resolved.formatting(), TextFormatting::STRIKE_OUT);

        // Test all flags combined
        style.bold = "1";
        style.italic = "1";
        style.underline = "1";
        style.strikeout = "1";
        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!(resolved.is_bold());
        assert!(resolved.is_italic());
        assert!(resolved.is_underline());
        assert!(resolved.is_strike_out());
        let expected = TextFormatting::BOLD
            | TextFormatting::ITALIC
            | TextFormatting::UNDERLINE
            | TextFormatting::STRIKE_OUT;
        assert_eq!(resolved.formatting(), expected);
    }

    #[test]
    fn resolved_style_empty_font_name_uses_default() {
        let mut style = create_test_style();
        style.fontname = "";

        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert_eq!(resolved.font_name(), "Arial");
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn resolved_style_getters_comprehensive() {
        let style = create_test_style();
        let resolved = ResolvedStyle::from_style(&style).unwrap();

        // Test all getter methods
        assert_eq!(resolved.font_name(), "Arial");
        assert_eq!(resolved.font_size(), 20.0);
        assert_eq!(resolved.primary_color(), [255, 255, 255, 0]); // &H00FFFFFF
        assert!(!resolved.has_performance_issues()); // Low complexity

        let formatting = resolved.formatting();
        assert!(!resolved.is_bold());
        assert!(!resolved.is_italic());
        assert!(!resolved.is_underline());
        assert!(!resolved.is_strike_out());
        assert_eq!(formatting, TextFormatting::empty());
    }

    #[test]
    fn resolved_style_apply_resolution_scaling_symmetric() {
        let style = create_test_style();
        let mut resolved = ResolvedStyle::from_style(&style).unwrap();

        // Apply 2x scaling
        resolved.apply_resolution_scaling(2.0, 2.0);

        assert!((resolved.font_size() - 40.0).abs() < f32::EPSILON); // 20 * 2
        assert!((resolved.spacing() - 0.0).abs() < f32::EPSILON); // 0 * 2
        assert!((resolved.outline() - 4.0).abs() < f32::EPSILON); // 2 * 2
        assert!((resolved.shadow() - 0.0).abs() < f32::EPSILON); // 0 * 2
        assert_eq!(resolved.margin_l(), 20); // 10 * 2
        assert_eq!(resolved.margin_r(), 20); // 10 * 2
        assert_eq!(resolved.margin_t(), 20); // 10 * 2
        assert_eq!(resolved.margin_b(), 20); // 10 * 2
    }

    #[test]
    fn resolved_style_apply_resolution_scaling_asymmetric() {
        let mut style = create_test_style();
        style.spacing = "4";
        style.shadow = "2";
        style.margin_l = "10";
        style.margin_r = "20";
        style.margin_v = "30";

        let mut resolved = ResolvedStyle::from_style(&style).unwrap();

        // Apply asymmetric scaling (3x horizontal, 2x vertical)
        resolved.apply_resolution_scaling(3.0, 2.0);

        // Average scale for font/outline/shadow: (3 + 2) / 2 = 2.5
        assert!((resolved.font_size() - 50.0).abs() < f32::EPSILON); // 20 * 2.5
        assert!((resolved.spacing() - 12.0).abs() < f32::EPSILON); // 4 * 3
        assert!((resolved.outline() - 5.0).abs() < f32::EPSILON); // 2 * 2.5
        assert!((resolved.shadow() - 5.0).abs() < f32::EPSILON); // 2 * 2.5
        assert_eq!(resolved.margin_l(), 30); // 10 * 3
        assert_eq!(resolved.margin_r(), 60); // 20 * 3
        assert_eq!(resolved.margin_t(), 60); // 30 * 2
        assert_eq!(resolved.margin_b(), 60); // 30 * 2
    }

    #[test]
    fn resolved_style_apply_resolution_scaling_downscale() {
        let style = create_test_style();
        let mut resolved = ResolvedStyle::from_style(&style).unwrap();

        // Apply 0.5x scaling (downscale)
        resolved.apply_resolution_scaling(0.5, 0.5);

        assert!((resolved.font_size() - 10.0).abs() < f32::EPSILON); // 20 * 0.5
        assert!((resolved.spacing() - 0.0).abs() < f32::EPSILON); // 0 * 0.5
        assert!((resolved.outline() - 1.0).abs() < f32::EPSILON); // 2 * 0.5
        assert!((resolved.shadow() - 0.0).abs() < f32::EPSILON); // 0 * 0.5
        assert_eq!(resolved.margin_l(), 5); // 10 * 0.5
        assert_eq!(resolved.margin_r(), 5); // 10 * 0.5
        assert_eq!(resolved.margin_t(), 5); // 10 * 0.5
        assert_eq!(resolved.margin_b(), 5); // 10 * 0.5
    }

    #[test]
    fn resolved_style_apply_resolution_scaling_updates_complexity() {
        let mut style = create_test_style();
        style.fontsize = "30"; // Not quite large enough to trigger complexity

        let mut resolved = ResolvedStyle::from_style(&style).unwrap();
        let initial_complexity = resolved.complexity_score();

        // Apply 3x scaling to push font size over complexity threshold
        resolved.apply_resolution_scaling(3.0, 3.0);

        assert!((resolved.font_size() - 90.0).abs() < f32::EPSILON); // 30 * 3
        assert!(resolved.complexity_score() > initial_complexity); // Should increase due to large font
    }

    #[test]
    fn resolved_style_apply_resolution_scaling_preserves_other_properties() {
        let mut style = create_test_style();
        style.bold = "1";
        style.italic = "1";
        style.primary_colour = "&H00FF0000"; // Red
        style.angle = "45";

        let mut resolved = ResolvedStyle::from_style(&style).unwrap();
        let initial_color = resolved.primary_color();
        let initial_angle = resolved.angle;
        let initial_formatting = resolved.formatting();

        // Apply scaling
        resolved.apply_resolution_scaling(2.0, 2.0);

        // These properties should not be affected by scaling
        assert_eq!(resolved.primary_color(), initial_color);
        assert!((resolved.angle - initial_angle).abs() < f32::EPSILON);
        assert_eq!(resolved.formatting(), initial_formatting);
        assert!(resolved.is_bold());
        assert!(resolved.is_italic());
    }

    #[test]
    fn resolved_style_spacing_getter() {
        let mut style = create_test_style();
        style.spacing = "5.5";

        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!((resolved.spacing() - 5.5).abs() < f32::EPSILON);
    }

    #[test]
    fn resolved_style_angle_getter() {
        let mut style = create_test_style();
        style.angle = "45.5";

        let resolved = ResolvedStyle::from_style(&style).unwrap();
        assert!((resolved.angle() - 45.5).abs() < f32::EPSILON);
    }
}
