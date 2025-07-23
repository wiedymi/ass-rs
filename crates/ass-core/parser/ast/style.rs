//! Style AST node for ASS style definitions
//!
//! Contains the Style struct representing style definitions from the
//! [V4+ Styles] section with zero-copy design and style property accessors.

#[cfg(debug_assertions)]
use core::ops::Range;

/// Style definition from [V4+ Styles] section
///
/// Represents a single style definition that can be referenced by events.
/// All fields are stored as zero-copy string references to the original
/// source text for maximum memory efficiency.
///
/// # Examples
///
/// ```rust
/// use ass_core::parser::ast::Style;
///
/// let style = Style {
///     name: "Default",
///     fontname: "Arial",
///     fontsize: "20",
///     ..Style::default()
/// };
///
/// assert_eq!(style.name, "Default");
/// assert_eq!(style.fontname, "Arial");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Style<'a> {
    /// Style name (must be unique within script)
    pub name: &'a str,

    /// Font name for text rendering
    pub fontname: &'a str,

    /// Font size in points
    pub fontsize: &'a str,

    /// Primary color in BGR format (&HBBGGRR)
    pub primary_colour: &'a str,

    /// Secondary color for collision effects
    pub secondary_colour: &'a str,

    /// Outline color
    pub outline_colour: &'a str,

    /// Shadow/background color
    pub back_colour: &'a str,

    /// Bold flag (-1/0 or weight)
    pub bold: &'a str,

    /// Italic flag (0/1)
    pub italic: &'a str,

    /// Underline flag (0/1)
    pub underline: &'a str,

    /// Strikeout flag (0/1)
    pub strikeout: &'a str,

    /// Horizontal scale percentage
    pub scale_x: &'a str,

    /// Vertical scale percentage
    pub scale_y: &'a str,

    /// Character spacing in pixels
    pub spacing: &'a str,

    /// Rotation angle in degrees
    pub angle: &'a str,

    /// Border style (1=outline+shadow, 3=opaque box)
    pub border_style: &'a str,

    /// Outline width in pixels
    pub outline: &'a str,

    /// Shadow depth in pixels
    pub shadow: &'a str,

    /// Alignment (1-3 + 4/8 for vertical positioning)
    pub alignment: &'a str,

    /// Left margin in pixels
    pub margin_l: &'a str,

    /// Right margin in pixels
    pub margin_r: &'a str,

    /// Vertical margin in pixels
    pub margin_v: &'a str,

    /// Font encoding identifier
    pub encoding: &'a str,
}

impl Style<'_> {
    /// Validate all spans in this Style reference valid source
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    /// Validates that all string references point to memory within
    /// the specified source range.
    ///
    /// Only available in debug builds to avoid performance overhead.
    #[cfg(debug_assertions)]
    #[must_use]
    pub fn validate_spans(&self, source_range: &Range<usize>) -> bool {
        let spans = [
            self.name,
            self.fontname,
            self.fontsize,
            self.primary_colour,
            self.secondary_colour,
            self.outline_colour,
            self.back_colour,
            self.bold,
            self.italic,
            self.underline,
            self.strikeout,
            self.scale_x,
            self.scale_y,
            self.spacing,
            self.angle,
            self.border_style,
            self.outline,
            self.shadow,
            self.alignment,
            self.margin_l,
            self.margin_r,
            self.margin_v,
            self.encoding,
        ];

        spans.iter().all(|span| {
            let ptr = span.as_ptr() as usize;
            source_range.contains(&ptr)
        })
    }
}

impl Default for Style<'_> {
    /// Create default ASS style with standard values
    ///
    /// Provides the standard ASS default style values as defined
    /// in the ASS specification for maximum compatibility.
    fn default() -> Self {
        Self {
            name: "Default",
            fontname: "Arial",
            fontsize: "20",
            primary_colour: "&Hffffff",
            secondary_colour: "&H0000ff",
            outline_colour: "&H000000",
            back_colour: "&H000000",
            bold: "0",
            italic: "0",
            underline: "0",
            strikeout: "0",
            scale_x: "100",
            scale_y: "100",
            spacing: "0",
            angle: "0",
            border_style: "1",
            outline: "0",
            shadow: "0",
            alignment: "2",
            margin_l: "10",
            margin_r: "10",
            margin_v: "10",
            encoding: "1",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_default_values() {
        let style = Style::default();
        assert_eq!(style.name, "Default");
        assert_eq!(style.fontname, "Arial");
        assert_eq!(style.fontsize, "20");
        assert_eq!(style.primary_colour, "&Hffffff");
        assert_eq!(style.alignment, "2");
    }

    #[test]
    fn style_clone() {
        let style = Style::default();
        let cloned = style.clone();
        assert_eq!(style, cloned);
    }

    #[test]
    fn style_debug() {
        let style = Style::default();
        let debug_str = format!("{style:?}");
        assert!(debug_str.contains("Style"));
        assert!(debug_str.contains("Default"));
    }

    #[test]
    fn style_partial_eq() {
        let style1 = Style::default();
        let style2 = Style::default();
        assert_eq!(style1, style2);

        let style3 = Style {
            name: "Custom",
            ..Style::default()
        };
        assert_ne!(style1, style3);
    }

    #[test]
    fn style_field_access() {
        let style = Style {
            name: "TestStyle",
            fontname: "Comic Sans",
            fontsize: "24",
            bold: "1",
            italic: "1",
            ..Style::default()
        };

        assert_eq!(style.name, "TestStyle");
        assert_eq!(style.fontname, "Comic Sans");
        assert_eq!(style.fontsize, "24");
        assert_eq!(style.bold, "1");
        assert_eq!(style.italic, "1");
    }
}
