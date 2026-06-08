//! `WebVttFormat` handler definition and trait dispatch wiring.
//!
//! Defines the [`WebVttFormat`] type, its constructor, and the [`Format`]
//! dispatch implementation. Import and export behaviour live in sibling
//! modules.

use crate::formats::{Format, FormatExporter, FormatImporter, FormatInfo};

/// WebVTT format handler with style and positioning preservation
#[derive(Debug)]
pub struct WebVttFormat {
    pub(super) info: FormatInfo,
}

impl WebVttFormat {
    /// Create a new WebVTT format handler
    pub fn new() -> Self {
        Self {
            info: FormatInfo {
                name: "WebVTT".to_string(),
                extensions: vec!["vtt".to_string(), "webvtt".to_string()],
                mime_type: "text/vtt".to_string(),
                description: "WebVTT subtitle format with full style and positioning preservation"
                    .to_string(),
                supports_styling: true,
                supports_positioning: true,
            },
        }
    }
}

impl Default for WebVttFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl Format for WebVttFormat {
    fn as_importer(&self) -> &dyn FormatImporter {
        self
    }

    fn as_exporter(&self) -> &dyn FormatExporter {
        self
    }
}
