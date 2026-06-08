//! SubRip (`.srt`) and SubStation Alpha (`.ssa`) export.
//!
//! Renders ASS dialogue events to SRT, performs ASS-to-SRT timestamp and tag
//! conversion, and hosts the shared [`FormatConverter::strip_ass_tags`] helper.

use super::types::ConversionOptions;
use super::FormatConverter;
use crate::core::{EditorDocument, Result};
use ass_core::parser::ast::EventType;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

impl FormatConverter {
    /// Export to SSA format
    pub(super) fn export_ssa(
        document: &EditorDocument,
        _options: &ConversionOptions,
    ) -> Result<String> {
        // SSA is very similar to ASS, just with slightly different headers
        let content = document.text();
        let mut output = content.replace("[V4+ Styles]", "[V4 Styles]");
        output = output.replace("ScriptType: v4.00+", "ScriptType: v4.00");
        Ok(output)
    }

    /// Export to SRT format
    pub(super) fn export_srt(
        document: &EditorDocument,
        options: &ConversionOptions,
    ) -> Result<String> {
        let mut output = String::new();
        let mut index = 1;

        document.parse_script_with(|script| {
            for section in script.sections() {
                if let ass_core::parser::ast::Section::Events(events) = section {
                    for event in events {
                        if event.event_type == EventType::Dialogue {
                            // Add index
                            output.push_str(&format!("{index}\n"));
                            index += 1;

                            // Add timestamps
                            let start = Self::ass_time_to_srt(event.start);
                            let end = Self::ass_time_to_srt(event.end);
                            output.push_str(&format!("{start} --> {end}\n"));

                            // Add text
                            let text = if options.strip_formatting {
                                Self::strip_ass_tags(event.text)
                            } else {
                                Self::convert_ass_to_srt_formatting(event.text)
                            };
                            output.push_str(&text.replace("\\N", "\n"));
                            output.push_str("\n\n");
                        }
                    }
                }
            }
        })?;

        Ok(output)
    }

    /// Convert ASS time to SRT format
    fn ass_time_to_srt(time: &str) -> String {
        // ASS format: 0:00:00.00
        // SRT format: 00:00:00,000

        let parts: Vec<&str> = time.split(':').collect();
        if parts.len() != 3 {
            return time.to_string();
        }

        let hours = format!("{:02}", parts[0].parse::<u32>().unwrap_or(0));
        let minutes = parts[1];

        let seconds_parts: Vec<&str> = parts[2].split('.').collect();
        let seconds = seconds_parts[0];
        let centiseconds = seconds_parts.get(1).unwrap_or(&"00");

        // Convert centiseconds to milliseconds
        let millis = centiseconds.parse::<u32>().unwrap_or(0) * 10;

        format!("{hours}:{minutes}:{seconds},{millis:03}")
    }

    /// Convert ASS formatting to SRT
    fn convert_ass_to_srt_formatting(text: &str) -> String {
        let mut result = text.to_string();

        // Convert basic formatting
        result = result.replace("{\\i1}", "<i>");
        result = result.replace("{\\i0}", "</i>");
        result = result.replace("{\\b1}", "<b>");
        result = result.replace("{\\b0}", "</b>");
        result = result.replace("{\\u1}", "<u>");
        result = result.replace("{\\u0}", "</u>");

        // Remove all other ASS tags
        while let Some(start) = result.find('{') {
            if let Some(end) = result[start..].find('}') {
                result.replace_range(start..start + end + 1, "");
            } else {
                break;
            }
        }

        result
    }

    /// Strip all ASS tags from text
    pub(super) fn strip_ass_tags(text: &str) -> String {
        let mut result = text.to_string();
        while let Some(start) = result.find('{') {
            if let Some(end) = result[start..].find('}') {
                result.replace_range(start..start + end + 1, "");
            } else {
                break;
            }
        }
        result
    }
}
