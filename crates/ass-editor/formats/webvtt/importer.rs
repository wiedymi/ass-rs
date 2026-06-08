//! WebVTT import: parse `.vtt` content into an `EditorDocument`.
//!
//! Implements [`FormatImporter`] for [`WebVttFormat`], converting WebVTT cues
//! into an ASS script and validating the generated result.

use crate::core::{EditorDocument, EditorError};
use crate::formats::{FormatImporter, FormatInfo, FormatOptions, FormatResult};
use ass_core::parser::Script;
use std::io::Read;

use super::WebVttFormat;

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
