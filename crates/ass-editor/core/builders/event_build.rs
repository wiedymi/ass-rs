//! Event line serialization for [`EventBuilder`].

use super::EventBuilder;
use crate::core::errors::{EditorError, Result};
use ass_core::parser::ast::EventType;
use ass_core::ScriptVersion;

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::{
    borrow::Cow,
    format,
    string::{String, ToString},
    vec::Vec,
};

impl EventBuilder<'_> {
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
