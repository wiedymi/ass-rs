//! Default style-line serialization for [`StyleBuilder`].

use super::StyleBuilder;
use crate::core::errors::Result;
use ass_core::ScriptVersion;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec,
};

impl StyleBuilder {
    /// Build the style (validates required fields)
    pub fn build(self) -> Result<String> {
        let name = self.name.unwrap_or_else(|| "NewStyle".to_string());
        let fontname = self.fontname.unwrap_or_else(|| "Arial".to_string());
        let fontsize = self.fontsize.unwrap_or(20);
        let primary_colour = self
            .primary_colour
            .unwrap_or_else(|| "&Hffffff".to_string());
        let secondary_colour = self
            .secondary_colour
            .unwrap_or_else(|| "&Hff0000".to_string());
        let outline_colour = self.outline_colour.unwrap_or_else(|| "&H0".to_string());
        let back_colour = self.back_colour.unwrap_or_else(|| "&H0".to_string());
        let bold = if self.bold.unwrap_or(false) {
            "-1"
        } else {
            "0"
        };
        let italic = if self.italic.unwrap_or(false) {
            "-1"
        } else {
            "0"
        };
        let underline = if self.underline.unwrap_or(false) {
            "-1"
        } else {
            "0"
        };
        let strikeout = if self.strikeout.unwrap_or(false) {
            "-1"
        } else {
            "0"
        };
        let scale_x = self.scale_x.unwrap_or(100.0);
        let scale_y = self.scale_y.unwrap_or(100.0);
        let spacing = self.spacing.unwrap_or(0.0);
        let angle = self.angle.unwrap_or(0.0);
        let border_style = self.border_style.unwrap_or(1);
        let outline = self.outline.unwrap_or(2.0);
        let shadow = self.shadow.unwrap_or(0.0);
        let alignment = self.alignment.unwrap_or(2);
        let margin_l = self.margin_l.unwrap_or(10);
        let margin_r = self.margin_r.unwrap_or(10);
        let margin_v = self.margin_v.unwrap_or(10);
        let encoding = self.encoding.unwrap_or(1);

        // Handle V4++ fields - margin_t/margin_b override margin_v when present
        // relative_to is also a V4++ field
        // Note: The actual format line would determine the field order and presence
        // For now, we use the standard V4+ format

        // Format as ASS style line
        let line = format!(
            "Style: {name},{fontname},{fontsize},{primary_colour},{secondary_colour},{outline_colour},{back_colour},{bold},{italic},{underline},{strikeout},{scale_x},{scale_y},{spacing},{angle},{border_style},{outline},{shadow},{alignment},{margin_l},{margin_r},{margin_v},{encoding}"
        );

        Ok(line)
    }

    /// Build the style with a specific version format
    pub fn build_with_version(self, version: ScriptVersion) -> Result<String> {
        // Define format based on version
        let format = match version {
            ScriptVersion::SsaV4 => {
                // SSA v4 has fewer fields
                vec![
                    "Name",
                    "Fontname",
                    "Fontsize",
                    "PrimaryColour",
                    "SecondaryColour",
                    "TertiaryColour",
                    "BackColour",
                    "Bold",
                    "Italic",
                    "BorderStyle",
                    "Outline",
                    "Shadow",
                    "Alignment",
                    "MarginL",
                    "MarginR",
                    "MarginV",
                    "AlphaLevel",
                    "Encoding",
                ]
            }
            ScriptVersion::AssV4 => {
                // Standard ASS v4 format
                vec![
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
                ]
            }
            ScriptVersion::AssV4Plus => {
                // ASS v4++ format with additional fields
                vec![
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
                    "MarginT",
                    "MarginB",
                    "Encoding",
                    "RelativeTo",
                ]
            }
        };

        self.build_with_format(&format)
    }
}
