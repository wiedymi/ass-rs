//! Range-based partial reparse and span adjustment helpers.
//!
//! Hosts the streaming [`Script::parse_partial`] entry point used by editors for
//! sub-2ms edits, together with the `adjust_section_spans` helper that shifts
//! unchanged section spans after a text change.

use alloc::vec::Vec;
#[cfg(feature = "stream")]
use alloc::{format, string::ToString};
#[cfg(feature = "stream")]
use core::ops::Range;

use crate::parser::ast::Section;
#[cfg(feature = "stream")]
use crate::parser::streaming;
#[cfg(feature = "stream")]
use crate::Result;

#[cfg(feature = "stream")]
use super::delta::{calculate_delta, ScriptDeltaOwned};
use super::Script;

impl<'a> Script<'a> {
    /// Parse incrementally with range-based updates for editors
    ///
    /// Updates only the specified range, keeping other sections unchanged.
    /// Enables <2ms edit responsiveness for interactive editing.
    ///
    /// # Arguments
    ///
    /// * `range` - Byte range in source to re-parse
    /// * `new_text` - Replacement text for the range
    ///
    /// # Returns
    ///
    /// Delta containing changes that can be applied to existing script.
    ///
    /// # Errors
    ///
    /// Returns an error if the new text contains malformed section headers or
    /// other unrecoverable syntax errors in the specified range.
    #[cfg(feature = "stream")]
    pub fn parse_partial(&self, range: Range<usize>, new_text: &str) -> Result<ScriptDeltaOwned> {
        // Build the modified source
        let modified_source =
            streaming::build_modified_source(self.source, range.clone(), new_text);

        // Create a TextChange for incremental parsing
        let change = crate::parser::incremental::TextChange {
            range: range.clone(),
            new_text: new_text.to_string(),
            line_range: crate::parser::incremental::calculate_line_range(self.source, range),
        };

        // Parse incrementally
        let new_script = self.parse_incremental(&modified_source, &change)?;

        // Calculate delta
        let delta = calculate_delta(self, &new_script);

        // Convert to owned format
        let mut owned_delta = ScriptDeltaOwned {
            added: Vec::new(),
            modified: Vec::new(),
            removed: Vec::new(),
            new_issues: Vec::new(),
        };

        // Convert added sections
        for section in delta.added {
            owned_delta.added.push(format!("{section:?}"));
        }

        // Convert modified sections
        for (idx, section) in delta.modified {
            owned_delta.modified.push((idx, format!("{section:?}")));
        }

        // Convert removed sections
        owned_delta.removed = delta.removed;

        // Convert new issues
        owned_delta.new_issues = delta.new_issues;

        Ok(owned_delta)
    }

    /// Adjust section spans for unchanged sections after a text change
    pub(super) fn adjust_section_spans(
        section: &Section<'a>,
        change: &crate::parser::incremental::TextChange,
    ) -> Section<'a> {
        use crate::parser::ast::Span;

        // Calculate the offset caused by the change
        let new_len = change.new_text.len();
        let old_len = change.range.end - change.range.start;

        // Helper to adjust a span using safe arithmetic
        let adjust_span = |span: &Span| -> Span {
            let new_start = if new_len >= old_len {
                span.start + (new_len - old_len)
            } else {
                span.start.saturating_sub(old_len - new_len)
            };

            let new_end = if new_len >= old_len {
                span.end + (new_len - old_len)
            } else {
                span.end.saturating_sub(old_len - new_len)
            };

            Span::new(new_start, new_end, span.line, span.column)
        };

        // Adjust all spans in the section
        match section {
            Section::ScriptInfo(info) => {
                let mut new_info = info.clone();
                new_info.span = adjust_span(&info.span);
                Section::ScriptInfo(new_info)
            }
            Section::Styles(styles) => {
                let new_styles: Vec<_> = styles
                    .iter()
                    .map(|style| {
                        let mut new_style = style.clone();
                        new_style.span = adjust_span(&style.span);
                        new_style
                    })
                    .collect();
                Section::Styles(new_styles)
            }
            Section::Events(events) => {
                let new_events: Vec<_> = events
                    .iter()
                    .map(|event| {
                        let mut new_event = event.clone();
                        new_event.span = adjust_span(&event.span);
                        new_event
                    })
                    .collect();
                Section::Events(new_events)
            }
            Section::Fonts(fonts) => {
                let new_fonts: Vec<_> = fonts
                    .iter()
                    .map(|font| {
                        let mut new_font = font.clone();
                        new_font.span = adjust_span(&font.span);
                        new_font
                    })
                    .collect();
                Section::Fonts(new_fonts)
            }
            Section::Graphics(graphics) => {
                let new_graphics: Vec<_> = graphics
                    .iter()
                    .map(|graphic| {
                        let mut new_graphic = graphic.clone();
                        new_graphic.span = adjust_span(&graphic.span);
                        new_graphic
                    })
                    .collect();
                Section::Graphics(new_graphics)
            }
        }
    }
}
