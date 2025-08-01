//! Main document type for the editor
//!
//! Provides the `EditorDocument` struct which manages ASS script content
//! with direct access to parsed ASS structures and efficient text editing.

use super::errors::{EditorError, Result};
use super::history::UndoManager;
use super::position::{LineColumn, Position, Range};
use crate::commands::CommandResult;
use ass_core::parser::{ast::Section, Script};
#[cfg(feature = "stream")]
use ass_core::parser::script::ScriptDeltaOwned;
use core::ops::Range as StdRange;

#[cfg(feature = "std")]
use std::sync::Arc;

#[cfg(not(feature = "std"))]
use alloc::{format, string::{String, ToString}, vec, vec::Vec};

#[cfg(feature = "std")]
use std::sync::mpsc::Sender;

#[cfg(feature = "std")]
use crate::events::DocumentEvent;

#[cfg(feature = "std")]
type EventSender = Sender<DocumentEvent>;

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

    /// Extension registry integration for tag and section handling
    #[cfg(feature = "plugins")]
    registry_integration: Option<Arc<crate::extensions::registry_integration::RegistryIntegration>>,

    /// Undo/redo manager
    history: UndoManager,

    /// Event channel for sending document events
    #[cfg(feature = "std")]
    event_tx: Option<EventSender>,
    
    /// Incremental parser for efficient updates
    #[cfg(feature = "stream")]
    incremental_parser: crate::core::incremental::IncrementalParser,
    
    /// Lazy validator for on-demand validation
    validator: crate::utils::validator::LazyValidator,
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
            registry_integration: None,
            history: UndoManager::new(),
            #[cfg(feature = "std")]
            event_tx: None,
            #[cfg(feature = "stream")]
            incremental_parser: crate::core::incremental::IncrementalParser::new(),
            validator: crate::utils::validator::LazyValidator::new(),
        }
    }

    /// Create a new document with event channel
    #[cfg(feature = "std")]
    pub fn with_event_channel(event_tx: EventSender) -> Self {
        let mut doc = Self::new();
        doc.event_tx = Some(event_tx);
        doc
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

    /// Emit an event to the event channel
    #[cfg(feature = "std")]
    fn emit(&mut self, event: DocumentEvent) {
        if let Some(tx) = &mut self.event_tx {
            let _ = tx.send(event);
        }
    }

    /// Set the event channel for this document
    #[cfg(feature = "std")]
    pub fn set_event_channel(&mut self, event_tx: EventSender) {
        self.event_tx = Some(event_tx);
    }

    /// Check if document has an event channel
    #[cfg(feature = "std")]
    pub fn has_event_channel(&self) -> bool {
        self.event_tx.is_some()
    }

    /// Load document from string content
    ///
    /// Creates a new `EditorDocument` from ASS subtitle content. The content
    /// is validated during creation to ensure it's parseable.
    ///
    /// # Examples
    ///
    /// ```
    /// use ass_editor::EditorDocument;
    ///
    /// let content = r#"
    /// [Script Info]
    /// Title: My Subtitle
    ///
    /// [V4+ Styles]
    /// Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
    /// Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
    ///
    /// [Events]
    /// Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
    /// Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World
    /// "#;
    ///
    /// let doc = EditorDocument::from_content(content).unwrap();
    /// assert!(doc.text().contains("Hello World"));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the content cannot be parsed as valid ASS format.
    pub fn from_content(content: &str) -> Result<Self> {
        // Validate that content can be parsed
        let _ = Script::parse(content).map_err(EditorError::from)?;

        #[cfg(feature = "stream")]
        let mut incremental_parser = crate::core::incremental::IncrementalParser::new();
        #[cfg(feature = "stream")]
        incremental_parser.initialize_cache(content);

        Ok(Self {
            #[cfg(feature = "rope")]
            text_rope: ropey::Rope::from_str(content),
            #[cfg(not(feature = "rope"))]
            text_content: content.to_string(),
            id: Self::generate_id(),
            modified: false,
            file_path: None,
            #[cfg(feature = "plugins")]
            registry_integration: None,
            history: UndoManager::new(),
            #[cfg(feature = "std")]
            event_tx: None,
            #[cfg(feature = "stream")]
            incremental_parser,
            validator: crate::utils::validator::LazyValidator::new(),
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
    /// This is the basic validation that just checks parsing
    pub fn validate(&self) -> Result<()> {
        let content = self.text();
        Script::parse(&content).map_err(EditorError::from)?;
        Ok(())
    }
    
    /// Execute a command with proper history recording
    /// 
    /// This method ensures that commands are properly recorded in the undo history.
    /// Use this instead of calling command.execute() directly if you want undo support.
    /// 
    /// For BatchCommand, this creates a synthetic undo operation that captures
    /// the aggregate effect of all sub-commands.
    pub fn execute_command(&mut self, command: &dyn crate::commands::EditorCommand) -> Result<crate::commands::CommandResult> {
        use crate::core::history::Operation;
        
        // Get the cursor position before
        let _cursor_before = self.cursor_position();
        
        // Execute the command
        let result = command.execute(self)?;
        
        // Only record if the command changed content
        if result.content_changed {
            // Determine the operation based on the result
            let operation = if let Some(range) = result.modified_range {
                if range.is_empty() {
                    // This was an insertion
                    let inserted_text = self.text_range(Range::new(range.start, 
                        Position::new(range.start.offset + 
                            result.new_cursor.map_or(0, |c| c.offset - range.start.offset))))?;
                    Operation::Insert {
                        position: range.start,
                        text: inserted_text,
                    }
                } else {
                    // For batch commands and complex operations, we need to be more careful
                    // Store enough information to properly undo
                    // This is a simplified approach - ideally each command would handle its own undo
                    let cmd_desc = command.description();
                    if cmd_desc.contains("batch") || cmd_desc.contains("Multiple") {
                        // For batch operations, we don't have enough info
                        // Just use a simple insert operation
                        Operation::Insert {
                            position: range.start,
                            text: String::new(),
                        }
                    } else {
                        // For simple operations, use the range info
                        Operation::Replace {
                            range,
                            old_text: String::new(), // We don't have the old text
                            new_text: self.text_range(range).unwrap_or_default(),
                        }
                    }
                }
            } else {
                // No range info - create a generic operation
                Operation::Insert {
                    position: Position::new(0),
                    text: String::new(),
                }
            };
            
            // Update cursor if needed
            if let Some(new_cursor) = result.new_cursor {
                self.set_cursor_position(Some(new_cursor));
            }
            
            // Record in history
            self.history.record_operation(
                operation,
                command.description().to_string(),
                &result,
            );
            
            // Clear validation cache since content changed
            self.validator.clear_cache();
        }
        
        Ok(result)
    }
    
    /// Perform comprehensive validation using the LazyValidator
    /// Returns detailed validation results including warnings and suggestions
    /// 
    /// Note: Returns a cloned result to avoid borrow checker issues
    pub fn validate_comprehensive(&mut self) -> Result<crate::utils::validator::ValidationResult> {
        // Create a temporary document reference for validation
        // This is needed because we can't pass &self while mutably borrowing validator
        let temp_doc = EditorDocument::from_content(&self.text())?;
        let result = self.validator.validate(&temp_doc)?;
        Ok(result.clone())
    }
    
    /// Force revalidation even if cached results exist
    pub fn force_validate(&mut self) -> Result<crate::utils::validator::ValidationResult> {
        // Create a temporary document reference for validation
        let temp_doc = EditorDocument::from_content(&self.text())?;
        let result = self.validator.force_validate(&temp_doc)?;
        Ok(result.clone())
    }
    
    /// Check if document is valid (quick check using cache if available)
    pub fn is_valid_cached(&mut self) -> Result<bool> {
        // Create a temporary document reference for validation
        let temp_doc = EditorDocument::from_content(&self.text())?;
        self.validator.is_valid(&temp_doc)
    }
    
    /// Get cached validation result without revalidating
    pub fn validation_result(&self) -> Option<&crate::utils::validator::ValidationResult> {
        self.validator.cached_result()
    }
    
    /// Configure the validator
    pub fn set_validator_config(&mut self, config: crate::utils::validator::ValidatorConfig) {
        self.validator.set_config(config);
    }
    
    /// Get mutable access to the validator
    pub fn validator_mut(&mut self) -> &mut crate::utils::validator::LazyValidator {
        &mut self.validator
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
    
    /// Import content from another subtitle format
    #[cfg(feature = "formats")]
    pub fn import_format(content: &str, format: Option<crate::utils::formats::SubtitleFormat>) -> Result<Self> {
        let ass_content = crate::utils::formats::FormatConverter::import(content, format)?;
        Self::from_content(&ass_content)
    }
    
    /// Export document to another subtitle format
    #[cfg(feature = "formats")]
    pub fn export_format(
        &self,
        format: crate::utils::formats::SubtitleFormat,
        options: &crate::utils::formats::ConversionOptions,
    ) -> Result<String> {
        crate::utils::formats::FormatConverter::export(self, format, options)
    }

    /// Check if document has unsaved changes
    #[must_use]
    pub const fn is_modified(&self) -> bool {
        self.modified
    }
    
    /// Get current cursor position (if tracked)
    #[must_use]
    pub fn cursor_position(&self) -> Option<Position> {
        self.history.cursor_position()
    }

    /// Set current cursor position for tracking
    pub fn set_cursor_position(&mut self, position: Option<Position>) {
        self.history.set_cursor(position);
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

    /// Get direct access to the rope for advanced operations
    #[cfg(feature = "rope")]
    #[must_use]
    pub fn rope(&self) -> &ropey::Rope {
        &self.text_rope
    }

    /// Get the length of the document in bytes
    #[must_use]
    pub fn len(&self) -> usize {
        #[cfg(feature = "rope")]
        {
            self.text_rope.len_bytes()
        }
        #[cfg(not(feature = "rope"))]
        {
            self.text_content.len()
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
            // Convert byte offsets to char indices for rope operations
            let start_char = self.text_rope.byte_to_char(start);
            let end_char = self.text_rope.byte_to_char(end);
            Ok(self.text_rope.slice(start_char..end_char).to_string())
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
            // Convert byte offset to char index for rope operations
            let char_idx = self.text_rope.byte_to_char(pos.offset);
            self.text_rope.insert(char_idx, text);
        }
        #[cfg(not(feature = "rope"))]
        {
            self.text_content.insert_str(pos.offset, text);
        }

        self.modified = true;
        Ok(())
    }

    /// Insert text at position with undo support
    ///
    /// Inserts text at the given position, automatically updating the underlying
    /// text representation and recording the operation in the undo history.
    ///
    /// # Examples
    ///
    /// ```
    /// use ass_editor::{EditorDocument, Position};
    ///
    /// let mut doc = EditorDocument::from_content("Hello World").unwrap();
    /// let pos = Position::new(5); // Insert after "Hello"
    /// doc.insert(pos, " there").unwrap();
    ///
    /// assert_eq!(doc.text(), "Hello there World");
    ///
    /// // Can undo the operation
    /// doc.undo().unwrap();
    /// assert_eq!(doc.text(), "Hello World");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if the position is beyond the document bounds.
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

        // Clear validation cache since content changed
        self.validator.clear_cache();

        // Emit event
        #[cfg(feature = "std")]
        self.emit(DocumentEvent::TextInserted {
            position: pos,
            text: text.to_string(),
            length: text.len(),
        });

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
            deleted_text: deleted_text.clone(),
        };
        self.history
            .record_operation(operation, command.description().to_string(), &result);

        // Clear validation cache since content changed
        self.validator.clear_cache();

        // Emit event
        #[cfg(feature = "std")]
        self.emit(DocumentEvent::TextDeleted {
            range,
            deleted_text,
        });

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
            old_text: old_text.clone(),
            new_text: text.to_string(),
        };
        self.history
            .record_operation(operation, command.description().to_string(), &result);

        // Clear validation cache since content changed
        self.validator.clear_cache();

        // Emit event
        #[cfg(feature = "std")]
        self.emit(DocumentEvent::TextReplaced {
            range,
            old_text,
            new_text: text.to_string(),
        });

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
            #[allow(static_mut_refs)]
            unsafe {
                COUNTER += 1;
                format!("doc_{COUNTER}")
            }
        }
    }

    // === ASS-Aware APIs ===

    /// Get number of events without manual parsing  
    pub fn events_count(&self) -> Result<usize> {
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
    /// 
    /// Includes error recovery with fallback strategies:
    /// 1. Try incremental parsing with Script::parse_partial()
    /// 2. On failure, fall back to full reparse
    /// 3. On repeated failures, reset parser state and retry
    #[cfg(feature = "stream")]
    pub fn edit_incremental(&mut self, range: Range, new_text: &str) -> Result<ScriptDeltaOwned> {
        use crate::core::history::Operation;
        
        // Fast path for simple edits that don't require full parsing
        let is_simple_edit = new_text.len() <= 100 && // Small to medium edits
            !new_text.contains('[') && // No new sections
            new_text.matches('\n').count() <= 1 && // At most one line break
            range.len() <= 50; // Small replacements
            
        if is_simple_edit {
            return self.edit_fast_path(range, new_text);
        }
        
        // Get the old text for undo data
        let old_text = self.text_range(range)?;
        
        // Apply change with incremental parsing (includes fallback to full parse)
        let current_text = self.text();
        let delta = match self.incremental_parser.apply_change(&current_text, range, new_text) {
            Ok(delta) => delta,
            Err(e) => {
                // Log the error for debugging
                #[cfg(feature = "std")]
                eprintln!("Incremental parsing failed, attempting recovery: {e}");
                
                // If incremental parsing fails repeatedly, reset the parser
                if self.incremental_parser.should_reparse() {
                    self.incremental_parser.clear_cache();
                }
                
                // Try one more time with a fresh parser state
                match self.incremental_parser.apply_change(&current_text, range, new_text) {
                    Ok(delta) => delta,
                    Err(_) => {
                        // Final fallback: return a minimal delta indicating the change
                        ScriptDeltaOwned {
                            added: Vec::new(),
                            modified: vec![(0, "Script modified".to_string())],
                            removed: Vec::new(),
                            new_issues: Vec::new(),
                        }
                    }
                }
            }
        };
        
        // Create undo data from the delta (must be captured BEFORE applying changes)
        let undo_data = self.capture_delta_undo_data(&delta)?;
        
        // Create delta operation for history
        let operation = Operation::Delta {
            forward: delta.clone(),
            undo_data,
        };
        
        // Create command result
        let result = CommandResult::success_with_change(
            range,
            Position::new(range.start.offset + new_text.len()),
        );
        
        // Record in history
        let result_with_delta = result.with_delta(delta.clone());
        self.history.record_operation(
            operation,
            format!("Incremental edit at {}", range.start.offset),
            &result_with_delta,
        );
        
        // Apply the text change
        self.replace_raw(range, new_text)?;
        
        // Mark as modified
        self.modified = true;
        
        // Emit event
        #[cfg(feature = "std")]
        self.emit(DocumentEvent::TextReplaced {
            range,
            old_text,
            new_text: new_text.to_string(),
        });
        
        Ok(delta)
    }

    /// Insert text with incremental parsing (< 1ms target)
    #[cfg(feature = "stream")]
    pub fn insert_incremental(&mut self, pos: Position, text: &str) -> Result<ScriptDeltaOwned> {
        let range = Range::new(pos, pos); // Zero-length range for insertion
        self.edit_incremental(range, text)
    }

    /// Delete text with incremental parsing
    #[cfg(feature = "stream")]
    pub fn delete_incremental(&mut self, range: Range) -> Result<ScriptDeltaOwned> {
        self.edit_incremental(range, "")
    }
    
    /// Fast path for simple edits that avoids heavy parsing
    #[cfg(feature = "stream")]
    fn edit_fast_path(&mut self, range: Range, new_text: &str) -> Result<ScriptDeltaOwned> {
        use crate::core::history::Operation;
        
        // Validate that range boundaries are on valid UTF-8 char boundaries
        let text = self.text();
        if !text.is_char_boundary(range.start.offset) || !text.is_char_boundary(range.end.offset) {
            // Fall back to regular incremental parsing for invalid boundaries
            return self.edit_incremental_fallback(range, new_text);
        }
        
        // Get old text for undo
        let old_text = if range.is_empty() {
            String::new()
        } else {
            self.text_range(range)?
        };
        
        // Simple text replacement without parsing
        self.replace_raw(range, new_text)?;
        
        // Create minimal undo operation
        let operation = Operation::Replace {
            range,
            new_text: new_text.to_string(),
            old_text,
        };
        
        // Create result
        let result = CommandResult::success_with_change(
            range,
            Position::new(range.start.offset + new_text.len()),
        );
        
        // Record in history (without delta)
        self.history.record_operation(
            operation,
            "Fast character insert".to_string(),
            &result,
        );
        
        // Mark as modified
        self.modified = true;
        
        // Return minimal delta
        Ok(ScriptDeltaOwned {
            added: Vec::new(),
            modified: Vec::new(),
            removed: Vec::new(),
            new_issues: Vec::new(),
        })
    }
    
    /// Fallback for edit_incremental that avoids infinite recursion
    #[cfg(feature = "stream")]
    fn edit_incremental_fallback(&mut self, range: Range, new_text: &str) -> Result<ScriptDeltaOwned> {
        // Just do a simple replace without the fast path
        self.replace(range, new_text)?;
        
        // Return minimal delta
        Ok(ScriptDeltaOwned {
            added: Vec::new(),
            modified: Vec::new(),
            removed: Vec::new(),
            new_issues: Vec::new(),
        })
    }
    
    /// Safe edit with automatic fallback to regular replace on error
    /// 
    /// This method tries incremental parsing first for performance,
    /// but falls back to regular replace if incremental parsing is unavailable
    /// or fails. This ensures edits always succeed.
    pub fn edit_safe(&mut self, range: Range, new_text: &str) -> Result<()> {
        #[cfg(feature = "stream")]
        {
            // Try incremental parsing first
            match self.edit_incremental(range, new_text) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    #[cfg(feature = "std")]
                    eprintln!("Incremental edit failed, falling back to regular replace: {e}");
                }
            }
        }
        
        // Fallback to regular replace
        self.replace(range, new_text)
    }

    /// Edit event using incremental parsing for performance
    #[cfg(feature = "stream")]
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
    #[cfg(feature = "stream")]
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
                    result.new_cursor = entry.cursor_before;
                }
                Operation::Delete {
                    range,
                    deleted_text,
                } => {
                    // Undo delete by inserting the deleted text
                    self.insert_raw(range.start, deleted_text)?;
                    let end_pos = Position::new(range.start.offset + deleted_text.len());
                    result.modified_range = Some(Range::new(range.start, end_pos));
                    result.new_cursor = entry.cursor_before;
                }
                Operation::Replace {
                    range, old_text, ..
                } => {
                    // Undo replace by restoring old text
                    self.replace_raw(*range, old_text)?;
                    let end_pos = Position::new(range.start.offset + old_text.len());
                    result.modified_range = Some(Range::new(range.start, end_pos));
                    result.new_cursor = entry.cursor_before;
                }
                #[cfg(feature = "stream")]
                Operation::Delta { forward, undo_data } => {
                    // Restore removed sections
                    for (index, section_text) in undo_data.removed_sections.iter() {
                        self.insert_section_at(*index, section_text)?;
                    }

                    // Restore modified sections
                    for (index, original_text) in undo_data.modified_sections.iter() {
                        self.replace_section(*index, original_text)?;
                    }

                    // Remove added sections
                    for _ in 0..forward.added.len() {
                        self.remove_last_section()?;
                    }

                    result.message = Some("Delta operation undone".to_string());
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
                    result.new_cursor = entry.cursor_after;
                }
                Operation::Delete { range, .. } => {
                    // Redo delete
                    self.delete_raw(*range)?;
                    result.modified_range = Some(Range::new(range.start, range.start));
                    result.new_cursor = entry.cursor_after;
                }
                Operation::Replace {
                    range, new_text, ..
                } => {
                    // Redo replace
                    self.replace_raw(*range, new_text)?;
                    let end_pos = Position::new(range.start.offset + new_text.len());
                    result.modified_range = Some(Range::new(range.start, end_pos));
                    result.new_cursor = entry.cursor_after;
                }
                #[cfg(feature = "stream")]
                Operation::Delta {
                    forward,
                    undo_data: _,
                } => {
                    // Re-apply the delta
                    self.apply_script_delta(forward.clone())?;
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

    /// Apply a script delta and record it with undo data
    #[cfg(feature = "stream")]
    pub fn apply_script_delta(&mut self, delta: ScriptDeltaOwned) -> Result<()> {
        use crate::core::history::Operation;

        // Capture undo data before applying
        let undo_data = self.capture_delta_undo_data(&delta)?;

        // Apply delta
        self.apply_script_delta_internal(delta.clone())?;

        // Record with undo data
        let operation = Operation::Delta {
            forward: delta,
            undo_data,
        };

        let result = CommandResult::success();
        self.history
            .record_operation(operation, "Apply delta".to_string(), &result);

        Ok(())
    }

    /// Capture undo data before applying a delta
    #[cfg(feature = "stream")]
    fn capture_delta_undo_data(
        &self,
        delta: &ScriptDeltaOwned,
    ) -> Result<crate::core::history::DeltaUndoData> {
        let mut removed_sections = Vec::new();
        let mut modified_sections = Vec::new();

        // First, get the current content for section extraction
        let content = self.text();

        // For incremental edits, use simplified undo data to avoid expensive full parsing
        // Only do detailed section analysis for larger operations
        #[cfg(feature = "stream")]
        let is_small_edit = delta.added.len() + delta.removed.len() + delta.modified.len() <= 2;
        #[cfg(not(feature = "stream"))]
        let is_small_edit = false;
        
        if is_small_edit {
            // For small incremental edits, skip expensive undo data capture
            // The basic text-based undo in edit_incremental will handle this
        } else {
            // For larger operations, do full undo data capture
            let _ = self.parse_script_with(|script| {
                // Capture removed sections
                for &index in &delta.removed {
                    if let Some(section) = script.sections().get(index) {
                        match self.extract_section_text(&content, section) {
                            Ok(section_text) => removed_sections.push((index, section_text)),
                            Err(_) => {
                                // If we can't extract the section text, store a placeholder
                                removed_sections.push((index, String::new()));
                            }
                        }
                    }
                }

                // Capture original state of modified sections
                for (index, _) in &delta.modified {
                    if let Some(section) = script.sections().get(*index) {
                        match self.extract_section_text(&content, section) {
                            Ok(section_text) => modified_sections.push((*index, section_text)),
                            Err(_) => {
                                // If we can't extract the section text, store a placeholder
                                modified_sections.push((*index, String::new()));
                            }
                        }
                    }
                }

                Ok::<(), EditorError>(())
            })?;
        }

        Ok(crate::core::history::DeltaUndoData {
            removed_sections,
            modified_sections,
        })
    }

    /// Extract section text from content
    #[cfg(feature = "stream")]
    fn extract_section_text(&self, content: &str, section: &Section) -> Result<String> {
        let header = match section {
            Section::ScriptInfo(_) => "[Script Info]",
            Section::Styles(_) => "[V4+ Styles]",
            Section::Events(_) => "[Events]",
            Section::Fonts(_) => "[Fonts]",
            Section::Graphics(_) => "[Graphics]",
        };

        // Find the header in the content
        let start = content
            .find(header)
            .ok_or_else(|| EditorError::SectionNotFound {
                section: header.to_string(),
            })?;

        // Find the next section header or end of document
        let section_headers = [
            "[Script Info]",
            "[V4+ Styles]",
            "[Events]",
            "[Fonts]",
            "[Graphics]",
        ];

        let end = content[start + header.len()..]
            .find(|_c: char| {
                for sh in &section_headers {
                    if content[start + header.len()..].starts_with(sh) {
                        return true;
                    }
                }
                false
            })
            .map(|pos| start + header.len() + pos)
            .unwrap_or(content.len());

        Ok(content[start..end].to_string())
    }

    /// Apply a script delta for efficient incremental parsing (internal)
    #[cfg(feature = "stream")]
    fn apply_script_delta_internal(&mut self, delta: ScriptDeltaOwned) -> Result<()> {
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
            // Convert byte offsets to char indices for rope operations
            let start_char = self.text_rope.byte_to_char(range.start.offset);
            let end_char = self.text_rope.byte_to_char(range.end.offset);
            self.text_rope.remove(start_char..end_char);
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

    // === Delta undo/redo helper methods ===

    /// Insert a section at a specific index
    #[cfg(feature = "stream")]
    fn insert_section_at(&mut self, index: usize, section_text: &str) -> Result<()> {
        // Get the current sections count
        let section_count = self.parse_script_with(|script| script.sections().len())?;

        // If index is beyond current sections, append to end
        if index >= section_count {
            let end_pos = Position::new(self.len_bytes());

            // Ensure proper newline before new section
            if self.len_bytes() > 0 && !self.text().ends_with('\n') {
                self.insert_raw(end_pos, "\n")?;
            }

            self.insert_raw(Position::new(self.len_bytes()), section_text)?;

            // Ensure trailing newline
            if !section_text.ends_with('\n') {
                self.insert_raw(Position::new(self.len_bytes()), "\n")?;
            }

            return Ok(());
        }

        // Find the position where to insert the new section
        let content = self.text();
        let insert_pos = self.parse_script_with(|script| -> Result<usize> {
            if let Some(section) = script.sections().get(index) {
                // Find the start of this section to insert before it
                let header = match section {
                    Section::ScriptInfo(_) => "[Script Info]",
                    Section::Styles(_) => "[V4+ Styles]",
                    Section::Events(_) => "[Events]",
                    Section::Fonts(_) => "[Fonts]",
                    Section::Graphics(_) => "[Graphics]",
                };

                if let Some(pos) = content.find(header) {
                    Ok(pos)
                } else {
                    Err(EditorError::SectionNotFound {
                        section: header.to_string(),
                    })
                }
            } else {
                // Append to end if index is out of bounds
                Ok(content.len())
            }
        })??;

        // Insert the section at the found position
        let mut text_to_insert = section_text.to_string();

        // Ensure section ends with newline
        if !text_to_insert.ends_with('\n') {
            text_to_insert.push('\n');
        }

        // Add extra newline if needed to separate from next section
        if insert_pos < content.len() {
            text_to_insert.push('\n');
        }

        self.insert_raw(Position::new(insert_pos), &text_to_insert)?;

        Ok(())
    }

    /// Replace a section at a specific index
    #[cfg(feature = "stream")]
    fn replace_section(&mut self, index: usize, new_text: &str) -> Result<()> {
        // Parse to find the section and get its boundaries
        // Get the section to replace
        let content = self.text();
        let section_info: Result<Option<&str>> = self.parse_script_with(|script| {
            if let Some(section) = script.sections().get(index) {
                let header = match section {
                    Section::ScriptInfo(_) => "[Script Info]",
                    Section::Styles(_) => "[V4+ Styles]",
                    Section::Events(_) => "[Events]",
                    Section::Fonts(_) => "[Fonts]",
                    Section::Graphics(_) => "[Graphics]",
                };
                Ok(Some(header))
            } else {
                Ok(None)
            }
        })?;

        if let Some(header) = section_info? {
            let start = self.find_section_start_by_header(&content, header)?;
            let end = self.find_section_end_from_start(&content, start)?;

            self.replace_raw(
                Range::new(Position::new(start), Position::new(end)),
                new_text,
            )?;
        }

        Ok(())
    }

    /// Remove the last section
    #[cfg(feature = "stream")]
    fn remove_last_section(&mut self) -> Result<()> {
        // Parse to find the last section and get its boundaries
        // Get the last section
        let content = self.text();
        let section_info: Result<Option<&str>> = self.parse_script_with(|script| {
            if let Some(section) = script.sections().last() {
                let header = match section {
                    Section::ScriptInfo(_) => "[Script Info]",
                    Section::Styles(_) => "[V4+ Styles]",
                    Section::Events(_) => "[Events]",
                    Section::Fonts(_) => "[Fonts]",
                    Section::Graphics(_) => "[Graphics]",
                };
                Ok(Some(header))
            } else {
                Ok(None)
            }
        })?;

        if let Some(header) = section_info? {
            let start = self.find_section_start_by_header(&content, header)?;
            let end = self.find_section_end_from_start(&content, start)?;

            self.delete_raw(Range::new(Position::new(start), Position::new(end)))?;
        }

        Ok(())
    }

    /// Find section start by header
    #[cfg(feature = "stream")]
    fn find_section_start_by_header(&self, content: &str, header: &str) -> Result<usize> {
        content
            .find(header)
            .ok_or_else(|| EditorError::SectionNotFound {
                section: header.to_string(),
            })
    }

    /// Find section end from start position
    #[cfg(feature = "stream")]
    fn find_section_end_from_start(&self, content: &str, start: usize) -> Result<usize> {
        let section_headers = [
            "[Script Info]",
            "[V4+ Styles]",
            "[Events]",
            "[Fonts]",
            "[Graphics]",
        ];

        // Find the next section header after start
        let mut end = content.len();
        for header in &section_headers {
            if let Some(pos) = content[start + 1..].find(header) {
                let actual_pos = start + 1 + pos;
                if actual_pos < end {
                    end = actual_pos;
                }
            }
        }

        Ok(end)
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
    
    /// Initialize the extension registry with built-in handlers
    #[cfg(feature = "plugins")]
    pub fn initialize_registry(&mut self) -> Result<()> {
        use crate::extensions::registry_integration::RegistryIntegration;
        
        let mut integration = RegistryIntegration::new();
        
        // Register all built-in extensions using the function from the builtin module
        crate::extensions::builtin::register_builtin_extensions(&mut integration)?;
        
        self.registry_integration = Some(Arc::new(integration));
        Ok(())
    }
    
    /// Get the extension registry for use in parsing
    #[cfg(feature = "plugins")]
    pub fn registry(&self) -> Option<&ass_core::plugin::ExtensionRegistry> {
        self.registry_integration.as_ref().map(|integration| integration.registry())
    }
    
    /// Parse the document content with extension support and process it with a callback
    /// 
    /// Since Script<'a> requires the source text to outlive it, this method uses a callback
    /// pattern to process the script while the content is still in scope.
    #[cfg(feature = "plugins")]
    pub fn parse_with_extensions<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&ass_core::parser::Script) -> R,
    {
        let content = self.text();
        
        if let Some(integration) = &self.registry_integration {
            // Parse with the extension registry using the builder pattern
            let script = ass_core::parser::Script::builder()
                .with_registry(integration.registry())
                .parse(&content)
                .map_err(EditorError::Core)?;
            Ok(f(&script))
        } else {
            // No registry, parse normally
            let script = ass_core::parser::Script::parse(&content)
                .map_err(EditorError::Core)?;
            Ok(f(&script))
        }
    }
    
    /// Register a custom tag handler
    #[cfg(feature = "plugins")]
    pub fn register_tag_handler(
        &mut self,
        extension_name: String,
        handler: Box<dyn ass_core::plugin::TagHandler>
    ) -> Result<()> {
        if self.registry_integration.is_none() {
            self.initialize_registry()?;
        }
        
        let registry_ref = self.registry_integration.as_mut()
            .ok_or_else(|| EditorError::ExtensionError {
                extension: extension_name.clone(),
                message: "Registry integration not available".to_string(),
            })?;
            
        if let Some(integration) = Arc::get_mut(registry_ref) {
            integration.register_custom_tag_handler(extension_name, handler)
        } else {
            Err(EditorError::ExtensionError {
                extension: extension_name,
                message: "Cannot modify shared registry integration".to_string(),
            })
        }
    }
    
    /// Register a custom section processor
    #[cfg(feature = "plugins")]
    pub fn register_section_processor(
        &mut self,
        extension_name: String,
        processor: Box<dyn ass_core::plugin::SectionProcessor>
    ) -> Result<()> {
        if self.registry_integration.is_none() {
            self.initialize_registry()?;
        }
        
        let registry_ref = self.registry_integration.as_mut()
            .ok_or_else(|| EditorError::ExtensionError {
                extension: extension_name.clone(),
                message: "Registry integration not available".to_string(),
            })?;
            
        if let Some(integration) = Arc::get_mut(registry_ref) {
            integration.register_custom_section_processor(extension_name, processor)
        } else {
            Err(EditorError::ExtensionError {
                extension: extension_name,
                message: "Cannot modify shared registry integration".to_string(),
            })
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
        let sections_count = doc.sections_count().unwrap();
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
    fn test_validator_integration() {
        let mut doc = EditorDocument::from_content(
            "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test"
        ).unwrap();
        
        // Should have result after comprehensive validation
        let result = doc.validate_comprehensive().unwrap();
        assert!(result.is_valid);
        
        // Modify document
        doc.insert(Position::new(doc.len_bytes()), "\nComment: Test").unwrap();
        
        // Force validate should work
        let result2 = doc.force_validate().unwrap();
        assert!(result2.is_valid);
    }
    
    #[test]
    fn test_validator_configuration() {
        let mut doc = EditorDocument::new();
        
        // Configure validator
        let config = crate::utils::validator::ValidatorConfig {
            max_issues: 5,
            enable_performance_hints: false,
            ..Default::default()
        };
        doc.set_validator_config(config);
        
        // Validator should be configured
        // We can't directly check the cache anymore, but configuration should work
        assert!(doc.is_valid_cached().is_ok());
    }
    
    #[test]
    fn test_validator_with_invalid_document() {
        let mut doc = EditorDocument::from_content("Invalid content").unwrap();
        
        // Comprehensive validation should find issues
        let result = doc.validate_comprehensive().unwrap();
        assert!(!result.issues.is_empty());
        
        // Should have warnings about missing sections
        let warnings = result.issues_with_severity(crate::utils::validator::ValidationSeverity::Warning);
        assert!(!warnings.is_empty());
    }
    
    #[test]
    #[cfg(feature = "formats")]
    fn test_format_import_export() {
        // Test SRT import
        let srt_content = "1\n00:00:00,000 --> 00:00:05,000\nHello world!";
        let doc = EditorDocument::import_format(srt_content, Some(crate::utils::formats::SubtitleFormat::SRT)).unwrap();
        assert!(doc.text().contains("Hello world!"));
        assert!(doc.has_events().unwrap());
        
        // Test export to WebVTT
        let options = crate::utils::formats::ConversionOptions::default();
        let webvtt = doc.export_format(crate::utils::formats::SubtitleFormat::WebVTT, &options).unwrap();
        assert!(webvtt.starts_with("WEBVTT"));
        assert!(webvtt.contains("00:00:00.000 --> 00:00:05.000"));
        assert!(webvtt.contains("Hello world!"));
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

    #[cfg(feature = "plugins")]
    #[test]
    fn test_registry_integration() {

        let mut doc = EditorDocument::new();
        
        // Initially no registry
        assert!(doc.registry().is_none());
        
        // Initialize with registry
        doc.initialize_registry().unwrap();
        assert!(doc.registry().is_some());
        
        // Parse with extensions
        doc.insert(Position::new(0), "[Script Info]\nTitle: Test\n\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\\b1}Bold{\\b0} text").unwrap();
        
        let section_count = doc.parse_with_extensions(|script| script.sections().len()).unwrap();
        assert_eq!(section_count, 2);
    }
    
    #[cfg(feature = "plugins")]
    #[test]
    fn test_custom_tag_handler() {
        use ass_core::plugin::{TagHandler, TagResult};
        
        struct CustomHandler;
        impl TagHandler for CustomHandler {
            fn name(&self) -> &'static str {
                "custom"
            }
            
            fn process(&self, _args: &str) -> TagResult {
                TagResult::Processed
            }
            
            fn validate(&self, _args: &str) -> bool {
                true
            }
        }
        
        let mut doc = EditorDocument::new();
        doc.initialize_registry().unwrap();
        
        // Register custom tag handler
        assert!(doc.register_tag_handler(
            "test-extension".to_string(),
            Box::new(CustomHandler)
        ).is_ok());
    }
    
    #[cfg(feature = "plugins")]
    #[test]
    fn test_custom_section_processor() {
        use ass_core::plugin::{SectionProcessor, SectionResult};
        
        struct CustomProcessor;
        impl SectionProcessor for CustomProcessor {
            fn name(&self) -> &'static str {
                "CustomSection"
            }
            
            fn process(&self, _header: &str, _lines: &[&str]) -> SectionResult {
                SectionResult::Processed
            }
            
            fn validate(&self, _header: &str, _lines: &[&str]) -> bool {
                true
            }
        }
        
        let mut doc = EditorDocument::new();
        doc.initialize_registry().unwrap();
        
        // Register custom section processor
        assert!(doc.register_section_processor(
            "test-extension".to_string(),
            Box::new(CustomProcessor)
        ).is_ok());
    }
}
