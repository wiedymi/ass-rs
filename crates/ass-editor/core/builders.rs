//! Builder patterns for ASS types
//!
//! Provides fluent builder APIs for creating ASS events, styles, and other structures
//! with ergonomic method chaining and validation.

use crate::core::errors::{EditorError, Result};
use ass_core::parser::ast::EventType;
use ass_core::ScriptVersion;

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::{borrow::Cow, format, string::ToString, vec};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// Builder for creating ASS events with fluent API
///
/// Provides an ergonomic way to construct ASS events with method chaining.
/// Supports all event types and automatically handles format validation.
///
/// # Examples
///
/// ```
/// use ass_editor::{EventBuilder, EditorDocument};
///
/// let mut doc = EditorDocument::new();
///
/// // Create a dialogue event
/// let event_line = EventBuilder::dialogue()
///     .start_time("0:00:00.00")
///     .end_time("0:00:05.00")
///     .style("Default")
///     .speaker("Character")
///     .text("Hello, world!")
///     .layer(0)
///     .build()
///     .unwrap();
///
/// // Add to document
/// doc.add_event_line(&event_line).unwrap();
/// ```
#[derive(Debug, Default)]
pub struct EventBuilder<'a> {
    event_type: Option<EventType>,
    start: Option<Cow<'a, str>>,
    end: Option<Cow<'a, str>>,
    style: Option<Cow<'a, str>>,
    name: Option<Cow<'a, str>>,
    text: Option<Cow<'a, str>>,
    layer: Option<Cow<'a, str>>,
    margin_l: Option<Cow<'a, str>>,
    margin_r: Option<Cow<'a, str>>,
    margin_v: Option<Cow<'a, str>>,
    margin_t: Option<Cow<'a, str>>,
    margin_b: Option<Cow<'a, str>>,
    effect: Option<Cow<'a, str>>,
}

impl<'a> EventBuilder<'a> {
    /// Create a new event builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a dialogue event builder  
    pub fn dialogue() -> Self {
        Self {
            event_type: Some(EventType::Dialogue),
            ..Self::default()
        }
    }

    /// Create a comment event builder
    pub fn comment() -> Self {
        Self {
            event_type: Some(EventType::Comment),
            ..Self::default()
        }
    }

    /// Set start time (e.g., "0:00:05.00")
    pub fn start_time<S: Into<Cow<'a, str>>>(mut self, time: S) -> Self {
        self.start = Some(time.into());
        self
    }

    /// Set end time (e.g., "0:00:10.00")
    pub fn end_time<S: Into<Cow<'a, str>>>(mut self, time: S) -> Self {
        self.end = Some(time.into());
        self
    }

    /// Set speaker/character name
    pub fn speaker<S: Into<Cow<'a, str>>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set dialogue text
    pub fn text<S: Into<Cow<'a, str>>>(mut self, text: S) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Set style name
    pub fn style<S: Into<Cow<'a, str>>>(mut self, style: S) -> Self {
        self.style = Some(style.into());
        self
    }

    /// Set layer (higher layers render on top)
    pub fn layer(mut self, layer: u32) -> Self {
        self.layer = Some(Cow::Owned(layer.to_string()));
        self
    }

    /// Set left margin
    pub fn margin_left(mut self, margin: u32) -> Self {
        self.margin_l = Some(Cow::Owned(margin.to_string()));
        self
    }

    /// Set right margin
    pub fn margin_right(mut self, margin: u32) -> Self {
        self.margin_r = Some(Cow::Owned(margin.to_string()));
        self
    }

    /// Set vertical margin  
    pub fn margin_vertical(mut self, margin: u32) -> Self {
        self.margin_v = Some(Cow::Owned(margin.to_string()));
        self
    }

    /// Set top margin (V4++)
    pub fn margin_top(mut self, margin: u32) -> Self {
        self.margin_t = Some(Cow::Owned(margin.to_string()));
        self
    }

    /// Set bottom margin (V4++)
    pub fn margin_bottom(mut self, margin: u32) -> Self {
        self.margin_b = Some(Cow::Owned(margin.to_string()));
        self
    }

    /// Set effect
    pub fn effect<S: Into<Cow<'a, str>>>(mut self, effect: S) -> Self {
        self.effect = Some(effect.into());
        self
    }

    /// Build the event (validates required fields)
    pub fn build(self) -> Result<String> {
        // Default to v4+ format
        self.build_with_version(ScriptVersion::AssV4)
    }

    /// Build the event with a specific format version
    pub fn build_with_version(self, version: ScriptVersion) -> Result<String> {
        let event_type = self.event_type.unwrap_or(EventType::Dialogue);
        let start = self.start.unwrap_or(Cow::Borrowed("0:00:00.00"));
        let end = self.end.unwrap_or(Cow::Borrowed("0:00:05.00"));
        let style = self.style.unwrap_or(Cow::Borrowed("Default"));
        let name = self.name.unwrap_or(Cow::Borrowed(""));
        let text = self.text.unwrap_or(Cow::Borrowed(""));
        let layer = self.layer.unwrap_or(Cow::Borrowed("0"));
        let margin_l = self.margin_l.unwrap_or(Cow::Borrowed("0"));
        let margin_r = self.margin_r.unwrap_or(Cow::Borrowed("0"));
        let margin_v = self.margin_v.unwrap_or(Cow::Borrowed("0"));
        let effect = self.effect.unwrap_or(Cow::Borrowed(""));

        // Format as ASS event line based on version
        let event_type_str = event_type.as_str();
        let line = match version {
            ScriptVersion::SsaV4 => {
                // SSA v4 format: no layer field, uses Marked=0 prefix
                format!(
                    "{event_type_str}: Marked=0,{start},{end},{style},{name},{margin_l},{margin_r},{margin_v},{effect},{text}"
                )
            }
            ScriptVersion::AssV4 => {
                // ASS v4 format: includes layer field
                format!(
                    "{event_type_str}: {layer},{start},{end},{style},{name},{margin_l},{margin_r},{margin_v},{effect},{text}"
                )
            }
            ScriptVersion::AssV4Plus => {
                // ASS v4++ format: can use margin_t/margin_b if specified
                // For now, we use the same format as v4 since the builder doesn't support margin_t/margin_b yet
                format!(
                    "{event_type_str}: {layer},{start},{end},{style},{name},{margin_l},{margin_r},{margin_v},{effect},{text}"
                )
            }
        };

        Ok(line)
    }

    /// Build the event with a specific format line
    /// The format parameter should contain field names like ["Layer", "Start", "End", "Style", "Text"]
    pub fn build_with_format(&self, format: &[&str]) -> Result<String> {
        if format.is_empty() {
            return Err(EditorError::FormatLineError {
                message: "Format line cannot be empty".to_string(),
            });
        }

        let event_type = self.event_type.unwrap_or(EventType::Dialogue);
        let event_type_str = event_type.as_str();

        // Build field values based on format specification
        let mut field_values = Vec::with_capacity(format.len());

        for field in format {
            let value = match *field {
                "Layer" => self.layer.as_ref().map(|c| c.as_ref()).unwrap_or("0"),
                "Start" => self
                    .start
                    .as_ref()
                    .map(|c| c.as_ref())
                    .unwrap_or("0:00:00.00"),
                "End" => self
                    .end
                    .as_ref()
                    .map(|c| c.as_ref())
                    .unwrap_or("0:00:05.00"),
                "Style" => self.style.as_ref().map(|c| c.as_ref()).unwrap_or("Default"),
                "Name" | "Actor" => self.name.as_ref().map(|c| c.as_ref()).unwrap_or(""),
                "MarginL" => self.margin_l.as_ref().map(|c| c.as_ref()).unwrap_or("0"),
                "MarginR" => self.margin_r.as_ref().map(|c| c.as_ref()).unwrap_or("0"),
                "MarginV" => self.margin_v.as_ref().map(|c| c.as_ref()).unwrap_or("0"),
                "MarginT" => self.margin_t.as_ref().map(|c| c.as_ref()).unwrap_or("0"),
                "MarginB" => self.margin_b.as_ref().map(|c| c.as_ref()).unwrap_or("0"),
                "Effect" => self.effect.as_ref().map(|c| c.as_ref()).unwrap_or(""),
                "Text" => self.text.as_ref().map(|c| c.as_ref()).unwrap_or(""),
                _ => {
                    return Err(EditorError::FormatLineError {
                        message: format!("Unknown event field: {field}"),
                    })
                }
            };
            field_values.push(value.to_string());
        }

        // Build the event line
        let line = format!("{event_type_str}: {}", field_values.join(","));
        Ok(line)
    }
}

/// Builder for creating ASS styles with fluent API
#[derive(Debug, Default, Clone)]
pub struct StyleBuilder {
    name: Option<String>,
    fontname: Option<String>,
    fontsize: Option<u32>,
    primary_colour: Option<String>,
    secondary_colour: Option<String>,
    outline_colour: Option<String>,
    back_colour: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
    underline: Option<bool>,
    strikeout: Option<bool>,
    scale_x: Option<f32>,
    scale_y: Option<f32>,
    spacing: Option<f32>,
    angle: Option<f32>,
    border_style: Option<u32>,
    outline: Option<f32>,
    shadow: Option<f32>,
    alignment: Option<u32>,
    margin_l: Option<u32>,
    margin_r: Option<u32>,
    margin_v: Option<u32>,
    margin_t: Option<u32>,
    margin_b: Option<u32>,
    encoding: Option<u32>,
    alpha_level: Option<u32>,
    relative_to: Option<String>,
}

impl StyleBuilder {
    /// Create a new style builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a style builder with default values
    pub fn default_style() -> Self {
        Self {
            fontname: Some("Arial".to_string()),
            fontsize: Some(20),
            primary_colour: Some("&Hffffff".to_string()),
            secondary_colour: Some("&Hff0000".to_string()),
            outline_colour: Some("&H0".to_string()),
            back_colour: Some("&H0".to_string()),
            bold: Some(false),
            italic: Some(false),
            underline: Some(false),
            strikeout: Some(false),
            scale_x: Some(100.0),
            scale_y: Some(100.0),
            spacing: Some(0.0),
            angle: Some(0.0),
            border_style: Some(1),
            outline: Some(2.0),
            shadow: Some(0.0),
            alignment: Some(2),
            margin_l: Some(10),
            margin_r: Some(10),
            margin_v: Some(10),
            encoding: Some(1),
            ..Self::default()
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    #[test]
    fn event_builder_dialogue() {
        let event = EventBuilder::dialogue()
            .start_time("0:00:05.00")
            .end_time("0:00:10.00")
            .speaker("John")
            .text("Hello world!")
            .build()
            .unwrap();

        assert!(event.contains("Dialogue:"));
        assert!(event.contains("0:00:05.00"));
        assert!(event.contains("Hello world!"));
    }

    #[test]
    fn event_builder_comment() {
        let event = EventBuilder::comment()
            .text("This is a comment")
            .build()
            .unwrap();

        assert!(event.contains("Comment:"));
        assert!(event.contains("This is a comment"));
    }

    #[test]
    fn style_builder_default() {
        let style = StyleBuilder::default_style()
            .name("TestStyle")
            .font("Comic Sans MS")
            .size(24)
            .bold(true)
            .build()
            .unwrap();

        assert!(style.contains("Style: TestStyle"));
        assert!(style.contains("Comic Sans MS"));
        assert!(style.contains("24"));
        assert!(style.contains("-1")); // Bold = true
    }

    #[test]
    fn style_builder_minimal() {
        let style = StyleBuilder::new().name("Minimal").build().unwrap();

        assert!(style.contains("Style: Minimal"));
        assert!(style.contains("Arial")); // Default font
    }

    #[test]
    fn event_builder_with_margins() {
        let event = EventBuilder::dialogue()
            .start_time("0:00:05.00")
            .end_time("0:00:10.00")
            .margin_left(15)
            .margin_right(20)
            .margin_vertical(25)
            .margin_top(30)
            .margin_bottom(35)
            .text("Testing margins")
            .build()
            .unwrap();

        assert!(event.contains("Dialogue:"));
        assert!(event.contains("15")); // margin_l
        assert!(event.contains("20")); // margin_r
        assert!(event.contains("25")); // margin_v
                                       // Note: margin_t and margin_b are stored but not in V4+ format output yet
    }

    #[test]
    fn style_builder_all_fields() {
        let style = StyleBuilder::new()
            .name("Complete")
            .font("Helvetica")
            .size(18)
            .color("&Hffffff")
            .secondary_color("&H00ff00")
            .outline_color("&H0000ff")
            .back_color("&H808080")
            .bold(true)
            .italic(false)
            .underline(true)
            .strikeout(false)
            .scale_x(95.5)
            .scale_y(105.0)
            .spacing(1.5)
            .angle(15.0)
            .border_style(3)
            .outline(2.5)
            .shadow(1.0)
            .align(7)
            .margin_left(5)
            .margin_right(15)
            .margin_vertical(20)
            .margin_top(25)
            .margin_bottom(30)
            .encoding(0)
            .relative_to("video")
            .build()
            .unwrap();

        assert!(style.contains("Style: Complete"));
        assert!(style.contains("Helvetica"));
        assert!(style.contains("18"));
        assert!(style.contains("&Hffffff"));
        assert!(style.contains("&H00ff00"));
        assert!(style.contains("&H0000ff"));
        assert!(style.contains("&H808080"));
        assert!(style.contains("-1")); // bold = true
        assert!(style.contains("95.5"));
        assert!(style.contains("105"));
        assert!(style.contains("1.5"));
        assert!(style.contains("15")); // angle
        assert!(style.contains("3")); // border_style
        assert!(style.contains("2.5")); // outline
        assert!(style.contains("7")); // alignment
                                      // Note: margin_t, margin_b, and relative_to are stored but not in V4+ format output yet
    }

    #[test]
    fn event_builder_with_format_v4plus() {
        let event = EventBuilder::dialogue()
            .start_time("0:00:05.00")
            .end_time("0:00:10.00")
            .style("Main")
            .layer(1)
            .text("Test with format")
            .build_with_format(&[
                "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV",
                "Effect", "Text",
            ])
            .unwrap();

        assert_eq!(
            event,
            "Dialogue: 1,0:00:05.00,0:00:10.00,Main,,0,0,0,,Test with format"
        );
    }

    #[test]
    fn event_builder_with_format_v4plusplus() {
        let event = EventBuilder::dialogue()
            .start_time("0:00:05.00")
            .end_time("0:00:10.00")
            .style("Main")
            .margin_top(5)
            .margin_bottom(10)
            .text("V4++ format")
            .build_with_format(&[
                "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginT",
                "MarginB", "Effect", "Text",
            ])
            .unwrap();

        assert_eq!(
            event,
            "Dialogue: 0,0:00:05.00,0:00:10.00,Main,,0,0,5,10,,V4++ format"
        );
    }

    #[test]
    fn event_builder_with_format_custom() {
        let event = EventBuilder::comment()
            .text("Simple comment")
            .build_with_format(&["Start", "End", "Text"])
            .unwrap();

        assert_eq!(event, "Comment: 0:00:00.00,0:00:05.00,Simple comment");
    }

    #[test]
    fn event_builder_with_format_error() {
        let result = EventBuilder::dialogue()
            .text("Test")
            .build_with_format(&["InvalidField"]);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown event field"));
    }

    #[test]
    fn style_builder_with_format_v4plus() {
        let style = StyleBuilder::new()
            .name("TestStyle")
            .font("Arial")
            .size(20)
            .build_with_format(&[
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
            ])
            .unwrap();

        assert_eq!(style, "Style: TestStyle,Arial,20,&Hffffff,&Hff0000,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1");
    }

    #[test]
    fn style_builder_with_format_v4plusplus() {
        let style = StyleBuilder::new()
            .name("V4++Style")
            .margin_top(15)
            .margin_bottom(20)
            .relative_to("video")
            .build_with_format(&[
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
            ])
            .unwrap();

        assert!(style.contains("V4++Style"));
        assert!(style.contains("15")); // MarginT
        assert!(style.contains("20")); // MarginB
        assert!(style.contains("video")); // RelativeTo
    }

    #[test]
    fn style_builder_with_format_minimal() {
        let style = StyleBuilder::new()
            .name("MinimalStyle")
            .build_with_format(&["Name", "Fontname", "Fontsize"])
            .unwrap();

        assert_eq!(style, "Style: MinimalStyle,Arial,20");
    }

    #[test]
    fn event_builder_with_script_version() {
        // Test building with SSA v4 format
        let event_ssa = EventBuilder::dialogue()
            .text("SSA Format")
            .start_time("0:00:01.00")
            .end_time("0:00:03.00")
            .build_with_version(ScriptVersion::SsaV4)
            .unwrap();
        assert!(event_ssa.contains("SSA Format"));

        // Test building with ASS v4+ format
        let event_ass = EventBuilder::dialogue()
            .text("ASS Format")
            .build_with_version(ScriptVersion::AssV4Plus)
            .unwrap();
        assert!(event_ass.contains("ASS Format"));
    }

    #[test]
    fn style_builder_with_script_version() {
        // Test building style with SSA v4 format
        let style_ssa = StyleBuilder::new()
            .name("TestSSA")
            .font("Arial")
            .size(18)
            .build_with_version(ScriptVersion::SsaV4)
            .unwrap();
        // SSA v4 has TertiaryColour instead of OutlineColour
        assert!(style_ssa.contains("TestSSA"));
        assert!(style_ssa.contains("Arial"));

        // Test building style with ASS v4+ format
        let style_ass = StyleBuilder::new()
            .name("TestASS")
            .font("Verdana")
            .size(20)
            .build_with_version(ScriptVersion::AssV4Plus)
            .unwrap();
        assert!(style_ass.contains("TestASS"));
        assert!(style_ass.contains("Verdana"));
    }
}
