//! Format import/export functionality for subtitle files.
//!
//! This module provides traits and implementations for importing and exporting
//! various subtitle formats, reusing ass-core's parsing capabilities where possible.

use crate::core::{EditorDocument, EditorError};
use std::collections::HashMap;
use std::fmt;
use std::io::{Read, Write};
use std::path::Path;

/// Metadata about a subtitle format
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatInfo {
    /// Format name (e.g., "ASS", "SRT", "WebVTT")
    pub name: String,
    /// File extensions supported by this format
    pub extensions: Vec<String>,
    /// MIME type for this format
    pub mime_type: String,
    /// Brief description of the format
    pub description: String,
    /// Whether this format supports styling
    pub supports_styling: bool,
    /// Whether this format supports positioning
    pub supports_positioning: bool,
}

/// Configuration options for format import/export operations
#[derive(Debug, Clone)]
pub struct FormatOptions {
    /// Encoding to use (defaults to UTF-8)
    pub encoding: String,
    /// Whether to preserve formatting when possible
    pub preserve_formatting: bool,
    /// Custom options specific to each format
    pub custom_options: HashMap<String, String>,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            encoding: "UTF-8".to_string(),
            preserve_formatting: true,
            custom_options: HashMap::new(),
        }
    }
}

/// Result of an import/export operation
#[derive(Debug)]
pub struct FormatResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// Number of lines/entries processed
    pub lines_processed: usize,
    /// Warnings encountered during processing
    pub warnings: Vec<String>,
    /// Additional metadata from the operation
    pub metadata: HashMap<String, String>,
}

impl FormatResult {
    pub fn success(lines_processed: usize) -> Self {
        Self {
            success: true,
            lines_processed,
            warnings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

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

/// Registry for managing available formats
#[derive(Debug, Default)]
pub struct FormatRegistry {
    importers: HashMap<String, Box<dyn FormatImporter>>,
    exporters: HashMap<String, Box<dyn FormatExporter>>,
    formats: HashMap<String, Box<dyn Format>>,
}

impl FormatRegistry {
    /// Create a new format registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a format that supports both import and export
    pub fn register_format(&mut self, format: Box<dyn Format>) {
        let name = format.name().to_string();
        self.formats.insert(name, format);
    }

    /// Register an importer
    pub fn register_importer(&mut self, importer: Box<dyn FormatImporter>) {
        let name = importer.format_info().name.clone();
        self.importers.insert(name, importer);
    }

    /// Register an exporter
    pub fn register_exporter(&mut self, exporter: Box<dyn FormatExporter>) {
        let name = exporter.format_info().name.clone();
        self.exporters.insert(name, exporter);
    }

    /// Find an importer for the given file extension
    pub fn find_importer(&self, extension: &str) -> Option<&dyn FormatImporter> {
        // Check combined formats first
        for format in self.formats.values() {
            if format.can_import(extension) {
                return Some(format.as_importer());
            }
        }

        // Check dedicated importers
        for importer in self.importers.values() {
            if importer.can_import(extension) {
                return Some(importer.as_ref());
            }
        }

        None
    }

    /// Find an exporter for the given file extension
    pub fn find_exporter(&self, extension: &str) -> Option<&dyn FormatExporter> {
        // Check combined formats first
        for format in self.formats.values() {
            if format.can_export(extension) {
                return Some(format.as_exporter());
            }
        }

        // Check dedicated exporters
        for exporter in self.exporters.values() {
            if exporter.can_export(extension) {
                return Some(exporter.as_ref());
            }
        }

        None
    }

    /// Get all supported import extensions
    pub fn supported_import_extensions(&self) -> Vec<String> {
        let mut extensions = Vec::new();

        for format in self.formats.values() {
            extensions.extend(
                FormatImporter::format_info(format.as_ref())
                    .extensions
                    .clone(),
            );
        }

        for importer in self.importers.values() {
            extensions.extend(importer.format_info().extensions.clone());
        }

        extensions.sort();
        extensions.dedup();
        extensions
    }

    /// Get all supported export extensions
    pub fn supported_export_extensions(&self) -> Vec<String> {
        let mut extensions = Vec::new();

        for format in self.formats.values() {
            extensions.extend(
                FormatExporter::format_info(format.as_ref())
                    .extensions
                    .clone(),
            );
        }

        for exporter in self.exporters.values() {
            extensions.extend(exporter.format_info().extensions.clone());
        }

        extensions.sort();
        extensions.dedup();
        extensions
    }

    /// Import a file using the appropriate format
    pub fn import_file(
        &self,
        path: &Path,
        options: Option<&FormatOptions>,
    ) -> Result<(EditorDocument, FormatResult), EditorError> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| EditorError::InvalidFormat("No file extension found".to_string()))?;

        let importer = self
            .find_importer(extension)
            .ok_or_else(|| EditorError::UnsupportedFormat(extension.to_string()))?;

        let default_options = FormatOptions::default();
        let options = options.unwrap_or(&default_options);
        importer.import_from_path(path, options)
    }

    /// Export a document to a file using the appropriate format
    pub fn export_file(
        &self,
        document: &EditorDocument,
        path: &Path,
        options: Option<&FormatOptions>,
    ) -> Result<FormatResult, EditorError> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| EditorError::InvalidFormat("No file extension found".to_string()))?;

        let exporter = self
            .find_exporter(extension)
            .ok_or_else(|| EditorError::UnsupportedFormat(extension.to_string()))?;

        let default_options = FormatOptions::default();
        let options = options.unwrap_or(&default_options);
        exporter.export_to_path(document, path, options)
    }
}

// Individual format modules
pub mod ass;
pub mod srt;
pub mod webvtt;

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;
    #[cfg(not(feature = "std"))]
    use alloc::{format, string::String, vec};

    #[test]
    fn test_format_info_creation() {
        let info = FormatInfo {
            name: "Test Format".to_string(),
            extensions: vec!["test".to_string(), "tst".to_string()],
            mime_type: "text/test".to_string(),
            description: "A test format".to_string(),
            supports_styling: true,
            supports_positioning: false,
        };

        assert_eq!(info.name, "Test Format");
        assert_eq!(info.extensions.len(), 2);
        assert!(info.supports_styling);
        assert!(!info.supports_positioning);
    }

    #[test]
    fn test_format_options_default() {
        let options = FormatOptions::default();
        assert_eq!(options.encoding, "UTF-8");
        assert!(options.preserve_formatting);
        assert!(options.custom_options.is_empty());
    }

    #[test]
    fn test_format_result_creation() {
        let result = FormatResult::success(42)
            .with_warnings(vec!["Warning 1".to_string()])
            .with_metadata("key".to_string(), "value".to_string());

        assert!(result.success);
        assert_eq!(result.lines_processed, 42);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_format_registry_creation() {
        let registry = FormatRegistry::new();
        assert!(registry.formats.is_empty());
        assert!(registry.importers.is_empty());
        assert!(registry.exporters.is_empty());
    }

    #[test]
    fn test_format_registry_extensions() {
        let registry = FormatRegistry::new();
        let import_exts = registry.supported_import_extensions();
        let export_exts = registry.supported_export_extensions();

        assert!(import_exts.is_empty());
        assert!(export_exts.is_empty());
    }
}
