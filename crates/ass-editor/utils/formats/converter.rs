//! Format converter entry points.
//!
//! Hosts the [`FormatConverter`] type and its public `import`/`export`
//! dispatchers, which delegate to the per-format helpers in sibling modules.

use super::types::{ConversionOptions, SubtitleFormat};
use crate::core::{EditorDocument, Result};

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// Format converter for subtitle import/export
pub struct FormatConverter;

impl FormatConverter {
    /// Import subtitle content from various formats into ASS
    pub fn import(content: &str, format: Option<SubtitleFormat>) -> Result<String> {
        let detected_format = format.unwrap_or_else(|| SubtitleFormat::from_content(content));

        match detected_format {
            SubtitleFormat::ASS | SubtitleFormat::SSA => {
                // Already in ASS/SSA format, just return
                Ok(content.to_string())
            }
            SubtitleFormat::SRT => Self::import_srt(content),
            SubtitleFormat::WebVTT => Self::import_webvtt(content),
            SubtitleFormat::PlainText => Self::import_plain_text(content),
        }
    }

    /// Export ASS content to another subtitle format
    pub fn export(
        document: &EditorDocument,
        format: SubtitleFormat,
        options: &ConversionOptions,
    ) -> Result<String> {
        match format {
            SubtitleFormat::ASS => Ok(document.text()),
            SubtitleFormat::SSA => Self::export_ssa(document, options),
            SubtitleFormat::SRT => Self::export_srt(document, options),
            SubtitleFormat::WebVTT => Self::export_webvtt(document, options),
            SubtitleFormat::PlainText => Self::export_plain_text(document, options),
        }
    }
}
