//! Main document type for the editor
//!
//! Provides the `EditorDocument` struct which manages ASS script content
//! with direct access to parsed ASS structures and efficient text editing.

use super::errors::{EditorError, Result};
use super::history::UndoManager;
use super::position::{LineColumn, Position, Range};
use crate::commands::CommandResult;
use ass_core::parser::{ast::Section, script::ScriptDeltaOwned, Script};
use core::ops::Range as StdRange;

#[cfg(feature = "std")]
use std::sync::Arc;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};

/// Main document container for editing ASS scripts
///
/// Manages both text content and parsed ASS structures with direct access to
/// events, styles, and script info without manual parsing.
#[derive(Debug)]
pub struct EditorDocument {
    /// Rope for efficient text editing operations
    #[cfg(feature = "rope")]
    text_rope: ropey::Rope,

    /// Raw text content (fallback when rope feature is disabled)
    #[cfg(not(feature = "rope"))]
    text_content: String,

    /// Document identifier for session management
    id: String,

    /// Whether document has unsaved changes
    modified: bool,

    /// Optional file path if loaded from/saved to disk
    file_path: Option<String>,

    /// Extension registry reference (shared across session)
    #[cfg(feature = "plugins")]
    _registry: Option<Arc<ass_core::plugin::ExtensionRegistry>>,

    /// Undo/redo manager
    history: UndoManager,
}

impl EditorDocument {
    /// Create a new empty document
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "rope")]
            text_rope: ropey::Rope::new(),
            #[cfg(not(feature = "rope"))]
            text_content: String::new(),
            id: Self::generate_id(),
            modified: false,
            file_path: None,
            #[cfg(feature = "plugins")]
            _registry: None,
            history: UndoManager::new(),
        }
    }

    /// Create document from file path
    #[cfg(feature = "std")]
    pub fn from_file(path: &str) -> Result<Self> {
        use std::fs;
        let content = fs::read_to_string(path).map_err(|e| EditorError::IoError(e.to_string()))?;
        let mut doc = Self::from_content(&content)?;
        doc.file_path = Some(path.to_string());
        Ok(doc)
    }

    /// Save document to file
    #[cfg(feature = "std")]
    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = self.file_path.clone() {
            self.save_to_file(&path)
        } else {
            Err(EditorError::IoError(
                "No file path set for document".to_string(),
            ))
        }
    }

    /// Save document to specific file path
    #[cfg(feature = "std")]
    pub fn save_to_file(&mut self, path: &str) -> Result<()> {
        use std::fs;
        let content = self.text();
        fs::write(path, content).map_err(|e| EditorError::IoError(e.to_string()))?;
        self.modified = false;
        self.file_path = Some(path.to_string());
        Ok(())
    }

    /// Create document with specific ID
    pub fn with_id(id: String) -> Self {
        let mut doc = Self::new();
        doc.id = id;
        doc
    }

    /// Load document from string content
    pub fn from_content(content: &str) -> Result<Self> {
        // Validate that content can be parsed
        let _ = Script::parse(content).map_err(EditorError::from)?;

        Ok(Self {
            #[cfg(feature = "rope")]
            text_rope: ropey::Rope::from_str(content),
            #[cfg(not(feature = "rope"))]
            text_content: content.to_string(),
            id: Self::generate_id(),
            modified: false,
            file_path: None,
            #[cfg(feature = "plugins")]
            _registry: None,
            history: UndoManager::new(),
        })
    }

    /// Parse the current document content into a Script
    ///
    /// Returns a boxed closure that provides the parsed Script.
    /// This avoids lifetime issues by ensuring the content outlives the Script.
    pub fn parse_script_with<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Script) -> R,
    {
        let content = self.text();
        match Script::parse(&content) {
            Ok(script) => Ok(f(&script)),
            Err(e) => Err(EditorError::from(e)),
        }
    }

    /// Validate the document content can be parsed as valid ASS
    pub fn validate(&self) -> Result<()> {
        let content = self.text();
        Script::parse(&content).map_err(EditorError::from)?;
        Ok(())
    }

    /// Get document ID
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get file path if document is associated with a file
    #[must_use]
    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    /// Set file path for the document
    pub fn set_file_path(&mut self, path: Option<String>) {
        self.file_path = path;
    }

    /// Check if document has unsaved changes
    #[must_use]
    pub const fn is_modified(&self) -> bool {
        self.modified
    }

    /// Mark document as modified
    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    /// Get total length in bytes
    #[must_use]
    pub fn len_bytes(&self) -> usize {
        #[cfg(feature = "rope")]
        {
            self.text_rope.len_bytes()
        }
        #[cfg(not(feature = "rope"))]
        {
            self.text_content.len()
        }
    }

    /// Get total number of lines
    #[must_use]
    pub fn len_lines(&self) -> usize {
        #[cfg(feature = "rope")]
        {
            self.text_rope.len_lines()
        }
        #[cfg(not(feature = "rope"))]
        {
            self.text_content.lines().count().max(1)
        }
    }

    /// Check if document is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len_bytes() == 0
    }

    /// Get text content as string
    #[must_use]
    pub fn text(&self) -> String {
        #[cfg(feature = "rope")]
        {
            self.text_rope.to_string()
        }
        #[cfg(not(feature = "rope"))]
        {
            self.text_content.clone()
        }
    }

    /// Get text content for a range
    pub fn text_range(&self, range: Range) -> Result<String> {
        let start = range.start.offset;
        let end = range.end.offset;

        if end > self.len_bytes() {
            return Err(EditorError::InvalidRange {
                start,
                end,
                length: self.len_bytes(),
            });
        }

        #[cfg(feature = "rope")]
        {
            Ok(self.text_rope.slice(start..end).to_string())
        }
        #[cfg(not(feature = "rope"))]
        {
            Ok(self.text_content[start..end].to_string())
        }
    }

    /// Convert byte position to line/column
    #[cfg(feature = "rope")]
    pub fn position_to_line_column(&self, pos: Position) -> Result<LineColumn> {
        if pos.offset > self.len_bytes() {
            return Err(EditorError::PositionOutOfBounds {
                position: pos.offset,
                length: self.len_bytes(),
            });
        }

        let line_idx = self.text_rope.byte_to_line(pos.offset);
        let line_start = self.text_rope.line_to_byte(line_idx);
        let col_offset = pos.offset - line_start;

        // Convert byte offset to character offset within line
        let line = self.text_rope.line(line_idx);
        let mut char_col = 0;
        let mut byte_count = 0;

        for ch in line.chars() {
            if byte_count >= col_offset {
                break;
            }
            byte_count += ch.len_utf8();
            char_col += 1;
        }

        // Convert to 1-indexed
        LineColumn::new(line_idx + 1, char_col + 1)
    }

    /// Convert byte position to line/column (without rope)
    #[cfg(not(feature = "rope"))]
    pub fn position_to_line_column(&self, pos: Position) -> Result<LineColumn> {
        if pos.offset > self.len_bytes() {
            return Err(EditorError::PositionOutOfBounds {
                position: pos.offset,
                length: self.len_bytes(),
            });
        }

        let mut line = 1;
        let mut col = 1;
        let mut byte_pos = 0;

        for ch in self.text_content.chars() {
            if byte_pos >= pos.offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }

            byte_pos += ch.len_utf8();
        }

        LineColumn::new(line, col)
    }

    /// Insert text at position (low-level operation without undo)
    pub(crate) fn insert_raw(&mut self, pos: Position, text: &str) -> Result<()> {
        if pos.offset > self.len_bytes() {
            return Err(EditorError::PositionOutOfBounds {
                position: pos.offset,
                length: self.len_bytes(),
            });
        }

        #[cfg(feature = "rope")]
        {
            self.text_rope.insert(pos.offset, text);
        }
        #[cfg(not(feature = "rope"))]
        {
            self.text_content.insert_str(pos.offset, text);
        }

        self.modified = true;
        Ok(())
    }

    /// Insert text at position with undo support
    pub fn insert(&mut self, pos: Position, text: &str) -> Result<()> {
        use crate::commands::{EditorCommand, InsertTextCommand};
        use crate::core::history::Operation;

        let command = InsertTextCommand::new(pos, text.to_string());
        let result = command.execute(self)?;

        // Record the operation in history
        let operation = Operation::Insert {
            position: pos,
            text: text.to_string(),
        };
        self.history
            .record_operation(operation, command.description().to_string(), &result);

        Ok(())
    }

    /// Delete text in range with undo support
    pub fn delete(&mut self, range: Range) -> Result<()> {
        use crate::commands::{DeleteTextCommand, EditorCommand};
        use crate::core::history::Operation;

        // Capture the text that will be deleted BEFORE deletion
        let deleted_text = self.text_range(range)?;

        let command = DeleteTextCommand::new(range);
        let result = command.execute(self)?;

        // Record the operation in history
        let operation = Operation::Delete {
            range,
            deleted_text,
        };
        self.history
            .record_operation(operation, command.description().to_string(), &result);

        Ok(())
    }

    /// Replace text in range with undo support
    pub fn replace(&mut self, range: Range, text: &str) -> Result<()> {
        use crate::commands::{EditorCommand, ReplaceTextCommand};
        use crate::core::history::Operation;

        // Capture the old text BEFORE replacement
        let old_text = self.text_range(range)?;

        let command = ReplaceTextCommand::new(range, text.to_string());
        let result = command.execute(self)?;

        // Record the operation in history
        let operation = Operation::Replace {
            range,
            old_text,
            new_text: text.to_string(),
        };
        self.history
            .record_operation(operation, command.description().to_string(), &result);

        Ok(())
    }

    /// Generate unique document ID
    fn generate_id() -> String {
        // Simple ID generation - in production might use UUID
        #[cfg(feature = "std")]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            format!("doc_{timestamp}")
        }
        #[cfg(not(feature = "std"))]
        {
            static mut COUNTER: u32 = 0;
            unsafe {
                COUNTER += 1;
                format!("doc_{COUNTER}")
            }
        }
    }

    // === ASS-Aware APIs ===

    /// Get events directly without manual parsing  
    pub fn events(&self) -> Result<usize> {
        self.parse_script_with(|script| {
            let mut count = 0;
            for section in script.sections() {
                if let Section::Events(events) = section {
                    count += events.len();
                }
            }
            count
        })
    }

    /// Get number of styles without manual parsing
    pub fn styles_count(&self) -> Result<usize> {
        self.parse_script_with(|script| {
            let mut count = 0;
            for section in script.sections() {
                if let Section::Styles(styles) = section {
                    count += styles.len();
                }
            }
            count
        })
    }

    /// Get script info field names
    pub fn script_info_fields(&self) -> Result<Vec<String>> {
        self.parse_script_with(|script| {
            let mut fields = Vec::new();
            for section in script.sections() {
                if let Section::ScriptInfo(info) = section {
                    for (key, _) in &info.fields {
                        fields.push(key.to_string());
                    }
                }
            }
            fields
        })
    }

    /// Get number of sections
    pub fn sections_count(&self) -> Result<usize> {
        self.parse_script_with(|script| script.sections().len())
    }

    /// Check if document has events section
    pub fn has_events(&self) -> Result<bool> {
        self.parse_script_with(|script| {
            script
                .sections()
                .iter()
                .any(|section| matches!(section, Section::Events(_)))
        })
    }

    /// Check if document has styles section
    pub fn has_styles(&self) -> Result<bool> {
        self.parse_script_with(|script| {
            script
                .sections()
                .iter()
                .any(|section| matches!(section, Section::Styles(_)))
        })
    }

    /// Get event text by line pattern (simplified search)
    pub fn find_event_text(&self, pattern: &str) -> Result<Vec<String>> {
        self.parse_script_with(|script| {
            let mut matches = Vec::new();
            for section in script.sections() {
                if let Section::Events(events) = section {
                    for event in events {
                        if event.text.contains(pattern) {
                            matches.push(event.text.to_string());
                        }
                    }
                }
            }
            matches
        })
    }

    /// Edit event by index with full field support
    ///
    /// Allows structured editing of specific event fields by index.
    /// Returns the modified event line for undo support.
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based index of the event to edit
    /// * `update_fn` - Function that receives the current event and returns modifications
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ass_editor::core::EditorDocument;
    /// # let content = r#"[Script Info]
    /// # Title: Test
    /// #
    /// # [Events]
    /// # Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
    /// # Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Original text"#;
    /// # let mut doc = EditorDocument::from_content(content).unwrap();
    /// doc.edit_event_by_index(0, |event| {
    ///     vec![
    ///         ("text", "New dialogue text".to_string()),
    ///         ("style", "NewStyle".to_string()),
    ///     ]
    /// })?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn edit_event_by_index<F>(&mut self, index: usize, update_fn: F) -> Result<String>
    where
        F: for<'a> FnOnce(&ass_core::parser::ast::Event<'a>) -> Vec<(&'static str, String)>,
    {
        let content = self.text();
        let mut event_info = None;
        let mut event_count = 0;

        // Find the event and its location in the document
        self.parse_script_with(|script| -> Result<()> {
            for section in script.sections() {
                if let Section::Events(events) = section {
                    for event in events {
                        if event_count == index {
                            // Get the modifications from the update function
                            let modifications = update_fn(event);

                            // Build a pattern to search for this specific event
                            let event_type_str = event.event_type.as_str();
                            let pattern = format!(
                                "{}: {},{},{}",
                                event_type_str, event.layer, event.start, event.end
                            );

                            let event_line = if let Some(pos) = content.find(&pattern) {
                                let line_end = content[pos..]
                                    .find('\n')
                                    .map(|n| pos + n)
                                    .unwrap_or(content.len());
                                let line = content[pos..line_end].to_string();
                                (pos, line_end, line)
                            } else {
                                return Err(EditorError::ValidationError {
                                    message: "Could not find event line in document".to_string(),
                                });
                            };

                            // Store the event data we need instead of cloning
                            let event_data = (
                                event.event_type,
                                event.layer.to_string(),
                                event.start.to_string(),
                                event.end.to_string(),
                                event.style.to_string(),
                                event.name.to_string(),
                                event.margin_l.to_string(),
                                event.margin_r.to_string(),
                                event.margin_v.to_string(),
                                event.effect.to_string(),
                                event.text.to_string(),
                            );
                            event_info = Some((event_data, event_line, modifications));
                            return Ok(());
                        }
                        event_count += 1;
                    }
                }
            }
            Ok(())
        })??;

        if let Some((event_data, (line_start, line_end, original_line), modifications)) = event_info
        {
            // Build the new event line with modifications
            let new_line = self.build_modified_event_line_from_data(
                event_data,
                &original_line,
                modifications,
            )?;

            // Replace the line in the document
            let range = Range::new(Position::new(line_start), Position::new(line_end));
            self.replace(range, &new_line)?;

            Ok(new_line)
        } else {
            Err(EditorError::InvalidRange {
                start: index,
                end: index + 1,
                length: event_count,
            })
        }
    }

    /// Helper to build a modified event line from event data
    fn build_modified_event_line_from_data(
        &self,
        event_data: (
            ass_core::parser::ast::EventType,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
        ),
        _original_line: &str,
        modifications: Vec<(&'static str, String)>,
    ) -> Result<String> {
        let (
            event_type,
            layer,
            start,
            end,
            style,
            name,
            margin_l,
            margin_r,
            margin_v,
            effect,
            text,
        ) = event_data;

        // Apply modifications
        let mut layer = layer;
        let mut start = start;
        let mut end = end;
        let mut style = style;
        let mut name = name;
        let mut margin_l = margin_l;
        let mut margin_r = margin_r;
        let mut margin_v = margin_v;
        let mut effect = effect;
        let mut text = text;

        for (field, value) in modifications {
            match field {
                "layer" => layer = value,
                "start" => start = value,
                "end" => end = value,
                "style" => style = value,
                "name" => name = value,
                "margin_l" => margin_l = value,
                "margin_r" => margin_r = value,
                "margin_v" => margin_v = value,
                "effect" => effect = value,
                "text" => text = value,
                _ => {
                    return Err(EditorError::ValidationError {
                        message: format!("Unknown event field: {field}"),
                    });
                }
            }
        }

        // Rebuild the line
        let event_type_str = event_type.as_str();
        Ok(format!("{event_type_str}: {layer},{start},{end},{style},{name},{margin_l},{margin_r},{margin_v},{effect},{text}"))
    }

    /// Add event line to document
    pub fn add_event_line(&mut self, event_line: &str) -> Result<()> {
        let content = self.text();
        if let Some(events_pos) = content.find("[Events]") {
            // Find end of format line and add after it
            let format_start = content[events_pos..].find("Format:").unwrap_or(0) + events_pos;
            let line_end = content[format_start..].find('\n').unwrap_or(0) + format_start + 1;

            let insert_pos = Position::new(line_end);
            self.insert(insert_pos, &format!("{event_line}\n"))
        } else {
            // Add Events section if it doesn't exist
            let content_len = self.len_bytes();
            let events_section = format!("\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n{event_line}\n");
            self.insert(Position::new(content_len), &events_section)
        }
    }

    /// Edit style line
    pub fn edit_style_line(&mut self, style_name: &str, new_style_line: &str) -> Result<()> {
        let content = self.text();
        let pattern = format!("Style: {style_name},");

        if let Some(pos) = content.find(&pattern) {
            // Find end of line
            let line_end = content[pos..].find('\n').map_or(content.len(), |n| pos + n);
            let range = Range::new(Position::new(pos), Position::new(line_end));
            self.replace(range, new_style_line)
        } else {
            // Add style if it doesn't exist
            self.add_style_line(new_style_line)
        }
    }

    /// Add style line to document
    pub fn add_style_line(&mut self, style_line: &str) -> Result<()> {
        let content = self.text();
        if let Some(styles_pos) = content
            .find("[V4+ Styles]")
            .or_else(|| content.find("[V4 Styles]"))
        {
            // Find end of format line and add after it
            let format_start = content[styles_pos..].find("Format:").unwrap_or(0) + styles_pos;
            let line_end = content[format_start..].find('\n').unwrap_or(0) + format_start + 1;

            let insert_pos = Position::new(line_end);
            self.insert(insert_pos, &format!("{style_line}\n"))
        } else {
            // Add Styles section if it doesn't exist
            let script_info_end = content.find("\n[Events]").unwrap_or(content.len());
            let styles_section = format!("\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n{style_line}\n");
            self.insert(Position::new(script_info_end), &styles_section)
        }
    }

    // === INCREMENTAL PARSING WITH CORE INTEGRATION ===

    /// Perform incremental edit using core's parse_partial for optimal performance
    pub fn edit_incremental(&mut self, range: Range, new_text: &str) -> Result<ScriptDeltaOwned> {
        // Convert our Range to std::ops::Range for core
        let std_range = StdRange {
            start: range.start.offset,
            end: range.end.offset,
        };

        // Get delta using core's incremental parsing
        let delta =
            self.parse_script_with(|script| script.parse_partial(std_range, new_text))??;

        // Apply the text change to our rope/content
        self.replace(range, new_text)?;

        Ok(delta)
    }

    /// Insert text with incremental parsing (< 1ms target)
    pub fn insert_incremental(&mut self, pos: Position, text: &str) -> Result<ScriptDeltaOwned> {
        let range = Range::new(pos, pos); // Zero-length range for insertion
        self.edit_incremental(range, text)
    }

    /// Delete text with incremental parsing
    pub fn delete_incremental(&mut self, range: Range) -> Result<ScriptDeltaOwned> {
        self.edit_incremental(range, "")
    }

    /// Edit event using incremental parsing for performance
    pub fn edit_event_incremental(
        &mut self,
        event_text: &str,
        new_text: &str,
    ) -> Result<ScriptDeltaOwned> {
        let content = self.text();
        if let Some(pos) = content.find(event_text) {
            let range = Range::new(Position::new(pos), Position::new(pos + event_text.len()));
            self.edit_incremental(range, new_text)
        } else {
            Err(EditorError::ValidationError {
                message: format!("Event text not found: {event_text}"),
            })
        }
    }

    /// Parse with delta tracking for command system integration
    pub fn parse_with_delta_tracking<F, R>(
        &self,
        range: Option<StdRange<usize>>,
        new_text: Option<&str>,
        f: F,
    ) -> Result<R>
    where
        F: FnOnce(&Script, Option<&ScriptDeltaOwned>) -> R,
    {
        let content = self.text();
        let script = Script::parse(&content).map_err(EditorError::from)?;

        if let (Some(range), Some(text)) = (range, new_text) {
            // Get delta for the change
            match script.parse_partial(range, text) {
                Ok(delta) => Ok(f(&script, Some(&delta))),
                Err(_) => {
                    // Fallback to full re-parse if incremental fails
                    Ok(f(&script, None))
                }
            }
        } else {
            Ok(f(&script, None))
        }
    }

    /// Edit event using a builder for structured modifications
    ///
    /// Allows editing events using the EventBuilder fluent API. The builder
    /// is pre-populated with the current event's values, allowing selective
    /// field updates.
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based index of the event to edit  
    /// * `builder_fn` - Function that receives a pre-populated EventBuilder
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ass_editor::core::EditorDocument;
    /// # let content = r#"[Script Info]
    /// # Title: Test
    /// #
    /// # [Events]
    /// # Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
    /// # Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Original text"#;
    /// # let mut doc = EditorDocument::from_content(content).unwrap();
    /// # use ass_editor::core::builders::EventBuilder;
    /// doc.edit_event_with_builder(0, |builder| {
    ///     builder
    ///         .text("New dialogue text")
    ///         .style("NewStyle")
    ///         .end_time("0:00:10.00")
    /// })?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn edit_event_with_builder<F>(&mut self, index: usize, builder_fn: F) -> Result<String>
    where
        F: for<'a> FnOnce(
            crate::core::builders::EventBuilder,
        ) -> crate::core::builders::EventBuilder,
    {
        use crate::core::builders::EventBuilder;

        let content = self.text();
        let mut event_info = None;
        let mut event_count = 0;
        let mut format_line = None;

        // Find the event and extract format line
        self.parse_script_with(|script| -> Result<()> {
            for section in script.sections() {
                if let Section::Events(events) = section {
                    // Get format line if available
                    if format_line.is_none() {
                        // Find Events section header and format line in raw text
                        if let Some(events_pos) = content.find("[Events]") {
                            let after_header = &content[events_pos + 8..];
                            if let Some(format_pos) = after_header.find("Format:") {
                                let format_start = events_pos + 8 + format_pos + 7; // Skip "Format:"
                                if let Some(format_end) = content[format_start..].find('\n') {
                                    let format_str =
                                        content[format_start..format_start + format_end].trim();
                                    let fields: Vec<&str> =
                                        format_str.split(',').map(str::trim).collect();
                                    format_line = Some(fields);
                                }
                            }
                        }
                    }

                    for event in events {
                        if event_count == index {
                            // Create a builder pre-populated with current values
                            let mut builder = match event.event_type {
                                ass_core::parser::ast::EventType::Dialogue => {
                                    EventBuilder::dialogue()
                                }
                                ass_core::parser::ast::EventType::Comment => {
                                    EventBuilder::comment()
                                }
                                _ => EventBuilder::new(),
                            };

                            // Pre-populate builder with current event values
                            builder = builder
                                .layer(event.layer.parse::<u32>().unwrap_or(0))
                                .start_time(event.start)
                                .end_time(event.end)
                                .style(event.style)
                                .speaker(event.name)
                                .margin_left(event.margin_l.parse::<u32>().unwrap_or(0))
                                .margin_right(event.margin_r.parse::<u32>().unwrap_or(0))
                                .margin_vertical(event.margin_v.parse::<u32>().unwrap_or(0))
                                .effect(event.effect)
                                .text(event.text);

                            if let Some(margin_t) = event.margin_t {
                                builder = builder.margin_top(margin_t.parse::<u32>().unwrap_or(0));
                            }
                            if let Some(margin_b) = event.margin_b {
                                builder =
                                    builder.margin_bottom(margin_b.parse::<u32>().unwrap_or(0));
                            }

                            // Apply user modifications
                            let modified_builder = builder_fn(builder);

                            // Build the new event line
                            let new_line = if let Some(ref format_fields) = format_line {
                                modified_builder.build_with_format(format_fields)?
                            } else {
                                modified_builder.build()?
                            };

                            // Find the event line in the raw text
                            let event_type_str = event.event_type.as_str();
                            let pattern = format!(
                                "{}: {},{},{}",
                                event_type_str, event.layer, event.start, event.end
                            );

                            let event_line = if let Some(pos) = content.find(&pattern) {
                                let line_end = content[pos..]
                                    .find('\n')
                                    .map(|n| pos + n)
                                    .unwrap_or(content.len());
                                (pos, line_end)
                            } else {
                                return Err(EditorError::ValidationError {
                                    message: "Could not find event line in document".to_string(),
                                });
                            };

                            event_info = Some((event_line, new_line));
                            return Ok(());
                        }
                        event_count += 1;
                    }
                }
            }
            Ok(())
        })??;

        if let Some(((line_start, line_end), new_line)) = event_info {
            // Replace the line in the document
            let range = Range::new(Position::new(line_start), Position::new(line_end));
            self.replace(range, &new_line)?;

            Ok(new_line)
        } else {
            Err(EditorError::InvalidRange {
                start: index,
                end: index + 1,
                length: event_count,
            })
        }
    }

    /// Edit an event by finding and replacing text (simplified ASS-aware editing)
    pub fn edit_event_text(&mut self, old_text: &str, new_text: &str) -> Result<()> {
        let content = self.text();

        if let Some(pos) = content.find(old_text) {
            let range = Range::new(Position::new(pos), Position::new(pos + old_text.len()));
            self.replace(range, new_text)?;
        }

        Ok(())
    }

    /// Get script info field value by key
    pub fn get_script_info_field(&self, key: &str) -> Result<Option<String>> {
        self.parse_script_with(|script| {
            script.sections().iter().find_map(|section| {
                if let Section::ScriptInfo(info) = section {
                    info.fields
                        .iter()
                        .find(|(k, _)| *k == key)
                        .map(|(_, v)| v.to_string())
                } else {
                    None
                }
            })
        })
    }

    /// Set script info field (ASS-aware editing)
    pub fn set_script_info_field(&mut self, key: &str, value: &str) -> Result<()> {
        // Simplified implementation - find and replace the field
        let content = self.text();
        let field_pattern = format!("{key}:");

        if let Some(pos) = content.find(&field_pattern) {
            // Find end of line
            let line_start = pos;
            let line_end = content[pos..].find('\n').map_or(content.len(), |n| pos + n);

            let range = Range::new(Position::new(line_start), Position::new(line_end));

            let new_line = format!("{key}: {value}");
            self.replace(range, &new_line)?;
        }

        Ok(())
    }

    /// Perform an undo operation
    ///
    /// Retrieves the most recent operation from the undo stack and reverses it.
    /// If the operation includes a script delta, it will be applied for efficient updates.
    pub fn undo(&mut self) -> Result<CommandResult> {
        use crate::core::history::Operation;

        // Pop from undo stack
        if let Some(entry) = self.history.pop_undo_entry() {
            let mut result = CommandResult::success();
            result.content_changed = true;

            // Execute the inverse of the operation
            match &entry.operation {
                Operation::Insert { position, text } => {
                    // Undo insert by deleting the inserted text
                    let end_pos = Position::new(position.offset + text.len());
                    let range = Range::new(*position, end_pos);
                    self.delete_raw(range)?;
                    result.modified_range = Some(Range::new(*position, *position));
                    result.new_cursor = Some(*position);
                }
                Operation::Delete {
                    range,
                    deleted_text,
                } => {
                    // Undo delete by inserting the deleted text
                    self.insert_raw(range.start, deleted_text)?;
                    let end_pos = Position::new(range.start.offset + deleted_text.len());
                    result.modified_range = Some(Range::new(range.start, end_pos));
                    result.new_cursor = Some(end_pos);
                }
                Operation::Replace {
                    range, old_text, ..
                } => {
                    // Undo replace by restoring old text
                    self.replace_raw(*range, old_text)?;
                    let end_pos = Position::new(range.start.offset + old_text.len());
                    result.modified_range = Some(Range::new(range.start, end_pos));
                    result.new_cursor = Some(end_pos);
                }
                #[cfg(feature = "stream")]
                Operation::ApplyDelta { delta: _ } => {
                    // For delta operations, we need to apply the inverse delta
                    // This is a complex operation that would require storing inverse deltas
                    return Err(EditorError::HistoryError {
                        message: "Undo of delta operations not yet implemented".to_string(),
                    });
                }
            }

            // Push to redo stack for future redo
            self.history.push_redo_entry(entry);

            // Apply script delta if available
            #[cfg(feature = "stream")]
            if let Some(delta) = result.script_delta.as_ref() {
                self.apply_script_delta(delta.clone())?;
            }

            result.message = Some("Undo successful".to_string());
            Ok(result)
        } else {
            Err(EditorError::NothingToUndo)
        }
    }

    /// Perform a redo operation
    ///
    /// Retrieves the most recent operation from the redo stack and re-executes it.
    /// If the operation includes a script delta, it will be applied for efficient updates.
    pub fn redo(&mut self) -> Result<CommandResult> {
        use crate::core::history::Operation;

        // Pop from redo stack
        if let Some(entry) = self.history.pop_redo_entry() {
            let mut result = CommandResult::success();
            result.content_changed = true;

            // Re-execute the original operation
            match &entry.operation {
                Operation::Insert { position, text } => {
                    // Redo insert
                    self.insert_raw(*position, text)?;
                    let end_pos = Position::new(position.offset + text.len());
                    result.modified_range = Some(Range::new(*position, end_pos));
                    result.new_cursor = Some(end_pos);
                }
                Operation::Delete { range, .. } => {
                    // Redo delete
                    self.delete_raw(*range)?;
                    result.modified_range = Some(Range::new(range.start, range.start));
                    result.new_cursor = Some(range.start);
                }
                Operation::Replace {
                    range, new_text, ..
                } => {
                    // Redo replace
                    self.replace_raw(*range, new_text)?;
                    let end_pos = Position::new(range.start.offset + new_text.len());
                    result.modified_range = Some(Range::new(range.start, end_pos));
                    result.new_cursor = Some(end_pos);
                }
                #[cfg(feature = "stream")]
                Operation::ApplyDelta { delta } => {
                    // Re-apply the delta
                    self.apply_script_delta(delta.clone())?;
                    result.message = Some("Delta re-applied".to_string());
                }
            }

            // Record in history without using the public methods (to avoid recursion)
            // We need to manually update the history manager's cursor
            if let Some(cursor) = result.new_cursor {
                self.history.set_cursor(Some(cursor));
            }

            // Create a new history entry for the redo operation
            let new_entry = crate::core::history::HistoryEntry::new(
                entry.operation,
                entry.description,
                &result,
                entry.cursor_before,
            );

            // Push back to undo stack
            self.history.stack_mut().push(new_entry);

            result.message = Some("Redo successful".to_string());
            Ok(result)
        } else {
            Err(EditorError::NothingToRedo)
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Get description of the next undo operation
    pub fn next_undo_description(&self) -> Option<&str> {
        self.history.next_undo_description()
    }

    /// Get description of the next redo operation
    pub fn next_redo_description(&self) -> Option<&str> {
        self.history.next_redo_description()
    }

    /// Get mutable reference to the undo manager for configuration
    pub fn undo_manager_mut(&mut self) -> &mut UndoManager {
        &mut self.history
    }

    /// Get reference to the undo manager
    pub fn undo_manager(&self) -> &UndoManager {
        &self.history
    }

    /// Apply a script delta for efficient incremental parsing
    #[cfg(feature = "stream")]
    fn apply_script_delta(&mut self, delta: ScriptDeltaOwned) -> Result<()> {
        // Parse the current script to get sections
        let current_content = self.text();
        let script = Script::parse(&current_content).map_err(EditorError::from)?;

        // Apply removals first (in reverse order to maintain indices)
        let mut removed_indices = delta.removed.clone();
        removed_indices.sort_by(|a, b| b.cmp(a)); // Sort descending

        for index in removed_indices {
            if index < script.sections().len() {
                // Find the section's text range and remove it
                let section = &script.sections()[index];
                let start_offset = self.find_section_start(section)?;
                let end_offset = self.find_section_end(section)?;

                self.delete_raw(Range::new(
                    Position::new(start_offset),
                    Position::new(end_offset),
                ))?;
            }
        }

        // Apply modifications
        for (index, new_section_text) in delta.modified {
            if index < script.sections().len() {
                // Find the section's text range
                let section = &script.sections()[index];
                let start_offset = self.find_section_start(section)?;
                let end_offset = self.find_section_end(section)?;

                // Replace with new section text
                self.replace_raw(
                    Range::new(Position::new(start_offset), Position::new(end_offset)),
                    &new_section_text,
                )?;
            }
        }

        // Apply additions
        for section_text in delta.added {
            // Add new sections at the end of the document
            let end_pos = Position::new(self.len_bytes());

            // Ensure proper newline before new section
            if self.len_bytes() > 0 && !self.text().ends_with('\n') {
                self.insert_raw(end_pos, "\n")?;
            }

            self.insert_raw(Position::new(self.len_bytes()), &section_text)?;

            // Ensure trailing newline
            if !section_text.ends_with('\n') {
                self.insert_raw(Position::new(self.len_bytes()), "\n")?;
            }
        }

        // Validate the result
        let _ = Script::parse(&self.text()).map_err(EditorError::from)?;

        Ok(())
    }

    /// Find the start offset of a section in the document
    #[cfg(feature = "stream")]
    fn find_section_start(&self, section: &Section) -> Result<usize> {
        // Get the section header text
        let header = match section {
            Section::ScriptInfo(_) => "[Script Info]",
            Section::Styles(_) => "[V4+ Styles]",
            Section::Events(_) => "[Events]",
            Section::Fonts(_) => "[Fonts]",
            Section::Graphics(_) => "[Graphics]",
        };

        // Find the header in the document
        if let Some(pos) = self.text().find(header) {
            Ok(pos)
        } else {
            Err(EditorError::SectionNotFound {
                section: header.to_string(),
            })
        }
    }

    /// Find the end offset of a section in the document
    #[cfg(feature = "stream")]
    fn find_section_end(&self, section: &Section) -> Result<usize> {
        let start = self.find_section_start(section)?;
        let content = &self.text()[start..];

        // Find the next section header or end of document
        let section_headers = [
            "[Script Info]",
            "[V4+ Styles]",
            "[Events]",
            "[Fonts]",
            "[Graphics]",
        ];

        let mut end_offset = content.len();
        for header in &section_headers {
            if let Some(pos) = content.find(header) {
                if pos > 0 {
                    end_offset = end_offset.min(pos);
                }
            }
        }

        Ok(start + end_offset)
    }

    /// Delete text in range (low-level operation without undo)
    pub(crate) fn delete_raw(&mut self, range: Range) -> Result<()> {
        if range.end.offset > self.len_bytes() {
            return Err(EditorError::InvalidRange {
                start: range.start.offset,
                end: range.end.offset,
                length: self.len_bytes(),
            });
        }

        #[cfg(feature = "rope")]
        {
            self.text_rope.remove(range.start.offset..range.end.offset);
        }
        #[cfg(not(feature = "rope"))]
        {
            self.text_content
                .drain(range.start.offset..range.end.offset);
        }

        self.modified = true;
        Ok(())
    }

    /// Replace text in range (low-level operation without undo)
    pub(crate) fn replace_raw(&mut self, range: Range, text: &str) -> Result<()> {
        self.delete_raw(range)?;
        self.insert_raw(range.start, text)?;
        Ok(())
    }
}

impl Default for EditorDocument {
    fn default() -> Self {
        Self::new()
    }
}

/// Fluent position API for editor operations
pub struct DocumentPosition<'a> {
    document: &'a mut EditorDocument,
    position: Position,
}

impl<'a> DocumentPosition<'a> {
    /// Insert text at this position
    pub fn insert_text(self, text: &str) -> Result<()> {
        self.document.insert(self.position, text)
    }

    /// Delete text range starting from this position
    pub fn delete_range(self, len: usize) -> Result<()> {
        let end_pos = Position::new(self.position.offset + len);
        let range = Range::new(self.position, end_pos);
        self.document.delete(range)
    }

    /// Replace text at this position
    pub fn replace_text(self, len: usize, new_text: &str) -> Result<()> {
        let end_pos = Position::new(self.position.offset + len);
        let range = Range::new(self.position, end_pos);
        self.document.replace(range, new_text)
    }
}

impl EditorDocument {
    /// Get fluent API for position-based operations
    pub fn at(&mut self, pos: Position) -> DocumentPosition {
        DocumentPosition {
            document: self,
            position: pos,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_creation() {
        let doc = EditorDocument::new();
        assert!(doc.is_empty());
        assert_eq!(doc.len_lines(), 1);
        assert!(!doc.is_modified());
    }

    #[test]
    fn document_from_content() {
        let content = "[Script Info]\nTitle: Test";
        let doc = EditorDocument::from_content(content).unwrap();
        assert_eq!(doc.text(), content);
        assert_eq!(doc.len_bytes(), content.len());
    }

    #[test]
    fn document_modification() {
        let mut doc = EditorDocument::new();
        doc.insert(Position::new(0), "Hello").unwrap();
        assert!(doc.is_modified());
        assert_eq!(doc.text(), "Hello");
    }

    #[test]
    fn position_conversion() {
        let content = "Line 1\nLine 2\nLine 3";
        let doc = EditorDocument::from_content(content).unwrap();

        // Start of second line
        let pos = Position::new(7); // After "Line 1\n"
        let lc = doc.position_to_line_column(pos).unwrap();
        assert_eq!(lc.line, 2);
        assert_eq!(lc.column, 1);
    }

    #[test]
    fn range_operations() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();

        // Delete "World"
        let range = Range::new(Position::new(6), Position::new(11));
        doc.delete(range).unwrap();
        assert_eq!(doc.text(), "Hello ");

        // Replace with "Rust"
        doc.insert(Position::new(6), "Rust").unwrap();
        assert_eq!(doc.text(), "Hello Rust");
    }

    #[test]
    fn parse_script_test() {
        let content = "[Script Info]\nTitle: Test\n[Events]\nDialogue: test";
        let doc = EditorDocument::from_content(content).unwrap();

        // Validate should succeed
        doc.validate().unwrap();

        // Parse and use script
        let sections_count = doc
            .parse_script_with(|script| script.sections().len())
            .unwrap();
        assert!(sections_count > 0);
    }

    #[test]
    fn test_edit_event_by_index() {
        let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Second event
Dialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,Third event"#;

        let mut doc = EditorDocument::from_content(content).unwrap();

        // Edit the second event (index 1)
        let result = doc.edit_event_by_index(1, |_event| {
            vec![
                ("text", "Modified second event".to_string()),
                ("style", "NewStyle".to_string()),
                ("start", "0:00:06.00".to_string()),
            ]
        });

        assert!(result.is_ok());
        let new_line = result.unwrap();
        assert!(new_line.contains("Modified second event"));
        assert!(new_line.contains("NewStyle"));
        assert!(new_line.contains("0:00:06.00"));

        // Verify the document was updated
        let updated_content = doc.text();
        assert!(updated_content.contains("Modified second event"));
        assert!(updated_content.contains("NewStyle"));
        assert!(updated_content.contains("0:00:06.00"));
        assert!(!updated_content.contains("Second event")); // Old text should be gone
    }

    #[test]
    fn test_edit_event_by_index_out_of_bounds() {
        let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,First event"#;

        let mut doc = EditorDocument::from_content(content).unwrap();

        // Try to edit non-existent event
        let result =
            doc.edit_event_by_index(5, |_event| vec![("text", "This should fail".to_string())]);

        assert!(result.is_err());
        match result.err().unwrap() {
            EditorError::InvalidRange { start, end, length } => {
                assert_eq!(start, 5);
                assert_eq!(end, 6);
                assert_eq!(length, 1); // Only 1 event exists
            }
            _ => panic!("Expected InvalidRange error"),
        }
    }

    #[test]
    fn test_edit_event_with_builder() {
        let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,Speaker,0,0,0,,Original text
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Second event"#;

        let mut doc = EditorDocument::from_content(content).unwrap();

        // Edit the first event using builder
        let result = doc.edit_event_with_builder(0, |builder| {
            builder
                .text("Modified with builder")
                .style("NewStyle")
                .end_time("0:00:08.00")
                .speaker("NewSpeaker")
        });

        assert!(result.is_ok());
        let new_line = result.unwrap();
        assert!(new_line.contains("Modified with builder"));
        assert!(new_line.contains("NewStyle"));
        assert!(new_line.contains("0:00:08.00"));
        assert!(new_line.contains("NewSpeaker"));

        // Verify the document was updated
        let updated_content = doc.text();
        assert!(updated_content.contains("Modified with builder"));
        assert!(updated_content.contains("0:00:08.00"));
        assert!(!updated_content.contains("Original text"));
    }

    #[test]
    fn test_edit_event_with_builder_preserves_format() {
        // Test with V4++ format that includes MarginT and MarginB
        let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginT, MarginB, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,10,20,5,15,fade,Original text"#;

        let mut doc = EditorDocument::from_content(content).unwrap();

        // Edit preserving the V4++ format
        let result = doc.edit_event_with_builder(0, |builder| {
            builder.text("New text").margin_top(30).margin_bottom(40)
        });

        assert!(result.is_ok());
        let new_line = result.unwrap();

        // Should use MarginT and MarginB fields based on format line
        assert!(new_line.contains("30")); // margin_top
        assert!(new_line.contains("40")); // margin_bottom
        assert!(new_line.contains("New text"));
        assert!(new_line.contains("10,20,30,40")); // All margins in correct order
    }

    #[test]
    fn test_edit_event_with_builder_comment() {
        let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Comment: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,This is a comment"#;

        let mut doc = EditorDocument::from_content(content).unwrap();

        // Edit comment event
        let result = doc.edit_event_with_builder(0, |builder| builder.text("Updated comment"));

        assert!(result.is_ok());
        let new_line = result.unwrap();
        assert!(new_line.starts_with("Comment:"));
        assert!(new_line.contains("Updated comment"));
    }

    #[test]
    fn test_edit_event_by_index_all_fields() {
        let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,Speaker,10,20,30,fade,Original text"#;

        let mut doc = EditorDocument::from_content(content).unwrap();

        // Edit all possible fields
        let result = doc.edit_event_by_index(0, |_event| {
            vec![
                ("layer", "1".to_string()),
                ("start", "0:00:01.00".to_string()),
                ("end", "0:00:06.00".to_string()),
                ("style", "Custom".to_string()),
                ("name", "NewSpeaker".to_string()),
                ("margin_l", "15".to_string()),
                ("margin_r", "25".to_string()),
                ("margin_v", "35".to_string()),
                ("effect", "scroll".to_string()),
                ("text", "Completely new text".to_string()),
            ]
        });

        assert!(result.is_ok());
        let new_line = result.unwrap();

        // Verify all fields were updated
        assert!(new_line.contains("Dialogue: 1,"));
        assert!(new_line.contains("0:00:01.00"));
        assert!(new_line.contains("0:00:06.00"));
        assert!(new_line.contains("Custom"));
        assert!(new_line.contains("NewSpeaker"));
        assert!(new_line.contains("15"));
        assert!(new_line.contains("25"));
        assert!(new_line.contains("35"));
        assert!(new_line.contains("scroll"));
        assert!(new_line.contains("Completely new text"));
    }

    #[test]
    fn test_undo_redo_basic() {
        let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();
        let initial_len = doc.len_bytes();
        println!(
            "Initial doc length: {}, content: {:?}",
            initial_len,
            doc.text()
        );

        // Insert some text
        doc.insert(Position::new(initial_len), "\nAuthor: John")
            .unwrap();
        println!(
            "After insert: length: {}, content: {:?}",
            doc.len_bytes(),
            doc.text()
        );
        assert!(doc.text().contains("Author: John"));
        assert!(doc.can_undo());
        assert!(!doc.can_redo());

        // Undo the insert
        let result = doc.undo().unwrap();
        println!(
            "After undo: length: {}, content: {:?}",
            doc.len_bytes(),
            doc.text()
        );
        assert!(result.success);
        assert!(!doc.text().contains("Author: John"));
        assert!(!doc.can_undo());
        assert!(doc.can_redo());

        // Redo the insert
        println!("About to redo...");
        let result = doc.redo().unwrap();
        println!(
            "After redo: length: {}, content: {:?}",
            doc.len_bytes(),
            doc.text()
        );
        assert!(result.success);
        assert!(doc.text().contains("Author: John"));
        assert!(doc.can_undo());
        assert!(!doc.can_redo());
    }

    #[test]
    fn test_undo_redo_multiple_operations() {
        let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

        // Multiple operations
        doc.insert(Position::new(doc.len_bytes()), "\nAuthor: John")
            .unwrap();
        doc.insert(Position::new(doc.len_bytes()), "\nVersion: 1.0")
            .unwrap();
        doc.insert(Position::new(doc.len_bytes()), "\nComment: Test script")
            .unwrap();

        assert!(doc.text().contains("Author: John"));
        assert!(doc.text().contains("Version: 1.0"));
        assert!(doc.text().contains("Comment: Test script"));

        // Undo all operations
        doc.undo().unwrap();
        assert!(!doc.text().contains("Comment: Test script"));

        doc.undo().unwrap();
        assert!(!doc.text().contains("Version: 1.0"));

        doc.undo().unwrap();
        assert!(!doc.text().contains("Author: John"));

        // Redo one operation
        doc.redo().unwrap();
        assert!(doc.text().contains("Author: John"));
        assert!(!doc.text().contains("Version: 1.0"));
    }

    #[test]
    fn test_undo_redo_replace() {
        let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Original").unwrap();

        // Find and replace "Original" with "Modified"
        let start = doc.text().find("Original").unwrap();
        let range = Range::new(Position::new(start), Position::new(start + 8));
        doc.replace(range, "Modified").unwrap();

        assert!(doc.text().contains("Title: Modified"));
        assert!(!doc.text().contains("Original"));

        // Undo the replace
        doc.undo().unwrap();
        assert!(doc.text().contains("Title: Original"));
        assert!(!doc.text().contains("Modified"));

        // Redo the replace
        doc.redo().unwrap();
        assert!(doc.text().contains("Title: Modified"));
        assert!(!doc.text().contains("Original"));
    }

    #[test]
    fn test_undo_redo_delete() {
        let mut doc =
            EditorDocument::from_content("[Script Info]\nTitle: Test\nAuthor: John").unwrap();

        // Delete the Author line
        let start = doc.text().find("\nAuthor: John").unwrap();
        let range = Range::new(Position::new(start), Position::new(start + 13));
        doc.delete(range).unwrap();

        assert!(!doc.text().contains("Author: John"));

        // Undo the delete
        doc.undo().unwrap();
        assert!(doc.text().contains("Author: John"));

        // Redo the delete
        doc.redo().unwrap();
        assert!(!doc.text().contains("Author: John"));
    }
}
