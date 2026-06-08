//! Event builder type with fluent constructors and field setters.

use ass_core::parser::ast::EventType;

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::{borrow::Cow, string::ToString};

/// Builder for creating ASS events with fluent API
///
/// Provides an ergonomic way to construct ASS events with method chaining.
/// Supports all event types and automatically handles format validation.
///
/// # Examples
///
/// ```
/// use ass_editor::{EventBuilder, EditorDocument};
///
/// let mut doc = EditorDocument::new();
///
/// // Create a dialogue event
/// let event_line = EventBuilder::dialogue()
///     .start_time("0:00:00.00")
///     .end_time("0:00:05.00")
///     .style("Default")
///     .speaker("Character")
///     .text("Hello, world!")
///     .layer(0)
///     .build()
///     .unwrap();
///
/// // Add to document
/// doc.add_event_line(&event_line).unwrap();
/// ```
#[derive(Debug, Default)]
pub struct EventBuilder<'a> {
    pub(super) event_type: Option<EventType>,
    pub(super) start: Option<Cow<'a, str>>,
    pub(super) end: Option<Cow<'a, str>>,
    pub(super) style: Option<Cow<'a, str>>,
    pub(super) name: Option<Cow<'a, str>>,
    pub(super) text: Option<Cow<'a, str>>,
    pub(super) layer: Option<Cow<'a, str>>,
    pub(super) margin_l: Option<Cow<'a, str>>,
    pub(super) margin_r: Option<Cow<'a, str>>,
    pub(super) margin_v: Option<Cow<'a, str>>,
    pub(super) margin_t: Option<Cow<'a, str>>,
    pub(super) margin_b: Option<Cow<'a, str>>,
    pub(super) effect: Option<Cow<'a, str>>,
}

impl<'a> EventBuilder<'a> {
    /// Create a new event builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a dialogue event builder
    pub fn dialogue() -> Self {
        Self {
            event_type: Some(EventType::Dialogue),
            ..Self::default()
        }
    }

    /// Create a comment event builder
    pub fn comment() -> Self {
        Self {
            event_type: Some(EventType::Comment),
            ..Self::default()
        }
    }

    /// Set start time (e.g., "0:00:05.00")
    pub fn start_time<S: Into<Cow<'a, str>>>(mut self, time: S) -> Self {
        self.start = Some(time.into());
        self
    }

    /// Set end time (e.g., "0:00:10.00")
    pub fn end_time<S: Into<Cow<'a, str>>>(mut self, time: S) -> Self {
        self.end = Some(time.into());
        self
    }

    /// Set speaker/character name
    pub fn speaker<S: Into<Cow<'a, str>>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set dialogue text
    pub fn text<S: Into<Cow<'a, str>>>(mut self, text: S) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Set style name
    pub fn style<S: Into<Cow<'a, str>>>(mut self, style: S) -> Self {
        self.style = Some(style.into());
        self
    }

    /// Set layer (higher layers render on top)
    pub fn layer(mut self, layer: u32) -> Self {
        self.layer = Some(Cow::Owned(layer.to_string()));
        self
    }

    /// Set left margin
    pub fn margin_left(mut self, margin: u32) -> Self {
        self.margin_l = Some(Cow::Owned(margin.to_string()));
        self
    }

    /// Set right margin
    pub fn margin_right(mut self, margin: u32) -> Self {
        self.margin_r = Some(Cow::Owned(margin.to_string()));
        self
    }

    /// Set vertical margin
    pub fn margin_vertical(mut self, margin: u32) -> Self {
        self.margin_v = Some(Cow::Owned(margin.to_string()));
        self
    }

    /// Set top margin (V4++)
    pub fn margin_top(mut self, margin: u32) -> Self {
        self.margin_t = Some(Cow::Owned(margin.to_string()));
        self
    }

    /// Set bottom margin (V4++)
    pub fn margin_bottom(mut self, margin: u32) -> Self {
        self.margin_b = Some(Cow::Owned(margin.to_string()));
        self
    }

    /// Set effect
    pub fn effect<S: Into<Cow<'a, str>>>(mut self, effect: S) -> Self {
        self.effect = Some(effect.into());
        self
    }
}
