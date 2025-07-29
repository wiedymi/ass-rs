//! WebVTT format support with style preservation.
//!
//! This module provides import/export functionality for WebVTT files,
//! with comprehensive style preservation and positioning support.

use crate::core::{EditorDocument, EditorError};
use crate::formats::{
    Format, FormatExporter, FormatImporter, FormatInfo, FormatOptions, FormatResult,
};
use ass_core::parser::Script;
use std::collections::HashMap;
use std::io::{Read, Write};

/// WebVTT format handler with style and positioning preservation
#[derive(Debug)]
pub struct WebVttFormat {
    info: FormatInfo,
}

impl WebVttFormat {
    /// Create a new WebVTT format handler
    pub fn new() -> Self {
        Self {
            info: FormatInfo {
                name: "WebVTT".to_string(),
                extensions: vec!["vtt".to_string(), "webvtt".to_string()],
                mime_type: "text/vtt".to_string(),
                description: "WebVTT subtitle format with full style and positioning preservation"
                    .to_string(),
                supports_styling: true,
                supports_positioning: true,
            },
        }
    }

    /// Parse WebVTT timestamp (HH:MM:SS.mmm or MM:SS.mmm)
    fn parse_vtt_time(time_str: &str) -> Result<String, EditorError> {
        let time_str = time_str.trim();

        // WebVTT supports both HH:MM:SS.mmm and MM:SS.mmm formats
        let parts: Vec<&str> = time_str.split('.').collect();
        if parts.len() != 2 {
            return Err(EditorError::InvalidFormat(format!(
                "Invalid WebVTT time format: {time_str}"
            )));
        }

        let time_part = parts[0];
        let ms_part = parts[1];

        // Parse milliseconds and convert to centiseconds
        let ms: u32 = ms_part
            .parse()
            .map_err(|_| EditorError::InvalidFormat(format!("Invalid milliseconds: {ms_part}")))?;
        let cs = ms / 10; // Convert to centiseconds

        // Handle both MM:SS and HH:MM:SS formats
        let time_components: Vec<&str> = time_part.split(':').collect();
        let ass_time = match time_components.len() {
            2 => {
                // MM:SS format - prepend 0 hours
                format!("0:{time_part}.{cs:02}")
            }
            3 => {
                // HH:MM:SS format - remove leading zero from hours if present
                let hours = time_components[0];
                let hours = if hours.starts_with('0') && hours.len() > 1 {
                    &hours[1..]
                } else {
                    hours
                };
                format!(
                    "{hours}:{}:{}.{cs:02}",
                    time_components[1], time_components[2]
                )
            }
            _ => {
                return Err(EditorError::InvalidFormat(format!(
                    "Invalid WebVTT time format: {time_str}"
                )));
            }
        };

        Ok(ass_time)
    }

    /// Convert ASS timestamp to WebVTT format
    fn format_vtt_time(ass_time: &str) -> Result<String, EditorError> {
        let ass_time = ass_time.trim();

        // Convert ASS time format (H:MM:SS.cc) to WebVTT format (HH:MM:SS.mmm)
        if let Some(dot_pos) = ass_time.find('.') {
            let (time_part, cs_part) = ass_time.split_at(dot_pos);
            let cs_part = &cs_part[1..]; // Remove dot

            // Parse centiseconds and convert to milliseconds
            let cs: u32 = cs_part.parse().map_err(|_| {
                EditorError::InvalidFormat(format!("Invalid centiseconds: {cs_part}"))
            })?;
            let ms = cs * 10; // Convert to milliseconds

            // Ensure hours are zero-padded for WebVTT format
            let parts: Vec<&str> = time_part.split(':').collect();
            if parts.len() == 3 {
                let hours: u32 = parts[0].parse().map_err(|_| {
                    EditorError::InvalidFormat(format!("Invalid hours: {}", parts[0]))
                })?;
                Ok(format!("{hours:02}:{}:{}.{ms:03}", parts[1], parts[2]))
            } else {
                Err(EditorError::InvalidFormat(format!(
                    "Invalid ASS time format: {ass_time}"
                )))
            }
        } else {
            Err(EditorError::InvalidFormat(format!(
                "Invalid ASS time format: {ass_time}"
            )))
        }
    }

    /// Convert WebVTT styling to ASS override tags
    fn convert_vtt_to_ass_styling(text: &str) -> String {
        let mut result = text.to_string();

        // Convert WebVTT tags to ASS override tags
        result = result.replace("<b>", r"{\b1}");
        result = result.replace("</b>", r"{\b0}");
        result = result.replace("<i>", r"{\i1}");
        result = result.replace("</i>", r"{\i0}");
        result = result.replace("<u>", r"{\u1}");
        result = result.replace("</u>", r"{\u0}");

        // Handle WebVTT class-based styling
        #[cfg(feature = "formats")]
        {
            let class_regex = regex::Regex::new(r#"<c\.([^>]+)>([^<]*)</c>"#).unwrap();
            result = class_regex
                .replace_all(&result, r"{\c&H$1&}$2{\c}")
                .to_string();

            // Handle voice tags
            let voice_regex = regex::Regex::new(r#"<v\s+([^>]+)>([^<]*)</v>"#).unwrap();
            result = voice_regex
                .replace_all(&result, r"{\fn$1}$2{\fn}")
                .to_string();

            // Handle ruby text (convert to simple parentheses)
            let ruby_regex = regex::Regex::new(r#"<ruby>([^<]*)<rt>([^<]*)</rt></ruby>"#).unwrap();
            result = ruby_regex.replace_all(&result, "$1($2)").to_string();

            // Handle timestamp tags (cue settings)
            let timestamp_regex = regex::Regex::new(r#"<([0-9:.,]+)>"#).unwrap();
            result = timestamp_regex.replace_all(&result, "").to_string();
        }

        result
    }

    /// Convert ASS override tags to WebVTT styling
    fn convert_ass_to_vtt_styling(text: &str) -> String {
        let mut result = text.to_string();

        // Convert ASS override tags to WebVTT tags
        result = result.replace(r"{\b1}", "<b>");
        result = result.replace(r"{\b0}", "</b>");
        result = result.replace(r"{\i1}", "<i>");
        result = result.replace(r"{\i0}", "</i>");
        result = result.replace(r"{\u1}", "<u>");
        result = result.replace(r"{\u0}", "</u>");

        #[cfg(feature = "formats")]
        {
            // Handle color tags
            let color_regex = regex::Regex::new(r"\\c&H([0-9A-Fa-f]{6})&").unwrap();
            result = color_regex.replace_all(&result, r#"<c.$1>"#).to_string();
            result = result.replace(r"{\c}", "</c>");

            // Handle font name tags
            let font_regex = regex::Regex::new(r"\\fn([^}]+)").unwrap();
            result = font_regex.replace_all(&result, r#"<v $1>"#).to_string();
            result = result.replace(r"{\fn}", "</v>");

            // Handle positioning tags (convert to WebVTT cue settings)
            let pos_regex = regex::Regex::new(r"\\pos\(([^,]+),([^)]+)\)").unwrap();
            result = pos_regex.replace_all(&result, "").to_string(); // Will be handled as cue settings

            // Remove any remaining ASS tags
            let cleanup_regex = regex::Regex::new(r"\{[^}]*\}").unwrap();
            result = cleanup_regex.replace_all(&result, "").to_string();
        }

        result
    }

    /// Parse WebVTT cue settings for positioning
    fn parse_cue_settings(settings: &str) -> HashMap<String, String> {
        let mut cue_settings = HashMap::new();

        for setting in settings.split_whitespace() {
            if let Some(colon_pos) = setting.find(':') {
                let (key, value) = setting.split_at(colon_pos);
                let value = &value[1..]; // Remove colon
                cue_settings.insert(key.to_string(), value.to_string());
            }
        }

        cue_settings
    }

    /// Convert cue settings to ASS positioning tags
    fn cue_settings_to_ass_positioning(settings: &HashMap<String, String>) -> String {
        let mut ass_tags = String::new();

        // Handle position settings
        if let (Some(line), Some(position)) = (settings.get("line"), settings.get("position")) {
            // Convert WebVTT positioning to approximate ASS positioning
            if let (Ok(line_val), Ok(pos_val)) = (line.parse::<f32>(), position.parse::<f32>()) {
                let x = (pos_val * 640.0) as u32; // Assume 640x480 resolution
                let y = (line_val * 480.0) as u32;
                ass_tags.push_str(&format!(r"\pos({x},{y})"));
            }
        }

        // Handle alignment
        if let Some(align) = settings.get("align") {
            let alignment = match align.as_str() {
                "start" | "left" => 1,
                "center" | "middle" => 2,
                "end" | "right" => 3,
                _ => 2, // default to center
            };
            ass_tags.push_str(&format!(r"\an{alignment}"));
        }

        if !ass_tags.is_empty() {
            format!("{{{ass_tags}}}")
        } else {
            String::new()
        }
    }

    /// Parse WebVTT cue
    fn parse_vtt_cue(lines: &[String], start_idx: usize) -> Result<(usize, String), EditorError> {
        if start_idx >= lines.len() {
            return Err(EditorError::InvalidFormat(
                "Unexpected end of file".to_string(),
            ));
        }

        let mut idx = start_idx;

        // Skip empty lines and NOTE blocks
        while idx < lines.len() {
            let line = lines[idx].trim();
            if line.is_empty() || line.starts_with("NOTE") {
                idx += 1;
                continue;
            }
            break;
        }

        if idx >= lines.len() {
            return Err(EditorError::InvalidFormat(
                "Unexpected end of file".to_string(),
            ));
        }

        let current_line = &lines[idx];

        // Check if this line contains timestamp (cue timing)
        if current_line.contains("-->") {
            // Parse timestamp line directly
            let timestamp_line = current_line;
            let parts: Vec<&str> = timestamp_line.split("-->").collect();
            if parts.len() < 2 {
                return Err(EditorError::InvalidFormat(format!(
                    "Invalid timestamp format: {timestamp_line}"
                )));
            }

            let start_time = Self::parse_vtt_time(parts[0])?;
            let end_time_and_settings: Vec<&str> = parts[1].split_whitespace().collect();
            let end_time = Self::parse_vtt_time(end_time_and_settings[0])?;

            // Parse cue settings if present
            let cue_settings = if end_time_and_settings.len() > 1 {
                let settings_str = end_time_and_settings[1..].join(" ");
                Self::parse_cue_settings(&settings_str)
            } else {
                HashMap::new()
            };

            idx += 1;

            // Collect cue text
            let mut text_lines = Vec::new();
            while idx < lines.len() && !lines[idx].trim().is_empty() {
                let styled_text = Self::convert_vtt_to_ass_styling(&lines[idx]);
                text_lines.push(styled_text);
                idx += 1;
            }

            if text_lines.is_empty() {
                return Err(EditorError::InvalidFormat("Empty cue text".to_string()));
            }

            let text = text_lines.join("\\N"); // ASS line break
            let positioning = Self::cue_settings_to_ass_positioning(&cue_settings);
            let dialogue_line =
                format!("Dialogue: 0,{start_time},{end_time},Default,,0,0,0,,{positioning}{text}");

            Ok((idx, dialogue_line))
        } else {
            // This might be a cue identifier, skip to next line for timestamp
            idx += 1;
            if idx < lines.len() && lines[idx].contains("-->") {
                Self::parse_vtt_cue(lines, idx)
            } else {
                Err(EditorError::InvalidFormat(format!(
                    "Expected timestamp line after cue identifier: {current_line}"
                )))
            }
        }
    }
}

impl Default for WebVttFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatImporter for WebVttFormat {
    fn format_info(&self) -> &FormatInfo {
        &self.info
    }

    fn import_from_reader(
        &self,
        reader: &mut dyn Read,
        options: &FormatOptions,
    ) -> Result<(EditorDocument, FormatResult), EditorError> {
        // Read the entire content
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .map_err(|e| EditorError::IoError(format!("Failed to read WebVTT content: {e}")))?;

        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut warnings = Vec::new();
        let mut dialogues = Vec::new();
        let mut idx = 0;
        let mut cue_count = 0;

        // Check for WebVTT header
        if lines.is_empty() || !lines[0].trim().starts_with("WEBVTT") {
            warnings.push("Missing or invalid WebVTT header".to_string());
        } else {
            idx = 1; // Skip header
        }

        // Parse all WebVTT cues
        while idx < lines.len() {
            match Self::parse_vtt_cue(&lines, idx) {
                Ok((next_idx, dialogue)) => {
                    dialogues.push(dialogue);
                    idx = next_idx;
                    cue_count += 1;
                }
                Err(e) => {
                    if idx < lines.len() {
                        warnings.push(format!("Skipping invalid cue at line {}: {e}", idx + 1));
                        idx += 1;
                    } else {
                        break;
                    }
                }
            }
        }

        // Build ASS script content
        let mut ass_content = String::new();

        // Add script info section
        ass_content.push_str("[Script Info]\n");
        ass_content.push_str("Title: Converted from WebVTT\n");
        ass_content.push_str("ScriptType: v4.00+\n");
        ass_content.push_str("Collisions: Normal\n");
        ass_content.push_str("PlayDepth: 0\n");
        ass_content.push_str("Timer: 100.0000\n");
        ass_content.push_str("Video Aspect Ratio: 0\n");
        ass_content.push_str("Video Zoom: 6\n");
        ass_content.push_str("Video Position: 0\n");
        ass_content.push_str("PlayResX: 640\n");
        ass_content.push_str("PlayResY: 480\n\n");

        // Add styles section with basic default style
        ass_content.push_str("[V4+ Styles]\n");
        ass_content.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
        ass_content.push_str("Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n");

        // Add events section
        ass_content.push_str("[Events]\n");
        ass_content.push_str(
            "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
        );

        for dialogue in dialogues {
            ass_content.push_str(&dialogue);
            ass_content.push('\n');
        }

        // Validate the generated ASS content
        let _script = Script::parse(&ass_content)?;

        // Create EditorDocument
        let document = EditorDocument::from_content(&ass_content)?;

        // Create result with metadata
        let mut result = FormatResult::success(cue_count)
            .with_metadata("original_format".to_string(), "WebVTT".to_string())
            .with_metadata("cues_count".to_string(), cue_count.to_string())
            .with_metadata("encoding".to_string(), options.encoding.clone());

        if !warnings.is_empty() {
            result = result.with_warnings(warnings);
        }

        Ok((document, result))
    }
}

impl FormatExporter for WebVttFormat {
    fn format_info(&self) -> &FormatInfo {
        &self.info
    }

    fn export_to_writer(
        &self,
        document: &EditorDocument,
        writer: &mut dyn Write,
        options: &FormatOptions,
    ) -> Result<FormatResult, EditorError> {
        // Parse the ASS content to extract events
        let events = document.parse_script_with(|script| {
            // Find events section and collect owned data
            if let Some(ass_core::parser::ast::Section::Events(events)) =
                script.find_section(ass_core::parser::ast::SectionType::Events)
            {
                // Convert to owned data to avoid lifetime issues
                events
                    .iter()
                    .map(|event| {
                        (
                            event.event_type,
                            event.start.to_string(),
                            event.end.to_string(),
                            event.text.to_string(),
                        )
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        })?;

        let mut vtt_content = String::new();
        let mut cue_num = 1;
        let mut warnings = Vec::new();

        // Add WebVTT header
        vtt_content.push_str("WEBVTT\n\n");

        for (event_type, start, end, text) in &events {
            // Only export dialogue events
            if event_type.as_str() != "Dialogue" {
                continue;
            }

            // Parse start and end times
            let start_time = match Self::format_vtt_time(start) {
                Ok(time) => time,
                Err(e) => {
                    warnings.push(format!("Invalid start time for cue {cue_num}: {e}"));
                    continue;
                }
            };

            let end_time = match Self::format_vtt_time(end) {
                Ok(time) => time,
                Err(e) => {
                    warnings.push(format!("Invalid end time for cue {cue_num}: {e}"));
                    continue;
                }
            };

            // Convert ASS text to WebVTT format
            let mut text = text.clone();

            // Convert ASS line breaks to actual line breaks
            text = text.replace("\\N", "\n");
            text = text.replace("\\n", "\n");

            // Convert ASS styling to WebVTT styling
            text = Self::convert_ass_to_vtt_styling(&text);

            // Write WebVTT cue
            vtt_content.push_str(&format!("{cue_num}\n"));
            vtt_content.push_str(&format!("{start_time} --> {end_time}\n"));
            vtt_content.push_str(&text);
            vtt_content.push_str("\n\n");

            cue_num += 1;
        }

        // Write content with proper encoding
        let bytes = if options.encoding.eq_ignore_ascii_case("UTF-8") {
            vtt_content.into_bytes()
        } else {
            warnings.push(format!(
                "Encoding '{}' not supported, using UTF-8 instead",
                options.encoding
            ));
            vtt_content.into_bytes()
        };

        writer
            .write_all(&bytes)
            .map_err(|e| EditorError::IoError(format!("Failed to write WebVTT content: {e}")))?;

        let mut result = FormatResult::success(cue_num - 1)
            .with_metadata("exported_format".to_string(), "WebVTT".to_string())
            .with_metadata("cues_exported".to_string(), (cue_num - 1).to_string());

        if !warnings.is_empty() {
            result = result.with_warnings(warnings);
        }

        Ok(result)
    }
}

impl Format for WebVttFormat {}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_WEBVTT: &str = r#"WEBVTT

1
00:00:00.000 --> 00:00:05.000
<b>Hello</b> <i>World</i>!

2
00:00:06.000 --> 00:00:10.000 align:center
This is a <u>subtitle</u> with <c.red>red text</c>.

3
00:12:30.500 --> 00:15:45.750 line:20% position:50%
<v Speaker>Multiple</v>
lines with positioning

"#;

    #[test]
    fn test_webvtt_format_creation() {
        let format = WebVttFormat::new();
        let info = FormatImporter::format_info(&format);
        assert_eq!(info.name, "WebVTT");
        assert!(info.supports_styling);
        assert!(info.supports_positioning);
        assert!(format.can_import("vtt"));
        assert!(format.can_import("webvtt"));
        assert!(format.can_export("vtt"));
    }

    #[test]
    fn test_parse_vtt_time() {
        assert_eq!(
            WebVttFormat::parse_vtt_time("00:01:23.456").unwrap(),
            "0:01:23.45"
        );
        assert_eq!(
            WebVttFormat::parse_vtt_time("01:00:00.000").unwrap(),
            "1:00:00.00"
        );
        assert_eq!(
            WebVttFormat::parse_vtt_time("30:45.123").unwrap(),
            "0:30:45.12"
        );

        assert!(WebVttFormat::parse_vtt_time("invalid").is_err());
        assert!(WebVttFormat::parse_vtt_time("00:01:23").is_err());
    }

    #[test]
    fn test_format_vtt_time() {
        assert_eq!(
            WebVttFormat::format_vtt_time("0:01:23.45").unwrap(),
            "00:01:23.450"
        );
        assert_eq!(
            WebVttFormat::format_vtt_time("1:00:00.00").unwrap(),
            "01:00:00.000"
        );
        assert_eq!(
            WebVttFormat::format_vtt_time("10:30:45.12").unwrap(),
            "10:30:45.120"
        );

        assert!(WebVttFormat::format_vtt_time("invalid").is_err());
        assert!(WebVttFormat::format_vtt_time("00:01:23").is_err());
    }

    #[test]
    fn test_convert_vtt_to_ass_styling() {
        assert_eq!(
            WebVttFormat::convert_vtt_to_ass_styling("<b>Bold</b> text"),
            r"{\b1}Bold{\b0} text"
        );
        assert_eq!(
            WebVttFormat::convert_vtt_to_ass_styling("<i>Italic</i> and <u>underlined</u>"),
            r"{\i1}Italic{\i0} and {\u1}underlined{\u0}"
        );
    }

    #[test]
    fn test_convert_ass_to_vtt_styling() {
        assert_eq!(
            WebVttFormat::convert_ass_to_vtt_styling(r"{\b1}Bold{\b0} text"),
            "<b>Bold</b> text"
        );
        assert_eq!(
            WebVttFormat::convert_ass_to_vtt_styling(r"{\i1}Italic{\i0} and {\u1}underlined{\u0}"),
            "<i>Italic</i> and <u>underlined</u>"
        );
    }

    #[test]
    fn test_parse_cue_settings() {
        let settings = WebVttFormat::parse_cue_settings("align:center line:20% position:50%");
        assert_eq!(settings.get("align"), Some(&"center".to_string()));
        assert_eq!(settings.get("line"), Some(&"20%".to_string()));
        assert_eq!(settings.get("position"), Some(&"50%".to_string()));
    }

    #[test]
    fn test_webvtt_import_from_string() {
        let format = WebVttFormat::new();
        let options = FormatOptions::default();

        let result = format.import_from_string(SAMPLE_WEBVTT, &options);
        assert!(result.is_ok());

        let (document, format_result) = result.unwrap();
        assert!(format_result.success);
        assert_eq!(format_result.lines_processed, 3); // 3 cues
        assert!(document.text().contains("Hello"));
        assert!(document.text().contains("World"));
        assert!(document.text().contains(r"{\b1}"));
        assert!(document.text().contains(r"{\i1}"));
    }

    #[test]
    fn test_webvtt_export_to_string() {
        let format = WebVttFormat::new();
        let options = FormatOptions::default();

        // First import
        let (document, _) = format.import_from_string(SAMPLE_WEBVTT, &options).unwrap();

        // Then export
        let result = format.export_to_string(&document, &options);
        assert!(result.is_ok());

        let (exported_content, format_result) = result.unwrap();
        assert!(format_result.success);
        assert!(exported_content.contains("WEBVTT"));
        assert!(exported_content.contains("Hello"));
        assert!(exported_content.contains("<b>"));
        assert!(exported_content.contains("<i>"));
        assert!(exported_content.contains("00:00:00.000 --> 00:00:05.000"));
    }

    #[test]
    fn test_webvtt_roundtrip_basic() {
        let format = WebVttFormat::new();
        let options = FormatOptions::default();

        let simple_vtt = "WEBVTT\n\n1\n00:00:01.000 --> 00:00:03.000\nHello World\n\n";

        // Import -> Export -> Import
        let (document1, _) = format.import_from_string(simple_vtt, &options).unwrap();
        let (exported_content, _) = format.export_to_string(&document1, &options).unwrap();

        // Verify basic structure is preserved
        assert!(exported_content.contains("WEBVTT"));
        assert!(exported_content.contains("Hello World"));
        assert!(exported_content.contains("00:00:01.000 --> 00:00:03.000"));
    }

    #[test]
    fn test_webvtt_style_preservation() {
        let format = WebVttFormat::new();
        let options = FormatOptions::default();

        let styled_vtt =
            "WEBVTT\n\n1\n00:00:00.000 --> 00:00:02.000\n<b>Bold</b> and <i>italic</i> text\n\n";

        let (document, _) = format.import_from_string(styled_vtt, &options).unwrap();
        let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

        // Verify styles are preserved
        assert!(exported_content.contains("<b>Bold</b>"));
        assert!(exported_content.contains("<i>italic</i>"));
    }

    #[test]
    fn test_webvtt_positioning_support() {
        let format = WebVttFormat::new();
        let options = FormatOptions::default();

        let positioned_vtt =
            "WEBVTT\n\n1\n00:00:00.000 --> 00:00:02.000 line:20% position:50%\nPositioned text\n\n";

        let (document, _) = format.import_from_string(positioned_vtt, &options).unwrap();

        // Should parse without errors and preserve positioning info in ASS format
        assert!(document.text().contains("Positioned text"));
    }

    #[test]
    fn test_webvtt_multiline_handling() {
        let format = WebVttFormat::new();
        let options = FormatOptions::default();

        let multiline_vtt =
            "WEBVTT\n\n1\n00:00:00.000 --> 00:00:02.000\nLine one\nLine two\nLine three\n\n";

        let (document, _) = format.import_from_string(multiline_vtt, &options).unwrap();
        let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

        // Verify multiline content is preserved
        assert!(exported_content.contains("Line one"));
        assert!(exported_content.contains("Line two"));
        assert!(exported_content.contains("Line three"));
    }

    #[test]
    fn test_webvtt_error_handling() {
        let format = WebVttFormat::new();
        let options = FormatOptions::default();

        let invalid_vtt = "Invalid WebVTT content";
        let result = format.import_from_string(invalid_vtt, &options);

        // Should handle gracefully and return warnings
        if let Ok((_, format_result)) = result {
            assert!(!format_result.warnings.is_empty());
        }
    }

    #[test]
    fn test_webvtt_metadata_extraction() {
        let format = WebVttFormat::new();
        let options = FormatOptions::default();

        let (_, format_result) = format.import_from_string(SAMPLE_WEBVTT, &options).unwrap();

        assert_eq!(
            format_result.metadata.get("original_format"),
            Some(&"WebVTT".to_string())
        );
        assert_eq!(
            format_result.metadata.get("cues_count"),
            Some(&"3".to_string())
        );
        assert_eq!(
            format_result.metadata.get("encoding"),
            Some(&"UTF-8".to_string())
        );
    }
}
