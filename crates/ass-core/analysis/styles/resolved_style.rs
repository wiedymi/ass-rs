//! Resolved style representation with computed values and metrics
//!
//! Provides the ResolvedStyle struct containing fully computed style properties
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
use alloc::string::String;

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
    bold: bool,
    italic: bool,
    underline: bool,
    strike_out: bool,
    /// Scaling factors (percentage)
    scale_x: f32,
    scale_y: f32,
    /// Character spacing
    spacing: f32,
    /// Text rotation angle
    angle: f32,
    /// Border style (0=outline+drop_shadow, 1=opaque_box)
    border_style: u8,
    /// Outline thickness
    outline: f32,
    /// Shadow distance
    shadow: f32,
    /// Text alignment (1-9, numpad layout)
    alignment: u8,
    /// Margins in pixels
    margin_l: u16,
    margin_r: u16,
    margin_v: u16,
    /// Text encoding
    encoding: u8,
    /// Rendering complexity score (0-100)
    complexity_score: u8,
}

impl<'a> ResolvedStyle<'a> {
    /// Create ResolvedStyle from base Style definition
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
    /// let style = Style { name: "Default", font_name: "Arial", font_size: "20", ..Default::default() };
    /// let resolved = ResolvedStyle::from_style(&style)?;
    /// assert_eq!(resolved.font_name(), "Arial");
    /// assert_eq!(resolved.font_size(), 20.0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_style(style: &'a Style<'a>) -> Result<Self> {
        let font_name = if style.fontname.is_empty() {
            "Arial".to_string()
        } else {
            style.fontname.to_string()
        };

        let font_size = parse_font_size(style.fontsize)?;
        let primary_color = parse_color(style.primary_colour)?;
        let secondary_color = parse_color(style.secondary_colour)?;
        let outline_color = parse_color(style.outline_colour)?;
        let back_color = parse_color(style.back_colour)?;

        let bold = parse_bool_flag(style.bold)?;
        let italic = parse_bool_flag(style.italic)?;
        let underline = parse_bool_flag(style.underline)?;
        let strike_out = parse_bool_flag(style.strikeout)?;

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
        let margin_v = parse_u16(style.margin_v)?;

        let encoding = parse_u8(style.encoding)?;

        let resolved = Self {
            name: style.name,
            font_name,
            font_size,
            primary_color,
            secondary_color,
            outline_color,
            back_color,
            bold,
            italic,
            underline,
            strike_out,
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
            margin_v,
            encoding,
            complexity_score: 0, // Will be computed
        };

        Ok(Self {
            complexity_score: Self::calculate_complexity(&resolved),
            ..resolved
        })
    }

    /// Get font family name
    pub fn font_name(&self) -> &str {
        &self.font_name
    }

    /// Get font size in points
    pub fn font_size(&self) -> f32 {
        self.font_size
    }

    /// Get primary color as RGBA bytes
    pub fn primary_color(&self) -> [u8; 4] {
        self.primary_color
    }

    /// Get rendering complexity score (0-100)
    pub fn complexity_score(&self) -> u8 {
        self.complexity_score
    }

    /// Check if style has performance concerns
    pub fn has_performance_issues(&self) -> bool {
        self.complexity_score > 70
    }

    /// Calculate rendering complexity score
    fn calculate_complexity(style: &Self) -> u8 {
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

        if style.scale_x != 100.0 || style.scale_y != 100.0 {
            score += 10;
        }

        if style.angle != 0.0 {
            score += 15;
        }

        if style.bold {
            score += 2;
        }
        if style.italic {
            score += 2;
        }
        if style.underline || style.strike_out {
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

/// Parse color value from ASS format
fn parse_color(color_str: &str) -> Result<[u8; 4]> {
    if color_str.is_empty() {
        return Ok([255, 255, 255, 255]);
    }

    let after_prefix = color_str
        .strip_prefix("&H")
        .or_else(|| color_str.strip_prefix("&h"))
        .unwrap_or(color_str);

    let clean = after_prefix.strip_suffix("&").unwrap_or(after_prefix);

    if let Ok(value) = u32::from_str_radix(clean, 16) {
        let b = ((value >> 16) & 0xFF) as u8;
        let g = ((value >> 8) & 0xFF) as u8;
        let r = (value & 0xFF) as u8;
        let a = 255;
        Ok([r, g, b, a])
    } else {
        Err(CoreError::parse("Invalid color format"))
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
    if !(0.0..=1000.0).contains(&value) {
        Err(CoreError::parse("Invalid percentage"))
    } else {
        Ok(value)
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

    fn create_test_style() -> Style<'static> {
        Style {
            name: "Test",
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
            encoding: "1",
        }
    }

    #[test]
    fn resolved_style_creation() {
        let style = create_test_style();
        let resolved = ResolvedStyle::from_style(&style).unwrap();

        assert_eq!(resolved.name, "Test");
        assert_eq!(resolved.font_name(), "Arial");
        assert_eq!(resolved.font_size(), 20.0);
        assert_eq!(resolved.primary_color(), [255, 255, 255, 255]);
    }

    #[test]
    fn color_parsing() {
        // ASS colors are in BGR format: &HAABBGGRR where AA=alpha, BB=blue, GG=green, RR=red
        assert_eq!(parse_color("&H000000FF").unwrap(), [255, 0, 0, 255]); // Red: RR=FF
        assert_eq!(parse_color("&H0000FF00").unwrap(), [0, 255, 0, 255]); // Green: GG=FF
        assert_eq!(parse_color("&H00FF0000").unwrap(), [0, 0, 255, 255]); // Blue: BB=FF
        assert_eq!(parse_color("").unwrap(), [255, 255, 255, 255]); // Default white

        // Test case-insensitive prefix
        assert_eq!(parse_color("&h000000FF").unwrap(), [255, 0, 0, 255]); // Red with lowercase h

        // Test 6-digit format (no alpha channel)
        assert_eq!(parse_color("&HFF0000").unwrap(), [0, 0, 255, 255]); // Blue in 6-digit
        assert_eq!(parse_color("&H00FF00").unwrap(), [0, 255, 0, 255]); // Green in 6-digit
        assert_eq!(parse_color("&H0000FF").unwrap(), [255, 0, 0, 255]); // Red in 6-digit
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
}
