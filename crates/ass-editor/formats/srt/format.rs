//! `SrtFormat` handler definition and trait dispatch wiring.
//!
//! Defines the [`SrtFormat`] type, its constructor, and the [`Format`]
//! dispatch implementation. Import and export behaviour live in sibling
//! modules.

use crate::formats::{Format, FormatExporter, FormatImporter, FormatInfo};

/// SRT format handler with style preservation
#[derive(Debug)]
pub struct SrtFormat {
    pub(super) info: FormatInfo,
}

impl SrtFormat {
    /// Create a new SRT format handler
    pub fn new() -> Self {
        Self {
            info: FormatInfo {
                name: "SRT".to_string(),
                extensions: vec!["srt".to_string()],
                mime_type: "text/srt".to_string(),
                description: "SubRip subtitle format with style preservation".to_string(),
                supports_styling: true,
                supports_positioning: false,
            },
        }
    }
}

impl Default for SrtFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl Format for SrtFormat {
    fn as_importer(&self) -> &dyn FormatImporter {
        self
    }

    fn as_exporter(&self) -> &dyn FormatExporter {
        self
    }
}
