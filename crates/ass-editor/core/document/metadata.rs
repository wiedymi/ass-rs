//! Document metadata accessors and format conversion entry points
//!
//! Covers the identity, file-path, modification, and cursor-tracking
//! accessors along with the optional subtitle-format import/export bridges.

use super::EditorDocument;
use crate::core::position::Position;

#[cfg(feature = "formats")]
use crate::core::errors::Result;

#[cfg(not(feature = "std"))]
use alloc::string::String;

impl EditorDocument {
    /// Get document ID
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get file path if document is associated with a file
    #[must_use]
    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    /// Set file path for the document
    pub fn set_file_path(&mut self, path: Option<String>) {
        self.file_path = path;
    }

    /// Import content from another subtitle format
    #[cfg(feature = "formats")]
    pub fn import_format(
        content: &str,
        format: Option<crate::utils::formats::SubtitleFormat>,
    ) -> Result<Self> {
        let ass_content = crate::utils::formats::FormatConverter::import(content, format)?;
        Self::from_content(&ass_content)
    }

    /// Export document to another subtitle format
    #[cfg(feature = "formats")]
    pub fn export_format(
        &self,
        format: crate::utils::formats::SubtitleFormat,
        options: &crate::utils::formats::ConversionOptions,
    ) -> Result<String> {
        crate::utils::formats::FormatConverter::export(self, format, options)
    }

    /// Check if document has unsaved changes
    #[must_use]
    pub const fn is_modified(&self) -> bool {
        self.modified
    }

    /// Get current cursor position (if tracked)
    #[must_use]
    pub fn cursor_position(&self) -> Option<Position> {
        self.history.cursor_position()
    }

    /// Set current cursor position for tracking
    pub fn set_cursor_position(&mut self, position: Option<Position>) {
        self.history.set_cursor(position);
    }

    /// Mark document as modified
    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }
}
