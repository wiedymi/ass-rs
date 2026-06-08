//! SRT export: serialize an ASS [`EditorDocument`] into SRT text.

use super::SrtFormat;
use crate::core::{EditorDocument, EditorError};
use crate::formats::{FormatExporter, FormatInfo, FormatOptions, FormatResult};
use std::io::Write;

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
