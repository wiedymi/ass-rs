//! SRT import: parse SRT entries into an ASS [`EditorDocument`].

use super::SrtFormat;
use crate::core::{EditorDocument, EditorError};
use crate::formats::{FormatImporter, FormatInfo, FormatOptions, FormatResult};
use ass_core::parser::Script;
use std::io::Read;

impl SrtFormat {
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
