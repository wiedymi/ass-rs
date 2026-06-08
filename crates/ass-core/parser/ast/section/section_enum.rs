//! Top-level section enum with span and validation helpers.
//!
//! Defines the [`Section`] enum representing the main sections of an ASS
//! script along with span computation and zero-copy span validation used
//! for debugging.

use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
extern crate alloc;

use super::SectionType;
use crate::parser::ast::{Event, Font, Graphic, ScriptInfo, Span, Style};
#[cfg(debug_assertions)]
use core::ops::Range;

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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
