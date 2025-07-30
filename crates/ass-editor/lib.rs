//! High-performance, ergonomic editor layer for ASS subtitles
//!
//! `ass-editor` provides an interactive editing interface built on top of `ass-core`,
//! featuring zero-copy efficiency, incremental updates, and multi-document support.
//!
//! # Features
//!
//! - **Zero-copy editing**: Uses borrowed spans from core with optional rope for text
//! - **Incremental parsing**: <1ms edits, <5ms re-parses via core's partial parse
//! - **Multi-document sessions**: Manage multiple documents with shared resources
//! - **Fluent APIs**: Ergonomic builders and method chaining for commands
//! - **Undo/redo**: History management with configurable depth and arena pooling
//! - **Search indexing**: FST-based trie indexing for fast regex queries
//! - **Plugin system**: Extends core's plugin system with editor-specific hooks
//! - **Thread-safe**: Optional multi-threading support with Arc/Mutex
//!
//! # Example
//!
//! ```
//! use ass_editor::{EditorDocument, Position};
//!
//! // Create a new document with ASS content
//! let mut doc = EditorDocument::from_content(
//!     "[Script Info]\nTitle: My Subtitle\n\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World"
//! ).unwrap();
//!
//! // Edit at specific position
//! let pos = Position::new(50); // Position in the text
//! doc.insert(pos, " there").unwrap();
//!
//! // Undo last operation
//! doc.undo().unwrap();
//!
//! assert!(doc.text().contains("Hello World"));
//! assert!(!doc.text().contains("Hello there World"));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod commands;
pub mod core;
pub mod events;
pub mod extensions;
pub mod utils;

#[cfg(feature = "std")]
pub mod sessions;

#[cfg(feature = "std")]
pub mod formats;

// Re-export ass-core types as first-class citizens
pub use ass_core::parser::ast::{Event, EventType, ScriptInfo, Section, SectionType, Span, Style};
pub use ass_core::parser::Script;

// Public API exports
pub use commands::{
    BatchCommand, CommandResult, DeleteTextCommand, DocumentCommandExt, EditorCommand,
    InsertTextCommand, ReplaceTextCommand, TextCommand,
};
pub use core::{
    DocumentPosition, EditorDocument, EditorError, EventBuilder, HistoryEntry, HistoryStats,
    Position, PositionBuilder, Range, Result, Selection, StyleBuilder, UndoManager, UndoStack,
    UndoStackConfig,
};
pub use events::{
    DocumentEvent, EventChannel, EventChannelConfig, EventFilter, EventHandler, EventStats,
};
pub use extensions::{
    EditorContext, EditorExtension, ExtensionCapability, ExtensionCommand, ExtensionContext,
    ExtensionInfo, ExtensionManager, ExtensionResult, ExtensionState, MessageLevel,
};

pub use utils::{
    LazyValidator, ValidationIssue, ValidationResult, ValidationSeverity, ValidatorConfig,
};

#[cfg(feature = "std")]
pub use sessions::{EditorSession, EditorSessionManager, SessionConfig, SessionStats};

#[cfg(feature = "std")]
pub use formats::{
    Format, FormatExporter, FormatImporter, FormatInfo, FormatOptions, FormatRegistry, FormatResult,
};
