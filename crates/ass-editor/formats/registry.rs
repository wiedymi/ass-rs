//! Runtime registry for available subtitle formats.
//!
//! Provides [`FormatRegistry`], which tracks registered importers, exporters,
//! and combined formats and dispatches import/export by file extension.

use super::{Format, FormatExporter, FormatImporter, FormatOptions, FormatResult};
use crate::core::{EditorDocument, EditorError};
use std::collections::HashMap;
use std::path::Path;

/// Registry for managing available formats
#[derive(Debug, Default)]
pub struct FormatRegistry {
    pub(super) importers: HashMap<String, Box<dyn FormatImporter>>,
    pub(super) exporters: HashMap<String, Box<dyn FormatExporter>>,
    pub(super) formats: HashMap<String, Box<dyn Format>>,
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
