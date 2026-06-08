//! `Style` struct definition for ASS style nodes.
//!
//! Defines the zero-copy `Style` struct representing a single style from the
//! V4+ Styles section, together with its `Default` implementation providing
//! standard ASS style values.

use super::super::Span;

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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Style<'a> {
    /// Style name (must be unique within script)
    pub name: &'a str,

    /// Parent style name for inheritance (None if no inheritance)
    pub parent: Option<&'a str>,

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

    /// Vertical margin in pixels (V4+)
    pub margin_v: &'a str,

    /// Top margin in pixels (V4++)
    pub margin_t: Option<&'a str>,

    /// Bottom margin in pixels (V4++)
    pub margin_b: Option<&'a str>,

    /// Font encoding identifier
    pub encoding: &'a str,

    /// Positioning context (V4++)
    pub relative_to: Option<&'a str>,

    /// Span in source text where this style is defined
    pub span: Span,
}

impl Default for Style<'_> {
    /// Create default ASS style with standard values
    ///
    /// Provides the standard ASS default style values as defined
    /// in the ASS specification for maximum compatibility.
    fn default() -> Self {
        Self {
            name: "Default",
            parent: None,
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
            margin_t: None,
            margin_b: None,
            encoding: "1",
            relative_to: None,
            span: Span::new(0, 0, 0, 0),
        }
    }
}
