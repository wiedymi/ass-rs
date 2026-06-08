//! SubRip (`.srt`) import support.
//!
//! Parses SRT cues into ASS dialogue lines, including timestamp conversion and
//! translation of basic HTML-like formatting into ASS override tags.

use super::FormatConverter;
use crate::core::errors::EditorError;
use crate::core::Result;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

impl FormatConverter {
    /// Import SRT format
    pub(super) fn import_srt(content: &str) -> Result<String> {
        let mut output = String::new();

        // Add ASS header
        output.push_str("[Script Info]\n");
        output.push_str("Title: Imported from SRT\n");
        output.push_str("ScriptType: v4.00+\n");
        output.push_str("WrapStyle: 0\n");
        output.push_str("PlayResX: 640\n");
        output.push_str("PlayResY: 480\n");
        output.push_str("ScaledBorderAndShadow: yes\n\n");

        // Add default style
        output.push_str("[V4+ Styles]\n");
        output.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
        output.push_str("Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n");

        // Add events section
        output.push_str("[Events]\n");
        output.push_str(
            "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
        );

        // Parse SRT entries
        let entries = Self::parse_srt_entries(content)?;
        for entry in entries {
            output.push_str(&format!(
                "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
                entry.start, entry.end, entry.text
            ));
        }

        Ok(output)
    }

    /// Parse SRT entries
    fn parse_srt_entries(content: &str) -> Result<Vec<SrtEntry>> {
        let mut entries = Vec::new();
        let mut current_entry: Option<SrtEntry> = None;
        let mut in_text = false;

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() {
                if let Some(entry) = current_entry.take() {
                    entries.push(entry);
                }
                in_text = false;
                continue;
            }

            // Check if it's a number (subtitle index)
            if line.chars().all(|c| c.is_ascii_digit()) && !in_text {
                // Start new entry
                current_entry = Some(SrtEntry::default());
                continue;
            }

            // Check if it's a timestamp line
            if line.contains("-->") {
                if let Some(ref mut entry) = current_entry {
                    let parts: Vec<&str> = line.split("-->").collect();
                    if parts.len() == 2 {
                        entry.start = Self::parse_srt_time(parts[0].trim())?;
                        entry.end = Self::parse_srt_time(parts[1].trim())?;
                        in_text = true;
                    }
                }
                continue;
            }

            // Otherwise it's subtitle text
            if in_text {
                if let Some(ref mut entry) = current_entry {
                    if !entry.text.is_empty() {
                        entry.text.push_str("\\N");
                    }
                    // Convert basic HTML-like tags to ASS
                    let converted_text = Self::convert_srt_formatting(line);
                    entry.text.push_str(&converted_text);
                }
            }
        }

        // Don't forget the last entry
        if let Some(entry) = current_entry {
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Parse SRT timestamp to ASS format
    fn parse_srt_time(time: &str) -> Result<String> {
        // SRT format: 00:00:00,000
        // ASS format: 0:00:00.00

        let time = time.replace(',', ".");
        let parts: Vec<&str> = time.split(':').collect();

        if parts.len() != 3 {
            return Err(EditorError::ValidationError {
                message: format!("Invalid SRT timestamp: {time}"),
            });
        }

        let hours: u32 = parts[0].parse().map_err(|_| EditorError::ValidationError {
            message: format!("Invalid hours in timestamp: {}", parts[0]),
        })?;

        let minutes: u32 = parts[1].parse().map_err(|_| EditorError::ValidationError {
            message: format!("Invalid minutes in timestamp: {}", parts[1]),
        })?;

        let seconds_parts: Vec<&str> = parts[2].split('.').collect();
        let seconds: u32 = seconds_parts[0]
            .parse()
            .map_err(|_| EditorError::ValidationError {
                message: format!("Invalid seconds in timestamp: {}", seconds_parts[0]),
            })?;

        let centiseconds = if seconds_parts.len() > 1 {
            // Convert milliseconds to centiseconds
            let millis: u32 = seconds_parts[1].parse().unwrap_or(0);
            millis / 10
        } else {
            0
        };

        Ok(format!(
            "{hours}:{minutes:02}:{seconds:02}.{centiseconds:02}"
        ))
    }

    /// Convert SRT formatting to ASS
    fn convert_srt_formatting(text: &str) -> String {
        let mut result = text.to_string();

        // Convert basic HTML-like tags
        result = result.replace("<i>", "{\\i1}");
        result = result.replace("</i>", "{\\i0}");
        result = result.replace("<b>", "{\\b1}");
        result = result.replace("</b>", "{\\b0}");
        result = result.replace("<u>", "{\\u1}");
        result = result.replace("</u>", "{\\u0}");

        // Remove any other HTML tags
        #[cfg(feature = "formats")]
        {
            result = regex::Regex::new(r"<[^>]+>")
                .unwrap()
                .replace_all(&result, "")
                .to_string();
        }

        result
    }
}

/// Helper struct for SRT entries
#[derive(Default)]
struct SrtEntry {
    start: String,
    end: String,
    text: String,
}
