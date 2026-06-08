//! Format-driven style-line serialization for [`StyleBuilder`].

use super::StyleBuilder;
use crate::core::errors::{EditorError, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

impl StyleBuilder {
    /// Build the style with a specific format line
    /// The format parameter should contain field names like ["Name", "Fontname", "Fontsize", ...]
    pub fn build_with_format(&self, format: &[&str]) -> Result<String> {
        if format.is_empty() {
            return Err(EditorError::FormatLineError {
                message: "Format line cannot be empty".to_string(),
            });
        }

        // Build field values based on format specification
        let mut field_values = Vec::with_capacity(format.len());

        for field in format {
            let value = match *field {
                "Name" => self.name.clone().unwrap_or_else(|| "NewStyle".to_string()),
                "Fontname" => self.fontname.clone().unwrap_or_else(|| "Arial".to_string()),
                "Fontsize" => self.fontsize.unwrap_or(20).to_string(),
                "PrimaryColour" => self
                    .primary_colour
                    .clone()
                    .unwrap_or_else(|| "&Hffffff".to_string()),
                "SecondaryColour" => self
                    .secondary_colour
                    .clone()
                    .unwrap_or_else(|| "&Hff0000".to_string()),
                "OutlineColour" | "TertiaryColour" => self
                    .outline_colour
                    .clone()
                    .unwrap_or_else(|| "&H0".to_string()),
                "BackColour" => self
                    .back_colour
                    .clone()
                    .unwrap_or_else(|| "&H0".to_string()),
                "Bold" => if self.bold.unwrap_or(false) {
                    "-1"
                } else {
                    "0"
                }
                .to_string(),
                "Italic" => if self.italic.unwrap_or(false) {
                    "-1"
                } else {
                    "0"
                }
                .to_string(),
                "Underline" => if self.underline.unwrap_or(false) {
                    "-1"
                } else {
                    "0"
                }
                .to_string(),
                "Strikeout" | "StrikeOut" => if self.strikeout.unwrap_or(false) {
                    "-1"
                } else {
                    "0"
                }
                .to_string(),
                "ScaleX" => self.scale_x.unwrap_or(100.0).to_string(),
                "ScaleY" => self.scale_y.unwrap_or(100.0).to_string(),
                "Spacing" => self.spacing.unwrap_or(0.0).to_string(),
                "Angle" => self.angle.unwrap_or(0.0).to_string(),
                "BorderStyle" => self.border_style.unwrap_or(1).to_string(),
                "Outline" => self.outline.unwrap_or(2.0).to_string(),
                "Shadow" => self.shadow.unwrap_or(0.0).to_string(),
                "Alignment" => self.alignment.unwrap_or(2).to_string(),
                "MarginL" => self.margin_l.unwrap_or(10).to_string(),
                "MarginR" => self.margin_r.unwrap_or(10).to_string(),
                "MarginV" => self.margin_v.unwrap_or(10).to_string(),
                "MarginT" => self.margin_t.unwrap_or(0).to_string(),
                "MarginB" => self.margin_b.unwrap_or(0).to_string(),
                "Encoding" => self.encoding.unwrap_or(1).to_string(),
                "AlphaLevel" => self.alpha_level.unwrap_or(0).to_string(),
                "RelativeTo" => self.relative_to.clone().unwrap_or_else(|| "0".to_string()),
                _ => {
                    return Err(EditorError::FormatLineError {
                        message: format!("Unknown style field: {field}"),
                    })
                }
            };
            field_values.push(value);
        }

        // Build the style line
        let line = format!("Style: {}", field_values.join(","));
        Ok(line)
    }
}
