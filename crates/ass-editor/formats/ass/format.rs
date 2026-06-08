//! ASS format handler implementation.
//!
//! Provides the [`AssFormat`] type, which reuses ass-core's parsing and
//! serialization to import and export Advanced SubStation Alpha files.

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

impl Format for AssFormat {
    fn as_importer(&self) -> &dyn FormatImporter {
        self
    }

    fn as_exporter(&self) -> &dyn FormatExporter {
        self
    }
}
