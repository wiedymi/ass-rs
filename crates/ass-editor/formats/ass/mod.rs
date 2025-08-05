//! ASS (Advanced SubStation Alpha) format support.
//!
//! This module provides import/export functionality for ASS files,
//! leveraging ass-core's native parsing and serialization capabilities.

use crate::core::{EditorDocument, EditorError};
use crate::formats::{
    Format, FormatExporter, FormatImporter, FormatInfo, FormatOptions, FormatResult,
};
use ass_core::parser::Script;
use std::io::{Read, Write};

/// ASS format handler that reuses ass-core functionality
#[derive(Debug)]
pub struct AssFormat {
    info: FormatInfo,
}

impl AssFormat {
    /// Create a new ASS format handler
    pub fn new() -> Self {
        Self {
            info: FormatInfo {
                name: "ASS".to_string(),
                extensions: vec!["ass".to_string()],
                mime_type: "text/x-ass".to_string(),
                description: "Advanced SubStation Alpha subtitle format".to_string(),
                supports_styling: true,
                supports_positioning: true,
            },
        }
    }
}

impl Default for AssFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatImporter for AssFormat {
    fn format_info(&self) -> &FormatInfo {
        &self.info
    }

    fn import_from_reader(
        &self,
        reader: &mut dyn Read,
        _options: &FormatOptions,
    ) -> Result<(EditorDocument, FormatResult), EditorError> {
        // Read the entire content
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .map_err(|e| EditorError::IoError(format!("Failed to read content: {e}")))?;

        // Validate that it's parseable by ass-core
        let script = Script::parse(&content)?;

        // Count lines for result metadata
        let line_count = content.lines().count();

        // Create EditorDocument from the content
        let document = EditorDocument::from_content(&content)?;

        // Gather metadata from the parsed script
        let mut result = FormatResult::success(line_count);

        // Add script info as metadata
        if let Some(ass_core::parser::ast::Section::ScriptInfo(script_info)) =
            script.find_section(ass_core::parser::ast::SectionType::ScriptInfo)
        {
            if let Some(title) = script_info.get_field("Title") {
                result = result.with_metadata("title".to_string(), title.to_string());
            }
            if let Some(script_type) = script_info.get_field("ScriptType") {
                result = result.with_metadata("script_type".to_string(), script_type.to_string());
            }
        }

        // Count sections for additional metadata
        let section_count = script.sections().len();

        result = result.with_metadata("sections".to_string(), section_count.to_string());

        Ok((document, result))
    }
}

impl FormatExporter for AssFormat {
    fn format_info(&self) -> &FormatInfo {
        &self.info
    }

    fn export_to_writer(
        &self,
        document: &EditorDocument,
        writer: &mut dyn Write,
        options: &FormatOptions,
    ) -> Result<FormatResult, EditorError> {
        let content = if options.preserve_formatting {
            // Use the raw content to preserve exact formatting
            document.text()
        } else {
            // Use ass-core's serialization for normalized output
            document.parse_script_with(|script| script.to_ass_string())?
        };

        // Count lines before using content
        let line_count = content.lines().count();

        // Write content with proper encoding
        let bytes = if options.encoding.eq_ignore_ascii_case("UTF-8") {
            content.into_bytes()
        } else {
            // For non-UTF-8 encodings, we'd need additional encoding support
            // For now, default to UTF-8 with a warning
            let mut warnings = Vec::new();
            if !options.encoding.eq_ignore_ascii_case("UTF-8") {
                warnings.push(format!(
                    "Encoding '{}' not supported, using UTF-8 instead",
                    options.encoding
                ));
            }

            let result = FormatResult::success(line_count).with_warnings(warnings);

            writer
                .write_all(&content.into_bytes())
                .map_err(|e| EditorError::IoError(format!("Failed to write content: {e}")))?;

            return Ok(result);
        };

        writer
            .write_all(&bytes)
            .map_err(|e| EditorError::IoError(format!("Failed to write content: {e}")))?;

        Ok(FormatResult::success(line_count))
    }
}

impl Format for AssFormat {}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;
    #[cfg(not(feature = "std"))]
    use alloc::{format, string::String, vec};
    use std::io::Cursor;

    const SAMPLE_ASS: &str = r#"[Script Info]
Title: Test Script
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
"#;

    #[test]
    fn test_ass_format_creation() {
        let format = AssFormat::new();
        let info = FormatImporter::format_info(&format);
        assert_eq!(info.name, "ASS");
        assert!(info.supports_styling);
        assert!(info.supports_positioning);
        assert!(format.can_import("ass"));
        assert!(format.can_export("ass"));
    }

    #[test]
    fn test_ass_import_from_string() {
        let format = AssFormat::new();
        let options = FormatOptions::default();

        let result = format.import_from_string(SAMPLE_ASS, &options);
        assert!(result.is_ok());

        let (document, format_result) = result.unwrap();
        assert!(format_result.success);
        assert!(format_result.lines_processed > 0);
        assert!(document.text().contains("Hello World!"));
    }

    #[test]
    fn test_ass_export_to_string() {
        let format = AssFormat::new();
        let options = FormatOptions::default();

        // First import
        let (document, _) = format.import_from_string(SAMPLE_ASS, &options).unwrap();

        // Then export
        let result = format.export_to_string(&document, &options);
        assert!(result.is_ok());

        let (exported_content, format_result) = result.unwrap();
        assert!(format_result.success);
        assert!(exported_content.contains("Hello World!"));
        assert!(exported_content.contains("[Script Info]"));
    }

    #[test]
    fn test_ass_roundtrip() {
        let format = AssFormat::new();
        let options = FormatOptions::default();

        // Import -> Export -> Import
        let (document1, _) = format.import_from_string(SAMPLE_ASS, &options).unwrap();
        let (exported_content, _) = format.export_to_string(&document1, &options).unwrap();
        let (document2, _) = format
            .import_from_string(&exported_content, &options)
            .unwrap();

        // Should be equivalent
        assert_eq!(document1.text().trim(), document2.text().trim());
    }

    #[test]
    fn test_ass_import_with_reader() {
        let format = AssFormat::new();
        let options = FormatOptions::default();
        let mut cursor = Cursor::new(SAMPLE_ASS.as_bytes());

        let result = format.import_from_reader(&mut cursor, &options);
        assert!(result.is_ok());

        let (document, format_result) = result.unwrap();
        assert!(format_result.success);
        assert!(document.text().contains("Test Script"));
    }

    #[test]
    fn test_ass_export_with_writer() {
        let format = AssFormat::new();
        let options = FormatOptions::default();

        let (document, _) = format.import_from_string(SAMPLE_ASS, &options).unwrap();

        let mut buffer = Vec::new();
        let result = format.export_to_writer(&document, &mut buffer, &options);
        assert!(result.is_ok());

        let exported_content = String::from_utf8(buffer).unwrap();
        assert!(exported_content.contains("Hello World!"));
    }

    #[test]
    fn test_ass_export_preserve_formatting() {
        let format = AssFormat::new();
        let options = FormatOptions {
            preserve_formatting: true,
            ..FormatOptions::default()
        };

        let (document, _) = format.import_from_string(SAMPLE_ASS, &options).unwrap();
        let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

        // Should preserve original formatting
        assert_eq!(exported_content.trim(), SAMPLE_ASS.trim());
    }

    #[test]
    fn test_ass_export_normalized() {
        let format = AssFormat::new();
        let options = FormatOptions {
            preserve_formatting: false,
            ..FormatOptions::default()
        };

        // Import with some non-standard formatting
        let messy_ass = r#"[Script Info]

Title: Test Script
ScriptType: v4.00+


[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
"#;

        let (document, _) = format.import_from_string(messy_ass, &options).unwrap();
        let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

        // Should be normalized (no extra blank lines)
        assert!(exported_content.contains("[Script Info]\nTitle: Test Script"));
        assert!(!exported_content.contains("\n\n\n")); // No triple newlines
        assert!(exported_content.contains("Hello World!"));
    }

    #[test]
    fn test_ass_metadata_extraction() {
        let format = AssFormat::new();
        let options = FormatOptions::default();

        let (_, format_result) = format.import_from_string(SAMPLE_ASS, &options).unwrap();

        assert_eq!(
            format_result.metadata.get("title"),
            Some(&"Test Script".to_string())
        );
        assert_eq!(
            format_result.metadata.get("script_type"),
            Some(&"v4.00+".to_string())
        );
        assert!(format_result.metadata.contains_key("sections"));
    }

    #[test]
    fn test_ass_export_normalized_format_lines() {
        let format = AssFormat::new();
        let options = FormatOptions {
            preserve_formatting: false,
            ..FormatOptions::default()
        };

        // ASS without format lines - Script::to_ass_string() should add default ones
        let minimal_ass = r#"[Script Info]
Title: Test Script

[V4+ Styles]
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
"#;

        let (document, _) = format.import_from_string(minimal_ass, &options).unwrap();
        let (exported_content, _) = format.export_to_string(&document, &options).unwrap();

        // The normalized output should NOT include format lines if the parser
        // didn't preserve them (ass-core's default behavior)
        assert!(exported_content.contains("[V4+ Styles]\n"));
        assert!(exported_content.contains("Style: Default,Arial,20"));
        assert!(exported_content.contains("[Events]\n"));
        assert!(exported_content.contains("Dialogue: 0,0:00:00.00,0:00:05.00"));
    }
}
