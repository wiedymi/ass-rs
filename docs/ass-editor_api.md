# `ass-editor` API Documentation

Welcome to the comprehensive API documentation for `ass-editor`, a high-performance, ergonomic editing layer for ASS (Advanced SubStation Alpha) subtitles. This document provides a detailed overview of the crate's public API, designed to be friendly for both human developers and Large Language Models (LLMs).

## 1. Introduction

`ass-editor` is built on top of `ass-core` and provides a rich set of tools for creating, editing, and manipulating ASS subtitle files programmatically. It is designed for performance, with features like incremental parsing and zero-copy operations, while offering a high-level, fluent API for ease of use.

**Key Features:**

-   **Interactive Editing**: A robust API for text manipulation, including undo/redo history.
-   **ASS-Aware Operations**: Directly interact with ASS structures like events, styles, and script info without manual parsing.
-   **Full Format Support**: Complete support for both ASS v4+ and v4++ formats with automatic format detection.
-   **Fluent API**: An ergonomic, chainable API for performing complex edits with simple code.
-   **Powerful Event Querying**: A flexible system for filtering, sorting, and accessing events based on various criteria.
-   **Extensible Command System**: A solid foundation for building custom operations and editor functionality.
-   **Builders for ASS Types**: Easily construct valid ASS events and styles programmatically for both v4+ and v4++ formats.
-   **Format Conversion**: Built-in support for importing from and exporting to other popular subtitle formats like SRT and WebVTT.

---

## 2. Core Concepts

The `ass-editor` crate revolves around a few central types:

-   **`EditorDocument`**: The main container for an ASS script. It holds the text content and provides all methods for editing and querying.
-   **`Position`**: Represents a specific location within the document, measured as a byte offset from the beginning.
-   **`Range`**: Represents a span of text between two `Position`s. Most editing operations target a `Range`.
-   **Commands**: All modifications are performed via commands (e.g., `InsertTextCommand`, `SplitEventCommand`). This ensures that every operation is atomic and can be undone.
-   **Fluent API**: A set of builder-like methods attached to `EditorDocument` that allow for intuitive, chainable operations (e.g., `doc.events().query().filter_by_style("Default").execute()?`).

---

## 3. Getting Started

Here’s a quick example of how to load, edit, and save an ASS document.

```rust
use ass_editor::{EditorDocument, Position, Range};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a document from string content.
    let mut doc = EditorDocument::from_content(r#"
[Script Info]
Title: My First Subtitle

[Events]
Format: Layer, Start, End, Style, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Hello World
"#)?;

    // 2. Perform a simple text insertion.
    let pos = Position::new(doc.text().find("World").unwrap_or(0));
    doc.insert(pos, "Brave New ")?;

    assert!(doc.text().contains("Hello Brave New World"));

    // 3. Undo the last operation.
    doc.undo()?;
    assert_eq!(doc.text().contains("Hello Brave New World"), false);

    // 4. Redo the operation.
    doc.redo()?;
    assert!(doc.text().contains("Hello Brave New World"));

    println!("Document edited successfully!");
    Ok(())
}
```

---

## 4. `EditorDocument` API

The `EditorDocument` is the heart of the editor.

### Creation and I/O

-   **`EditorDocument::new()`**: Creates a new, empty document.
-   **`EditorDocument::from_content(content: &str)`**: Creates a document from a string slice.
-   **`EditorDocument::from_file(path: &str)`** (`std` feature): Creates a document by reading from a file path.
-   **`save(&mut self)`** (`std` feature): Saves the document to its original file path.
-   **`save_to_file(&mut self, path: &str)`** (`std` feature): Saves the document to a new file path.

### Text and Content Access

-   **`text(&self) -> String`**: Returns the full text content of the document.
-   **`text_range(&self, range: Range) -> Result<String>`**: Returns the text within a specific range.
-   **`len_bytes(&self) -> usize`**: Returns the total length of the document in bytes.
-   **`len_lines(&self) -> usize`**: Returns the total number of lines in the document.
-   **`is_empty(&self) -> bool`**: Checks if the document is empty.

### Basic Editing

These methods form the low-level foundation for edits and include undo/redo support.

-   **`insert(&mut self, pos: Position, text: &str) -> Result<()>`**: Inserts text at a given position.
-   **`delete(&mut self, range: Range) -> Result<()>`**: Deletes text within a given range.
-   **`replace(&mut self, range: Range, text: &str) -> Result<()>`**: Replaces the text in a range with new text.

### History Management

-   **`undo(&mut self) -> Result<CommandResult>`**: Undoes the last modification.
-   **`redo(&mut self) -> Result<CommandResult>`**: Redoes the last undone modification.
-   **`can_undo(&self) -> bool`**: Checks if there are operations to undo.
-   **`can_redo(&self) -> bool`**: Checks if there are operations to redo.
-   **`undo_manager_mut(&mut self) -> &mut UndoManager`**: Gets mutable access to the undo manager for configuration.

### ASS-Aware Accessors

These methods provide direct access to parsed ASS data without manual parsing.

-   **`events_count(&self) -> Result<usize>`**: Gets the number of events in the `[Events]` section.
-   **`styles_count(&self) -> Result<usize>`**: Gets the number of styles in the `[V4+ Styles]` section.
-   **`script_info_fields(&self) -> Result<Vec<String>>`**: Gets a list of all field names in the `[Script Info]` section.
-   **`get_script_info_field(&self, key: &str) -> Result<Option<String>>`**: Gets the value of a specific script info field.
-   **`set_script_info_field(&mut self, key: &str, value: &str) -> Result<()>`**: Sets the value of a script info field.

---

## 5. Fluent API

For more ergonomic editing, `ass-editor` provides a fluent, chainable API.

### Position-Based Operations: `at_pos()`

-   **`doc.at_pos(position)`**: Starts a fluent operation at a specific `Position`.
    -   **.insert_text(text: &str)**: Inserts text.
    -   **.delete(count: usize)**: Deletes a number of characters forward.
    -   **.backspace(count: usize)**: Deletes characters backward.

```rust
# use ass_editor::{EditorDocument, Position};
# let mut doc = EditorDocument::from_content("Hello World").unwrap();
doc.at_pos(Position::new(5))
    .insert_text(" Beautiful")?;
assert_eq!(doc.text(), "Hello Beautiful World");
# Ok::<(), ass_editor::EditorError>(())
```

### Range-Based Operations: `select()`

-   **`doc.select(range)`**: Starts a fluent operation on a `Range`.
    -   **.replace_with(text: &str)**: Replaces the selected text.
    -   **.delete()**: Deletes the selected text.
    -   **.wrap_with_tag(open: &str, close: &str)**: Wraps the selection with tags.
    -   **.text() -> String**: Gets the selected text.

```rust
# use ass_editor::{EditorDocument, Position, Range};
# let mut doc = EditorDocument::from_content("Hello World").unwrap();
let range = Range::new(Position::new(6), Position::new(11));
doc.select(range)
    .replace_with("Rust")?;
assert_eq!(doc.text(), "Hello Rust");
# Ok::<(), ass_editor::EditorError>(())
```

### Style Operations: `styles()`

-   **`doc.styles()`**: Accesses the style management API.
    -   **.create(name: &str, builder: StyleBuilder)**: Creates a new style.
    -   **.edit(name: &str) -> StyleEditor`**: Starts a fluent editor for an existing style.
    -   **.delete(name: &str)**: Deletes a style.
    -   **.clone(source: &str, target: &str)**: Clones a style.
    -   **.apply(old_style: &str, new_style: &str) -> StyleApplicator`**: Applies a new style to events using an old style.

```rust
# use ass_editor::{EditorDocument, StyleBuilder};
# let mut doc = EditorDocument::from_content("[V4+ Styles]\nStyle: Default,Arial,20,,,,,,,,,,,,,,,,,,,,,,\n[Events]\nDialogue: 0,0:00:01.00,0:00:05.00,Default,Hello").unwrap();
doc.styles()
    .edit("Default")
    .font("Comic Sans MS")
    .size(24)
    .bold(true)
    .apply()
    .unwrap();

// For v4++ format with separate top/bottom margins

```

### Event Operations: `events()`

-   **`doc.events()`**: Accesses the event management and querying API. This is the entry point for the powerful event query system.

### Tag Operations: `tags()`

-   **`doc.tags()`**: Accesses the override tag management API.
    -   **.at(position).insert(tag: &str)**: Inserts a tag.
    -   **.in_range(range).remove_all()**: Removes all tags in a range.
    -   **.in_range(range).replace_all(find: &str, replace: &str)**: Replaces tags.
    -   **.in_range(range).wrap(open_tag: &str)**: Wraps text with tags.

```rust
# use ass_editor::{EditorDocument, Position, Range};
# let mut doc = EditorDocument::from_content("Hello World").unwrap();
let range = Range::new(Position::new(6), Position::new(11));
doc.select(range)
    .wrap_with_tag("{\\b1}", "{\\b0}")?;
assert_eq!(doc.text(), "Hello {\\b1}World{\\b0}");
# Ok::<(), ass_editor::EditorError>(())
```

### Karaoke Operations: `karaoke()`

-   **`doc.karaoke()`**: Accesses the karaoke timing API.
    -   **.in_range(range).generate(duration: u32)**: Generates karaoke timing.
    -   **.in_range(range).adjust().scale(factor: f32)**: Adjusts existing timings.

---

## 6. Event Querying API

The `events()` fluent API provides a powerful way to filter, sort, and access events without manual parsing.

### Basic Querying

-   **`doc.events().all() -> Result<Vec<EventInfo>>`**: Gets all events.
-   **`doc.events().get(index) -> Result<Option<EventInfo>>`**: Gets a single event by its index.
-   **`doc.events().count() -> Result<usize>`**: Gets the total number of events.

### Filtering

You can chain multiple filters together.

-   **`.query().filter_by_type(EventType)`**: Filters by `Dialogue` or `Comment`.
-   **`.query().filter_by_style(pattern: &str)`**: Filters by style name.
-   **`.query().filter_by_speaker(pattern: &str)`**: Filters by speaker/actor name.
-   **`.query().filter_by_text(pattern: &str)`**: Filters by text content.
-   **`.query().filter_by_time_range(start_cs: u32, end_cs: u32)`**: Filters by a time range in centiseconds.
-   **`.query().case_sensitive(bool)`**: Toggles case sensitivity for pattern matching.

### Sorting

-   **`.query().sort(criteria: EventSortCriteria)`**: Sorts results. `EventSortCriteria` can be `StartTime`, `EndTime`, `Duration`, `Style`, `Speaker`, `Layer`, `Index`, or `Text`.
-   **`.query().descending()`**: Sorts in descending order.
-   **`.query().then_by(criteria: EventSortCriteria)`**: Adds a secondary sort criterion.

### Executing Queries

-   **`.execute() -> Result<Vec<EventInfo>>`**: Executes the query and returns a vector of `EventInfo` structs.
-   **`.indices() -> Result<Vec<usize>>`**: Returns just the indices of the matching events.
-   **`.first() -> Result<Option<EventInfo>>`**: Returns the first matching event.

### Example: Complex Query

```rust
# use ass_editor::{EditorDocument, EventType, EventSortCriteria};
# let content = "[Events]\nDialogue: 0,0:00:05.50,0:00:08.00,Default,Alice,0,0,0,,Hello everyone\nDialogue: 0,0:00:12.50,0:00:15.00,Default,Alice,0,0,0,,I'm doing great, thanks!";
# let mut doc = EditorDocument::from_content(content).unwrap();
// Find all dialogue lines by "Alice", sort them by start time, and get the results.
let alice_lines = doc.events()
    .query()
    .filter_by_type(EventType::Dialogue)
    .filter_by_speaker("Alice")
    .sort(EventSortCriteria::StartTime)
    .execute()
    .unwrap();

assert_eq!(alice_lines.len(), 2);
assert_eq!(alice_lines[0].event.text, "Hello everyone");
```

---

## 7. Command System

For advanced use cases, you can work with the command system directly. This is useful for building complex, undoable operations.

-   **`InsertTextCommand`**, **`DeleteTextCommand`**, **`ReplaceTextCommand`**: Basic text operations.
-   **`SplitEventCommand`**, **`MergeEventsCommand`**, **`TimingAdjustCommand`**: Event-specific commands.
-   **`CreateStyleCommand`**, **`EditStyleCommand`**, **`DeleteStyleCommand`**: Style commands.
-   **`BatchCommand`**: Groups multiple commands into a single, atomic, undoable operation.

```rust
use ass_editor::{EditorDocument, Position, InsertTextCommand, BatchCommand, EditorCommand};

let mut doc = EditorDocument::new();

// Create a batch of commands
let batch = BatchCommand::new("Initial setup".to_string())
    .add_command(Box::new(InsertTextCommand::new(
        Position::new(0),
        "[Script Info]\n".to_string(),
    )))
    .add_command(Box::new(InsertTextCommand::new(
        Position::new(14),
        "[Events]\n".to_string(),
    )));

// Execute the batch command
batch.execute(&mut doc).unwrap();

assert!(doc.text().contains("[Script Info]"));
assert!(doc.text().contains("[Events]"));
```

---

## 8. Builders

Programmatically create valid ASS elements with fluent builders.

### `EventBuilder`

```rust
use ass_editor::EventBuilder;
use ass_core::ScriptVersion;

// v4+ format event
let event_line_v4plus = EventBuilder::dialogue()
    .start_time("0:00:10.00")
    .end_time("0:00:15.00")
    .style("Default")
    .speaker("API Demo")
    .margin_vertical(10) // Single vertical margin for v4+
    .text("This event was created by a builder!")
    .build()
    .unwrap();

// v4++ format event with separate top/bottom margins
let event_line_v4plusplus = EventBuilder::dialogue()
    .start_time("0:00:10.00")
    .end_time("0:00:15.00")
    .style("Default")
    .speaker("API Demo")
    .margin_top(15) // Top margin for v4++
    .margin_bottom(20) // Bottom margin for v4++
    .text("This event was created by a builder!")
    .build_with_version(ScriptVersion::AssV4Plus) // Build with v4++ format
    .unwrap();
```

### `StyleBuilder`

```rust
use ass_editor::StyleBuilder;
use ass_core::ScriptVersion;

// v4+ format style
let style_line_v4plus = StyleBuilder::new()
    .name("MyCustomStyle")
    .font("Roboto")
    .size(28)
    .color("&H00A0FF")
    .bold(true)
    .outline(1.5)
    .shadow(0.5)
    .margin_vertical(15) // Single vertical margin for v4+
    .build()
    .unwrap();

// v4++ format style with enhanced features
let style_line_v4plusplus = StyleBuilder::new()
    .name("MyAdvancedStyle")
    .font("Roboto")
    .size(28)
    .color("&H00A0FF")
    .bold(true)
    .outline(1.5)
    .shadow(0.5)
    .margin_top(20) // Separate top margin for v4++
    .margin_bottom(25) // Separate bottom margin for v4++
    .relative_to("video") // Positioning context for v4++
    .build_with_version(ScriptVersion::AssV4Plus) // Build with v4++ format
    .unwrap();
```

---

## 9. Utilities

### Validator

-   **`doc.validate()`**: Performs a basic parsing check.
-   **`doc.validate_comprehensive()`**: Performs a full linting and analysis, returning detailed issues.

### Search

`ass-editor` includes a powerful search system accessible via `utils::search::create_search()`.

### Format Converter

-   **`utils::formats::FormatConverter::import(content, format)`**: Imports content from another format into an ASS string.
-   **`utils::formats::FormatConverter::export(document, format, options)`**: Exports an `EditorDocument` to another format.

---

## 10. Format Detection and Automatic Handling

`ass-editor` automatically detects whether a script uses v4+ or v4++ format and handles both seamlessly:

```rust
use ass_editor::EditorDocument;

// Load a v4+ script - automatically detected
let doc_v4plus = EditorDocument::from_content(r#"
[Script Info]
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&HFFFFFF,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,15,1
"#).unwrap();

// Load a v4++ script - automatically detected
let doc_v4plusplus = EditorDocument::from_content(r#"
[Script Info]
ScriptType: v4.00++

[V4++ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginT, MarginB, Encoding, RelativeTo
Style: Default,Arial,20,&HFFFFFF,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,15,25,1,0
"#).unwrap();

// Both formats work transparently
assert!(doc_v4plus.text().contains("ScriptType: v4.00+"));
assert!(doc_v4plusplus.text().contains("ScriptType: v4.00++"));
```

## 11. Error Handling

`ass-editor` uses a structured `EditorError` enum for all fallible operations. It wraps `ass-core::CoreError` and adds editor-specific error types. All functions that can fail return a `Result<T, EditorError>`.

---

## 12. Feature Flags

-   **`full`** (default): Enables all features for a rich desktop editor experience.
-   **`minimal`**: A lightweight, `no_std`-compatible core for embedded use or simple tools.
-   **`stream`**: Enables incremental parsing for high-performance editing.
-   **`formats`**: Enables import/export functionality for SRT, WebVTT, etc.
-   **`search-index`**: Enables the high-performance FST-based search index.
-   **`concurrency`**: Enables thread-safe types like `SyncDocument` and `EditorSessionManager`.

This documentation provides a thorough overview of the `ass-editor` API. For more details on specific functions, please refer to the inline Rustdoc comments in the source code.