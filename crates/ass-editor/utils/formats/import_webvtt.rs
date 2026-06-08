//! WebVTT (`.vtt`) import support.
//!
//! Parses WebVTT cues into ASS dialogue lines, handling both `HH:MM:SS.mmm`
//! and `MM:SS.mmm` timestamps and converting WebVTT markup to ASS tags.

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
    /// Import WebVTT format
    pub(super) fn import_webvtt(content: &str) -> Result<String> {
        let mut output = String::new();

        // Add ASS header
        output.push_str("[Script Info]\n");
        output.push_str("Title: Imported from WebVTT\n");
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

        // Parse WebVTT cues
        let cues = Self::parse_webvtt_cues(content)?;
        for cue in cues {
            output.push_str(&format!(
                "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
                cue.start, cue.end, cue.text
            ));
        }

        Ok(output)
    }

    /// Parse WebVTT cues
    fn parse_webvtt_cues(content: &str) -> Result<Vec<WebVttCue>> {
        let mut cues = Vec::new();
        let mut current_cue: Option<WebVttCue> = None;
        let mut in_cue = false;

        for line in content.lines() {
            let line = line.trim();

            // Skip WEBVTT header and empty lines
            if line.starts_with("WEBVTT") || line.starts_with("NOTE") || line.is_empty() {
                if let Some(cue) = current_cue.take() {
                    cues.push(cue);
                }
                in_cue = false;
                continue;
            }

            // Check if it's a timestamp line
            if line.contains("-->") {
                current_cue = Some(WebVttCue::default());
                if let Some(ref mut cue) = current_cue {
                    let parts: Vec<&str> = line.split("-->").collect();
                    if parts.len() >= 2 {
                        cue.start = Self::parse_webvtt_time(parts[0].trim())?;
                        cue.end = Self::parse_webvtt_time(parts[1].trim())?;
                        in_cue = true;
                    }
                }
                continue;
            }

            // Otherwise it's cue text
            if in_cue {
                if let Some(ref mut cue) = current_cue {
                    if !cue.text.is_empty() {
                        cue.text.push_str("\\N");
                    }
                    let converted_text = Self::convert_webvtt_formatting(line);
                    cue.text.push_str(&converted_text);
                }
            }
        }

        // Don't forget the last cue
        if let Some(cue) = current_cue {
            cues.push(cue);
        }

        Ok(cues)
    }

    /// Parse WebVTT timestamp to ASS format
    fn parse_webvtt_time(time: &str) -> Result<String> {
        // WebVTT format: 00:00:00.000 or 00:00.000
        // ASS format: 0:00:00.00

        let parts: Vec<&str> = time.split(':').collect();

        let (hours, minutes, seconds_str) = if parts.len() == 3 {
            // HH:MM:SS.mmm
            (parts[0].parse::<u32>().unwrap_or(0), parts[1], parts[2])
        } else if parts.len() == 2 {
            // MM:SS.mmm
            (0, parts[0], parts[1])
        } else {
            return Err(EditorError::ValidationError {
                message: format!("Invalid WebVTT timestamp: {time}"),
            });
        };

        let minutes: u32 = minutes.parse().map_err(|_| EditorError::ValidationError {
            message: format!("Invalid minutes in timestamp: {minutes}"),
        })?;

        let seconds_parts: Vec<&str> = seconds_str.split('.').collect();
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

    /// Convert WebVTT formatting to ASS
    fn convert_webvtt_formatting(text: &str) -> String {
        let mut result = text.to_string();

        // Convert WebVTT tags
        result = result.replace("<i>", "{\\i1}");
        result = result.replace("</i>", "{\\i0}");
        result = result.replace("<b>", "{\\b1}");
        result = result.replace("</b>", "{\\b0}");
        result = result.replace("<u>", "{\\u1}");
        result = result.replace("</u>", "{\\u0}");

        // Convert voice spans
        result = regex::Regex::new(r"<v\s+([^>]+)>")
            .unwrap()
            .replace_all(&result, "")
            .to_string();
        result = result.replace("</v>", "");

        // Remove any other tags
        result = regex::Regex::new(r"<[^>]+>")
            .unwrap()
            .replace_all(&result, "")
            .to_string();

        result
    }
}

/// Helper struct for WebVTT cues
#[derive(Default)]
struct WebVttCue {
    start: String,
    end: String,
    text: String,
}
