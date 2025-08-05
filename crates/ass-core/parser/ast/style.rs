//! Style AST node for ASS style definitions
//!
//! Contains the Style struct representing style definitions from the
//! [V4+ Styles] section with zero-copy design and style property accessors.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{format, vec::Vec};

use super::Span;
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

impl Style<'_> {
    /// Convert style to ASS string representation
    ///
    /// Generates the standard ASS style line format for V4+ styles.
    /// Uses `margin_v` by default, but will use `margin_t/margin_b` if provided (V4++ format).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::Style;
    /// let style = Style {
    ///     name: "TestStyle",
    ///     fontname: "Arial",
    ///     fontsize: "20",
    ///     ..Style::default()
    /// };
    /// let ass_string = style.to_ass_string();
    /// assert!(ass_string.starts_with("Style: TestStyle,Arial,20,"));
    /// ```
    #[must_use]
    pub fn to_ass_string(&self) -> alloc::string::String {

        // Use standard V4+ format by default
        // TODO: Support custom format lines
        format!(
            "Style: {},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
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
            self.encoding
        )
    }

    /// Convert style to ASS string with specific format
    ///
    /// Generates an ASS style line according to the provided format specification.
    /// This allows handling both V4+ and V4++ formats, as well as custom formats.
    ///
    /// # Arguments
    ///
    /// * `format` - Field names in order (e.g., ["Name", "Fontname", "Fontsize", ...])
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::Style;
    /// let style = Style {
    ///     name: "Simple",
    ///     fontname: "Arial",
    ///     fontsize: "16",
    ///     ..Style::default()
    /// };
    /// let format = vec!["Name", "Fontname", "Fontsize"];
    /// assert_eq!(
    ///     style.to_ass_string_with_format(&format),
    ///     "Style: Simple,Arial,16"
    /// );
    /// ```
    #[must_use]
    pub fn to_ass_string_with_format(&self, format: &[&str]) -> alloc::string::String {

        let mut field_values = Vec::with_capacity(format.len());

        for field in format {
            let value = match *field {
                "Name" => self.name,
                "Fontname" => self.fontname,
                "Fontsize" => self.fontsize,
                "PrimaryColour" => self.primary_colour,
                "SecondaryColour" => self.secondary_colour,
                "OutlineColour" | "TertiaryColour" => self.outline_colour,
                "BackColour" => self.back_colour,
                "Bold" => self.bold,
                "Italic" => self.italic,
                "Underline" => self.underline,
                "Strikeout" | "StrikeOut" => self.strikeout,
                "ScaleX" => self.scale_x,
                "ScaleY" => self.scale_y,
                "Spacing" => self.spacing,
                "Angle" => self.angle,
                "BorderStyle" => self.border_style,
                "Outline" => self.outline,
                "Shadow" => self.shadow,
                "Alignment" => self.alignment,
                "MarginL" => self.margin_l,
                "MarginR" => self.margin_r,
                "MarginV" => self.margin_v,
                "MarginT" => self.margin_t.unwrap_or("0"),
                "MarginB" => self.margin_b.unwrap_or("0"),
                "Encoding" => self.encoding,
                "RelativeTo" => self.relative_to.unwrap_or("0"),
                _ => "", // Unknown fields default to empty
            };
            field_values.push(value);
        }

        let joined = field_values.join(",");
        format!("Style: {joined}")
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::String;
    #[cfg(not(feature = "std"))]
    use alloc::vec;

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
    fn style_default_all_fields() {
        let style = Style::default();

        // Test all default field values
        assert_eq!(style.name, "Default");
        assert_eq!(style.fontname, "Arial");
        assert_eq!(style.fontsize, "20");
        assert_eq!(style.primary_colour, "&Hffffff");
        assert_eq!(style.secondary_colour, "&H0000ff");
        assert_eq!(style.outline_colour, "&H000000");
        assert_eq!(style.back_colour, "&H000000");
        assert_eq!(style.bold, "0");
        assert_eq!(style.italic, "0");
        assert_eq!(style.underline, "0");
        assert_eq!(style.strikeout, "0");
        assert_eq!(style.scale_x, "100");
        assert_eq!(style.scale_y, "100");
        assert_eq!(style.spacing, "0");
        assert_eq!(style.angle, "0");
        assert_eq!(style.border_style, "1");
        assert_eq!(style.outline, "0");
        assert_eq!(style.shadow, "0");
        assert_eq!(style.alignment, "2");
        assert_eq!(style.margin_l, "10");
        assert_eq!(style.margin_r, "10");
        assert_eq!(style.margin_v, "10");
        assert_eq!(style.encoding, "1");
    }

    #[test]
    fn style_clone() {
        let style = Style::default();
        let cloned = style.clone();
        assert_eq!(style, cloned);
    }

    #[test]
    fn style_clone_custom() {
        let style = Style {
            name: "CustomStyle",
            parent: None,
            fontname: "Times New Roman",
            fontsize: "18",
            primary_colour: "&H00ff00",
            secondary_colour: "&Hff0000",
            outline_colour: "&H0000ff",
            back_colour: "&H808080",
            bold: "1",
            italic: "1",
            underline: "0",
            strikeout: "1",
            scale_x: "110",
            scale_y: "90",
            spacing: "2",
            angle: "15",
            border_style: "3",
            outline: "2",
            shadow: "1",
            alignment: "5",
            margin_l: "15",
            margin_r: "25",
            margin_v: "20",
            margin_t: None,
            margin_b: None,
            encoding: "0",
            relative_to: None,
            span: Span::new(0, 0, 0, 0),
        };

        let cloned = style.clone();
        assert_eq!(style, cloned);
        assert_eq!(cloned.name, "CustomStyle");
        assert_eq!(cloned.fontname, "Times New Roman");
        assert_eq!(cloned.bold, "1");
    }

    #[test]
    fn style_debug() {
        let style = Style::default();
        let debug_str = format!("{style:?}");
        assert!(debug_str.contains("Style"));
        assert!(debug_str.contains("Default"));
    }

    #[test]
    fn style_debug_custom() {
        let style = Style {
            name: "DebugTest",
            parent: None,
            fontname: "Helvetica",
            fontsize: "18",
            ..Style::default()
        };

        let debug_str = format!("{style:?}");
        assert!(debug_str.contains("Style"));
        assert!(debug_str.contains("DebugTest"));
        assert!(debug_str.contains("Helvetica"));
        assert!(debug_str.contains("18"));
    }

    #[test]
    fn style_partial_eq() {
        let style1 = Style::default();
        let style2 = Style::default();
        assert_eq!(style1, style2);

        let style3 = Style {
            name: "Custom",
            parent: None,
            ..Style::default()
        };
        assert_ne!(style1, style3);
    }

    #[test]
    fn style_partial_eq_different_fields() {
        let base = Style::default();

        // Test inequality with different fields
        let name_diff = Style {
            name: "Different",
            parent: None,
            ..Style::default()
        };
        assert_ne!(base, name_diff);

        let font_diff = Style {
            fontname: "Comic Sans",
            parent: None,
            ..Style::default()
        };
        assert_ne!(base, font_diff);

        let size_diff = Style {
            fontsize: "24",
            parent: None,
            ..Style::default()
        };
        assert_ne!(base, size_diff);

        let color_diff = Style {
            primary_colour: "&H00ff00",
            parent: None,
            ..Style::default()
        };
        assert_ne!(base, color_diff);
    }

    #[test]
    fn style_field_access() {
        let style = Style {
            name: "TestStyle",
            parent: None,
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

    #[test]
    fn style_field_access_comprehensive() {
        let style = Style {
            name: "ComprehensiveTest",
            parent: None,
            fontname: "Verdana",
            fontsize: "14",
            primary_colour: "&Hff0000",
            secondary_colour: "&H00ff00",
            outline_colour: "&H0000ff",
            back_colour: "&Hffffff",
            bold: "-1",
            italic: "1",
            underline: "1",
            strikeout: "0",
            scale_x: "125",
            scale_y: "75",
            spacing: "3",
            angle: "45",
            border_style: "3",
            outline: "3",
            shadow: "2",
            alignment: "9",
            margin_l: "20",
            margin_r: "30",
            margin_v: "15",
            margin_t: None,
            margin_b: None,
            encoding: "2",
            relative_to: None,
            span: Span::new(0, 0, 0, 0),
        };

        // Test all field accesses
        assert_eq!(style.name, "ComprehensiveTest");
        assert_eq!(style.fontname, "Verdana");
        assert_eq!(style.fontsize, "14");
        assert_eq!(style.primary_colour, "&Hff0000");
        assert_eq!(style.secondary_colour, "&H00ff00");
        assert_eq!(style.outline_colour, "&H0000ff");
        assert_eq!(style.back_colour, "&Hffffff");
        assert_eq!(style.bold, "-1");
        assert_eq!(style.italic, "1");
        assert_eq!(style.underline, "1");
        assert_eq!(style.strikeout, "0");
        assert_eq!(style.scale_x, "125");
        assert_eq!(style.scale_y, "75");
        assert_eq!(style.spacing, "3");
        assert_eq!(style.angle, "45");
        assert_eq!(style.border_style, "3");
        assert_eq!(style.outline, "3");
        assert_eq!(style.shadow, "2");
        assert_eq!(style.alignment, "9");
        assert_eq!(style.margin_l, "20");
        assert_eq!(style.margin_r, "30");
        assert_eq!(style.margin_v, "15");
        assert_eq!(style.encoding, "2");
    }

    #[test]
    fn style_empty_strings() {
        let style = Style {
            name: "",
            parent: None,
            fontname: "",
            fontsize: "",
            primary_colour: "",
            secondary_colour: "",
            outline_colour: "",
            back_colour: "",
            bold: "",
            italic: "",
            underline: "",
            strikeout: "",
            scale_x: "",
            scale_y: "",
            spacing: "",
            angle: "",
            border_style: "",
            outline: "",
            shadow: "",
            alignment: "",
            margin_l: "",
            margin_r: "",
            margin_v: "",
            margin_t: None,
            margin_b: None,
            encoding: "",
            relative_to: None,
            span: Span::new(0, 0, 0, 0),
        };

        // All fields should be empty strings
        assert_eq!(style.name, "");
        assert_eq!(style.fontname, "");
        assert_eq!(style.fontsize, "");
        assert_eq!(style.primary_colour, "");
        assert_eq!(style.alignment, "");
        assert_eq!(style.encoding, "");
    }

    #[cfg(debug_assertions)]
    #[test]
    fn style_validate_spans() {
        let source = "Default,Arial,20,&Hffffff,&H0000ff,&H000000,&H000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1";
        let source_start = source.as_ptr() as usize;
        let source_end = source_start + source.len();
        let source_range = source_start..source_end;

        // Parse all fields from the source to ensure all references are within range
        let fields: Vec<&str> = source.split(',').collect();
        assert_eq!(fields.len(), 23); // ASS v4+ style has 23 fields

        // Create style with all references within the source range
        let style = Style {
            name: fields[0],
            parent: None,
            fontname: fields[1],
            fontsize: fields[2],
            primary_colour: fields[3],
            secondary_colour: fields[4],
            outline_colour: fields[5],
            back_colour: fields[6],
            bold: fields[7],
            italic: fields[8],
            underline: fields[9],
            strikeout: fields[10],
            scale_x: fields[11],
            scale_y: fields[12],
            spacing: fields[13],
            angle: fields[14],
            border_style: fields[15],
            outline: fields[16],
            shadow: fields[17],
            alignment: fields[18],
            margin_l: fields[19],
            margin_r: fields[20],
            margin_v: fields[21],
            margin_t: None,
            margin_b: None,
            encoding: fields[22],
            relative_to: None,
            span: Span::new(0, 0, 0, 0),
        };

        // Actually call the validate_spans method
        assert!(style.validate_spans(&source_range));

        // Verify the fields are correct
        assert_eq!(style.name, "Default");
        assert_eq!(style.fontname, "Arial");
        assert_eq!(style.fontsize, "20");
    }

    #[cfg(debug_assertions)]
    #[test]
    fn style_validate_spans_invalid() {
        let source1 = "Default,Arial,20";
        let source2 = "Other,Times,16";
        let source1_start = source1.as_ptr() as usize;
        let source1_end = source1_start + source1.len();
        let source1_range = source1_start..source1_end;

        // Create style with references from different source
        let style = Style {
            name: &source2[0..5], // "Other" - from different source
            parent: None,
            fontname: &source1[8..13],  // "Arial" - from source1
            fontsize: &source1[14..16], // "20" - from source1
            ..Style::default()
        };

        // This should fail since name is from different source
        assert!(!style.validate_spans(&source1_range));
    }

    #[test]
    fn style_lifetimes() {
        let source = String::from("TestStyle,Times,16");
        let style = {
            let parts: Vec<&str> = source.split(',').collect();
            Style {
                name: parts[0],
                parent: None,
                fontname: parts[1],
                fontsize: parts[2],
                ..Style::default()
            }
        };

        assert_eq!(style.name, "TestStyle");
        assert_eq!(style.fontname, "Times");
        assert_eq!(style.fontsize, "16");
    }

    #[test]
    fn style_equality_all_combinations() {
        let style1 = Style::default();
        let mut style2 = Style::default();

        // Should be equal initially
        assert_eq!(style1, style2);

        // Test each field for inequality
        style2.name = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.fontname = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.fontsize = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.primary_colour = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.secondary_colour = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.outline_colour = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.back_colour = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.bold = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.italic = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.underline = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.strikeout = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.scale_x = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.scale_y = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.spacing = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.angle = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.border_style = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.outline = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.shadow = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.alignment = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.margin_l = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.margin_r = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.margin_v = "Different";
        assert_ne!(style1, style2);
        style2 = Style::default();

        style2.encoding = "Different";
        assert_ne!(style1, style2);
    }

    #[test]
    fn style_default_construction() {
        // Test that Default::default() works correctly
        let style: Style = Style::default();
        assert_eq!(style.name, "Default");
        assert_eq!(style.fontname, "Arial");
        assert_eq!(style.fontsize, "20");
        assert_eq!(style.primary_colour, "&Hffffff");
        assert_eq!(style.secondary_colour, "&H0000ff");
        assert_eq!(style.outline_colour, "&H000000");
        assert_eq!(style.back_colour, "&H000000");
        assert_eq!(style.bold, "0");
        assert_eq!(style.italic, "0");
        assert_eq!(style.underline, "0");
        assert_eq!(style.strikeout, "0");
        assert_eq!(style.scale_x, "100");
        assert_eq!(style.scale_y, "100");
        assert_eq!(style.spacing, "0");
        assert_eq!(style.angle, "0");
        assert_eq!(style.border_style, "1");
        assert_eq!(style.outline, "0");
        assert_eq!(style.shadow, "0");
        assert_eq!(style.alignment, "2");
        assert_eq!(style.margin_l, "10");
        assert_eq!(style.margin_r, "10");
        assert_eq!(style.margin_v, "10");
        assert_eq!(style.encoding, "1");
    }

    #[test]
    fn style_struct_creation() {
        // Test direct struct creation syntax
        let style = Style {
            name: "TestName",
            parent: None,
            fontname: "TestFont",
            fontsize: "12",
            primary_colour: "&H123456",
            secondary_colour: "&H654321",
            outline_colour: "&Habcdef",
            back_colour: "&Hfedcba",
            bold: "1",
            italic: "1",
            underline: "1",
            strikeout: "1",
            scale_x: "150",
            scale_y: "75",
            spacing: "5",
            angle: "90",
            border_style: "2",
            outline: "1",
            shadow: "3",
            alignment: "5",
            margin_l: "25",
            margin_r: "35",
            margin_v: "20",
            margin_t: None,
            margin_b: None,
            encoding: "3",
            relative_to: None,
            span: Span::new(0, 0, 0, 0),
        };

        // Verify all fields are set correctly
        assert_eq!(style.name, "TestName");
        assert_eq!(style.fontname, "TestFont");
        assert_eq!(style.fontsize, "12");
        assert_eq!(style.primary_colour, "&H123456");
        assert_eq!(style.secondary_colour, "&H654321");
        assert_eq!(style.outline_colour, "&Habcdef");
        assert_eq!(style.back_colour, "&Hfedcba");
        assert_eq!(style.bold, "1");
        assert_eq!(style.italic, "1");
        assert_eq!(style.underline, "1");
        assert_eq!(style.strikeout, "1");
        assert_eq!(style.scale_x, "150");
        assert_eq!(style.scale_y, "75");
        assert_eq!(style.spacing, "5");
        assert_eq!(style.angle, "90");
        assert_eq!(style.border_style, "2");
        assert_eq!(style.outline, "1");
        assert_eq!(style.shadow, "3");
        assert_eq!(style.alignment, "5");
        assert_eq!(style.margin_l, "25");
        assert_eq!(style.margin_r, "35");
        assert_eq!(style.margin_v, "20");
        assert_eq!(style.encoding, "3");
    }

    #[test]
    fn style_mix_default_and_custom() {
        // Test struct update syntax with defaults
        let style = Style {
            name: "MixedStyle",
            parent: None,
            fontsize: "22",
            bold: "1",
            italic: "1",
            primary_colour: "&Hff00ff",
            alignment: "7",
            ..Style::default()
        };

        // Custom fields
        assert_eq!(style.name, "MixedStyle");
        assert_eq!(style.fontsize, "22");
        assert_eq!(style.bold, "1");
        assert_eq!(style.italic, "1");
        assert_eq!(style.primary_colour, "&Hff00ff");
        assert_eq!(style.alignment, "7");

        // Default fields
        assert_eq!(style.fontname, "Arial");
        assert_eq!(style.underline, "0");
        assert_eq!(style.strikeout, "0");
        assert_eq!(style.scale_x, "100");
        assert_eq!(style.encoding, "1");
    }

    #[test]
    fn style_to_ass_string() {
        let style = Style::default();
        let ass_string = style.to_ass_string();

        assert_eq!(
            ass_string,
            "Style: Default,Arial,20,&Hffffff,&H0000ff,&H000000,&H000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1"
        );
    }

    #[test]
    fn style_to_ass_string_custom() {
        let style = Style {
            name: "Custom",
            fontname: "Times New Roman",
            fontsize: "24",
            primary_colour: "&H00ff00",
            bold: "-1",
            italic: "1",
            scale_x: "95",
            scale_y: "105",
            alignment: "5",
            ..Style::default()
        };

        let ass_string = style.to_ass_string();
        assert!(ass_string.contains("Custom,Times New Roman,24"));
        assert!(ass_string.contains("&H00ff00"));
        assert!(ass_string.contains("-1,1")); // bold, italic
        assert!(ass_string.contains("95,105")); // scale_x, scale_y
        assert!(ass_string.contains(",5,")); // alignment
    }

    #[test]
    fn style_to_ass_string_with_format() {
        let style = Style {
            name: "TestStyle",
            fontname: "Arial",
            fontsize: "20",
            ..Style::default()
        };

        // V4+ standard format
        let v4_format = vec![
            "Name",
            "Fontname",
            "Fontsize",
            "PrimaryColour",
            "SecondaryColour",
            "OutlineColour",
            "BackColour",
            "Bold",
            "Italic",
            "Underline",
            "StrikeOut",
            "ScaleX",
            "ScaleY",
            "Spacing",
            "Angle",
            "BorderStyle",
            "Outline",
            "Shadow",
            "Alignment",
            "MarginL",
            "MarginR",
            "MarginV",
            "Encoding",
        ];
        let v4_string = style.to_ass_string_with_format(&v4_format);
        assert_eq!(
            v4_string,
            "Style: TestStyle,Arial,20,&Hffffff,&H0000ff,&H000000,&H000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1"
        );

        // Minimal format
        let min_format = vec!["Name", "Fontname", "Fontsize"];
        let min_string = style.to_ass_string_with_format(&min_format);
        assert_eq!(min_string, "Style: TestStyle,Arial,20");

        // V4++ format with margin_t/margin_b
        let style_v4pp = Style {
            name: "V4++Style",
            margin_t: Some("15"),
            margin_b: Some("20"),
            relative_to: Some("video"),
            ..Style::default()
        };
        let v4pp_format = vec![
            "Name",
            "Fontname",
            "Fontsize",
            "PrimaryColour",
            "SecondaryColour",
            "OutlineColour",
            "BackColour",
            "Bold",
            "Italic",
            "Underline",
            "StrikeOut",
            "ScaleX",
            "ScaleY",
            "Spacing",
            "Angle",
            "BorderStyle",
            "Outline",
            "Shadow",
            "Alignment",
            "MarginL",
            "MarginR",
            "MarginT",
            "MarginB",
            "Encoding",
            "RelativeTo",
        ];
        let v4pp_string = style_v4pp.to_ass_string_with_format(&v4pp_format);
        assert!(v4pp_string.contains("V4++Style"));
        assert!(v4pp_string.contains(",15,20,")); // margin_t, margin_b
        assert!(v4pp_string.contains(",video")); // relative_to
    }

    #[cfg(debug_assertions)]
    #[test]
    fn style_validate_spans_comprehensive() {
        let source = "Name,Font,Size,Primary,Secondary,Outline,Back,Bold,Italic,Under,Strike,ScX,ScY,Sp,Ang,Border,Out,Shad,Align,ML,MR,MV,Enc";
        let source_start = source.as_ptr() as usize;
        let source_end = source_start + source.len();
        let source_range = source_start..source_end;

        let fields: Vec<&str> = source.split(',').collect();
        let style = Style {
            name: fields[0],
            parent: None,
            fontname: fields[1],
            fontsize: fields[2],
            primary_colour: fields[3],
            secondary_colour: fields[4],
            outline_colour: fields[5],
            back_colour: fields[6],
            bold: fields[7],
            italic: fields[8],
            underline: fields[9],
            strikeout: fields[10],
            scale_x: fields[11],
            scale_y: fields[12],
            spacing: fields[13],
            angle: fields[14],
            border_style: fields[15],
            outline: fields[16],
            shadow: fields[17],
            alignment: fields[18],
            margin_l: fields[19],
            margin_r: fields[20],
            margin_v: fields[21],
            margin_t: None,
            margin_b: None,
            encoding: fields[22],
            relative_to: None,
            span: Span::new(0, 0, 0, 0),
        };

        // Should validate successfully since all fields are from source
        assert!(style.validate_spans(&source_range));
    }
}
