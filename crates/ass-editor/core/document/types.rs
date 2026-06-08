//! `EditorDocument` struct definition and `Default` implementation
//!
//! Defines the core document container holding text content and editing
//! state. The struct fields are exposed as `pub(super)` so the sibling
//! submodules implementing its methods can access them across module
//! boundaries.

use crate::core::history::UndoManager;

#[cfg(feature = "std")]
use std::sync::Arc;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::sync::mpsc::Sender;

#[cfg(feature = "std")]
use crate::events::DocumentEvent;

#[cfg(feature = "std")]
pub(super) type EventSender = Sender<DocumentEvent>;

/// Main document container for editing ASS scripts
///
/// Manages both text content and parsed ASS structures with direct access to
/// events, styles, and script info without manual parsing.
#[derive(Debug)]
pub struct EditorDocument {
    /// Rope for efficient text editing operations
    #[cfg(feature = "rope")]
    pub(super) text_rope: ropey::Rope,

    /// Raw text content (fallback when rope feature is disabled)
    #[cfg(not(feature = "rope"))]
    pub(super) text_content: String,

    /// Document identifier for session management
    pub(super) id: String,

    /// Whether document has unsaved changes
    pub(super) modified: bool,

    /// Optional file path if loaded from/saved to disk
    pub(super) file_path: Option<String>,

    /// Extension registry integration for tag and section handling
    #[cfg(feature = "plugins")]
    pub(super) registry_integration:
        Option<Arc<crate::extensions::registry_integration::RegistryIntegration>>,

    /// Undo/redo manager
    pub(super) history: UndoManager,

    /// Event channel for sending document events
    #[cfg(feature = "std")]
    pub(super) event_tx: Option<EventSender>,

    /// Incremental parser for efficient updates
    #[cfg(feature = "stream")]
    pub(super) incremental_parser: crate::core::incremental::IncrementalParser,

    /// Lazy validator for on-demand validation
    pub(super) validator: crate::utils::validator::LazyValidator,
}

impl Default for EditorDocument {
    fn default() -> Self {
        Self::new()
    }
}
