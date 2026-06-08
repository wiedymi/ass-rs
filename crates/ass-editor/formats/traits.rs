//! Import/export trait definitions for subtitle formats.
//!
//! Defines [`FormatImporter`], [`FormatExporter`], and the combined [`Format`]
//! trait used to read and write subtitle documents.

use super::{FormatInfo, FormatOptions, FormatResult};
use crate::core::{EditorDocument, EditorError};
use std::fmt;
use std::io::{Read, Write};
use std::path::Path;

/// Trait for importing subtitle files into EditorDocument
pub trait FormatImporter: fmt::Debug + Send + Sync {
    /// Get information about this format
    fn format_info(&self) -> &FormatInfo;

    /// Check if this importer can handle the given file extension
    fn can_import(&self, extension: &str) -> bool {
        self.format_info()
            .extensions
            .iter()
            .any(|ext| ext.eq_ignore_ascii_case(extension))
    }

    /// Import from a reader with the given options
    fn import_from_reader(
        &self,
        reader: &mut dyn Read,
        options: &FormatOptions,
    ) -> Result<(EditorDocument, FormatResult), EditorError>;

    /// Import from a file path
    fn import_from_path(
        &self,
        path: &Path,
        options: &FormatOptions,
    ) -> Result<(EditorDocument, FormatResult), EditorError> {
        let mut file = std::fs::File::open(path)
            .map_err(|e| EditorError::IoError(format!("Failed to open file: {e}")))?;
        self.import_from_reader(&mut file, options)
    }

    /// Import from a string
    fn import_from_string(
        &self,
        content: &str,
        options: &FormatOptions,
    ) -> Result<(EditorDocument, FormatResult), EditorError> {
        let mut cursor = std::io::Cursor::new(content.as_bytes());
        self.import_from_reader(&mut cursor, options)
    }
}

/// Trait for exporting EditorDocument to subtitle files
pub trait FormatExporter: fmt::Debug + Send + Sync {
    /// Get information about this format
    fn format_info(&self) -> &FormatInfo;

    /// Check if this exporter can handle the given file extension
    fn can_export(&self, extension: &str) -> bool {
        self.format_info()
            .extensions
            .iter()
            .any(|ext| ext.eq_ignore_ascii_case(extension))
    }

    /// Export to a writer with the given options
    fn export_to_writer(
        &self,
        document: &EditorDocument,
        writer: &mut dyn Write,
        options: &FormatOptions,
    ) -> Result<FormatResult, EditorError>;

    /// Export to a file path
    fn export_to_path(
        &self,
        document: &EditorDocument,
        path: &Path,
        options: &FormatOptions,
    ) -> Result<FormatResult, EditorError> {
        let mut file = std::fs::File::create(path)
            .map_err(|e| EditorError::IoError(format!("Failed to create file: {e}")))?;
        self.export_to_writer(document, &mut file, options)
    }

    /// Export to a string
    fn export_to_string(
        &self,
        document: &EditorDocument,
        options: &FormatOptions,
    ) -> Result<(String, FormatResult), EditorError> {
        let mut buffer = Vec::new();
        let result = self.export_to_writer(document, &mut buffer, options)?;
        let content = String::from_utf8(buffer)
            .map_err(|e| EditorError::InvalidFormat(format!("Invalid UTF-8 output: {e}")))?;
        Ok((content, result))
    }
}

/// Combined trait for formats that support both import and export
pub trait Format: FormatImporter + FormatExporter {
    /// Get the format name
    fn name(&self) -> &str {
        &FormatImporter::format_info(self).name
    }

    /// Check if this format supports the given file extension
    fn supports_extension(&self, extension: &str) -> bool {
        self.can_import(extension) || self.can_export(extension)
    }

    /// Get self as an importer (workaround for trait upcasting)
    fn as_importer(&self) -> &dyn FormatImporter;

    /// Get self as an exporter (workaround for trait upcasting)
    fn as_exporter(&self) -> &dyn FormatExporter;
}
