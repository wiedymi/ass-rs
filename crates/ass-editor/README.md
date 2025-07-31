# ASS-Editor

[![Crates.io](https://img.shields.io/crates/v/ass-editor.svg)](https://crates.io/crates/ass-editor)  
[![Documentation](https://docs.rs/ass-editor/badge.svg)](https://docs.rs/ass-editor)  
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

A high-performance, ergonomic editor layer for ASS (Advanced SubStation Alpha) subtitles, built on top of `ass-core`. Designed for interactive subtitle editing with zero-copy efficiency, incremental updates, and comprehensive command support.

## 🚀 Key Features

- **📝 Interactive Editing**: Fluent API with undo/redo, multi-document sessions, and incremental parsing
- **⚡ High Performance**: <1ms edits, <5ms re-parses, zero-copy spans from ass-core
- **🔍 Advanced Search**: FST-based indexing for fast regex queries across large scripts
- **🔌 Extensible**: Plugin system for custom commands, syntax highlighting, and auto-completion
- **🧵 Thread-Safe**: Optional multi-threading support with Arc/Mutex (feature-gated)
- **📦 Zero Dependencies**: Core functionality with minimal external dependencies
- **🌐 Platform Support**: Native, WASM, and no_std compatibility

## 📋 Performance Targets

- **Edit Operations**: <1ms for single-event modifications
- **Incremental Parsing**: <5ms for typical script changes
- **Memory Usage**: ~1.2x input size (including undo history)
- **Session Switching**: <100µs between documents
- **Search Queries**: <10ms for regex across 1000+ events

## 🎯 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
ass-editor = "0.1"

# With additional features
ass-editor = { version = "0.1", features = ["search-index", "plugins"] }
```

### Basic Usage

```rust
use ass_editor::{EditorDocument, Position, Range};

// Create a new document
let mut doc = EditorDocument::from_content(r#"
[Script Info]
Title: My Subtitle

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World
"#).unwrap();

// Basic text operations
let pos = Position::new(200);
doc.insert_at(pos, " there").unwrap();

// Undo/redo support
doc.undo().unwrap();
doc.redo().unwrap();

println!("Content: {}", doc.text());
```

### Fluent API

```rust
use ass_editor::{EditorDocument, Position, Range, commands::*};

let mut doc = EditorDocument::new();

// Fluent command chaining
doc.at_pos(Position::new(0))
    .insert_text("Hello ")
    .and_then(|_| doc.at_pos(Position::new(6)))
    .and_then(|_| doc.insert_text("World"))
    .unwrap();

// Range operations
let range = Range::new(Position::new(0), Position::new(5));
doc.command()
    .range(range)
    .replace("Hi")
    .unwrap();

assert_eq!(doc.text(), "Hi World");
```

### Style and Event Management

```rust
use ass_editor::{EditorDocument, StyleBuilder, EventBuilder};

let mut doc = EditorDocument::from_content("[V4+ Styles]\n[Events]\n").unwrap();

// Create and apply styles
let style = StyleBuilder::new("Title")
    .fontname("Arial")
    .fontsize(24)
    .primary_colour("&H00FFFFFF")
    .build();

doc.styles()
    .create(style)
    .execute()
    .unwrap();

// Create events with the fluent API
doc.events()
    .create_dialogue()
    .start_time("0:00:00.00")
    .end_time("0:00:05.00")
    .style("Title")
    .text("Hello World")
    .execute()
    .unwrap();
```

### Karaoke Support

```rust
use ass_editor::{EditorDocument, Position, Range, commands::karaoke_commands::*};

let mut doc = EditorDocument::from_content("Hello World").unwrap();
let range = Range::new(Position::new(0), Position::new(11));

// Generate karaoke timing
doc.karaoke()
    .in_range(range)
    .generate(50) // 50 centiseconds per syllable
    .karaoke_type(KaraokeType::Fill)
    .execute()
    .unwrap();

// Adjust existing karaoke timing
doc.karaoke()
    .in_range(range)
    .adjust()
    .scale(1.5) // Make 50% longer
    .execute()
    .unwrap();
```

### Search and Indexing

```rust
use ass_editor::{EditorDocument, utils::search::*};

let mut doc = EditorDocument::from_content("Large subtitle script...").unwrap();

// Build search index for fast queries
doc.build_search_index().unwrap();

// Fast regex search
let results = doc.search()
    .pattern(r"\\b\d+")  // Find bold tags with numbers
    .case_insensitive()
    .execute()
    .unwrap();

for result in results {
    println!("Match at {}: {}", result.position(), result.text());
}
```

### Multi-Document Sessions

```rust
use ass_editor::{EditorSessionManager, SessionConfig};

let mut manager = EditorSessionManager::new(SessionConfig::default());

// Create multiple sessions
let session1 = manager.create_session("subtitle1.ass").unwrap();
let session2 = manager.create_session("subtitle2.ass").unwrap();

// Switch between sessions with shared resources
manager.activate_session(&session1).unwrap();
// Edit session1...

manager.activate_session(&session2).unwrap();  
// Edit session2...

// Sessions share extension registry and memory pools
```

## 🔧 Feature Flags

ass-editor uses feature flags to enable optional functionality:

### Core Features (enabled by default)
- **`minimal`**: Core editing with rope, arena, and stream support (no_std compatible)
- **`full`**: All features including std, analysis, plugins, formats, search, concurrency, serde

### Optional Features
- **`std`**: Standard library support (required for most features)
- **`analysis`**: Script analysis and linting integration from ass-core
- **`plugins`**: Extension system with syntax highlighting and auto-completion
- **`search-index`**: FST-based advanced search indexing
- **`formats`**: Import/export support for SRT, WebVTT formats
- **`serde`**: Serialization support for editor state
- **`concurrency`**: Multi-threading and async support
- **`simd`**: SIMD acceleration for parsing performance
- **`stream`**: Incremental parsing for large files

### Platform Features
- **`nostd`**: No-standard library support for embedded/WASM
- **`dev-benches`**: Development benchmarking

### Usage Examples

```toml
# Minimal editor for lightweight integrations
ass-editor = { version = "0.1", default-features = false, features = ["minimal"] }

# Full-featured desktop editor
ass-editor = { version = "0.1", features = ["full", "simd"] }

# WASM/embedded build  
ass-editor = { version = "0.1", default-features = false, features = ["minimal", "nostd"] }

# Server-side processing
ass-editor = { version = "0.1", features = ["full", "concurrency", "formats"] }
```

## 🏗️ Architecture

ASS-Editor is built in layers:

```
┌─────────────────────────────────────────────────────────┐
│                    User Applications                     │
├─────────────────────────────────────────────────────────┤
│  EditorSessionManager  │  Multi-document Management     │
├─────────────────────────────────────────────────────────┤
│      EditorDocument     │  Single Document Editing      │
├─────────────────────────────────────────────────────────┤
│  Commands │ Extensions │ Events │ Search │ Validation   │
├─────────────────────────────────────────────────────────┤
│                      ass-core                           │
│        (Zero-copy parsing & AST manipulation)           │
└─────────────────────────────────────────────────────────┘
```

### Key Components

- **EditorDocument**: Core document wrapper around ass-core::Script
- **Commands**: Undoable operations (text edits, style changes, event management)
- **Extensions**: Plugin system for syntax highlighting, auto-completion
- **Events**: Reactive event system for UI updates and notifications
- **Search**: FST-based indexing for fast regex queries
- **Sessions**: Multi-document management with shared resources

## 🎮 Command System

All operations in ass-editor go through a command system that provides:

- **Undo/Redo**: Every operation is reversible
- **Batching**: Multiple operations can be grouped atomically
- **Progress Tracking**: Long operations report progress
- **Validation**: Commands validate their inputs before execution

### Available Commands

| Category | Commands | Description |
|----------|----------|-------------|
| **Text** | Insert, Delete, Replace | Basic text operations |
| **Events** | Create, Split, Merge, Timing | Event management |
| **Styles** | Create, Edit, Delete, Clone, Apply | Style operations |
| **Tags** | Insert, Remove, Replace, Wrap, Parse | Override tag handling |
| **Karaoke** | Generate, Split, Adjust, Apply | Karaoke timing management |

## 🔍 Search System

ASS-Editor provides powerful search capabilities:

- **Text Search**: Simple string matching with case sensitivity options
- **Regex Search**: Full regex support with capture groups
- **Tag Search**: Search within override tags and parameters
- **Event Search**: Search by timing, style, or other event properties
- **Indexed Search**: FST-based indexing for large scripts (1000+ events)

## 🧪 Testing

Run the comprehensive test suite:

```bash
# Unit tests
cargo test

# Integration tests  
cargo test --test integration

# Performance tests
cargo test --test performance_targets

# All features
cargo test --all-features

# Specific feature combinations
cargo test --no-default-features --features minimal
```

## 📊 Benchmarks

Run performance benchmarks:

```bash
# All benchmarks
cargo bench

# Specific benchmarks
cargo bench --bench editor_commands
cargo bench --bench search_performance
cargo bench --bench memory_usage
```

Expected performance on typical subtitle files:

| Operation | Target | Typical Result |
|-----------|--------|----------------|
| Document creation | <5ms | ~2ms |
| Single edit | <1ms | ~0.3ms |
| Undo/redo | <1ms | ~0.2ms |
| Search (indexed) | <10ms | ~3ms |
| Session switch | <100µs | ~50µs |

## 🤝 Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/your-org/ass-rs.git
cd ass-rs/crates/ass-editor

# Run tests
cargo test --all-features

# Run benchmarks
cargo bench

# Check code quality
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## 📄 License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

## 🔗 Related Crates

- **[ass-core](../ass-core/)**: Zero-copy ASS parsing and analysis
- **[ass-renderer](../ass-renderer/)**: High-performance subtitle rendering
- **[ass-cli](../ass-cli/)**: Command-line tools for subtitle processing
- **[ass-wasm](../ass-wasm/)**: WebAssembly bindings for browser use

---

**Built with ❤️ in Rust for subtitle editors worldwide**