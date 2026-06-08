//! Subtitle format types and conversion options.
//!
//! Defines the supported [`SubtitleFormat`] variants along with the
//! [`ConversionOptions`] and [`FormatOptions`] used to drive import and export.

/// Supported subtitle formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtitleFormat {
    /// Advanced SubStation Alpha (.ass)
    ASS,
    /// SubStation Alpha (.ssa)
    SSA,
    /// SubRip Text (.srt)
    SRT,
    /// WebVTT (.vtt)
    WebVTT,
    /// Plain text
    PlainText,
}

impl SubtitleFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "ass" => Some(Self::ASS),
            "ssa" => Some(Self::SSA),
            "srt" => Some(Self::SRT),
            "vtt" | "webvtt" => Some(Self::WebVTT),
            "txt" => Some(Self::PlainText),
            _ => None,
        }
    }

    /// Detect format from content
    pub fn from_content(content: &str) -> Self {
        if content.contains("[Script Info]") || content.contains("[Events]") {
            Self::ASS
        } else if content.starts_with("WEBVTT") {
            Self::WebVTT
        } else if content.contains("-->") && !content.starts_with("WEBVTT") {
            Self::SRT
        } else {
            Self::PlainText
        }
    }

    /// Get the standard file extension for this format
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::ASS => "ass",
            Self::SSA => "ssa",
            Self::SRT => "srt",
            Self::WebVTT => "vtt",
            Self::PlainText => "txt",
        }
    }
}

/// Options for format conversion
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Preserve styling information when possible
    pub preserve_styling: bool,

    /// Preserve positioning information when possible
    pub preserve_positioning: bool,

    /// Convert karaoke timing to inline format
    pub inline_karaoke: bool,

    /// Strip all formatting tags
    pub strip_formatting: bool,

    /// Target format-specific options
    pub format_options: FormatOptions,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            preserve_styling: true,
            preserve_positioning: true,
            inline_karaoke: false,
            strip_formatting: false,
            format_options: FormatOptions::default(),
        }
    }
}

/// Format-specific conversion options
#[derive(Debug, Clone, Default)]
pub enum FormatOptions {
    /// No format-specific options
    #[default]
    None,

    /// SRT-specific options
    SRT {
        /// Include sequential numbering
        include_numbers: bool,
        /// Use millisecond precision (3 digits)
        millisecond_precision: bool,
    },

    /// WebVTT-specific options
    WebVTT {
        /// Include STYLE block for CSS
        include_style_block: bool,
        /// Include NOTE comments
        include_notes: bool,
        /// Use cue settings for positioning
        use_cue_settings: bool,
    },
}
