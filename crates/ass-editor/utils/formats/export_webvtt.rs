//! WebVTT (`.vtt`) export.
//!
//! Renders ASS dialogue events to WebVTT cues with optional style blocks and
//! cue settings, plus ASS-to-WebVTT timestamp and tag conversion.

use super::types::{ConversionOptions, FormatOptions};
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
    /// Export to WebVTT format
    pub(super) fn export_webvtt(
        document: &EditorDocument,
        options: &ConversionOptions,
    ) -> Result<String> {
        let mut output = String::new();

        // Add WebVTT header
        output.push_str("WEBVTT\n\n");

        // Add style block if requested
        if let FormatOptions::WebVTT {
            include_style_block: true,
            ..
        } = &options.format_options
        {
            output.push_str("STYLE\n");
            output.push_str("::cue {\n");
            output
                .push_str("  background-image: linear-gradient(to bottom, dimgray, lightgray);\n");
            output.push_str("  color: papayawhip;\n");
            output.push_str("}\n\n");
        }

        document.parse_script_with(|script| {
            for section in script.sections() {
                if let ass_core::parser::ast::Section::Events(events) = section {
                    for event in events {
                        if event.event_type == EventType::Dialogue {
                            // Add timestamps
                            let start = Self::ass_time_to_webvtt(event.start);
                            let end = Self::ass_time_to_webvtt(event.end);
                            output.push_str(&format!("{start} --> {end}"));

                            // Add cue settings if requested
                            if let FormatOptions::WebVTT {
                                use_cue_settings: true,
                                ..
                            } = &options.format_options
                            {
                                // Parse margins as integers for positioning
                                let margin_v: i32 = event.margin_v.parse().unwrap_or(0);
                                if margin_v != 0 {
                                    output.push_str(&format!(" line:{}", 100 - margin_v));
                                }
                            }

                            output.push('\n');

                            // Add text
                            let text = if options.strip_formatting {
                                Self::strip_ass_tags(event.text)
                            } else {
                                Self::convert_ass_to_webvtt_formatting(event.text)
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

    /// Convert ASS time to WebVTT format
    fn ass_time_to_webvtt(time: &str) -> String {
        // ASS format: 0:00:00.00
        // WebVTT format: 00:00:00.000

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

        format!("{hours}:{minutes}:{seconds}.{millis:03}")
    }

    /// Convert ASS formatting to WebVTT
    fn convert_ass_to_webvtt_formatting(text: &str) -> String {
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
}
