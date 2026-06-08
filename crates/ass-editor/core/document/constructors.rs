//! Constructors, file IO, event-channel wiring, and ID generation
//!
//! Provides the various ways to build an `EditorDocument` (empty, from a
//! string, from disk), the persistence helpers, and the private `emit`
//! helper shared with the editing submodules.

use super::EditorDocument;
#[cfg(feature = "std")]
use super::EventSender;
use crate::core::errors::{EditorError, Result};
use crate::core::history::UndoManager;
use ass_core::parser::Script;

#[cfg(feature = "std")]
use crate::events::DocumentEvent;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

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
    pub(super) fn emit(&mut self, event: DocumentEvent) {
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
            use core::sync::atomic::{AtomicU32, Ordering};
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let id = COUNTER.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
            format!("doc_{id}")
        }
    }
}
