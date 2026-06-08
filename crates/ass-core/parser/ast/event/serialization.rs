//! ASS string serialization and span validation for [`Event`]
//!
//! Provides the default and format-driven `to_ass_string` conversions plus the
//! debug-only `validate_spans` zero-copy invariant check.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{format, vec::Vec};

use super::Event;
#[cfg(debug_assertions)]
use core::ops::Range;

impl Event<'_> {
    /// Convert event to ASS string representation
    ///
    /// Generates the standard ASS event line format. Uses `margin_v` by default,
    /// but will use `margin_t/margin_b` if provided (V4++ format).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::{Event, EventType};
    /// let event = Event {
    ///     event_type: EventType::Dialogue,
    ///     layer: "0",
    ///     start: "0:00:05.00",
    ///     end: "0:00:10.00",
    ///     style: "Default",
    ///     text: "Hello",
    ///     ..Event::default()
    /// };
    /// assert_eq!(
    ///     event.to_ass_string(),
    ///     "Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Hello"
    /// );
    /// ```
    #[must_use]
    pub fn to_ass_string(&self) -> alloc::string::String {
        let event_type_str = self.event_type.as_str();

        // Use standard V4+ format by default
        // TODO: Support custom format lines
        format!(
            "{event_type_str}: {},{},{},{},{},{},{},{},{},{}",
            self.layer,
            self.start,
            self.end,
            self.style,
            self.name,
            self.margin_l,
            self.margin_r,
            self.margin_v,
            self.effect,
            self.text
        )
    }

    /// Convert event to ASS string with specific format
    ///
    /// Generates an ASS event line according to the provided format specification.
    /// This allows handling both V4+ and V4++ formats, as well as custom formats.
    ///
    /// # Arguments
    ///
    /// * `format` - Field names in order (e.g., `["Layer", "Start", "End", "Style", "Text"]`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ass_core::parser::ast::{Event, EventType};
    /// let event = Event {
    ///     event_type: EventType::Comment,
    ///     start: "0:00:00.00",
    ///     end: "0:00:05.00",
    ///     text: "Note",
    ///     ..Event::default()
    /// };
    /// let format = vec!["Start", "End", "Text"];
    /// assert_eq!(
    ///     event.to_ass_string_with_format(&format),
    ///     "Comment: 0:00:00.00,0:00:05.00,Note"
    /// );
    /// ```
    #[must_use]
    pub fn to_ass_string_with_format(&self, format: &[&str]) -> alloc::string::String {
        let event_type_str = self.event_type.as_str();
        let mut field_values = Vec::with_capacity(format.len());

        for field in format {
            let value = match *field {
                "Layer" => self.layer,
                "Start" => self.start,
                "End" => self.end,
                "Style" => self.style,
                "Name" | "Actor" => self.name,
                "MarginL" => self.margin_l,
                "MarginR" => self.margin_r,
                "MarginV" => self.margin_v,
                "MarginT" => self.margin_t.unwrap_or("0"),
                "MarginB" => self.margin_b.unwrap_or("0"),
                "Effect" => self.effect,
                "Text" => self.text,
                _ => "", // Unknown fields default to empty
            };
            field_values.push(value);
        }

        format!("{event_type_str}: {}", field_values.join(","))
    }

    /// Validate all spans in this Event reference valid source
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    /// Validates that all string references point to memory within
    /// the specified source range.
    ///
    /// Only available in debug builds to avoid performance overhead.
    #[cfg(debug_assertions)]
    #[must_use]
    pub fn validate_spans(&self, source_range: &Range<usize>) -> bool {
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
