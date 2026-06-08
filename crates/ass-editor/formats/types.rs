//! Core data types describing subtitle formats and operation results.
//!
//! Defines [`FormatInfo`] (format metadata), [`FormatOptions`] (import/export
//! configuration), and [`FormatResult`] (the outcome of an operation).

use std::collections::HashMap;

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
