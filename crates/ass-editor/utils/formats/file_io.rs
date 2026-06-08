//! Filesystem helpers for importing and exporting subtitle files.
//!
//! Provides convenience wrappers that read from and write to disk, detecting
//! the subtitle format from the file extension when one is not supplied.

use super::types::{ConversionOptions, SubtitleFormat};
use super::FormatConverter;
use crate::core::errors::EditorError;
use crate::core::{EditorDocument, Result};

/// Import content from a file path
#[cfg(feature = "std")]
pub fn import_from_file(path: &str) -> Result<EditorDocument> {
    use std::fs;

    let content = fs::read_to_string(path).map_err(|e| EditorError::IoError(e.to_string()))?;

    let format = path
        .rfind('.')
        .and_then(|pos| SubtitleFormat::from_extension(&path[pos + 1..]));

    let ass_content = FormatConverter::import(&content, format)?;
    EditorDocument::from_content(&ass_content)
}

/// Export document to a file
#[cfg(feature = "std")]
pub fn export_to_file(
    document: &EditorDocument,
    path: &str,
    format: Option<SubtitleFormat>,
    options: &ConversionOptions,
) -> Result<()> {
    use std::fs;

    let detected_format = format
        .or_else(|| {
            path.rfind('.')
                .and_then(|pos| SubtitleFormat::from_extension(&path[pos + 1..]))
        })
        .unwrap_or(SubtitleFormat::ASS);

    let content = FormatConverter::export(document, detected_format, options)?;

    fs::write(path, content).map_err(|e| EditorError::IoError(e.to_string()))?;

    Ok(())
}
