//! Fluent API for document editing
//!
//! Provides an ergonomic builder pattern for document edits:
//! ```
//! # use ass_editor::{EditorDocument, Position, Range};
//! # let mut doc = EditorDocument::new();
//! # let pos = Position::new(0);
//! # let range = Range::new(Position::new(0), Position::new(5));
//! doc.at(pos).insert_text("Hello").unwrap();
//! // doc.at_line(5).replace_text("New content"); // Not yet implemented
//! doc.select(range).wrap_with_tag("\\b1", "\\b0").unwrap();
//! ```

use super::{EditorDocument, Position, Range, Result, StyleBuilder};
use crate::commands::{
    AddFontCommand, AddGraphicCommand, AdjustKaraokeCommand, ApplyKaraokeCommand,
    ApplyStyleCommand, BatchDeleteEventsCommand, ClearFontsCommand, ClearGraphicsCommand,
    CloneStyleCommand, CreateStyleCommand, DeleteEventCommand, DeleteScriptInfoCommand,
    DeleteStyleCommand, EditStyleCommand, EditorCommand, EffectOperation, EventEffectCommand,
    GenerateKaraokeCommand, GetAllScriptInfoCommand, GetScriptInfoCommand, InsertTagCommand,
    KaraokeType, ListFontsCommand, ListGraphicsCommand, MergeEventsCommand, ParseTagCommand,
    ParsedTag, RemoveFontCommand, RemoveGraphicCommand, RemoveTagCommand, ReplaceTagCommand,
    SetScriptInfoCommand, SplitEventCommand, SplitKaraokeCommand, TimingAdjustCommand,
    ToggleEventTypeCommand, WrapTagCommand,
};
use crate::core::errors::EditorError;
use ass_core::parser::ast::{Event, EventType, Section};
use core::cmp::Ordering;

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[cfg(feature = "std")]
use std::vec;

// ============================================================================
// Event Query and Filtering Types
// ============================================================================

/// Event information with index for filtering/sorting results
#[derive(Debug, Clone, PartialEq)]
pub struct EventInfo {
    /// Zero-based index in the document
    pub index: usize,
    /// Owned copy of the event data
    pub event: OwnedEvent,
    /// Line number in the document (1-based)
    pub line_number: usize,
    /// Character position range in document
    pub range: Range,
}

/// Owned version of Event for use in EventInfo
#[derive(Debug, Clone, PartialEq)]
pub struct OwnedEvent {
    /// Event type (Dialogue, Comment, etc.)
    pub event_type: EventType,
    /// Layer for drawing order (higher layers drawn on top)
    pub layer: String,
    /// Start time in ASS time format (H:MM:SS.CS)
    pub start: String,
    /// End time in ASS time format (H:MM:SS.CS)
    pub end: String,
    /// Style name reference
    pub style: String,
    /// Character name or speaker
    pub name: String,
    /// Left margin override (pixels)
    pub margin_l: String,
    /// Right margin override (pixels)
    pub margin_r: String,
    /// Vertical margin override (pixels) (V4+)
    pub margin_v: String,
    /// Top margin override (pixels) (V4++) - optional
    pub margin_t: Option<String>,
    /// Bottom margin override (pixels) (V4++) - optional
    pub margin_b: Option<String>,
    /// Effect specification for special rendering
    pub effect: String,
    /// Text content with possible style overrides
    pub text: String,
}

impl<'a> From<&Event<'a>> for OwnedEvent {
    fn from(event: &Event<'a>) -> Self {
        Self {
            event_type: event.event_type,
            layer: event.layer.to_string(),
            start: event.start.to_string(),
            end: event.end.to_string(),
            style: event.style.to_string(),
            name: event.name.to_string(),
            margin_l: event.margin_l.to_string(),
            margin_r: event.margin_r.to_string(),
            margin_v: event.margin_v.to_string(),
            margin_t: event.margin_t.map(|s| s.to_string()),
            margin_b: event.margin_b.map(|s| s.to_string()),
            effect: event.effect.to_string(),
            text: event.text.to_string(),
        }
    }
}

/// Filter criteria for events
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Filter by event type (Dialogue, Comment)
    pub event_type: Option<EventType>,
    /// Filter by style name pattern
    pub style_pattern: Option<String>,
    /// Filter by speaker/actor name pattern  
    pub speaker_pattern: Option<String>,
    /// Filter by text content pattern
    pub text_pattern: Option<String>,
    /// Filter by time range (start_cs, end_cs)
    pub time_range: Option<(u32, u32)>,
    /// Filter by layer
    pub layer: Option<u32>,
    /// Filter by effect presence/pattern
    pub effect_pattern: Option<String>,
    /// Use regex for pattern matching
    pub use_regex: bool,
    /// Case sensitive matching
    pub case_sensitive: bool,
}

/// Sort criteria for events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventSortCriteria {
    /// Sort by start time (default)
    StartTime,
    /// Sort by end time
    EndTime,
    /// Sort by duration (end - start)
    Duration,
    /// Sort by style name
    Style,
    /// Sort by speaker/actor name
    Speaker,
    /// Sort by layer
    Layer,
    /// Sort by document order (original index)
    Index,
    /// Sort by text content (alphabetical)
    Text,
}

/// Sort options
#[derive(Debug, Clone)]
pub struct EventSortOptions {
    /// Primary sort criteria
    pub criteria: EventSortCriteria,
    /// Secondary sort criteria (for ties)
    pub secondary: Option<EventSortCriteria>,
    /// Sort in ascending order (default true)
    pub ascending: bool,
}

impl Default for EventSortOptions {
    fn default() -> Self {
        Self {
            criteria: EventSortCriteria::Index,
            secondary: None,
            ascending: true,
        }
    }
}

// ============================================================================
// Existing Fluent API Types
// ============================================================================

/// Fluent API builder for document operations at a specific position
pub struct AtPosition<'a> {
    document: &'a mut EditorDocument,
    position: Position,
}

impl<'a> AtPosition<'a> {
    /// Create a new fluent API at position
    pub(crate) fn new(document: &'a mut EditorDocument, position: Position) -> Self {
        Self { document, position }
    }

    /// Insert text at the current position
    pub fn insert_text(self, text: &str) -> Result<&'a mut EditorDocument> {
        let range = Range::empty(self.position);
        self.document.replace(range, text)?;
        Ok(self.document)
    }

    /// Insert a line break at the current position
    pub fn insert_line(self) -> Result<&'a mut EditorDocument> {
        self.insert_text("\n")
    }

    /// Delete a number of characters forward from position
    pub fn delete(self, count: usize) -> Result<&'a mut EditorDocument> {
        let end = self.position.advance(count);
        let range = Range::new(self.position, end);
        self.document.delete(range)?;
        Ok(self.document)
    }

    /// Delete characters backward from position (backspace)
    pub fn backspace(self, count: usize) -> Result<&'a mut EditorDocument> {
        let start = self.position.retreat(count);
        let range = Range::new(start, self.position);
        self.document.delete(range)?;
        Ok(self.document)
    }

    /// Replace text from position to end of line
    pub fn replace_to_line_end(self, text: &str) -> Result<&'a mut EditorDocument> {
        #[cfg(feature = "rope")]
        {
            let rope = self.document.rope();
            let line_idx = rope.byte_to_line(self.position.offset);
            let line_end_byte = if line_idx + 1 < rope.len_lines() {
                rope.line_to_byte(line_idx + 1).saturating_sub(1)
            } else {
                rope.len_bytes()
            };
            let range = Range::new(self.position, Position::new(line_end_byte));
            self.document.replace(range, text)?;
            Ok(self.document)
        }

        #[cfg(not(feature = "rope"))]
        {
            Err(EditorError::FeatureNotEnabled {
                feature: "line-based operations".to_string(),
                required_feature: "rope".to_string(),
            })
        }
    }

    /// Get the current position
    pub const fn position(&self) -> Position {
        self.position
    }

    /// Convert position to line/column
    #[cfg(feature = "rope")]
    pub fn to_line_column(&self) -> Result<(usize, usize)> {
        let rope = self.document.rope();
        let line_idx = rope.byte_to_line(self.position.offset);
        let line_start = rope.line_to_byte(line_idx);
        let col_offset = self.position.offset - line_start;

        // Convert byte offset to character offset
        let line = rope.line(line_idx);
        let mut char_col = 0;
        let mut byte_count = 0;

        for ch in line.chars() {
            if byte_count >= col_offset {
                break;
            }
            byte_count += ch.len_utf8();
            char_col += 1;
        }

        Ok((line_idx + 1, char_col + 1)) // Convert to 1-indexed
    }
}

/// Fluent API builder for operations on a selected range
pub struct SelectRange<'a> {
    document: &'a mut EditorDocument,
    range: Range,
}

impl<'a> SelectRange<'a> {
    /// Create a new fluent API for range
    pub(crate) fn new(document: &'a mut EditorDocument, range: Range) -> Self {
        Self { document, range }
    }

    /// Replace the selected range with text
    pub fn replace_with(self, text: &str) -> Result<&'a mut EditorDocument> {
        self.document.replace(self.range, text)?;
        Ok(self.document)
    }

    /// Delete the selected range
    pub fn delete(self) -> Result<&'a mut EditorDocument> {
        self.document.delete(self.range)?;
        Ok(self.document)
    }

    /// Wrap the selection with ASS tags
    pub fn wrap_with_tag(self, open_tag: &str, close_tag: &str) -> Result<&'a mut EditorDocument> {
        // Get the selected text
        let selected = self
            .document
            .rope()
            .byte_slice(self.range.start.offset..self.range.end.offset);
        let mut wrapped =
            String::with_capacity(open_tag.len() + selected.len_bytes() + close_tag.len());
        wrapped.push_str(open_tag);
        wrapped.push_str(&selected.to_string());
        wrapped.push_str(close_tag);

        self.document.replace(self.range, &wrapped)?;
        Ok(self.document)
    }

    /// Indent the selected lines
    #[cfg(feature = "rope")]
    pub fn indent(self, spaces: usize) -> Result<&'a mut EditorDocument> {
        // Get line information before mutating
        let start_line = self.document.rope().byte_to_line(self.range.start.offset);
        let end_line = self.document.rope().byte_to_line(self.range.end.offset);
        let indent = " ".repeat(spaces);

        // Collect line positions
        let mut line_positions = Vec::new();
        for line_idx in (start_line..=end_line).rev() {
            let line_start = self.document.rope().line_to_byte(line_idx);
            line_positions.push(line_start);
        }

        // Apply indentation
        for line_start in line_positions {
            let pos = Position::new(line_start);
            let range = Range::empty(pos);
            self.document.replace(range, &indent)?;
        }

        Ok(self.document)
    }

    /// Unindent the selected lines
    #[cfg(feature = "rope")]
    pub fn unindent(self, spaces: usize) -> Result<&'a mut EditorDocument> {
        // Get line information before mutating
        let start_line = self.document.rope().byte_to_line(self.range.start.offset);
        let end_line = self.document.rope().byte_to_line(self.range.end.offset);

        // Collect unindent operations
        let mut unindent_ops = Vec::new();
        for line_idx in (start_line..=end_line).rev() {
            let line_start = self.document.rope().line_to_byte(line_idx);
            let line = self.document.rope().line(line_idx);

            // Count spaces to remove
            let mut space_count = 0;
            for ch in line.chars().take(spaces) {
                if ch == ' ' {
                    space_count += 1;
                } else {
                    break;
                }
            }

            if space_count > 0 {
                unindent_ops.push((line_start, space_count));
            }
        }

        // Apply unindent operations
        for (line_start, space_count) in unindent_ops {
            let range = Range::new(
                Position::new(line_start),
                Position::new(line_start + space_count),
            );
            self.document.delete(range)?;
        }

        Ok(self.document)
    }

    /// Get the selected text
    pub fn text(&self) -> String {
        self.document
            .rope()
            .byte_slice(self.range.start.offset..self.range.end.offset)
            .to_string()
    }

    /// Get the range
    pub const fn range(&self) -> Range {
        self.range
    }
}

/// Fluent API builder for style operations
pub struct StyleOps<'a> {
    document: &'a mut EditorDocument,
}

impl<'a> StyleOps<'a> {
    /// Create a new style operations builder
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self { document }
    }

    /// Create a new style
    pub fn create(self, name: &str, builder: StyleBuilder) -> Result<&'a mut EditorDocument> {
        let command = CreateStyleCommand::new(name.to_string(), builder);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Edit an existing style
    pub fn edit(self, name: &str) -> StyleEditor<'a> {
        StyleEditor::new(self.document, name.to_string())
    }

    /// Delete a style
    pub fn delete(self, name: &str) -> Result<&'a mut EditorDocument> {
        let command = DeleteStyleCommand::new(name.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Clone a style
    pub fn clone(self, source: &str, target: &str) -> Result<&'a mut EditorDocument> {
        let command = CloneStyleCommand::new(source.to_string(), target.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Apply a style to events
    pub fn apply(self, old_style: &str, new_style: &str) -> StyleApplicator<'a> {
        StyleApplicator::new(self.document, old_style.to_string(), new_style.to_string())
    }
}

/// Fluent API builder for editing a specific style
pub struct StyleEditor<'a> {
    document: &'a mut EditorDocument,
    command: EditStyleCommand,
}

impl<'a> StyleEditor<'a> {
    /// Create a new style editor
    pub(crate) fn new(document: &'a mut EditorDocument, style_name: String) -> Self {
        let command = EditStyleCommand::new(style_name);
        Self { document, command }
    }

    /// Set font name
    pub fn font(mut self, font: &str) -> Self {
        self.command = self.command.set_font(font);
        self
    }

    /// Set font size
    pub fn size(mut self, size: u32) -> Self {
        self.command = self.command.set_size(size);
        self
    }

    /// Set primary color
    pub fn color(mut self, color: &str) -> Self {
        self.command = self.command.set_color(color);
        self
    }

    /// Set bold
    pub fn bold(mut self, bold: bool) -> Self {
        self.command = self.command.set_bold(bold);
        self
    }

    /// Set italic
    pub fn italic(mut self, italic: bool) -> Self {
        self.command = self.command.set_italic(italic);
        self
    }

    /// Set alignment
    pub fn alignment(mut self, alignment: u32) -> Self {
        self.command = self.command.set_alignment(alignment);
        self
    }

    /// Set a custom field
    pub fn field(mut self, name: &str, value: &str) -> Self {
        self.command = self.command.set_field(name, value.to_string());
        self
    }

    /// Apply the changes
    pub fn apply(self) -> Result<&'a mut EditorDocument> {
        self.command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Fluent API builder for applying styles to events
pub struct StyleApplicator<'a> {
    document: &'a mut EditorDocument,
    command: ApplyStyleCommand,
}

impl<'a> StyleApplicator<'a> {
    /// Create a new style applicator
    pub(crate) fn new(
        document: &'a mut EditorDocument,
        old_style: String,
        new_style: String,
    ) -> Self {
        let command = ApplyStyleCommand::new(old_style, new_style);
        Self { document, command }
    }

    /// Only apply to events containing specific text
    pub fn with_filter(mut self, filter: &str) -> Self {
        self.command = self.command.with_filter(filter.to_string());
        self
    }

    /// Apply the style changes
    pub fn apply(self) -> Result<&'a mut EditorDocument> {
        self.command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Fluent API builder for event operations
pub struct EventOps<'a> {
    document: &'a mut EditorDocument,
}

impl<'a> EventOps<'a> {
    /// Create a new event operations builder
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self { document }
    }

    /// Split an event at a specific time
    pub fn split(self, event_index: usize, split_time: &str) -> Result<&'a mut EditorDocument> {
        let command = SplitEventCommand::new(event_index, split_time.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Merge two consecutive events
    pub fn merge(self, first_index: usize, second_index: usize) -> EventMerger<'a> {
        EventMerger::new(self.document, first_index, second_index)
    }

    /// Adjust timing for events
    pub fn timing(self) -> EventTimer<'a> {
        EventTimer::new(self.document)
    }

    /// Toggle event types between Dialogue and Comment
    pub fn toggle_type(self) -> EventToggler<'a> {
        EventToggler::new(self.document)
    }

    /// Modify event effects
    pub fn effects(self) -> EventEffector<'a> {
        EventEffector::new(self.document)
    }

    // ============================================================================
    // Direct Event Access Methods (NEW!)
    // ============================================================================

    /// Get event information by index
    pub fn get(self, index: usize) -> Result<Option<EventInfo>> {
        self.document
            .parse_script_with(|script| -> Result<Option<EventInfo>> {
                let mut current_index = 0;

                for section in script.sections() {
                    if let Section::Events(events) = section {
                        for event in events {
                            if current_index == index {
                                let event_info = EventInfo {
                                    index,
                                    event: OwnedEvent::from(event),
                                    line_number: self.find_line_number_for_event(event)?,
                                    range: self.find_range_for_event(event)?,
                                };
                                return Ok(Some(event_info));
                            }
                            current_index += 1;
                        }
                    }
                }

                Ok(None)
            })?
    }

    /// Get event by index with fluent access to properties
    pub fn event(self, index: usize) -> EventAccessor<'a> {
        EventAccessor::new(self.document, index)
    }

    /// Get all events as a vector
    pub fn all(self) -> Result<Vec<EventInfo>> {
        EventQuery::new(self.document).execute()
    }

    /// Get event count
    pub fn count(self) -> Result<usize> {
        self.document.parse_script_with(|script| {
            let mut count = 0;

            for section in script.sections() {
                if let Section::Events(events) = section {
                    count += events.len();
                }
            }

            count
        })
    }

    // ============================================================================
    // Query and Filtering Methods (NEW!)
    // ============================================================================

    /// Start a query chain for filtering and sorting events
    pub fn query(self) -> EventQuery<'a> {
        EventQuery::new(self.document)
    }

    /// Shorthand for common filters
    pub fn dialogues(self) -> EventQuery<'a> {
        EventQuery::new(self.document).filter_by_type(EventType::Dialogue)
    }

    pub fn comments(self) -> EventQuery<'a> {
        EventQuery::new(self.document).filter_by_type(EventType::Comment)
    }

    pub fn in_time_range(self, start_cs: u32, end_cs: u32) -> EventQuery<'a> {
        EventQuery::new(self.document).filter_by_time_range(start_cs, end_cs)
    }

    pub fn with_style(self, pattern: &str) -> EventQuery<'a> {
        EventQuery::new(self.document).filter_by_style(pattern)
    }

    /// Find events by text pattern
    pub fn containing(self, text: &str) -> EventQuery<'a> {
        EventQuery::new(self.document).filter_by_text(text)
    }

    /// Get events in order they appear in document
    pub fn in_order(self) -> EventQuery<'a> {
        EventQuery::new(self.document).sort(EventSortCriteria::Index)
    }

    /// Get events sorted by time
    pub fn by_time(self) -> EventQuery<'a> {
        EventQuery::new(self.document).sort_by_time()
    }

    /// Delete a single event by index
    pub fn delete(self, index: usize) -> Result<&'a mut EditorDocument> {
        let command = DeleteEventCommand::new(index);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete multiple events by indices
    pub fn delete_multiple(self, indices: Vec<usize>) -> Result<&'a mut EditorDocument> {
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete all events matching a query
    pub fn delete_query(self) -> EventDeleter<'a> {
        EventDeleter::new(self.document)
    }

    // ============================================================================
    // Helper Methods
    // ============================================================================

    fn find_line_number_for_event(&self, _event: &Event) -> Result<usize> {
        // For now, return a placeholder. This would need to be implemented
        // by tracking line numbers during parsing or using rope byte-to-line conversion
        Ok(1)
    }

    fn find_range_for_event(&self, _event: &Event) -> Result<Range> {
        // For now, return a placeholder range. This would need to be implemented
        // by using the event's span information from the parser
        Ok(Range::new(Position::new(0), Position::new(0)))
    }
}

/// Fluent API builder for merging events
pub struct EventMerger<'a> {
    document: &'a mut EditorDocument,
    first_index: usize,
    second_index: usize,
    separator: String,
}

impl<'a> EventMerger<'a> {
    /// Create a new event merger
    pub(crate) fn new(
        document: &'a mut EditorDocument,
        first_index: usize,
        second_index: usize,
    ) -> Self {
        Self {
            document,
            first_index,
            second_index,
            separator: " ".to_string(),
        }
    }

    /// Set the text separator for merged text
    pub fn with_separator(mut self, separator: &str) -> Self {
        self.separator = separator.to_string();
        self
    }

    /// Execute the merge
    pub fn apply(self) -> Result<&'a mut EditorDocument> {
        let command = MergeEventsCommand::new(self.first_index, self.second_index)
            .with_separator(self.separator);
        command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Fluent API builder for timing adjustments
pub struct EventTimer<'a> {
    document: &'a mut EditorDocument,
    event_indices: Vec<usize>,
}

impl<'a> EventTimer<'a> {
    /// Create a new event timer
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            event_indices: Vec::new(), // Default to all events
        }
    }

    /// Specify which events to adjust
    pub fn events(mut self, indices: Vec<usize>) -> Self {
        self.event_indices = indices;
        self
    }

    /// Adjust a single event
    pub fn event(mut self, index: usize) -> Self {
        self.event_indices = vec![index];
        self
    }

    /// Shift start and end times by the same offset (preserves duration)
    pub fn shift(self, offset_cs: i32) -> Result<&'a mut EditorDocument> {
        let command = TimingAdjustCommand::new(self.event_indices, offset_cs, offset_cs);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Shift only start times (changes duration)
    pub fn shift_start(self, offset_cs: i32) -> Result<&'a mut EditorDocument> {
        let command = TimingAdjustCommand::new(self.event_indices, offset_cs, 0);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Shift only end times (changes duration)
    pub fn shift_end(self, offset_cs: i32) -> Result<&'a mut EditorDocument> {
        let command = TimingAdjustCommand::new(self.event_indices, 0, offset_cs);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Scale duration by a factor
    pub fn scale_duration(self, factor: f64) -> Result<&'a mut EditorDocument> {
        let command = TimingAdjustCommand::scale_duration(self.event_indices, factor);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Custom start and end offsets
    pub fn adjust(
        self,
        start_offset_cs: i32,
        end_offset_cs: i32,
    ) -> Result<&'a mut EditorDocument> {
        let command = TimingAdjustCommand::new(self.event_indices, start_offset_cs, end_offset_cs);
        command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Fluent API builder for toggling event types
pub struct EventToggler<'a> {
    document: &'a mut EditorDocument,
    event_indices: Vec<usize>,
}

impl<'a> EventToggler<'a> {
    /// Create a new event toggler
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            event_indices: Vec::new(), // Default to all events
        }
    }

    /// Specify which events to toggle
    pub fn events(mut self, indices: Vec<usize>) -> Self {
        self.event_indices = indices;
        self
    }

    /// Toggle a single event
    pub fn event(mut self, index: usize) -> Self {
        self.event_indices = vec![index];
        self
    }

    /// Execute the toggle
    pub fn apply(self) -> Result<&'a mut EditorDocument> {
        let command = ToggleEventTypeCommand::new(self.event_indices);
        command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Fluent API builder for event effects
pub struct EventEffector<'a> {
    document: &'a mut EditorDocument,
    event_indices: Vec<usize>,
}

impl<'a> EventEffector<'a> {
    /// Create a new event effector
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            event_indices: Vec::new(), // Default to all events
        }
    }

    /// Specify which events to modify
    pub fn events(mut self, indices: Vec<usize>) -> Self {
        self.event_indices = indices;
        self
    }

    /// Modify a single event
    pub fn event(mut self, index: usize) -> Self {
        self.event_indices = vec![index];
        self
    }

    /// Set the effect (replace existing)
    pub fn set(self, effect: &str) -> Result<&'a mut EditorDocument> {
        let command = EventEffectCommand::set_effect(self.event_indices, effect.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Clear all effects
    pub fn clear(self) -> Result<&'a mut EditorDocument> {
        let command = EventEffectCommand::clear_effect(self.event_indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Append to existing effect
    pub fn append(self, effect: &str) -> Result<&'a mut EditorDocument> {
        let command = EventEffectCommand::append_effect(self.event_indices, effect.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Prepend to existing effect
    pub fn prepend(self, effect: &str) -> Result<&'a mut EditorDocument> {
        let command = EventEffectCommand::new(
            self.event_indices,
            effect.to_string(),
            EffectOperation::Prepend,
        );
        command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Fluent API for ASS tag operations
pub struct TagOps<'a> {
    document: &'a mut EditorDocument,
    range: Option<Range>,
    position: Option<Position>,
}

impl<'a> TagOps<'a> {
    /// Create new tag operations
    fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            range: None,
            position: None,
        }
    }

    /// Set position for tag insertion
    #[must_use]
    pub fn at(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// Set range for tag operations
    #[must_use]
    pub fn in_range(mut self, range: Range) -> Self {
        self.range = Some(range);
        self
    }

    /// Insert ASS override tag at position
    pub fn insert(self, tag: &str) -> Result<&'a mut EditorDocument> {
        let position = self
            .position
            .ok_or_else(|| EditorError::command_failed("Position required for tag insertion"))?;

        let command = InsertTagCommand::new(position, tag.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Insert tag without auto-wrapping in {}
    pub fn insert_raw(self, tag: &str) -> Result<&'a mut EditorDocument> {
        let position = self
            .position
            .ok_or_else(|| EditorError::command_failed("Position required for tag insertion"))?;

        let command = InsertTagCommand::new(position, tag.to_string()).no_auto_wrap();
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Remove all tags from range
    pub fn remove_all(self) -> Result<&'a mut EditorDocument> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range required for tag removal"))?;

        let command = RemoveTagCommand::new(range);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Remove specific tag pattern from range
    pub fn remove_pattern(self, pattern: &str) -> Result<&'a mut EditorDocument> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range required for tag removal"))?;

        let command = RemoveTagCommand::new(range).pattern(pattern.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Replace tag pattern with another tag
    pub fn replace(self, find_pattern: &str, replace_with: &str) -> Result<&'a mut EditorDocument> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range required for tag replacement"))?;

        let command =
            ReplaceTagCommand::new(range, find_pattern.to_string(), replace_with.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Replace all occurrences of tag pattern
    pub fn replace_all(
        self,
        find_pattern: &str,
        replace_with: &str,
    ) -> Result<&'a mut EditorDocument> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range required for tag replacement"))?;

        let command =
            ReplaceTagCommand::new(range, find_pattern.to_string(), replace_with.to_string()).all();
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Wrap range with opening and closing tags
    pub fn wrap(self, opening_tag: &str) -> Result<&'a mut EditorDocument> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range required for tag wrapping"))?;

        let command = WrapTagCommand::new(range, opening_tag.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Wrap range with explicit opening and closing tags
    pub fn wrap_with(self, opening_tag: &str, closing_tag: &str) -> Result<&'a mut EditorDocument> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range required for tag wrapping"))?;

        let command = WrapTagCommand::new(range, opening_tag.to_string())
            .closing_tag(closing_tag.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Parse tags from range and return parsed information
    pub fn parse(self) -> Result<Vec<ParsedTag>> {
        let range = self.range.unwrap_or_else(|| {
            Range::new(Position::new(0), Position::new(self.document.text().len()))
        });

        let command = ParseTagCommand::new(range).with_positions();
        let text = self.document.text_range(range)?;
        command.parse_tags_from_text(&text)
    }
}

/// Fluent API for ASS karaoke operations
pub struct KaraokeOps<'a> {
    document: &'a mut EditorDocument,
    range: Option<Range>,
}

impl<'a> KaraokeOps<'a> {
    /// Create new karaoke operations
    fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            range: None,
        }
    }

    /// Set range for karaoke operations
    #[must_use]
    pub fn in_range(mut self, range: Range) -> Self {
        self.range = Some(range);
        self
    }

    /// Generate karaoke timing for text
    pub fn generate(self, default_duration: u32) -> KaraokeGenerator<'a> {
        let default_range = if self.range.is_none() {
            let doc_len = self.document.text().len();
            Range::new(Position::new(0), Position::new(doc_len))
        } else {
            Range::new(Position::new(0), Position::new(0)) // Placeholder, won't be used
        };

        KaraokeGenerator {
            document: self.document,
            range: self.range.unwrap_or(default_range),
            default_duration,
            karaoke_type: KaraokeType::Standard,
            auto_detect_syllables: true,
        }
    }

    /// Split existing karaoke timing
    pub fn split(self, split_positions: Vec<usize>) -> KaraokeSplitter<'a> {
        let default_range = if self.range.is_none() {
            let doc_len = self.document.text().len();
            Range::new(Position::new(0), Position::new(doc_len))
        } else {
            Range::new(Position::new(0), Position::new(0)) // Placeholder
        };

        KaraokeSplitter {
            document: self.document,
            range: self.range.unwrap_or(default_range),
            split_positions,
            new_duration: None,
        }
    }

    /// Adjust existing karaoke timing
    pub fn adjust(self) -> KaraokeAdjuster<'a> {
        let default_range = if self.range.is_none() {
            let doc_len = self.document.text().len();
            Range::new(Position::new(0), Position::new(doc_len))
        } else {
            Range::new(Position::new(0), Position::new(0)) // Placeholder
        };

        KaraokeAdjuster {
            document: self.document,
            range: self.range.unwrap_or(default_range),
        }
    }

    /// Apply karaoke template
    pub fn apply(self) -> KaraokeApplicator<'a> {
        let default_range = if self.range.is_none() {
            let doc_len = self.document.text().len();
            Range::new(Position::new(0), Position::new(doc_len))
        } else {
            Range::new(Position::new(0), Position::new(0)) // Placeholder
        };

        KaraokeApplicator {
            document: self.document,
            range: self.range.unwrap_or(default_range),
        }
    }
}

/// Karaoke generator builder
pub struct KaraokeGenerator<'a> {
    document: &'a mut EditorDocument,
    range: Range,
    default_duration: u32,
    karaoke_type: KaraokeType,
    auto_detect_syllables: bool,
}

impl<'a> KaraokeGenerator<'a> {
    /// Set karaoke type
    #[must_use]
    pub fn karaoke_type(mut self, karaoke_type: KaraokeType) -> Self {
        self.karaoke_type = karaoke_type;
        self
    }

    /// Use manual syllable splitting
    #[must_use]
    pub fn manual_syllables(mut self) -> Self {
        self.auto_detect_syllables = false;
        self
    }

    /// Execute the generation
    pub fn execute(self) -> Result<&'a mut EditorDocument> {
        let mut command = GenerateKaraokeCommand::new(self.range, self.default_duration)
            .karaoke_type(self.karaoke_type);

        if !self.auto_detect_syllables {
            command = command.manual_syllables();
        }

        command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Karaoke splitter builder
pub struct KaraokeSplitter<'a> {
    document: &'a mut EditorDocument,
    range: Range,
    split_positions: Vec<usize>,
    new_duration: Option<u32>,
}

impl<'a> KaraokeSplitter<'a> {
    /// Set new duration for split segments
    #[must_use]
    pub fn duration(mut self, duration: u32) -> Self {
        self.new_duration = Some(duration);
        self
    }

    /// Execute the split
    pub fn execute(self) -> Result<&'a mut EditorDocument> {
        let mut command = SplitKaraokeCommand::new(self.range, self.split_positions);

        if let Some(duration) = self.new_duration {
            command = command.duration(duration);
        }

        command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Karaoke adjuster builder
pub struct KaraokeAdjuster<'a> {
    document: &'a mut EditorDocument,
    range: Range,
}

impl<'a> KaraokeAdjuster<'a> {
    /// Scale timing by factor
    pub fn scale(self, factor: f32) -> Result<&'a mut EditorDocument> {
        let command = AdjustKaraokeCommand::scale(self.range, factor);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Offset timing by centiseconds
    pub fn offset(self, offset: i32) -> Result<&'a mut EditorDocument> {
        let command = AdjustKaraokeCommand::offset(self.range, offset);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Set all timing to specific duration
    pub fn set_all(self, duration: u32) -> Result<&'a mut EditorDocument> {
        let command = AdjustKaraokeCommand::set_all(self.range, duration);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Apply custom timing to each syllable
    pub fn custom(self, timings: Vec<u32>) -> Result<&'a mut EditorDocument> {
        let command = AdjustKaraokeCommand::custom(self.range, timings);
        command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Karaoke applicator builder
pub struct KaraokeApplicator<'a> {
    document: &'a mut EditorDocument,
    range: Range,
}

impl<'a> KaraokeApplicator<'a> {
    /// Apply equal timing
    pub fn equal(self, duration: u32, karaoke_type: KaraokeType) -> Result<&'a mut EditorDocument> {
        let command = ApplyKaraokeCommand::equal(self.range, duration, karaoke_type);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Apply beat-based timing
    pub fn beat(
        self,
        bpm: u32,
        beats_per_syllable: f32,
        karaoke_type: KaraokeType,
    ) -> Result<&'a mut EditorDocument> {
        let command = ApplyKaraokeCommand::beat(self.range, bpm, beats_per_syllable, karaoke_type);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Apply pattern-based timing
    pub fn pattern(
        self,
        durations: Vec<u32>,
        karaoke_type: KaraokeType,
    ) -> Result<&'a mut EditorDocument> {
        let command = ApplyKaraokeCommand::pattern(self.range, durations, karaoke_type);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Import timing from another event
    pub fn import_from(self, source_event_index: usize) -> Result<&'a mut EditorDocument> {
        let command = ApplyKaraokeCommand::import_from(self.range, source_event_index);
        command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Extension trait to add fluent API to EditorDocument
impl EditorDocument {
    /// Start a fluent operation at a position
    pub fn at_pos(&mut self, position: Position) -> AtPosition<'_> {
        AtPosition::new(self, position)
    }

    /// Start a fluent operation at a line
    #[cfg(feature = "rope")]
    pub fn at_line(&mut self, line: usize) -> Result<AtPosition<'_>> {
        let line_idx = line.saturating_sub(1);
        if line_idx >= self.rope().len_lines() {
            return Err(EditorError::InvalidPosition { line, column: 1 });
        }

        let byte_pos = self.rope().line_to_byte(line_idx);
        Ok(AtPosition::new(self, Position::new(byte_pos)))
    }

    /// Start a fluent operation at the start of the document
    pub fn at_start(&mut self) -> AtPosition<'_> {
        AtPosition::new(self, Position::start())
    }

    /// Start a fluent operation at the end of the document
    pub fn at_end(&mut self) -> AtPosition<'_> {
        let end_pos = Position::new(self.len());
        AtPosition::new(self, end_pos)
    }

    /// Start a fluent operation on a range
    pub fn select(&mut self, range: Range) -> SelectRange<'_> {
        SelectRange::new(self, range)
    }

    /// Start fluent style operations
    pub fn styles(&mut self) -> StyleOps<'_> {
        StyleOps::new(self)
    }

    /// Start fluent event operations
    pub fn events(&mut self) -> EventOps<'_> {
        EventOps::new(self)
    }

    /// Start fluent tag operations
    pub fn tags(&mut self) -> TagOps<'_> {
        TagOps::new(self)
    }

    /// Start fluent karaoke operations
    pub fn karaoke(&mut self) -> KaraokeOps<'_> {
        KaraokeOps::new(self)
    }

    /// Start fluent script info operations
    pub fn info(&mut self) -> ScriptInfoOps<'_> {
        ScriptInfoOps::new(self)
    }

    /// Start fluent fonts operations
    pub fn fonts(&mut self) -> FontsOps<'_> {
        FontsOps::new(self)
    }

    /// Start fluent graphics operations
    pub fn graphics(&mut self) -> GraphicsOps<'_> {
        GraphicsOps::new(self)
    }

    /// Convert a Position to line/column tuple
    #[cfg(feature = "rope")]
    pub fn position_to_line_col(&self, pos: Position) -> Result<(usize, usize)> {
        if pos.offset > self.len() {
            return Err(EditorError::PositionOutOfBounds {
                position: pos.offset,
                length: self.len(),
            });
        }

        let line_idx = self.rope().byte_to_line(pos.offset);
        let line_start = self.rope().line_to_byte(line_idx);
        let col_offset = pos.offset - line_start;

        // Convert byte offset to character offset
        let line = self.rope().line(line_idx);
        let mut char_col = 0;
        let mut byte_count = 0;

        for ch in line.chars() {
            if byte_count >= col_offset {
                break;
            }
            byte_count += ch.len_utf8();
            char_col += 1;
        }

        Ok((line_idx + 1, char_col + 1)) // Convert to 1-indexed
    }

    /// Convert line/column to Position
    #[cfg(feature = "rope")]
    pub fn line_column_to_position(&self, line: usize, column: usize) -> Result<Position> {
        use super::PositionBuilder;

        PositionBuilder::new()
            .line(line)
            .column(column)
            .build(self.rope())
    }
}

// ============================================================================
// Event Accessor for Individual Event Operations (NEW!)
// ============================================================================

/// Fluent accessor for individual event properties and operations
pub struct EventAccessor<'a> {
    document: &'a mut EditorDocument,
    index: usize,
}

impl<'a> EventAccessor<'a> {
    pub(crate) fn new(document: &'a mut EditorDocument, index: usize) -> Self {
        Self { document, index }
    }

    /// Get the full event information
    pub fn get(self) -> Result<Option<EventInfo>> {
        EventOps::new(self.document).get(self.index)
    }

    /// Get just the event text
    pub fn text(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.text))
    }

    /// Get event style name
    pub fn style(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.style))
    }

    /// Get event speaker/actor name
    pub fn speaker(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.name))
    }

    /// Get event timing as (start, end) in centiseconds
    pub fn timing(self) -> Result<Option<(String, String)>> {
        Ok(self.get()?.map(|info| (info.event.start, info.event.end)))
    }

    /// Get event start time
    pub fn start_time(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.start))
    }

    /// Get event end time  
    pub fn end_time(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.end))
    }

    /// Get event layer
    pub fn layer(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.layer))
    }

    /// Get event effect
    pub fn effect(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.effect))
    }

    /// Get event type (Dialogue/Comment)
    pub fn event_type(self) -> Result<Option<EventType>> {
        Ok(self.get()?.map(|info| info.event.event_type))
    }

    /// Check if event exists
    pub fn exists(self) -> Result<bool> {
        Ok(self.get()?.is_some())
    }

    /// Get event margins as (left, right, vertical)
    pub fn margins(self) -> Result<Option<(String, String, String)>> {
        Ok(self.get()?.map(|info| {
            (
                info.event.margin_l,
                info.event.margin_r,
                info.event.margin_v,
            )
        }))
    }

    /// Convert to timing operations for this specific event
    pub fn timing_ops(self) -> EventTimer<'a> {
        EventTimer::new(self.document).event(self.index)
    }

    /// Convert to toggle operations for this specific event
    pub fn toggle_ops(self) -> EventToggler<'a> {
        EventToggler::new(self.document).event(self.index)
    }

    /// Convert to effect operations for this specific event
    pub fn effect_ops(self) -> EventEffector<'a> {
        EventEffector::new(self.document).event(self.index)
    }
}

// ============================================================================
// Event Query Builder for Filtering and Sorting (NEW!)
// ============================================================================

/// Main query builder for filtering and sorting events
pub struct EventQuery<'a> {
    document: &'a mut EditorDocument,
    filters: EventFilter,
    sort_options: Option<EventSortOptions>,
    limit: Option<usize>,
}

impl<'a> EventQuery<'a> {
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            filters: EventFilter::default(),
            sort_options: None,
            limit: None,
        }
    }

    // Filter methods
    pub fn filter(mut self, filter: EventFilter) -> Self {
        self.filters = filter;
        self
    }

    pub fn filter_by_type(mut self, event_type: EventType) -> Self {
        self.filters.event_type = Some(event_type);
        self
    }

    pub fn filter_by_style(mut self, pattern: &str) -> Self {
        self.filters.style_pattern = Some(pattern.to_string());
        self
    }

    pub fn filter_by_speaker(mut self, pattern: &str) -> Self {
        self.filters.speaker_pattern = Some(pattern.to_string());
        self
    }

    pub fn filter_by_text(mut self, pattern: &str) -> Self {
        self.filters.text_pattern = Some(pattern.to_string());
        self
    }

    pub fn filter_by_time_range(mut self, start_cs: u32, end_cs: u32) -> Self {
        self.filters.time_range = Some((start_cs, end_cs));
        self
    }

    pub fn filter_by_layer(mut self, layer: u32) -> Self {
        self.filters.layer = Some(layer);
        self
    }

    pub fn filter_by_effect(mut self, pattern: &str) -> Self {
        self.filters.effect_pattern = Some(pattern.to_string());
        self
    }

    pub fn with_regex(mut self, use_regex: bool) -> Self {
        self.filters.use_regex = use_regex;
        self
    }

    pub fn case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.filters.case_sensitive = case_sensitive;
        self
    }

    // Sort methods
    pub fn sort(mut self, criteria: EventSortCriteria) -> Self {
        self.sort_options = Some(EventSortOptions {
            criteria,
            secondary: None,
            ascending: true,
        });
        self
    }

    pub fn sort_by(mut self, options: EventSortOptions) -> Self {
        self.sort_options = Some(options);
        self
    }

    pub fn sort_by_time(self) -> Self {
        self.sort(EventSortCriteria::StartTime)
    }

    pub fn sort_by_style(self) -> Self {
        self.sort(EventSortCriteria::Style)
    }

    pub fn sort_by_duration(self) -> Self {
        self.sort(EventSortCriteria::Duration)
    }

    pub fn descending(mut self) -> Self {
        if let Some(ref mut options) = self.sort_options {
            options.ascending = false;
        }
        self
    }

    pub fn then_by(mut self, criteria: EventSortCriteria) -> Self {
        if let Some(ref mut options) = self.sort_options {
            options.secondary = Some(criteria);
        }
        self
    }

    // Limit results
    pub fn limit(mut self, count: usize) -> Self {
        self.limit = Some(count);
        self
    }

    pub fn take(self, count: usize) -> Self {
        self.limit(count)
    }

    // Execution methods
    pub fn execute(self) -> Result<Vec<EventInfo>> {
        let mut results = self.collect_events()?;

        // Apply filters
        results = self.apply_filters(results)?;

        // Apply sorting
        if let Some(ref sort_options) = self.sort_options {
            self.apply_sort(&mut results, sort_options);
        }

        // Apply limit
        if let Some(limit) = self.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Execute and return only the indices
    pub fn indices(self) -> Result<Vec<usize>> {
        Ok(self.execute()?.into_iter().map(|info| info.index).collect())
    }

    /// Execute and return events with their indices as tuples
    pub fn with_indices(self) -> Result<Vec<(usize, OwnedEvent)>> {
        Ok(self
            .execute()?
            .into_iter()
            .map(|info| (info.index, info.event))
            .collect())
    }

    /// Execute and get the first matching event
    pub fn first(self) -> Result<Option<EventInfo>> {
        let mut results = self.limit(1).execute()?;
        Ok(results.pop())
    }

    /// Execute and get count of matching events
    pub fn count(self) -> Result<usize> {
        Ok(self.execute()?.len())
    }

    /// Chain with existing fluent operations
    pub fn timing(self) -> Result<EventTimer<'a>> {
        let _indices: Vec<usize> = self.execute()?.into_iter().map(|info| info.index).collect();
        // Need to create a new EventQuery to get document reference since self is consumed
        // This is a limitation of the current API design
        Err(EditorError::command_failed(
            "Cannot chain timing operations after query execution - use indices() first",
        ))
    }

    pub fn toggle_type(self) -> Result<EventToggler<'a>> {
        let _indices: Vec<usize> = self.execute()?.into_iter().map(|info| info.index).collect();
        Err(EditorError::command_failed(
            "Cannot chain toggle operations after query execution - use indices() first",
        ))
    }

    pub fn effects(self) -> Result<EventEffector<'a>> {
        let _indices: Vec<usize> = self.execute()?.into_iter().map(|info| info.index).collect();
        Err(EditorError::command_failed(
            "Cannot chain effect operations after query execution - use indices() first",
        ))
    }

    // ============================================================================
    // Implementation Details
    // ============================================================================

    fn collect_events(&self) -> Result<Vec<EventInfo>> {
        self.document
            .parse_script_with(|script| -> Result<Vec<EventInfo>> {
                let mut events = Vec::new();
                let mut event_index = 0;

                for section in script.sections() {
                    if let Section::Events(section_events) = section {
                        for event in section_events {
                            // Build EventInfo with position tracking
                            let event_info = EventInfo {
                                index: event_index,
                                event: OwnedEvent::from(event),
                                line_number: self.find_line_number(event)?,
                                range: self.find_event_range(event)?,
                            };
                            events.push(event_info);
                            event_index += 1;
                        }
                    }
                }

                Ok(events)
            })?
    }

    fn apply_filters(&self, events: Vec<EventInfo>) -> Result<Vec<EventInfo>> {
        let mut filtered = Vec::new();

        for event_info in events {
            if self.matches_filter(&event_info)? {
                filtered.push(event_info);
            }
        }

        Ok(filtered)
    }

    fn matches_filter(&self, event_info: &EventInfo) -> Result<bool> {
        // Apply each filter criteria
        if let Some(event_type) = self.filters.event_type {
            if event_info.event.event_type != event_type {
                return Ok(false);
            }
        }

        if let Some(ref pattern) = self.filters.style_pattern {
            if !self.matches_pattern(&event_info.event.style, pattern)? {
                return Ok(false);
            }
        }

        if let Some(ref pattern) = self.filters.text_pattern {
            if !self.matches_pattern(&event_info.event.text, pattern)? {
                return Ok(false);
            }
        }

        if let Some(ref pattern) = self.filters.speaker_pattern {
            if !self.matches_pattern(&event_info.event.name, pattern)? {
                return Ok(false);
            }
        }

        if let Some(ref pattern) = self.filters.effect_pattern {
            if !self.matches_pattern(&event_info.event.effect, pattern)? {
                return Ok(false);
            }
        }

        if let Some(layer) = self.filters.layer {
            if let Ok(event_layer) = event_info.event.layer.parse::<u32>() {
                if event_layer != layer {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        if let Some((start_cs, end_cs)) = self.filters.time_range {
            // Parse timing - this is a simplified implementation
            // In practice, you'd want proper time parsing from ass_core
            if let (Ok(event_start), Ok(event_end)) = (
                self.parse_time_to_cs(&event_info.event.start),
                self.parse_time_to_cs(&event_info.event.end),
            ) {
                if event_start < start_cs || event_end > end_cs {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn matches_pattern(&self, text: &str, pattern: &str) -> Result<bool> {
        if self.filters.use_regex {
            // For now, just do simple string matching
            // In a full implementation, you'd use regex crate
            Ok(if self.filters.case_sensitive {
                text.contains(pattern)
            } else {
                text.to_lowercase().contains(&pattern.to_lowercase())
            })
        } else {
            Ok(if self.filters.case_sensitive {
                text.contains(pattern)
            } else {
                text.to_lowercase().contains(&pattern.to_lowercase())
            })
        }
    }

    fn parse_time_to_cs(&self, time_str: &str) -> Result<u32> {
        // Simplified time parsing - in practice use ass_core utilities
        // Format: H:MM:SS.CS
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 3 {
            return Err(EditorError::command_failed("Invalid time format"));
        }

        let hours: u32 = parts[0]
            .parse()
            .map_err(|_| EditorError::command_failed("Invalid hours"))?;
        let minutes: u32 = parts[1]
            .parse()
            .map_err(|_| EditorError::command_failed("Invalid minutes"))?;

        let sec_cs_parts: Vec<&str> = parts[2].split('.').collect();
        if sec_cs_parts.len() != 2 {
            return Err(EditorError::command_failed("Invalid seconds format"));
        }

        let seconds: u32 = sec_cs_parts[0]
            .parse()
            .map_err(|_| EditorError::command_failed("Invalid seconds"))?;
        let centiseconds: u32 = sec_cs_parts[1]
            .parse()
            .map_err(|_| EditorError::command_failed("Invalid centiseconds"))?;

        Ok(hours * 360000 + minutes * 6000 + seconds * 100 + centiseconds)
    }

    fn apply_sort(&self, events: &mut [EventInfo], options: &EventSortOptions) {
        events.sort_by(|a, b| {
            let primary_cmp = self.compare_by_criteria(a, b, &options.criteria);

            match primary_cmp {
                Ordering::Equal => {
                    if let Some(secondary) = &options.secondary {
                        let secondary_cmp = self.compare_by_criteria(a, b, secondary);
                        if options.ascending {
                            secondary_cmp
                        } else {
                            secondary_cmp.reverse()
                        }
                    } else {
                        Ordering::Equal
                    }
                }
                other => {
                    if options.ascending {
                        other
                    } else {
                        other.reverse()
                    }
                }
            }
        });
    }

    fn compare_by_criteria(
        &self,
        a: &EventInfo,
        b: &EventInfo,
        criteria: &EventSortCriteria,
    ) -> Ordering {
        match criteria {
            EventSortCriteria::StartTime => {
                let a_time = self.parse_time_to_cs(&a.event.start).unwrap_or(0);
                let b_time = self.parse_time_to_cs(&b.event.start).unwrap_or(0);
                a_time.cmp(&b_time)
            }
            EventSortCriteria::EndTime => {
                let a_time = self.parse_time_to_cs(&a.event.end).unwrap_or(0);
                let b_time = self.parse_time_to_cs(&b.event.end).unwrap_or(0);
                a_time.cmp(&b_time)
            }
            EventSortCriteria::Duration => {
                let a_start = self.parse_time_to_cs(&a.event.start).unwrap_or(0);
                let a_end = self.parse_time_to_cs(&a.event.end).unwrap_or(0);
                let b_start = self.parse_time_to_cs(&b.event.start).unwrap_or(0);
                let b_end = self.parse_time_to_cs(&b.event.end).unwrap_or(0);
                let a_duration = a_end.saturating_sub(a_start);
                let b_duration = b_end.saturating_sub(b_start);
                a_duration.cmp(&b_duration)
            }
            EventSortCriteria::Style => a.event.style.cmp(&b.event.style),
            EventSortCriteria::Speaker => a.event.name.cmp(&b.event.name),
            EventSortCriteria::Layer => {
                let a_layer = a.event.layer.parse::<u32>().unwrap_or(0);
                let b_layer = b.event.layer.parse::<u32>().unwrap_or(0);
                a_layer.cmp(&b_layer)
            }
            EventSortCriteria::Index => a.index.cmp(&b.index),
            EventSortCriteria::Text => a.event.text.cmp(&b.event.text),
        }
    }

    fn find_line_number(&self, _event: &Event) -> Result<usize> {
        // For now, return a placeholder. This would need to be implemented
        // by tracking line numbers during parsing or using rope byte-to-line conversion
        Ok(1)
    }

    fn find_event_range(&self, _event: &Event) -> Result<Range> {
        // For now, return a placeholder range. This would need to be implemented
        // by using the event's span information from the parser
        Ok(Range::new(Position::new(0), Position::new(0)))
    }
}

/// Fluent API for managing Script Info properties
pub struct ScriptInfoOps<'a> {
    document: &'a mut EditorDocument,
}

impl<'a> ScriptInfoOps<'a> {
    /// Create new script info operations
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self { document }
    }

    /// Set a script info property
    pub fn set(self, property: &str, value: &str) -> Result<&'a mut EditorDocument> {
        let command = SetScriptInfoCommand::new(property.to_string(), value.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Get a script info property value
    pub fn get(&self, property: &str) -> Result<Option<String>> {
        let command = GetScriptInfoCommand::new(property.to_string());
        command.get_value(self.document)
    }

    /// Delete a script info property
    pub fn delete(self, property: &str) -> Result<&'a mut EditorDocument> {
        let command = DeleteScriptInfoCommand::new(property.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Get all script info properties
    pub fn all(&self) -> Result<Vec<(String, String)>> {
        let command = GetAllScriptInfoCommand::new();
        command.get_all(self.document)
    }

    /// Set the title
    pub fn title(self, title: &str) -> Result<&'a mut EditorDocument> {
        self.set("Title", title)
    }

    /// Get the title
    pub fn get_title(&self) -> Result<Option<String>> {
        self.get("Title")
    }

    /// Set the author
    pub fn author(self, author: &str) -> Result<&'a mut EditorDocument> {
        self.set("Original Script", author)
    }

    /// Get the author
    pub fn get_author(&self) -> Result<Option<String>> {
        self.get("Original Script")
    }

    /// Set the resolution
    pub fn resolution(self, width: u32, height: u32) -> Result<&'a mut EditorDocument> {
        let command1 = SetScriptInfoCommand::new("PlayResX".to_string(), width.to_string());
        let command2 = SetScriptInfoCommand::new("PlayResY".to_string(), height.to_string());
        command1.execute(self.document)?;
        command2.execute(self.document)?;
        Ok(self.document)
    }

    /// Get the resolution
    pub fn get_resolution(&self) -> Result<Option<(u32, u32)>> {
        let width_cmd = GetScriptInfoCommand::new("PlayResX".to_string());
        let height_cmd = GetScriptInfoCommand::new("PlayResY".to_string());

        let width = width_cmd.get_value(self.document)?;
        let height = height_cmd.get_value(self.document)?;

        match (width, height) {
            (Some(w), Some(h)) => {
                let width_val = w
                    .parse::<u32>()
                    .map_err(|_| EditorError::command_failed("Invalid PlayResX value"))?;
                let height_val = h
                    .parse::<u32>()
                    .map_err(|_| EditorError::command_failed("Invalid PlayResY value"))?;
                Ok(Some((width_val, height_val)))
            }
            _ => Ok(None),
        }
    }

    /// Set the wrap style
    pub fn wrap_style(self, style: u8) -> Result<&'a mut EditorDocument> {
        self.set("WrapStyle", &style.to_string())
    }

    /// Get the wrap style
    pub fn get_wrap_style(&self) -> Result<Option<u8>> {
        self.get("WrapStyle")?
            .map(|s| {
                s.parse::<u8>()
                    .map_err(|_| EditorError::command_failed("Invalid WrapStyle value"))
            })
            .transpose()
    }

    /// Set scaled border and shadow
    pub fn scaled_border_and_shadow(self, scaled: bool) -> Result<&'a mut EditorDocument> {
        let value = if scaled { "yes" } else { "no" };
        self.set("ScaledBorderAndShadow", value)
    }

    /// Get scaled border and shadow setting
    pub fn get_scaled_border_and_shadow(&self) -> Result<Option<bool>> {
        Ok(self
            .get("ScaledBorderAndShadow")?
            .map(|s| s.to_lowercase() == "yes" || s == "1"))
    }
}

/// Fluent API for managing fonts
pub struct FontsOps<'a> {
    document: &'a mut EditorDocument,
}

impl<'a> FontsOps<'a> {
    /// Create new fonts operations
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self { document }
    }

    /// Add a font from UU-encoded data
    pub fn add(self, filename: &str, data_lines: Vec<String>) -> Result<&'a mut EditorDocument> {
        let command = AddFontCommand::new(filename.to_string(), data_lines);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Add a font from binary data (will UU-encode it)
    pub fn add_binary(self, filename: &str, data: &[u8]) -> Result<&'a mut EditorDocument> {
        let command = AddFontCommand::from_binary(filename.to_string(), data);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Remove a font by filename
    pub fn remove(self, filename: &str) -> Result<&'a mut EditorDocument> {
        let command = RemoveFontCommand::new(filename.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// List all font filenames
    pub fn list(&self) -> Result<Vec<String>> {
        let command = ListFontsCommand::new();
        command.list(self.document)
    }

    /// Check if a font exists
    pub fn exists(&self, filename: &str) -> Result<bool> {
        Ok(self.list()?.contains(&filename.to_string()))
    }

    /// Clear all fonts
    pub fn clear(self) -> Result<&'a mut EditorDocument> {
        let command = ClearFontsCommand::new();
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Get count of fonts
    pub fn count(&self) -> Result<usize> {
        Ok(self.list()?.len())
    }
}

/// Fluent API for managing graphics
pub struct GraphicsOps<'a> {
    document: &'a mut EditorDocument,
}

impl<'a> GraphicsOps<'a> {
    /// Create new graphics operations
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self { document }
    }

    /// Add a graphic from UU-encoded data
    pub fn add(self, filename: &str, data_lines: Vec<String>) -> Result<&'a mut EditorDocument> {
        let command = AddGraphicCommand::new(filename.to_string(), data_lines);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Add a graphic from binary data (will UU-encode it)
    pub fn add_binary(self, filename: &str, data: &[u8]) -> Result<&'a mut EditorDocument> {
        let command = AddGraphicCommand::from_binary(filename.to_string(), data);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Remove a graphic by filename
    pub fn remove(self, filename: &str) -> Result<&'a mut EditorDocument> {
        let command = RemoveGraphicCommand::new(filename.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// List all graphic filenames
    pub fn list(&self) -> Result<Vec<String>> {
        let command = ListGraphicsCommand::new();
        command.list(self.document)
    }

    /// Check if a graphic exists
    pub fn exists(&self, filename: &str) -> Result<bool> {
        Ok(self.list()?.contains(&filename.to_string()))
    }

    /// Clear all graphics
    pub fn clear(self) -> Result<&'a mut EditorDocument> {
        let command = ClearGraphicsCommand::new();
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Get count of graphics
    pub fn count(&self) -> Result<usize> {
        Ok(self.list()?.len())
    }
}

/// Fluent API for deleting events based on queries
pub struct EventDeleter<'a> {
    document: &'a mut EditorDocument,
    indices: Vec<usize>,
}

impl<'a> EventDeleter<'a> {
    /// Create a new event deleter
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            indices: Vec::new(),
        }
    }

    /// Delete events by their indices
    pub fn by_indices(mut self, indices: Vec<usize>) -> Self {
        self.indices = indices;
        self
    }

    /// Delete all dialogue events
    pub fn dialogues(self) -> Result<&'a mut EditorDocument> {
        let indices = EventQuery::new(self.document)
            .filter_by_type(EventType::Dialogue)
            .indices()?;
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete all comment events
    pub fn comments(self) -> Result<&'a mut EditorDocument> {
        let indices = EventQuery::new(self.document)
            .filter_by_type(EventType::Comment)
            .indices()?;
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete events in a time range
    pub fn in_time_range(self, start_cs: u32, end_cs: u32) -> Result<&'a mut EditorDocument> {
        let indices = EventQuery::new(self.document)
            .filter_by_time_range(start_cs, end_cs)
            .indices()?;
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete events with a specific style
    pub fn with_style(self, style: &str) -> Result<&'a mut EditorDocument> {
        let indices = EventQuery::new(self.document)
            .filter_by_style(style)
            .indices()?;
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete events containing specific text
    pub fn containing(self, text: &str) -> Result<&'a mut EditorDocument> {
        let indices = EventQuery::new(self.document)
            .filter_by_text(text)
            .indices()?;
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete all events
    pub fn all(self) -> Result<&'a mut EditorDocument> {
        let indices = EventQuery::new(self.document).indices()?;
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Execute deletion with the configured indices
    pub fn execute(self) -> Result<&'a mut EditorDocument> {
        if self.indices.is_empty() {
            return Ok(self.document);
        }
        let command = BatchDeleteEventsCommand::new(self.indices);
        command.execute(self.document)?;
        Ok(self.document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;
    #[cfg(not(feature = "std"))]
    use alloc::vec;

    #[test]
    #[cfg(feature = "rope")]
    fn test_fluent_insert() {
        let mut doc = EditorDocument::new();
        doc.at_start().insert_text("Hello, ").unwrap();
        doc.at_end().insert_text("World!").unwrap();

        assert_eq!(doc.text(), "Hello, World!");
    }

    #[test]
    #[cfg(feature = "rope")]
    fn test_fluent_line_operations() {
        let mut doc = EditorDocument::from_content("Line 1\nLine 2\nLine 3").unwrap();

        // Insert at beginning of line 2
        doc.at_line(2).unwrap().insert_text("Start: ").unwrap();
        assert_eq!(doc.text(), "Line 1\nStart: Line 2\nLine 3");

        // Replace to end of line
        doc.at_line(2)
            .unwrap()
            .replace_to_line_end("New Line 2")
            .unwrap();
        assert_eq!(doc.text(), "Line 1\nNew Line 2\nLine 3");
    }

    #[test]
    #[cfg(feature = "rope")]
    fn test_fluent_selection() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();

        let range = Range::new(Position::new(6), Position::new(11));
        doc.select(range).replace_with("Rust").unwrap();
        assert_eq!(doc.text(), "Hello Rust");

        // Test wrapping
        let range = Range::new(Position::new(6), Position::new(10));
        doc.select(range).wrap_with_tag("{\\b1}", "{\\b0}").unwrap();
        assert_eq!(doc.text(), "Hello {\\b1}Rust{\\b0}");
    }

    #[test]
    #[cfg(feature = "rope")]
    fn test_position_conversion() {
        let doc = EditorDocument::from_content("Line 1\nLine 2\nLine 3").unwrap();

        // Test position to line/column
        let pos = Position::new(7); // Start of "Line 2"
        let (line, col) = doc.position_to_line_col(pos).unwrap();
        assert_eq!((line, col), (2, 1));

        // Test line/column to position
        let pos2 = doc.line_column_to_position(2, 1).unwrap();
        assert_eq!(pos2.offset, 7);
    }

    #[test]
    #[cfg(feature = "rope")]
    fn test_indent_unindent() {
        let mut doc = EditorDocument::from_content("Line 1\nLine 2\nLine 3").unwrap();

        // Select all and indent
        let range = Range::new(Position::start(), Position::new(doc.len()));
        doc.select(range).indent(2).unwrap();
        assert_eq!(doc.text(), "  Line 1\n  Line 2\n  Line 3");

        // Unindent
        let range = Range::new(Position::start(), Position::new(doc.len()));
        doc.select(range).unindent(2).unwrap();
        assert_eq!(doc.text(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_fluent_style_operations() {
        const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Hello world!
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test create style
        doc.styles()
            .create(
                "NewStyle",
                StyleBuilder::new()
                    .font("Comic Sans MS")
                    .size(24)
                    .bold(true),
            )
            .unwrap();

        assert!(doc.text().contains("Style: NewStyle"));
        assert!(doc.text().contains("Comic Sans MS"));

        // Test edit style
        doc.styles()
            .edit("Default")
            .font("Helvetica")
            .size(18)
            .bold(true)
            .apply()
            .unwrap();

        assert!(doc.text().contains("Helvetica"));
        assert!(doc.text().contains("18"));

        // Test clone style
        doc.styles().clone("Default", "DefaultCopy").unwrap();

        assert!(doc.text().contains("Style: DefaultCopy"));

        // Test apply style to events
        doc.styles().apply("Default", "NewStyle").apply().unwrap();

        // The dialogue event should now use NewStyle
        let text = doc.text();
        let events_section = text.split("[Events]").nth(1).unwrap();
        assert!(events_section.contains("NewStyle"));
    }

    #[test]
    fn test_fluent_style_delete() {
        const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: ToDelete,Times,22,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Verify style exists
        assert!(doc.text().contains("Style: ToDelete"));

        // Delete the style
        doc.styles().delete("ToDelete").unwrap();

        // Verify style is gone
        assert!(!doc.text().contains("Style: ToDelete"));
        assert!(doc.text().contains("Style: Default")); // Other styles should remain
    }

    #[test]
    fn test_fluent_style_apply_with_filter() {
        const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: FilterStyle,Times,22,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Hello world!
Dialogue: 0,0:00:06.00,0:00:10.00,Default,Speaker,0,0,0,,Goodbye world!
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Apply style only to events containing "Hello"
        doc.styles()
            .apply("Default", "FilterStyle")
            .with_filter("Hello")
            .apply()
            .unwrap();

        let content = doc.text();
        let lines: Vec<&str> = content.lines().collect();

        // Find the dialogue lines
        let hello_line = lines.iter().find(|line| line.contains("Hello")).unwrap();
        let goodbye_line = lines.iter().find(|line| line.contains("Goodbye")).unwrap();

        // Only the "Hello" line should use FilterStyle
        assert!(hello_line.contains("FilterStyle"));
        assert!(goodbye_line.contains("Default")); // Should still use Default
    }

    #[test]
    fn test_fluent_event_operations() {
        const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
Comment: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Third event
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test split event
        doc.events().split(0, "0:00:03.00").unwrap();

        // Should now have 4 events (first split into 2)
        let events_count = doc
            .text()
            .lines()
            .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
            .count();
        assert_eq!(events_count, 4);
        assert!(doc.text().contains("0:00:01.00,0:00:03.00"));
        assert!(doc.text().contains("0:00:03.00,0:00:05.00"));
    }

    #[test]
    fn test_fluent_event_merge() {
        const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
Comment: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Third event
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test merge events with custom separator
        doc.events()
            .merge(0, 1)
            .with_separator(" | ")
            .apply()
            .unwrap();

        // Should now have 2 events (first two merged)
        let events_count = doc
            .text()
            .lines()
            .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
            .count();
        assert_eq!(events_count, 2);
        assert!(doc.text().contains("First event | Second event"));
        assert!(doc.text().contains("0:00:01.00,0:00:10.00")); // Start of first, end of second
    }

    #[test]
    fn test_fluent_event_timing() {
        const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test shifting all events by 2 seconds (200 centiseconds)
        doc.events().timing().shift(200).unwrap();

        assert!(doc.text().contains("0:00:03.00,0:00:07.00")); // First event shifted
        assert!(doc.text().contains("0:00:07.00,0:00:12.00")); // Second event shifted
    }

    #[test]
    fn test_fluent_event_timing_specific() {
        const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test adjusting only first event
        doc.events()
            .timing()
            .event(0)
            .shift_start(100) // +1 second to start only
            .unwrap();

        // Only first event should change
        assert!(doc.text().contains("0:00:02.00,0:00:05.00")); // First event start shifted
        assert!(doc.text().contains("0:00:05.00,0:00:10.00")); // Second event unchanged
    }

    #[test]
    fn test_fluent_event_toggle() {
        const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Comment: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test toggling first event type
        doc.events().toggle_type().event(0).apply().unwrap();

        let text = doc.text();
        let lines: Vec<&str> = text.lines().collect();
        let event_lines: Vec<&str> = lines
            .iter()
            .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
            .copied()
            .collect();

        // First event should now be Comment (was Dialogue)
        assert_eq!(event_lines.len(), 2);
        assert!(event_lines[0].starts_with("Comment:"));
        assert!(event_lines[1].starts_with("Comment:")); // Second unchanged
    }

    #[test]
    fn test_fluent_event_effects() {
        const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test setting effects
        doc.events()
            .effects()
            .events(vec![0, 1])
            .set("Fade(255,0)")
            .unwrap();

        // Both events should have the effect
        let text = doc.text();
        let event_lines: Vec<&str> = text
            .lines()
            .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
            .collect();

        assert!(event_lines[0].contains("Fade(255,0)"));
        assert!(event_lines[1].contains("Fade(255,0)"));
    }

    #[test]
    fn test_fluent_event_effects_chaining() {
        const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test effect chaining: set, then append
        doc.events().effects().event(0).set("Fade(255,0)").unwrap();

        doc.events()
            .effects()
            .event(0)
            .append("Move(100,200)")
            .unwrap();

        // Should have both effects
        assert!(doc.text().contains("Fade(255,0) Move(100,200)"));

        // Test clearing
        doc.events().effects().event(0).clear().unwrap();

        // Effect field should be empty
        let text = doc.text();
        let event_line = text
            .lines()
            .find(|line| line.starts_with("Dialogue:"))
            .unwrap();
        let parts: Vec<&str> = event_line.split(',').collect();
        assert_eq!(parts[8].trim(), ""); // Effect field should be empty
    }

    #[test]
    fn test_fluent_event_complex_workflow() {
        const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Long event that needs splitting
Dialogue: 0,0:00:05.00,0:00:07.00,Default,Speaker,0,0,0,,Short event
Comment: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Comment to toggle
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Complex workflow: split, adjust timing, toggle type, add effects

        // 1. Split the first event
        doc.events().split(0, "0:00:03.00").unwrap();

        // Now we have 4 events: split first, original second, original comment

        // 2. Shift all events forward by 1 second
        doc.events()
            .timing()
            .shift(100) // 1 second
            .unwrap();

        // 3. Toggle the comment (now at index 3) to dialogue
        doc.events().toggle_type().event(3).apply().unwrap();

        // 4. Add fade effect to all events
        doc.events().effects().set("Fade(255,0)").unwrap();

        let content = doc.text();

        // Verify results
        let event_lines: Vec<&str> = content
            .lines()
            .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
            .collect();

        // Should have 4 events, all Dialogue (comment was toggled)
        assert_eq!(event_lines.len(), 4);
        assert!(event_lines.iter().all(|line| line.starts_with("Dialogue:")));

        // All should have timing shifted by 1 second
        assert!(content.contains("0:00:02.00,0:00:04.00")); // First part of split
        assert!(content.contains("0:00:04.00,0:00:06.00")); // Second part of split
        assert!(content.contains("0:00:06.00,0:00:08.00")); // Original second event
        assert!(content.contains("0:00:11.00,0:00:16.00")); // Original comment (now dialogue)

        // All should have fade effect
        assert!(event_lines.iter().all(|line| line.contains("Fade(255,0)")));
    }

    #[test]
    fn tag_operations() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();

        // Test tag insertion
        doc.tags().at(Position::new(5)).insert("\\b1").unwrap();
        assert_eq!(doc.text(), "Hello{\\b1} World");

        // Test raw tag insertion - need to account for the inserted tag
        doc.tags().at(Position::new(12)).insert_raw("\\i1").unwrap();
        assert_eq!(doc.text(), "Hello{\\b1} W\\i1orld");
    }

    #[test]
    fn tag_removal() {
        let mut doc =
            EditorDocument::from_content("Hello {\\b1\\i1}World{\\c&H00FF00&} test").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Remove specific pattern
        doc.tags().in_range(range).remove_pattern("\\b").unwrap();
        assert_eq!(doc.text(), "Hello {\\i1}World{\\c&H00FF00&} test");

        // Remove all tags
        let full_range = Range::new(Position::new(0), Position::new(doc.text().len()));
        doc.tags().in_range(full_range).remove_all().unwrap();
        assert_eq!(doc.text(), "Hello World test");
    }

    #[test]
    fn tag_replacement() {
        let mut doc = EditorDocument::from_content("Hello {\\b1}World{\\b1} test").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Replace all bold tags with italic
        doc.tags()
            .in_range(range)
            .replace_all("\\b1", "\\i1")
            .unwrap();
        assert_eq!(doc.text(), "Hello {\\i1}World{\\i1} test");
    }

    #[test]
    fn tag_wrapping() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();
        let range = Range::new(Position::new(6), Position::new(11));

        // Wrap with bold tags
        doc.tags().in_range(range).wrap("\\b1").unwrap();
        assert_eq!(doc.text(), "Hello {\\b1}World{\\b0}");

        // Test explicit closing tag
        let mut doc2 = EditorDocument::from_content("Hello World").unwrap();
        let range2 = Range::new(Position::new(6), Position::new(11));
        doc2.tags()
            .in_range(range2)
            .wrap_with("\\c&HFF0000&", "\\c")
            .unwrap();
        assert_eq!(doc2.text(), "Hello {\\c&HFF0000&}World{\\c}");
    }

    #[test]
    fn tag_parsing() {
        let mut doc =
            EditorDocument::from_content("Hello {\\b1\\c&H00FF00&\\pos(100,200)}World").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        let parsed_tags = doc.tags().in_range(range).parse().unwrap();

        assert_eq!(parsed_tags.len(), 3);
        assert_eq!(parsed_tags[0].tag, "\\b1");
        assert_eq!(parsed_tags[1].tag, "\\c&H00FF00&");
        assert_eq!(parsed_tags[2].tag, "\\pos");
        assert_eq!(parsed_tags[2].parameters.len(), 2);
        assert_eq!(parsed_tags[2].parameters[0], "100");
        assert_eq!(parsed_tags[2].parameters[1], "200");
    }

    #[test]
    fn karaoke_generate() {
        let mut doc = EditorDocument::from_content("Hello World Test").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Test basic karaoke generation with manual syllables to preserve text
        doc.karaoke()
            .in_range(range)
            .generate(50)
            .manual_syllables()
            .execute()
            .unwrap();

        let text = doc.text();
        assert!(text.contains("\\k50"));
        // With manual syllables, the entire text should be preserved
        assert!(text.contains("Hello World Test"));
    }

    #[test]
    fn karaoke_generate_with_types() {
        let mut doc = EditorDocument::from_content("Test Text").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Test with fill karaoke
        doc.karaoke()
            .in_range(range)
            .generate(40)
            .karaoke_type(KaraokeType::Fill)
            .execute()
            .unwrap();

        assert!(doc.text().contains("\\kf40"));

        // Test with outline karaoke
        let mut doc2 = EditorDocument::from_content("Test Text").unwrap();
        let range2 = Range::new(Position::new(0), Position::new(doc2.text().len()));

        doc2.karaoke()
            .in_range(range2)
            .generate(30)
            .karaoke_type(KaraokeType::Outline)
            .execute()
            .unwrap();

        assert!(doc2.text().contains("\\ko30"));
    }

    #[test]
    fn karaoke_generate_manual_syllables() {
        let mut doc = EditorDocument::from_content("Syllable Test").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Test with manual syllable detection disabled
        doc.karaoke()
            .in_range(range)
            .generate(60)
            .manual_syllables()
            .execute()
            .unwrap();

        let text = doc.text();
        assert!(text.contains("\\k60"));
        assert!(text.contains("Syllable Test"));
    }

    #[test]
    fn karaoke_split() {
        let mut doc = EditorDocument::from_content("{\\k100}Hello World").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Split at position 5 (between "Hello" and " World")
        doc.karaoke()
            .in_range(range)
            .split(vec![5])
            .duration(25)
            .execute()
            .unwrap();

        let text = doc.text();
        assert!(text.contains("\\k25"));
    }

    #[test]
    fn karaoke_adjust_scale() {
        let mut doc = EditorDocument::from_content("{\\k50}Hello {\\k30}World").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Scale timing by 2.0
        doc.karaoke().in_range(range).adjust().scale(2.0).unwrap();

        let text = doc.text();
        assert!(text.contains("\\k100")); // 50 * 2.0
        assert!(text.contains("\\k60")); // 30 * 2.0
    }

    #[test]
    fn karaoke_adjust_offset() {
        let mut doc = EditorDocument::from_content("{\\k50}Hello {\\k30}World").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Add 20 centiseconds to all timings
        doc.karaoke().in_range(range).adjust().offset(20).unwrap();

        let text = doc.text();
        assert!(text.contains("\\k70")); // 50 + 20
        assert!(text.contains("\\k50")); // 30 + 20
    }

    #[test]
    fn karaoke_adjust_set_all() {
        let mut doc = EditorDocument::from_content("{\\k50}Hello {\\k30}World").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Set all timings to 45 centiseconds
        doc.karaoke().in_range(range).adjust().set_all(45).unwrap();

        let text = doc.text();
        assert!(text.contains("\\k45"));
        // Should contain exactly two instances of \\k45
        assert_eq!(text.matches("\\k45").count(), 2);
    }

    #[test]
    fn karaoke_adjust_custom() {
        let mut doc = EditorDocument::from_content("{\\k50}Hello {\\k30}World").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Apply custom timings: 80cs for first, 40cs for second
        doc.karaoke()
            .in_range(range)
            .adjust()
            .custom(vec![80, 40])
            .unwrap();

        let text = doc.text();
        assert!(text.contains("\\k80"));
        assert!(text.contains("\\k40"));
    }

    #[test]
    fn karaoke_apply_equal() {
        let mut doc = EditorDocument::from_content("Hello World Test").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Apply equal timing of 35cs with fill karaoke
        doc.karaoke()
            .in_range(range)
            .apply()
            .equal(35, KaraokeType::Fill)
            .unwrap();

        let text = doc.text();
        assert!(text.contains("\\kf35"));
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(text.contains("Test"));
    }

    #[test]
    fn karaoke_apply_beat() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Apply beat-based timing: 120 BPM, 0.5 beats per syllable
        // Expected duration: (60/120) * 0.5 * 100 = 25 centiseconds
        doc.karaoke()
            .in_range(range)
            .apply()
            .beat(120, 0.5, KaraokeType::Standard)
            .unwrap();

        let text = doc.text();
        assert!(text.contains("\\k25"));
    }

    #[test]
    fn karaoke_apply_pattern() {
        let mut doc = EditorDocument::from_content("Hello World Test").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Apply pattern-based timing: 40cs, 60cs, repeating
        doc.karaoke()
            .in_range(range)
            .apply()
            .pattern(vec![40, 60], KaraokeType::Outline)
            .unwrap();

        let text = doc.text();
        assert!(text.contains("\\ko40"));
        assert!(text.contains("\\ko60"));
    }

    #[test]
    fn karaoke_apply_import() {
        let mut doc = EditorDocument::from_content("Source text for import").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // Apply import timing (simplified test - would import from event 0)
        doc.karaoke()
            .in_range(range)
            .apply()
            .import_from(0)
            .unwrap();

        // Since import is simplified and returns original text, verify no crash
        assert!(doc.text().contains("Source text for import"));
    }

    #[test]
    fn karaoke_complex_workflow() {
        let mut doc =
            EditorDocument::from_content("Complex karaoke test with multiple words").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        // 1. Generate initial karaoke with standard timing and manual syllables
        doc.karaoke()
            .in_range(range)
            .generate(50)
            .karaoke_type(KaraokeType::Standard)
            .manual_syllables()
            .execute()
            .unwrap();

        let mut text = doc.text();
        assert!(text.contains("\\k50"));

        // 2. Scale the timing by 1.5
        let current_range = Range::new(Position::new(0), Position::new(doc.text().len()));
        doc.karaoke()
            .in_range(current_range)
            .adjust()
            .scale(1.5)
            .unwrap();

        text = doc.text();
        assert!(text.contains("\\k75")); // 50 * 1.5

        // 3. Add 10cs offset
        let final_range = Range::new(Position::new(0), Position::new(doc.text().len()));
        doc.karaoke()
            .in_range(final_range)
            .adjust()
            .offset(10)
            .unwrap();

        text = doc.text();
        assert!(text.contains("\\k85")); // 75 + 10

        // With manual syllables, the entire original text is preserved
        assert!(text.contains("Complex karaoke test with multiple words"));
    }

    #[test]
    fn karaoke_different_types_workflow() {
        // Test all karaoke types in sequence
        let test_text = "Test karaoke types";

        // Standard karaoke
        let mut doc1 = EditorDocument::from_content(test_text).unwrap();
        let range1 = Range::new(Position::new(0), Position::new(doc1.text().len()));
        doc1.karaoke()
            .in_range(range1)
            .generate(30)
            .karaoke_type(KaraokeType::Standard)
            .execute()
            .unwrap();
        assert!(doc1.text().contains("\\k30"));

        // Fill karaoke
        let mut doc2 = EditorDocument::from_content(test_text).unwrap();
        let range2 = Range::new(Position::new(0), Position::new(doc2.text().len()));
        doc2.karaoke()
            .in_range(range2)
            .generate(40)
            .karaoke_type(KaraokeType::Fill)
            .execute()
            .unwrap();
        assert!(doc2.text().contains("\\kf40"));

        // Outline karaoke
        let mut doc3 = EditorDocument::from_content(test_text).unwrap();
        let range3 = Range::new(Position::new(0), Position::new(doc3.text().len()));
        doc3.karaoke()
            .in_range(range3)
            .generate(50)
            .karaoke_type(KaraokeType::Outline)
            .execute()
            .unwrap();
        assert!(doc3.text().contains("\\ko50"));

        // Transition karaoke
        let mut doc4 = EditorDocument::from_content(test_text).unwrap();
        let range4 = Range::new(Position::new(0), Position::new(doc4.text().len()));
        doc4.karaoke()
            .in_range(range4)
            .generate(60)
            .karaoke_type(KaraokeType::Transition)
            .execute()
            .unwrap();
        assert!(doc4.text().contains("\\kt60"));
    }

    #[test]
    fn karaoke_error_conditions() {
        // Test with text containing override blocks (should fail)
        let mut doc = EditorDocument::from_content("Hello {\\b1}World{\\b0}").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));

        let result = doc.karaoke().in_range(range).generate(50).execute();

        // Should fail because text contains override blocks
        assert!(result.is_err());
    }

    #[test]
    fn karaoke_edge_cases() {
        // Test with empty text
        let mut doc = EditorDocument::from_content("").unwrap();
        let range = Range::new(Position::new(0), Position::new(0));

        let result = doc.karaoke().in_range(range).generate(50).execute();

        // Should handle empty text gracefully
        assert!(result.is_ok());

        // Test with single character
        let mut doc2 = EditorDocument::from_content("A").unwrap();
        let range2 = Range::new(Position::new(0), Position::new(1));

        doc2.karaoke()
            .in_range(range2)
            .generate(25)
            .execute()
            .unwrap();

        assert!(doc2.text().contains("\\k25"));
        assert!(doc2.text().contains("A"));
    }

    #[test]
    fn test_new_event_api_direct_access() {
        const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
Comment: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Third event
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test direct event access
        let event_info = doc.events().get(0).unwrap();
        assert!(event_info.is_some());
        let info = event_info.unwrap();
        assert_eq!(info.index, 0);
        assert_eq!(info.event.text, "First event");
        assert_eq!(info.event.event_type, EventType::Dialogue);

        // Test event count
        let count = doc.events().count().unwrap();
        assert_eq!(count, 3);

        // Test fluent accessor
        let text = doc.events().event(1).text().unwrap();
        assert_eq!(text, Some("Second event".to_string()));

        let style = doc.events().event(1).style().unwrap();
        assert_eq!(style, Some("Default".to_string()));

        let exists = doc.events().event(5).exists().unwrap();
        assert!(!exists);
    }

    #[test]
    fn test_new_event_api_filtering() {
        const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First dialogue
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second dialogue
Comment: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,First comment
Comment: 0,0:00:15.00,0:00:20.00,Default,Speaker,0,0,0,,Second comment
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test filtering by type
        let dialogues = doc.events().dialogues().execute().unwrap();
        assert_eq!(dialogues.len(), 2);
        assert!(dialogues
            .iter()
            .all(|info| info.event.event_type == EventType::Dialogue));

        let comments = doc.events().comments().execute().unwrap();
        assert_eq!(comments.len(), 2);
        assert!(comments
            .iter()
            .all(|info| info.event.event_type == EventType::Comment));

        // Test text filtering
        let with_first = doc
            .events()
            .query()
            .filter_by_text("First")
            .execute()
            .unwrap();
        assert_eq!(with_first.len(), 2);
        assert!(with_first[0].event.text.contains("First"));
        assert!(with_first[1].event.text.contains("First"));

        // Test case insensitive filtering
        let with_first_insensitive = doc
            .events()
            .query()
            .filter_by_text("first")
            .case_sensitive(false)
            .execute()
            .unwrap();
        assert_eq!(with_first_insensitive.len(), 2);
    }

    #[test]
    fn test_new_event_api_sorting() {
        const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Third by time
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First by time
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second by time
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test sorting by time (should reorder events)
        let by_time = doc.events().by_time().execute().unwrap();
        assert_eq!(by_time.len(), 3);
        assert_eq!(by_time[0].event.text, "First by time");
        assert_eq!(by_time[1].event.text, "Second by time");
        assert_eq!(by_time[2].event.text, "Third by time");

        // Test original order
        let in_order = doc.events().in_order().execute().unwrap();
        assert_eq!(in_order.len(), 3);
        assert_eq!(in_order[0].event.text, "Third by time");
        assert_eq!(in_order[1].event.text, "First by time");
        assert_eq!(in_order[2].event.text, "Second by time");

        // Test descending sort
        let by_time_desc = doc
            .events()
            .query()
            .sort_by_time()
            .descending()
            .execute()
            .unwrap();
        assert_eq!(by_time_desc[0].event.text, "Third by time");
        assert_eq!(by_time_desc[1].event.text, "Second by time");
        assert_eq!(by_time_desc[2].event.text, "First by time");
    }

    #[test]
    fn test_new_event_api_combined_operations() {
        const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Important dialogue
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Another dialogue
Comment: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Important comment
Dialogue: 0,0:00:15.00,0:00:20.00,Default,Speaker,0,0,0,,Final dialogue
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test combined filtering and sorting with limit
        let important_dialogues = doc
            .events()
            .query()
            .filter_by_type(EventType::Dialogue)
            .filter_by_text("Important")
            .sort_by_time()
            .limit(1)
            .execute()
            .unwrap();

        assert_eq!(important_dialogues.len(), 1);
        assert_eq!(important_dialogues[0].event.text, "Important dialogue");
        assert_eq!(important_dialogues[0].event.event_type, EventType::Dialogue);

        // Test getting indices only
        let dialogue_indices = doc.events().dialogues().sort_by_time().indices().unwrap();

        assert_eq!(dialogue_indices.len(), 3);
        // Should be indices in time order: 1, 0, 3 (based on start times)
        assert_eq!(dialogue_indices, vec![1, 0, 3]);

        // Test count
        let dialogue_count = doc.events().dialogues().count().unwrap();
        assert_eq!(dialogue_count, 3);

        // Test first
        let first_dialogue = doc.events().dialogues().sort_by_time().first().unwrap();

        assert!(first_dialogue.is_some());
        let first = first_dialogue.unwrap();
        assert_eq!(first.event.text, "Another dialogue");
    }

    #[test]
    fn karaoke_chaining_operations() {
        let mut doc = EditorDocument::from_content("Chain test").unwrap();

        // Test that karaoke operations can be chained with other fluent operations
        doc.at_pos(Position::new(0))
            .insert_text("Prefix: ")
            .unwrap();

        assert_eq!(doc.text(), "Prefix: Chain test");

        // Now apply karaoke to the appended part with manual syllables
        let karaoke_range = Range::new(Position::new(8), Position::new(doc.text().len()));
        doc.karaoke()
            .in_range(karaoke_range)
            .generate(45)
            .manual_syllables()
            .execute()
            .unwrap();

        let text = doc.text();
        assert!(text.starts_with("Prefix: "));
        assert!(text.contains("\\k45"));
        // With manual syllables, the original appended text is preserved
        assert!(text.contains("Chain test"));
    }

    #[test]
    fn test_event_delete_single() {
        const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
Dialogue: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Third event
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Delete the second event (index 1)
        doc.events().delete(1).unwrap();

        let events = doc.events().all().unwrap();
        assert_eq!(events.len(), 2);
        assert!(doc.text().contains("First event"));
        assert!(!doc.text().contains("Second event"));
        assert!(doc.text().contains("Third event"));
    }

    #[test]
    fn test_event_delete_multiple() {
        const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Event 1
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Event 2
Dialogue: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Event 3
Dialogue: 0,0:00:15.00,0:00:20.00,Default,Speaker,0,0,0,,Event 4
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Delete events at indices 0 and 2
        doc.events().delete_multiple(vec![0, 2]).unwrap();

        let events = doc.events().all().unwrap();
        assert_eq!(events.len(), 2);
        assert!(!doc.text().contains("Event 1"));
        assert!(doc.text().contains("Event 2"));
        assert!(!doc.text().contains("Event 3"));
        assert!(doc.text().contains("Event 4"));
    }

    #[test]
    fn test_event_delete_query() {
        const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Keep this
Comment: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Delete this comment
Dialogue: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Keep this too
Comment: 0,0:00:15.00,0:00:20.00,Default,Speaker,0,0,0,,Delete this comment too
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Delete all comment events
        doc.events().delete_query().comments().unwrap();

        let events = doc.events().all().unwrap();
        assert_eq!(events.len(), 2);
        assert!(events
            .iter()
            .all(|e| e.event.event_type == EventType::Dialogue));
        assert!(!doc.text().contains("Delete this comment"));
    }

    #[test]
    fn test_script_info_operations() {
        const TEST_CONTENT: &str = r#"[Script Info]
Title: Original Title
ScriptType: v4.00+

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test getting existing property
        let title = doc.info().get_title().unwrap();
        assert_eq!(title, Some("Original Title".to_string()));

        // Test setting property
        doc.info().title("New Title").unwrap();
        let new_title = doc.info().get_title().unwrap();
        assert_eq!(new_title, Some("New Title".to_string()));

        // Test adding new property
        doc.info().author("John Doe").unwrap();
        let author = doc.info().get_author().unwrap();
        assert_eq!(author, Some("John Doe".to_string()));

        // Test resolution
        doc.info().resolution(1920, 1080).unwrap();
        let res = doc.info().get_resolution().unwrap();
        assert_eq!(res, Some((1920, 1080)));

        // Test getting all properties
        let all = doc.info().all().unwrap();
        assert!(all.contains(&("Title".to_string(), "New Title".to_string())));
        assert!(all.contains(&("Original Script".to_string(), "John Doe".to_string())));
        assert!(all.contains(&("PlayResX".to_string(), "1920".to_string())));
        assert!(all.contains(&("PlayResY".to_string(), "1080".to_string())));

        // Test deleting property
        doc.info().delete("ScriptType").unwrap();
        let script_type = doc.info().get("ScriptType").unwrap();
        assert_eq!(script_type, None);
    }

    #[test]
    fn test_script_info_special_properties() {
        const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test wrap style
        doc.info().wrap_style(2).unwrap();
        let wrap = doc.info().get_wrap_style().unwrap();
        assert_eq!(wrap, Some(2));

        // Test scaled border and shadow
        doc.info().scaled_border_and_shadow(true).unwrap();
        let scaled = doc.info().get_scaled_border_and_shadow().unwrap();
        assert_eq!(scaled, Some(true));

        // Test with "no" value
        doc.info().scaled_border_and_shadow(false).unwrap();
        let not_scaled = doc.info().get_scaled_border_and_shadow().unwrap();
        assert_eq!(not_scaled, Some(false));
    }

    #[test]
    fn test_fonts_operations() {
        const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test adding a font
        let font_data = vec![
            "begin 644 custom.ttf".to_string(),
            "M1234567890".to_string(),
            "end".to_string(),
        ];
        doc.fonts().add("custom.ttf", font_data.clone()).unwrap();

        // Verify font was added
        let fonts = doc.fonts().list().unwrap();
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0], "custom.ttf");
        assert!(doc.fonts().exists("custom.ttf").unwrap());

        // Test adding another font
        doc.fonts().add("another.otf", font_data).unwrap();
        assert_eq!(doc.fonts().count().unwrap(), 2);

        // Test removing a font
        doc.fonts().remove("custom.ttf").unwrap();
        assert_eq!(doc.fonts().count().unwrap(), 1);
        assert!(!doc.fonts().exists("custom.ttf").unwrap());
        assert!(doc.fonts().exists("another.otf").unwrap());

        // Test clearing all fonts
        doc.fonts().clear().unwrap();
        assert_eq!(doc.fonts().count().unwrap(), 0);
        assert!(doc.fonts().list().unwrap().is_empty());
    }

    #[test]
    fn test_fonts_binary_add() {
        let mut doc = EditorDocument::new();

        // Test adding font from binary data
        let binary_data = b"Hello Font Data!";
        doc.fonts().add_binary("test.ttf", binary_data).unwrap();

        // Verify font was added
        assert!(doc.fonts().exists("test.ttf").unwrap());
        assert!(doc.text().contains("[Fonts]"));
        assert!(doc.text().contains("fontname: test.ttf"));
        assert!(doc.text().contains("begin 644 test.ttf"));
        assert!(doc.text().contains("end"));
    }

    #[test]
    fn test_graphics_operations() {
        const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Test adding a graphic
        let graphic_data = vec![
            "begin 644 logo.png".to_string(),
            "M89PNG1234".to_string(),
            "end".to_string(),
        ];
        doc.graphics()
            .add("logo.png", graphic_data.clone())
            .unwrap();

        // Verify graphic was added
        let graphics = doc.graphics().list().unwrap();
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0], "logo.png");
        assert!(doc.graphics().exists("logo.png").unwrap());

        // Test adding another graphic
        doc.graphics().add("banner.jpg", graphic_data).unwrap();
        assert_eq!(doc.graphics().count().unwrap(), 2);

        // Test removing a graphic
        doc.graphics().remove("logo.png").unwrap();
        assert_eq!(doc.graphics().count().unwrap(), 1);
        assert!(!doc.graphics().exists("logo.png").unwrap());
        assert!(doc.graphics().exists("banner.jpg").unwrap());

        // Test clearing all graphics
        doc.graphics().clear().unwrap();
        assert_eq!(doc.graphics().count().unwrap(), 0);
        assert!(doc.graphics().list().unwrap().is_empty());
    }

    #[test]
    fn test_graphics_binary_add() {
        let mut doc = EditorDocument::new();

        // Test adding graphic from binary data
        let binary_data = b"PNG Image Data Here!";
        doc.graphics().add_binary("image.png", binary_data).unwrap();

        // Verify graphic was added
        assert!(doc.graphics().exists("image.png").unwrap());
        assert!(doc.text().contains("[Graphics]"));
        assert!(doc.text().contains("filename: image.png"));
        assert!(doc.text().contains("begin 644 image.png"));
        assert!(doc.text().contains("end"));
    }
}
