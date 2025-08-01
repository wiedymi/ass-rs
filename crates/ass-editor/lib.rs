//! High-performance, ergonomic editor layer for ASS subtitles
//!
//! `ass-editor` provides an interactive editing interface built on top of `ass-core`,
//! featuring zero-copy efficiency, incremental updates, and multi-document support.
//! Designed for building subtitle editors, conversion tools, and automation scripts.
//!
//! # Key Features
//!
//! ## 📝 Interactive Editing
//! - **Undo/redo**: Full history management with configurable depth
//! - **Multi-document sessions**: Manage multiple files with shared resources
//! - **Incremental parsing**: <1ms edits, <5ms re-parses via ass-core's streaming parser
//! - **Fluent APIs**: Ergonomic builders and method chaining for all operations
//!
//! ## ⚡ High Performance
//! - **Zero-copy editing**: Direct manipulation of ass-core's zero-copy spans
//! - **Memory efficient**: ~1.2x input size including undo history with arena pooling
//! - **SIMD acceleration**: Optional SIMD features for parsing performance
//! - **Thread-safe**: Optional multi-threading support with Arc/Mutex
//!
//! ## 🔍 Advanced Features
//! - **Search indexing**: FST-based trie indexing for fast regex queries across large scripts
//! - **Plugin system**: Extensible architecture with syntax highlighting and auto-completion
//! - **Format support**: Import/export SRT, WebVTT with configurable conversion options
//! - **Karaoke support**: Generate, split, adjust, and apply karaoke timing with syllable detection
//!
//! # Quick Start
//!
//! ```
//! use ass_editor::{EditorDocument, Position, Range};
//!
//! // Create a new document with ASS content
//! let mut doc = EditorDocument::from_content(r#"
//! [Script Info]
//! Title: My Subtitle
//!
//! [V4+ Styles]
//! Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
//! Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
//!
//! [Events]
//! Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
//! Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World
//! "#).unwrap();
//!
//! // Basic text editing with position-based operations
//! let pos = Position::new(doc.text().len() - 11); // Before "Hello World"
//! doc.insert(pos, "Welcome! ").unwrap();
//!
//! // Range-based operations
//! let range = Range::new(Position::new(pos.offset), Position::new(pos.offset + 8));
//! doc.replace(range, "Hi").unwrap();
//!
//! // Undo/redo support
//! assert!(doc.can_undo());
//! doc.undo().unwrap();
//! assert!(doc.can_redo());
//! doc.redo().unwrap();
//!
//! println!("Final text contains: {}", doc.text().contains("Hi World"));
//! ```
//!
//! # Advanced Usage Examples
//!
//! ## Style Management
//!
//! ```
//! # use ass_editor::{EditorDocument, StyleBuilder};
//! let mut doc = EditorDocument::new();
//!
//! // Create styles with builder pattern
//! let style = StyleBuilder::new()
//!     .font("Arial")
//!     .size(24)
//!     .color("&H00FFFFFF")
//!     .bold(true);
//!
//! // Apply styles to document
//! doc.styles().create("Title", style).unwrap();
//! ```
//!
//! ## Karaoke Timing
//!
//! ```
//! # use ass_editor::{EditorDocument, Position, Range, commands::karaoke_commands::*};
//! let mut doc = EditorDocument::from_content("Karaoke text here").unwrap();
//! let range = Range::new(Position::new(0), Position::new(17));
//!
//! // Generate karaoke with automatic syllable detection
//! doc.karaoke()
//!     .in_range(range)
//!     .generate(50) // 50 centiseconds per syllable
//!     .karaoke_type(KaraokeType::Fill)
//!     .execute()
//!     .unwrap();
//!
//! // Adjust timing
//! doc.karaoke()
//!     .in_range(range)
//!     .adjust()
//!     .scale(1.5) // Make 50% longer
//!     .unwrap();
//! ```
//!
//! ## Multi-Document Sessions
//!
//! ```
//! # use ass_editor::{EditorSessionManager, SessionConfig};
//! let mut manager = EditorSessionManager::new();
//!
//! manager.create_session("subtitle1.ass".to_string()).unwrap();
//! manager.create_session("subtitle2.ass".to_string()).unwrap();
//!
//! // Switch between sessions with <100µs overhead
//! manager.switch_session("subtitle1.ass").unwrap();
//! // Edit session1...
//!
//! manager.switch_session("subtitle2.ass").unwrap();
//! // Edit session2 with shared plugins and memory pools...
//! ```
//!
//! # Feature Flags
//!
//! ass-editor uses a two-tier feature system for maximum flexibility:
//!
//! ## Main Flavors
//! - **`default`**: Enables `full` for complete desktop functionality
//! - **`minimal`**: Core editing features - no_std compatible with alloc
//! - **`full`**: All features including std, analysis, plugins, formats, search, concurrency
//!
//! ## Optional Features
//! - **`simd`**: SIMD acceleration for parsing performance
//! - **`nostd`**: No-standard library support for embedded/WASM
//! - **`dev-benches`**: Development benchmarking
//!
//! ## Usage Examples
//!
//! ```toml
//! # Full-featured desktop editor (default)
//! ass-editor = "0.1"
//!
//! # Minimal editor for lightweight integrations
//! ass-editor = { version = "0.1", default-features = false, features = ["minimal"] }
//!
//! # Maximum performance with SIMD
//! ass-editor = { version = "0.1", features = ["full", "simd"] }
//!
//! # WASM/embedded build
//! ass-editor = { version = "0.1", default-features = false, features = ["minimal", "nostd"] }
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
    UndoStackConfig, EventInfo, EventSortCriteria, EventSortOptions, EventAccessor, 
    EventQuery, OwnedEvent,
};
// Re-export the fluent EventFilter directly
pub use core::fluent::EventFilter;
// Re-export the events EventFilter with a specific name to avoid conflict  
pub use events::{
    DocumentEvent, EventChannel, EventChannelConfig, EventFilter as DocumentEventFilter, 
    EventHandler, EventStats,
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
