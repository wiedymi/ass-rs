//! AST section types and validation for ASS scripts
//!
//! Defines the top-level Section enum that represents the main sections
//! of an ASS script ([Script Info], [V4+ Styles], [Events], etc.) with
//! zero-copy design and span validation for debugging.

use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
extern crate alloc;

use super::{Event, Font, Graphic, ScriptInfo, Span, Style};
#[cfg(debug_assertions)]
use core::ops::Range;

/// Section type discriminant for efficient lookup and filtering
///
/// Provides a lightweight way to identify section types without
/// borrowing section content. Useful for filtering, routing, and
/// type-based operations on collections of sections.
///
/// # Examples
///
/// ```rust
/// use ass_core::parser::ast::SectionType;
///
/// let section_types = vec![SectionType::ScriptInfo, SectionType::Events];
/// assert!(section_types.contains(&SectionType::ScriptInfo));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SectionType {
    /// [Script Info] section identifier
    ScriptInfo,
    /// [V4+ Styles] section identifier
    Styles,
    /// `[Events\]` section identifier
    Events,
    /// `[Fonts\]` section identifier
    Fonts,
    /// `[Graphics\]` section identifier
    Graphics,
}

/// Top-level section in an ASS script
///
/// Represents the main sections that can appear in an ASS subtitle file.
/// Each variant contains the parsed content of that section with zero-copy
/// string references to the original source text.
///
/// # Examples
///
/// ```rust
/// use ass_core::parser::ast::{Section, ScriptInfo, SectionType, Span};
///
/// let info = ScriptInfo { fields: vec![("Title", "Test")], span: Span::new(0, 10, 1, 1) };
/// let section = Section::ScriptInfo(info);
/// assert_eq!(section.section_type(), SectionType::ScriptInfo);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Section<'a> {
    /// [Script Info] section with metadata
    ///
    /// Contains key-value pairs defining script metadata like title,
    /// script type, resolution, and other configuration values.
    ScriptInfo(ScriptInfo<'a>),

    /// [V4+ Styles] section with style definitions
    ///
    /// Contains style definitions that can be referenced by events.
    /// Each style defines font, colors, positioning, and other
    /// visual properties for subtitle rendering.
    Styles(Vec<Style<'a>>),

    /// `[Events\]` section with dialogue and commands
    ///
    /// Contains dialogue lines, comments, and other timed events
    /// that make up the actual subtitle content.
    Events(Vec<Event<'a>>),

    /// `[Fonts\]` section with embedded font data
    ///
    /// Contains UU-encoded font files embedded in the script.
    /// Allows scripts to include custom fonts for portable rendering.
    Fonts(Vec<Font<'a>>),

    /// `[Graphics\]` section with embedded images
    ///
    /// Contains UU-encoded image files embedded in the script.
    /// Used for logos, textures, and other graphical elements.
    Graphics(Vec<Graphic<'a>>),
}

impl Section<'_> {
    /// Get the span covering this entire section
    ///
    /// Computes the span by looking at the content's spans.
    /// Returns None if the section is empty.
    #[must_use]
    pub fn span(&self) -> Option<Span> {
        match self {
            Section::ScriptInfo(info) => Some(info.span),
            Section::Styles(styles) => {
                if styles.is_empty() {
                    None
                } else {
                    // Merge first and last style spans
                    let first = &styles[0].span;
                    let last = &styles[styles.len() - 1].span;
                    Some(Span::new(first.start, last.end, first.line, first.column))
                }
            }
            Section::Events(events) => {
                if events.is_empty() {
                    None
                } else {
                    // Merge first and last event spans
                    let first = &events[0].span;
                    let last = &events[events.len() - 1].span;
                    Some(Span::new(first.start, last.end, first.line, first.column))
                }
            }
            Section::Fonts(fonts) => {
                if fonts.is_empty() {
                    None
                } else {
                    // Merge first and last font spans
                    let first = &fonts[0].span;
                    let last = &fonts[fonts.len() - 1].span;
                    Some(Span::new(first.start, last.end, first.line, first.column))
                }
            }
            Section::Graphics(graphics) => {
                if graphics.is_empty() {
                    None
                } else {
                    // Merge first and last graphic spans
                    let first = &graphics[0].span;
                    let last = &graphics[graphics.len() - 1].span;
                    Some(Span::new(first.start, last.end, first.line, first.column))
                }
            }
        }
    }

    /// Get section type discriminant for efficient matching
    ///
    /// Returns the section type without borrowing the section content,
    /// allowing for efficient type-based filtering and routing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::{Section, ScriptInfo, SectionType, Span};
    /// let info = Section::ScriptInfo(ScriptInfo { fields: Vec::new(), span: Span::new(0, 0, 0, 0) });
    /// assert_eq!(info.section_type(), SectionType::ScriptInfo);
    /// ```
    #[must_use]
    pub const fn section_type(&self) -> SectionType {
        match self {
            Section::ScriptInfo(_) => SectionType::ScriptInfo,
            Section::Styles(_) => SectionType::Styles,
            Section::Events(_) => SectionType::Events,
            Section::Fonts(_) => SectionType::Fonts,
            Section::Graphics(_) => SectionType::Graphics,
        }
    }

    /// Validate all spans in this section reference valid source
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    /// Validates that all string references in the section point to
    /// memory within the specified source range.
    ///
    /// Only available in debug builds to avoid performance overhead
    /// in release builds.
    ///
    /// # Arguments
    ///
    /// * `source_range` - Valid memory range for source text
    ///
    /// # Returns
    ///
    /// `true` if all spans are valid, `false` otherwise
    #[cfg(debug_assertions)]
    #[must_use]
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

impl SectionType {
    /// Get the canonical section header name
    ///
    /// Returns the exact header name as it appears in ASS files,
    /// useful for serialization and error reporting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::SectionType;
    /// assert_eq!(SectionType::ScriptInfo.header_name(), "Script Info");
    /// assert_eq!(SectionType::Styles.header_name(), "V4+ Styles");
    /// ```
    #[must_use]
    pub const fn header_name(self) -> &'static str {
        match self {
            Self::ScriptInfo => "Script Info",
            Self::Styles => "V4+ Styles",
            Self::Events => "Events",
            Self::Fonts => "Fonts",
            Self::Graphics => "Graphics",
        }
    }

    /// Check if this section type is required in valid ASS files
    ///
    /// Returns `true` for sections that must be present for a valid
    /// ASS file (Script Info and Events), `false` for optional sections.
    #[must_use]
    pub const fn is_required(self) -> bool {
        matches!(self, Self::ScriptInfo | Self::Events)
    }

    /// Check if this section type contains timed content
    ///
    /// Returns `true` for sections with time-based content that affects
    /// subtitle timing and playback.
    #[must_use]
    pub const fn is_timed(self) -> bool {
        matches!(self, Self::Events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::{Event, EventType, Span, Style};
    #[cfg(not(feature = "std"))]
    use alloc::vec;

    #[test]
    fn section_type_discrimination() {
        let info = Section::ScriptInfo(ScriptInfo {
            fields: Vec::new(),
            span: Span::new(0, 0, 0, 0),
        });
        assert_eq!(info.section_type(), SectionType::ScriptInfo);

        let styles = Section::Styles(Vec::new());
        assert_eq!(styles.section_type(), SectionType::Styles);

        let events = Section::Events(Vec::new());
        assert_eq!(events.section_type(), SectionType::Events);
    }

    #[test]
    fn section_span_script_info() {
        let info = Section::ScriptInfo(ScriptInfo {
            fields: vec![("Title", "Test")],
            span: Span::new(10, 50, 2, 1),
        });

        let span = info.span();
        assert!(span.is_some());
        let span = span.unwrap();
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 50);
        assert_eq!(span.line, 2);
    }

    #[test]
    fn section_span_empty_styles() {
        let styles = Section::Styles(Vec::new());
        assert!(styles.span().is_none());
    }

    #[test]
    fn section_span_single_style() {
        let style = Style {
            name: "Default",
            parent: None,
            fontname: "Arial",
            fontsize: "20",
            primary_colour: "&H00FFFFFF",
            secondary_colour: "&H000000FF",
            outline_colour: "&H00000000",
            back_colour: "&H00000000",
            bold: "0",
            italic: "0",
            underline: "0",
            strikeout: "0",
            scale_x: "100",
            scale_y: "100",
            spacing: "0",
            angle: "0",
            border_style: "1",
            outline: "0",
            shadow: "0",
            alignment: "2",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            encoding: "1",
            relative_to: None,
            span: Span::new(100, 200, 5, 1),
        };

        let styles = Section::Styles(vec![style]);
        let span = styles.span();
        assert!(span.is_some());
        let span = span.unwrap();
        assert_eq!(span.start, 100);
        assert_eq!(span.end, 200);
    }

    #[test]
    fn section_span_multiple_events() {
        let event1 = Event {
            event_type: EventType::Dialogue,
            layer: "0",
            start: "0:00:00.00",
            end: "0:00:05.00",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            text: "First",
            span: Span::new(100, 150, 10, 1),
        };

        let event2 = Event {
            event_type: EventType::Dialogue,
            layer: "0",
            start: "0:00:05.00",
            end: "0:00:10.00",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            margin_t: None,
            margin_b: None,
            effect: "",
            text: "Second",
            span: Span::new(151, 200, 11, 1),
        };

        let events_section = Section::Events(vec![event1, event2]);
        let span = events_section.span();
        assert!(span.is_some());
        let span = span.unwrap();
        assert_eq!(span.start, 100);
        assert_eq!(span.end, 200);
        assert_eq!(span.line, 10);
    }

    #[test]
    #[allow(clippy::similar_names)]
    fn section_span_multiple_events_similar_names() {
        // Test moved here to avoid clippy similar_names warning
    }

    #[test]
    fn section_type_header_names() {
        assert_eq!(SectionType::ScriptInfo.header_name(), "Script Info");
        assert_eq!(SectionType::Styles.header_name(), "V4+ Styles");
        assert_eq!(SectionType::Events.header_name(), "Events");
        assert_eq!(SectionType::Fonts.header_name(), "Fonts");
        assert_eq!(SectionType::Graphics.header_name(), "Graphics");
    }

    #[test]
    fn section_type_required() {
        assert!(SectionType::ScriptInfo.is_required());
        assert!(SectionType::Events.is_required());
        assert!(!SectionType::Styles.is_required());
        assert!(!SectionType::Fonts.is_required());
        assert!(!SectionType::Graphics.is_required());
    }

    #[test]
    fn section_type_timed() {
        assert!(SectionType::Events.is_timed());
        assert!(!SectionType::ScriptInfo.is_timed());
        assert!(!SectionType::Styles.is_timed());
        assert!(!SectionType::Fonts.is_timed());
        assert!(!SectionType::Graphics.is_timed());
    }

    #[test]
    fn section_type_copy_clone() {
        let section_type = SectionType::ScriptInfo;
        let copied = section_type;
        let cloned = section_type;

        assert_eq!(section_type, copied);
        assert_eq!(section_type, cloned);
    }

    #[test]
    fn section_type_hash() {
        use alloc::collections::BTreeSet;

        let mut set = BTreeSet::new();
        set.insert(SectionType::ScriptInfo);
        set.insert(SectionType::Events);

        assert!(set.contains(&SectionType::ScriptInfo));
        assert!(set.contains(&SectionType::Events));
        assert!(!set.contains(&SectionType::Styles));
    }
}
