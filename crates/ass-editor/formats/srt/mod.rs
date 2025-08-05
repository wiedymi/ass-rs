//! SRT (SubRip) format support with style preservation.
//!
//! This module provides import/export functionality for SRT files,
//! with comprehensive style preservation through ASS-style tags.

use crate::core::{EditorDocument, EditorError};
use crate::formats::{
    Format, FormatExporter, FormatImporter, FormatInfo, FormatOptions, FormatResult,
};
use ass_core::parser::Script;
use std::io::{Read, Write};

/// SRT format handler with style preservation
#[derive(Debug)]
pub struct SrtFormat {
    info: FormatInfo,
}

impl SrtFormat {
    /// Create a new SRT format handler
    pub fn new() -> Self {
        Self {
            info: FormatInfo {
                name: "SRT".to_string(),
                extensions: vec!["srt".to_string()],
                mime_type: "text/srt".to_string(),
                description: "SubRip subtitle format with style preservation".to_string(),
                supports_styling: true,
                supports_positioning: false,
            },
        }
    }

    /// Parse SRT timestamp (HH:MM:SS,mmm)
    fn parse_srt_time(time_str: &str) -> Result<String, EditorError> {
        let time_str = time_str.trim();

        // Convert SRT time format (HH:MM:SS,mmm) to ASS format (H:MM:SS.cc)
        if let Some(comma_pos) = time_str.find(',') {
            let (time_part, ms_part) = time_str.split_at(comma_pos);
            let ms_part = &ms_part[1..]; // Remove comma

            // Parse milliseconds and convert to centiseconds
            let ms: u32 = ms_part.parse().map_err(|_| {
                EditorError::InvalidFormat(format!("Invalid milliseconds: {ms_part}"))
            })?;
            let cs = ms / 10; // Convert to centiseconds

            // Remove leading zero from hours if present for ASS format
            let time_part = if time_part.starts_with("0") && time_part.len() > 1 {
                &time_part[1..]
            } else {
                time_part
            };

            Ok(format!("{time_part}.{cs:02}"))
        } else {
            Err(EditorError::InvalidFormat(format!(
                "Invalid SRT time format: {time_str}"
            )))
        }
    }

    /// Convert ASS timestamp to SRT format
    fn format_srt_time(ass_time: &str) -> Result<String, EditorError> {
        let ass_time = ass_time.trim();

        // Convert ASS time format (H:MM:SS.cc) to SRT format (HH:MM:SS,mmm)
        if let Some(dot_pos) = ass_time.find('.') {
            let (time_part, cs_part) = ass_time.split_at(dot_pos);
            let cs_part = &cs_part[1..]; // Remove dot

            // Parse centiseconds and convert to milliseconds
            let cs: u32 = cs_part.parse().map_err(|_| {
                EditorError::InvalidFormat(format!("Invalid centiseconds: {cs_part}"))
            })?;
            let ms = cs * 10; // Convert to milliseconds

            // Ensure hours are zero-padded for SRT format
            let parts: Vec<&str> = time_part.split(':').collect();
            if parts.len() == 3 {
                let hours: u32 = parts[0].parse().map_err(|_| {
                    EditorError::InvalidFormat(format!("Invalid hours: {}", parts[0]))
                })?;
                Ok(format!("{hours:02}:{}:{},{ms:03}", parts[1], parts[2]))
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

    /// Convert SRT styling to ASS override tags
    fn convert_srt_to_ass_styling(text: &str) -> String {
        let mut result = text.to_string();

        // Convert HTML-like tags to ASS override tags
        result = result.replace("<b>", r"{\b1}");
        result = result.replace("</b>", r"{\b0}");
        result = result.replace("<i>", r"{\i1}");
        result = result.replace("</i>", r"{\i0}");
        result = result.replace("<u>", r"{\u1}");
        result = result.replace("</u>", r"{\u0}");
        result = result.replace("<s>", r"{\s1}");
        result = result.replace("</s>", r"{\s0}");

        #[cfg(feature = "formats")]
        {
            // Handle font color tags
            let color_regex = regex::Regex::new(r#"<font color="?#?([0-9A-Fa-f]{6})"?>"#).unwrap();
            result = color_regex.replace_all(&result, r"{\c&H$1&}").to_string();
            result = result.replace("</font>", r"{\c}");

            // Handle font face tags
            let font_regex = regex::Regex::new(r#"<font face="([^"]+)">"#).unwrap();
            result = font_regex.replace_all(&result, r"{\fn$1}").to_string();
        }

        result
    }

    /// Convert ASS override tags to SRT styling
    fn convert_ass_to_srt_styling(text: &str) -> String {
        let mut result = text.to_string();

        // Convert ASS override tags to HTML-like tags
        result = result.replace(r"{\b1}", "<b>");
        result = result.replace(r"{\b0}", "</b>");
        result = result.replace(r"{\i1}", "<i>");
        result = result.replace(r"{\i0}", "</i>");
        result = result.replace(r"{\u1}", "<u>");
        result = result.replace(r"{\u0}", "</u>");
        result = result.replace(r"{\s1}", "<s>");
        result = result.replace(r"{\s0}", "</s>");

        #[cfg(feature = "formats")]
        {
            // Handle color tags
            let color_regex = regex::Regex::new(r"\\c&H([0-9A-Fa-f]{6})&").unwrap();
            result = color_regex
                .replace_all(&result, "<font color=\"#$1\">")
                .to_string();
            result = result.replace(r"{\c}", "</font>");

            // Handle font name tags
            let font_regex = regex::Regex::new(r"\\fn([^}]+)").unwrap();
            result = font_regex
                .replace_all(&result, "<font face=\"$1\">")
                .to_string();

            // Remove any remaining ASS tags
            let cleanup_regex = regex::Regex::new(r"\{[^}]*\}").unwrap();
            result = cleanup_regex.replace_all(&result, "").to_string();
        }

        result
    }

    /// Parse SRT subtitle entry
    fn parse_srt_subtitle(
        lines: &[String],
        start_idx: usize,
    ) -> Result<(usize, String), EditorError> {
        if start_idx >= lines.len() {
            return Err(EditorError::InvalidFormat(
                "Unexpected end of file".to_string(),
            ));
        }

        let mut idx = start_idx;

        // Skip empty lines
        while idx < lines.len() && lines[idx].trim().is_empty() {
            idx += 1;
        }

        if idx >= lines.len() {
            return Err(EditorError::InvalidFormat(
                "Unexpected end of file".to_string(),
            ));
        }

        // Parse subtitle number (optional validation)
        let _subtitle_num = lines[idx].trim();
        idx += 1;

        if idx >= lines.len() {
            return Err(EditorError::InvalidFormat(
                "Missing timestamp line".to_string(),
            ));
        }

        // Parse timestamp line
        let timestamp_line = &lines[idx];
        if !timestamp_line.contains("-->") {
            return Err(EditorError::InvalidFormat(format!(
                "Invalid timestamp line: {timestamp_line}"
            )));
        }

        let parts: Vec<&str> = timestamp_line.split("-->").collect();
        if parts.len() != 2 {
            return Err(EditorError::InvalidFormat(format!(
                "Invalid timestamp format: {timestamp_line}"
            )));
        }

        let start_time = Self::parse_srt_time(parts[0])?;
        let end_time = Self::parse_srt_time(parts[1])?;

        idx += 1;

        // Collect subtitle text lines
        let mut text_lines = Vec::new();
        while idx < lines.len() && !lines[idx].trim().is_empty() {
            let styled_text = Self::convert_srt_to_ass_styling(&lines[idx]);
            text_lines.push(styled_text);
            idx += 1;
        }

        if text_lines.is_empty() {
            return Err(EditorError::InvalidFormat(
                "Empty subtitle text".to_string(),
            ));
        }

        let text = text_lines.join("\\N"); // ASS line break
        let dialogue_line = format!("Dialogue: 0,{start_time},{end_time},Default,,0,0,0,,{text}");

        Ok((idx, dialogue_line))
    }
}

impl Default for SrtFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatImporter for SrtFormat {
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
            .map_err(|e| EditorError::IoError(format!("Failed to read SRT content: {e}")))?;

        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut warnings = Vec::new();
        let mut dialogues = Vec::new();
        let mut idx = 0;
        let mut subtitle_count = 0;

        // Parse all SRT subtitles
        while idx < lines.len() {
            match Self::parse_srt_subtitle(&lines, idx) {
                Ok((next_idx, dialogue)) => {
                    dialogues.push(dialogue);
                    idx = next_idx;
                    subtitle_count += 1;
                }
                Err(e) => {
                    if idx < lines.len() {
                        warnings.push(format!(
                            "Skipping invalid subtitle at line {}: {e}",
                            idx + 1
                        ));
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
        ass_content.push_str("Title: Converted from SRT\n");
        ass_content.push_str("ScriptType: v4.00+\n");
        ass_content.push_str("Collisions: Normal\n");
        ass_content.push_str("PlayDepth: 0\n");
        ass_content.push_str("Timer: 100.0000\n");
        ass_content.push_str("Video Aspect Ratio: 0\n");
        ass_content.push_str("Video Zoom: 6\n");
        ass_content.push_str("Video Position: 0\n\n");

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
        let mut result = FormatResult::success(subtitle_count)
            .with_metadata("original_format".to_string(), "SRT".to_string())
            .with_metadata("subtitles_count".to_string(), subtitle_count.to_string())
            .with_metadata("encoding".to_string(), options.encoding.clone());

        if !warnings.is_empty() {
            result = result.with_warnings(warnings);
        }

        Ok((document, result))
    }
}

impl FormatExporter for SrtFormat {
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

        let mut srt_content = String::new();
        let mut subtitle_num = 1;
        let mut warnings = Vec::new();

        for (event_type, start, end, text) in &events {
            // Only export dialogue events
            if event_type.as_str() != "Dialogue" {
                continue;
            }

            // Parse start and end times
            let start_time = match Self::format_srt_time(start) {
                Ok(time) => time,
                Err(e) => {
                    warnings.push(format!(
                        "Invalid start time for subtitle {subtitle_num}: {e}"
                    ));
                    continue;
                }
            };

            let end_time = match Self::format_srt_time(end) {
                Ok(time) => time,
                Err(e) => {
                    warnings.push(format!("Invalid end time for subtitle {subtitle_num}: {e}"));
                    continue;
                }
            };

            // Convert ASS text to SRT format
            let mut text = text.clone();

            // Convert ASS line breaks to actual line breaks
            text = text.replace("\\N", "\n");
            text = text.replace("\\n", "\n");

            // Convert ASS styling to SRT styling
            text = Self::convert_ass_to_srt_styling(&text);

            // Write SRT subtitle entry
            srt_content.push_str(&format!("{subtitle_num}\n"));
            srt_content.push_str(&format!("{start_time} --> {end_time}\n"));
            srt_content.push_str(&text);
            srt_content.push_str("\n\n");

            subtitle_num += 1;
        }

        // Write content with proper encoding
        let bytes = if options.encoding.eq_ignore_ascii_case("UTF-8") {
            srt_content.into_bytes()
        } else {
            warnings.push(format!(
                "Encoding '{}' not supported, using UTF-8 instead",
                options.encoding
            ));
            srt_content.into_bytes()
        };

        writer
            .write_all(&bytes)
            .map_err(|e| EditorError::IoError(format!("Failed to write SRT content: {e}")))?;

        let mut result = FormatResult::success(subtitle_num - 1)
            .with_metadata("exported_format".to_string(), "SRT".to_string())
            .with_metadata(
                "subtitles_exported".to_string(),
                (subtitle_num - 1).to_string(),
            );

        if !warnings.is_empty() {
            result = result.with_warnings(warnings);
        }

        Ok(result)
    }
}

impl Format for SrtFormat {
    fn as_importer(&self) -> &dyn FormatImporter {
        self
    }

    fn as_exporter(&self) -> &dyn FormatExporter {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;
    #[cfg(not(feature = "std"))]
    use alloc::{format, string::String, vec};

    const SAMPLE_SRT: &str = "1\n00:00:00,000 --> 00:00:05,000\n<b>Hello</b> <i>World</i>!\n\n2\n00:00:06,000 --> 00:00:10,000\nThis is a <u>subtitle</u> with <font color=\"#FF0000\">red text</font>.\n\n3\n00:00:12,500 --> 00:00:15,750\nMultiple\nlines\nhere\n\n";

    #[test]
    fn test_srt_format_creation() {
        let format = SrtFormat::new();
        let info = FormatImporter::format_info(&format);
        assert_eq!(info.name, "SRT");
        assert!(info.supports_styling);
        assert!(!info.supports_positioning);
        assert!(format.can_import("srt"));
        assert!(format.can_export("srt"));
    }

    #[test]
    fn test_parse_srt_time() {
        assert_eq!(
            SrtFormat::parse_srt_time("00:01:23,456").unwrap(),
            "0:01:23.45"
        );
        assert_eq!(
            SrtFormat::parse_srt_time("01:00:00,000").unwrap(),
            "1:00:00.00"
        );
        assert_eq!(
            SrtFormat::parse_srt_time("10:30:45,123").unwrap(),
            "10:30:45.12"
        );

        assert!(SrtFormat::parse_srt_time("invalid").is_err());
        assert!(SrtFormat::parse_srt_time("00:01:23").is_err());
    }

    #[test]
    fn test_format_srt_time() {
        assert_eq!(
            SrtFormat::format_srt_time("0:01:23.45").unwrap(),
            "00:01:23,450"
        );
        assert_eq!(
            SrtFormat::format_srt_time("1:00:00.00").unwrap(),
            "01:00:00,000"
        );
        assert_eq!(
            SrtFormat::format_srt_time("10:30:45.12").unwrap(),
            "10:30:45,120"
        );

        assert!(SrtFormat::format_srt_time("invalid").is_err());
        assert!(SrtFormat::format_srt_time("00:01:23").is_err());
    }

    #[test]
    fn test_convert_srt_to_ass_styling() {
        assert_eq!(
            SrtFormat::convert_srt_to_ass_styling("<b>Bold</b> text"),
            r"{\b1}Bold{\b0} text"
        );
        assert_eq!(
            SrtFormat::convert_srt_to_ass_styling("<i>Italic</i> and <u>underlined</u>"),
            r"{\i1}Italic{\i0} and {\u1}underlined{\u0}"
        );
        assert_eq!(
            SrtFormat::convert_srt_to_ass_styling("<font color=\"#FF0000\">Red text</font>"),
            r"{\c&HFF0000&}Red text{\c}"
        );
    }

    #[test]
    fn test_convert_ass_to_srt_styling() {
        assert_eq!(
            SrtFormat::convert_ass_to_srt_styling(r"{\b1}Bold{\b0} text"),
            "<b>Bold</b> text"
        );
        assert_eq!(
            SrtFormat::convert_ass_to_srt_styling(r"{\i1}Italic{\i0} and {\u1}underlined{\u0}"),
            "<i>Italic</i> and <u>underlined</u>"
        );
    }

    #[test]
    fn test_srt_import_from_string() {
        let format = SrtFormat::new();
        let options = FormatOptions::default();

        let result = format.import_from_string(SAMPLE_SRT, &options);
        assert!(result.is_ok());

        let (document, format_result) = result.unwrap();
        assert!(format_result.success);
        assert_eq!(format_result.lines_processed, 3); // 3 subtitles
        assert!(document.text().contains("Hello"));
        assert!(document.text().contains("World"));
        assert!(document.text().contains(r"{\b1}"));
        assert!(document.text().contains(r"{\i1}"));
    }

    #[test]
    fn test_srt_export_to_string() {
        let format = SrtFormat::new();
        let options = FormatOptions::default();

        // First import
        let (document, _) = format.import_from_string(SAMPLE_SRT, &options).unwrap();

        // Then export
        let result = format.export_to_string(&document, &options);
        assert!(result.is_ok());

        let (exported_content, format_result) = result.unwrap();
        assert!(format_result.success);
        assert!(exported_content.contains("Hello"));
        assert!(exported_content.contains("<b>"));
        assert!(exported_content.contains("<i>"));
        assert!(exported_content.contains("00:00:00,000 --> 00:00:05,000"));
    }

    #[test]
    fn test_srt_roundtrip_basic() {
        let format = SrtFormat::new();
        let options = FormatOptions::default();

        let simple_srt = "1\n00:00:01,000 --> 00:00:03,000\nHello World\n\n";

        // Import -> Export -> Import
        let (document1, _) = format.import_from_string(simple_srt, &options).unwrap();
        let (exported_content, _) = format.export_to_string(&document1, &options).unwrap();

        // Verify basic structure is preserved
        assert!(exported_content.contains("Hello World"));
        assert!(exported_content.contains("00:00:01,000 --> 00:00:03,000"));
    }

    #[test]
    fn test_srt_style_preservation() {
        let format = SrtFormat::new();
        let options = FormatOptions::default();

        let styled_srt = r#"1
00:00:00,000 --> 00:00:02,000
<b>Bold</b> and <i>italic</i> text

"#;

        let (document, _) = format.import_from_string(styled_srt, &options).unwrap();
        let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

        // Verify styles are preserved
        assert!(exported_content.contains("<b>Bold</b>"));
        assert!(exported_content.contains("<i>italic</i>"));
    }

    #[test]
    fn test_srt_multiline_handling() {
        let format = SrtFormat::new();
        let options = FormatOptions::default();

        let multiline_srt = r#"1
00:00:00,000 --> 00:00:02,000
Line one
Line two
Line three

"#;

        let (document, _) = format.import_from_string(multiline_srt, &options).unwrap();
        let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

        // Verify multiline content is preserved
        assert!(exported_content.contains("Line one"));
        assert!(exported_content.contains("Line two"));
        assert!(exported_content.contains("Line three"));
    }

    #[test]
    fn test_srt_error_handling() {
        let format = SrtFormat::new();
        let options = FormatOptions::default();

        let invalid_srt = "Invalid SRT content";
        let result = format.import_from_string(invalid_srt, &options);

        // Should handle gracefully and return warnings
        if let Ok((_, format_result)) = result {
            assert!(!format_result.warnings.is_empty());
        }
    }

    #[test]
    fn test_srt_metadata_extraction() {
        let format = SrtFormat::new();
        let options = FormatOptions::default();

        let (_, format_result) = format.import_from_string(SAMPLE_SRT, &options).unwrap();

        assert_eq!(
            format_result.metadata.get("original_format"),
            Some(&"SRT".to_string())
        );
        assert_eq!(
            format_result.metadata.get("subtitles_count"),
            Some(&"3".to_string())
        );
        assert_eq!(
            format_result.metadata.get("encoding"),
            Some(&"UTF-8".to_string())
        );
    }
}
