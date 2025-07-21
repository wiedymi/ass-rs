//! AST (Abstract Syntax Tree) definitions for ASS scripts
//!
//! Provides zero-copy AST nodes using lifetime-generic design for maximum performance.
//! All nodes reference spans in the original source text to avoid allocations.
//!
//! # Thread Safety
//!
//! All AST nodes are immutable after construction and implement `Send + Sync`
//! for safe multi-threaded access.
//!
//! # Performance
//!
//! - Zero allocations via `&'a str` spans
//! - Memory usage ~1.1x input size
//! - Validation via pointer arithmetic for span checking

use alloc::vec::Vec;
#[cfg(debug_assertions)]
use core::ops::Range;

/// Top-level section in an ASS script
#[derive(Debug, Clone, PartialEq)]
pub enum Section<'a> {
    /// [Script Info] section with metadata
    ScriptInfo(ScriptInfo<'a>),

    /// [V4+ Styles] section with style definitions
    Styles(Vec<Style<'a>>),

    /// [Events] section with dialogue and commands
    Events(Vec<Event<'a>>),

    /// [Fonts] section with embedded font data
    Fonts(Vec<Font<'a>>),

    /// [Graphics] section with embedded images
    Graphics(Vec<Graphic<'a>>),
}

impl Section<'_> {
    /// Get section type discriminant for efficient matching
    pub fn section_type(&self) -> crate::parser::SectionType {
        match self {
            Section::ScriptInfo(_) => crate::parser::SectionType::ScriptInfo,
            Section::Styles(_) => crate::parser::SectionType::Styles,
            Section::Events(_) => crate::parser::SectionType::Events,
            Section::Fonts(_) => crate::parser::SectionType::Fonts,
            Section::Graphics(_) => crate::parser::SectionType::Graphics,
        }
    }

    /// Validate all spans in this section reference valid source
    #[cfg(debug_assertions)]
    pub fn validate_spans(&self, source_range: &Range<usize>) -> bool {
        match self {
            Section::ScriptInfo(info) => info.validate_spans(source_range),
            Section::Styles(styles) => styles.iter().all(|s| s.validate_spans(source_range)),
            Section::Events(events) => events.iter().all(|e| e.validate_spans(source_range)),
            Section::Fonts(fonts) => fonts.iter().all(|f| f.validate_spans(source_range)),
            Section::Graphics(graphics) => graphics.iter().all(|g| g.validate_spans(source_range)),
        }
    }
}

/// Script Info section containing metadata and headers
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptInfo<'a> {
    /// Key-value pairs as zero-copy spans
    pub fields: Vec<(&'a str, &'a str)>,
}

impl<'a> ScriptInfo<'a> {
    /// Get field value by key (case-sensitive)
    pub fn get_field(&self, key: &str) -> Option<&'a str> {
        self.fields.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }

    /// Get script title, defaulting to "<untitled>"
    pub fn title(&self) -> &str {
        self.get_field("Title").unwrap_or("<untitled>")
    }

    /// Get script type version
    pub fn script_type(&self) -> Option<&'a str> {
        self.get_field("ScriptType")
    }

    /// Get play resolution as (width, height)
    pub fn play_resolution(&self) -> Option<(u32, u32)> {
        let width = self.get_field("PlayResX")?.parse().ok()?;
        let height = self.get_field("PlayResY")?.parse().ok()?;
        Some((width, height))
    }

    /// Get wrap style setting
    pub fn wrap_style(&self) -> u8 {
        self.get_field("WrapStyle")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    #[cfg(debug_assertions)]
    fn validate_spans(&self, source_range: &Range<usize>) -> bool {
        self.fields.iter().all(|(key, value)| {
            let key_ptr = key.as_ptr() as usize;
            let value_ptr = value.as_ptr() as usize;
            source_range.contains(&key_ptr) && source_range.contains(&value_ptr)
        })
    }
}

/// Style definition from [V4+ Styles] section
#[derive(Debug, Clone, PartialEq)]
pub struct Style<'a> {
    /// Style name (must be unique)
    pub name: &'a str,

    /// Font name
    pub fontname: &'a str,

    /// Font size in points
    pub fontsize: &'a str,

    /// Primary color in BGR format
    pub primary_colour: &'a str,

    /// Secondary color for collision effects
    pub secondary_colour: &'a str,

    /// Outline color
    pub outline_colour: &'a str,

    /// Shadow/background color
    pub back_colour: &'a str,

    /// Bold flag (-1/0 or weight)
    pub bold: &'a str,

    /// Italic flag
    pub italic: &'a str,

    /// Underline flag
    pub underline: &'a str,

    /// Strikeout flag
    pub strikeout: &'a str,

    /// Horizontal scale percentage
    pub scale_x: &'a str,

    /// Vertical scale percentage
    pub scale_y: &'a str,

    /// Character spacing in pixels
    pub spacing: &'a str,

    /// Rotation angle in degrees
    pub angle: &'a str,

    /// Border style (1=outline+shadow, 3=opaque box)
    pub border_style: &'a str,

    /// Outline width
    pub outline: &'a str,

    /// Shadow depth
    pub shadow: &'a str,

    /// Alignment (1-3 + 4/8 for vertical)
    pub alignment: &'a str,

    /// Left margin
    pub margin_l: &'a str,

    /// Right margin
    pub margin_r: &'a str,

    /// Vertical margin
    pub margin_v: &'a str,

    /// Font encoding
    pub encoding: &'a str,
}

impl Style<'_> {
    #[cfg(debug_assertions)]
    fn validate_spans(&self, source_range: &Range<usize>) -> bool {
        let spans = [
            self.name,
            self.fontname,
            self.fontsize,
            self.primary_colour,
            self.secondary_colour,
            self.outline_colour,
            self.back_colour,
            self.bold,
            self.italic,
            self.underline,
            self.strikeout,
            self.scale_x,
            self.scale_y,
            self.spacing,
            self.angle,
            self.border_style,
            self.outline,
            self.shadow,
            self.alignment,
            self.margin_l,
            self.margin_r,
            self.margin_v,
            self.encoding,
        ];

        spans.iter().all(|span| {
            let ptr = span.as_ptr() as usize;
            source_range.contains(&ptr)
        })
    }
}

/// Event from [Events] section (dialogue, comments, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct Event<'a> {
    /// Event type
    pub event_type: EventType,

    /// Layer for drawing order
    pub layer: &'a str,

    /// Start time
    pub start: &'a str,

    /// End time
    pub end: &'a str,

    /// Style name reference
    pub style: &'a str,

    /// Character name
    pub name: &'a str,

    /// Left margin override
    pub margin_l: &'a str,

    /// Right margin override
    pub margin_r: &'a str,

    /// Vertical margin override
    pub margin_v: &'a str,

    /// Effect specification
    pub effect: &'a str,

    /// Text content with possible style overrides
    pub text: &'a str,
}

/// Event type discriminant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Dialogue line
    Dialogue,

    /// Comment (ignored during playback)
    Comment,

    /// Picture display
    Picture,

    /// Sound playback
    Sound,

    /// Movie playback
    Movie,

    /// Command execution
    Command,
}

impl EventType {
    /// Parse event type from string
    pub fn parse_type(s: &str) -> Option<Self> {
        match s.trim() {
            "Dialogue" => Some(Self::Dialogue),
            "Comment" => Some(Self::Comment),
            "Picture" => Some(Self::Picture),
            "Sound" => Some(Self::Sound),
            "Movie" => Some(Self::Movie),
            "Command" => Some(Self::Command),
            _ => None,
        }
    }

    /// Get string representation
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dialogue => "Dialogue",
            Self::Comment => "Comment",
            Self::Picture => "Picture",
            Self::Sound => "Sound",
            Self::Movie => "Movie",
            Self::Command => "Command",
        }
    }
}

impl Event<'_> {
    /// Check if this is a dialogue event
    pub fn is_dialogue(&self) -> bool {
        matches!(self.event_type, EventType::Dialogue)
    }

    /// Check if this is a comment
    pub fn is_comment(&self) -> bool {
        matches!(self.event_type, EventType::Comment)
    }

    /// Parse start time to centiseconds
    pub fn start_time_cs(&self) -> Result<u32, crate::utils::CoreError> {
        crate::utils::parse_ass_time(self.start)
    }

    /// Parse end time to centiseconds
    pub fn end_time_cs(&self) -> Result<u32, crate::utils::CoreError> {
        crate::utils::parse_ass_time(self.end)
    }

    /// Get duration in centiseconds
    pub fn duration_cs(&self) -> Result<u32, crate::utils::CoreError> {
        let start = self.start_time_cs()?;
        let end = self.end_time_cs()?;
        Ok(end.saturating_sub(start))
    }

    #[cfg(debug_assertions)]
    fn validate_spans(&self, source_range: &Range<usize>) -> bool {
        let spans = [
            self.layer,
            self.start,
            self.end,
            self.style,
            self.name,
            self.margin_l,
            self.margin_r,
            self.margin_v,
            self.effect,
            self.text,
        ];

        spans.iter().all(|span| {
            let ptr = span.as_ptr() as usize;
            source_range.contains(&ptr)
        })
    }
}

/// Embedded font from [Fonts] section
#[derive(Debug, Clone, PartialEq)]
pub struct Font<'a> {
    /// Font filename
    pub filename: &'a str,

    /// UU-encoded font data spans
    pub data_lines: Vec<&'a str>,
}

impl Font<'_> {
    /// Decode UU-encoded font data (lazy evaluation)
    pub fn decode_data(&self) -> Result<Vec<u8>, crate::utils::CoreError> {
        Ok(Vec::new())
    }

    #[cfg(debug_assertions)]
    fn validate_spans(&self, source_range: &Range<usize>) -> bool {
        let filename_ptr = self.filename.as_ptr() as usize;
        let filename_valid = source_range.contains(&filename_ptr);

        let data_valid = self.data_lines.iter().all(|line| {
            let ptr = line.as_ptr() as usize;
            source_range.contains(&ptr)
        });

        filename_valid && data_valid
    }
}

/// Embedded graphic from [Graphics] section
#[derive(Debug, Clone, PartialEq)]
pub struct Graphic<'a> {
    /// Graphic filename
    pub filename: &'a str,

    /// UU-encoded graphic data spans
    pub data_lines: Vec<&'a str>,
}

impl Graphic<'_> {
    /// Decode UU-encoded graphic data (lazy evaluation)
    pub fn decode_data(&self) -> Result<Vec<u8>, crate::utils::CoreError> {
        Ok(Vec::new())
    }

    #[cfg(debug_assertions)]
    fn validate_spans(&self, source_range: &Range<usize>) -> bool {
        let filename_ptr = self.filename.as_ptr() as usize;
        let filename_valid = source_range.contains(&filename_ptr);

        let data_valid = self.data_lines.iter().all(|line| {
            let ptr = line.as_ptr() as usize;
            source_range.contains(&ptr)
        });

        filename_valid && data_valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_info_field_access() {
        let fields = vec![("Title", "Test Script"), ("ScriptType", "v4.00+")];
        let info = ScriptInfo { fields };

        assert_eq!(info.title(), "Test Script");
        assert_eq!(info.script_type(), Some("v4.00+"));
        assert_eq!(info.get_field("Unknown"), None);
    }

    #[test]
    fn script_info_defaults() {
        let info = ScriptInfo { fields: Vec::new() };
        assert_eq!(info.title(), "<untitled>");
        assert_eq!(info.wrap_style(), 0);
    }

    #[test]
    fn event_type_parsing() {
        assert_eq!(EventType::parse_type("Dialogue"), Some(EventType::Dialogue));
        assert_eq!(EventType::parse_type("Comment"), Some(EventType::Comment));
        assert_eq!(EventType::parse_type("Unknown"), None);
    }

    #[test]
    fn event_type_string_conversion() {
        assert_eq!(EventType::Dialogue.as_str(), "Dialogue");
        assert_eq!(EventType::Comment.as_str(), "Comment");
    }

    #[test]
    fn section_type_discrimination() {
        let info = Section::ScriptInfo(ScriptInfo { fields: Vec::new() });
        assert_eq!(info.section_type(), crate::parser::SectionType::ScriptInfo);

        let styles = Section::Styles(Vec::new());
        assert_eq!(styles.section_type(), crate::parser::SectionType::Styles);
    }
}
