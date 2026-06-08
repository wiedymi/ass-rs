//! Plain text import and export.
//!
//! Imports raw text into a single ASS dialogue line and exports dialogue text
//! with optional tag stripping.

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
    /// Import plain text
    pub(super) fn import_plain_text(content: &str) -> Result<String> {
        let mut output = String::new();

        // Add minimal ASS header
        output.push_str("[Script Info]\n");
        output.push_str("Title: Imported from Plain Text\n");
        output.push_str("ScriptType: v4.00+\n\n");

        output.push_str("[V4+ Styles]\n");
        output.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
        output.push_str("Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n");

        output.push_str("[Events]\n");
        output.push_str(
            "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
        );

        // Create a single dialogue line with all text
        let text = content.lines().collect::<Vec<_>>().join("\\N");
        output.push_str(&format!(
            "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{text}\n"
        ));

        Ok(output)
    }

    /// Export to plain text
    pub(super) fn export_plain_text(
        document: &EditorDocument,
        options: &ConversionOptions,
    ) -> Result<String> {
        let mut output = String::new();

        document.parse_script_with(|script| {
            for section in script.sections() {
                if let ass_core::parser::ast::Section::Events(events) = section {
                    for event in events {
                        if event.event_type == EventType::Dialogue {
                            let text = if options.strip_formatting {
                                Self::strip_ass_tags(event.text)
                            } else {
                                event.text.to_string()
                            };
                            output.push_str(&text.replace("\\N", "\n"));
                            output.push('\n');
                        }
                    }
                }
            }
        })?;

        Ok(output)
    }
}
