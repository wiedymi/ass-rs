# `ass-core` API Documentation

Welcome to the comprehensive API documentation for `ass-core`, the foundational crate of the `ass-rs` project. This document provides a detailed overview of the crate's public API, designed to be friendly for both human developers and Large Language Models (LLMs).

## 1. Overview

`ass-core` is a high-performance, memory-efficient Rust crate for parsing, analyzing, and manipulating ASS (Advanced SubStation Alpha) and SSA (SubStation Alpha) subtitle scripts. It is designed with a zero-copy architecture to minimize memory allocations and provide a thread-safe, immutable AST (Abstract Syntax Tree) for safe and efficient subtitle processing.

**Key Features:**

*   **Zero-Copy Parsing**: Utilizes `&str` spans to reference the original source text, avoiding costly allocations.
*   **Full ASS/SSA Compatibility**: Supports ASS v4.00+, ASS v4.00++, SSA v4, and libass extensions.
*   **Comprehensive Analysis**: Provides tools for style resolution, timing validation, and performance analysis.
*   **Extensible Linting**: A configurable, rule-based linting engine to detect common issues and spec violations.
*   **Plugin System**: Extensible architecture for custom tag handlers and section processors.

## 2. Core Concepts

The `ass-core` crate revolves around a few central data structures that represent the components of an ASS script.

### `Script`

The `Script` struct is the top-level container for a parsed ASS file. It holds all the sections of the script and provides methods for accessing and analyzing its content.

-   **`Script::parse(source: &str) -> Result<Script>`**: The primary entry point for parsing an ASS script from a string slice.
-   **`script.sections()`**: Returns a slice of `Section` enums, representing the different parts of the script.
-   **`script.issues()`**: Returns a slice of `ParseIssue` structs, detailing any warnings or recoverable errors encountered during parsing.

### `Section`

The `Section` enum represents the different sections of an ASS script, such as `[Script Info]`, `[V4+ Styles]`, and `[Events]`.

-   `Section::ScriptInfo(ScriptInfo)`: Contains script metadata.
-   `Section::Styles(Vec<Style>)`: Contains style definitions.
-   `Section::Events(Vec<Event>)`: Contains dialogue, comments, and other timed events.
-   `Section::Fonts(Vec<Font>)`: Contains embedded font data.
-   `Section::Graphics(Vec<Graphic>)`: Contains embedded graphic data.

### `Event`

The `Event` struct represents a single line in the `[Events]` section. This is typically a dialogue line, but can also be a comment, picture, sound, movie, or command.

-   **`event.event_type`**: An `EventType` enum (`Dialogue`, `Comment`, etc.).
-   **`event.start`**, **`event.end`**: The start and end timestamps of the event.
-   **`event.text`**: The text content of the event, including any override tags.
-   **`event.margin_v`**: Vertical margin override for v4+ format (pixels).
-   **`event.margin_t`**, **`event.margin_b`**: Top/bottom margin overrides for v4++ format (pixels).

### `Style`

The `Style` struct represents a single style definition from the `[V4+ Styles]` or `[V4++ Styles]` section. It defines the appearance of dialogue text, including font, colors, margins, and more.

-   **`style.name`**: The unique name of the style.
-   **`style.fontname`**, **`style.fontsize`**: Font properties.
-   **`style.primary_colour`**, **`style.secondary_colour`**, etc.: Color definitions.
-   **`style.margin_v`**: Single vertical margin for v4+ format (pixels).
-   **`style.margin_t`**, **`style.margin_b`**: Separate top/bottom margins for v4++ format (pixels).
-   **`style.relative_to`**: Positioning context for v4++ format (`0`=window, `1`=video, `2`=script).

## 3. Parsing a Script

Parsing is the first step in working with an ASS file. The `Script::parse` method provides a simple and efficient way to do this.

### Example: Basic Parsing

```rust
use ass_core::Script;
use ass_core::parser::ast::SectionType;

let script_text = r#"
[Script Info]
Title: Example Script
ScriptType: v4.00+

[Events]
Format: Layer, Start, End, Style, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,Hello World!
"#;

match Script::parse(script_text) {
    Ok(script) => {
        println!("Successfully parsed script!");
        assert!(script.find_section(SectionType::ScriptInfo).is_some());
        assert!(script.find_section(SectionType::Events).is_some());
    },
    Err(e) => {
        eprintln!("Failed to parse script: {}", e);
    }
}
```

## 4. Script Analysis

Once a script is parsed, you can perform a comprehensive analysis using the `ScriptAnalysis` struct. This provides resolved styles, timing information, and performance metrics.

### Example: Analyzing a Script

```rust
use ass_core::{Script, ScriptAnalysis};

fn get_script_text() -> &'static str {
    r#"[Script Info]
Title: Example Script
[V4+ Styles]
Format: Name, Fontname, Fontsize
Style: Default,Arial,20
[Events]
Format: Layer, Start, End, Style, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,Hello World!
"#
}

let script = Script::parse(get_script_text()).unwrap();
let analysis = ScriptAnalysis::analyze(&script).unwrap();

// Get resolved style information
if let Some(style) = analysis.resolve_style("Default") {
    println!("Default style font: {}, size: {}", style.font_name(), style.font_size());
}

// Get dialogue event analysis
for dialogue_info in analysis.dialogue_info() {
    println!(
        "Event text: '{}', Duration: {}ms",
        dialogue_info.text_analysis().plain_text(),
        dialogue_info.duration_ms()
    );
}
```

## 5. Linting

The `ass-core` crate includes a powerful linting engine to detect issues in your ASS scripts. You can configure the linter to check for specific problems.

### Example: Linting a Script

```rust
use ass_core::{Script, analysis::linting::{lint_script, LintConfig, IssueSeverity}};

fn get_script_with_issues() -> &'static str {
    r#"[Events]
Format: Layer, Start, End, Style, Text
Dialogue: 0,0:00:05.00,0:00:02.00,Default,Invalid timing!
"#
}

let script = Script::parse(get_script_with_issues()).unwrap();
let config = LintConfig {
    min_severity: IssueSeverity::Warning,
    ..Default::default()
};

let issues = lint_script(&script, &config).unwrap();
for issue in issues {
    println!("[{}] {}: {}", issue.severity(), issue.category(), issue.message());
}
```

## 6. API Reference

This section provides a detailed reference for the public API of `ass-core`.

### Module: `parser`

Contains the primary parsing structures and functions.

-   **`Script`**: The main struct representing a parsed script.
-   **`Section`**: An enum for the different sections of a script.
-   **`EventType`**: An enum for the different types of events.
-   **`Event`**: A struct for events in the `[Events]` section.
-   **`Style`**: A struct for styles in the `[V4+ Styles]` section.
-   **`ScriptInfo`**: A struct for the `[Script Info]` section.
-   **`Font`**: A struct for embedded fonts.
-   **`Graphic`**: A struct for embedded graphics.

### Module: `analysis`

Contains tools for script analysis.

-   **`ScriptAnalysis`**: The main analysis struct.
-   **`DialogueInfo`**: Detailed analysis of a single dialogue event.
-   **`TextAnalysis`**: Analysis of the text content of an event.
-   **`ResolvedStyle`**: A fully resolved style with all properties computed.

### Module: `analysis::linting`

Contains the linting engine and rule definitions.

-   **`lint_script`**: The main function for linting a script.
-   **`LintConfig`**: Configuration for the linter.
-   **`LintIssue`**: A single issue found by the linter.
-   **`LintRule`**: A trait for implementing custom linting rules.

### Module: `utils`

Contains utility functions for working with ASS data.

-   **`parse_ass_time`**: Parses an ASS timestamp into centiseconds.
-   **`format_ass_time`**: Formats centiseconds into an ASS timestamp.
-   **`parse_bgr_color`**: Parses an ASS BGR color string into RGBA.
-   **`decode_uu_data`**: Decodes UU-encoded data from `[Fonts]` and `[Graphics]` sections.

## 7. ASS v4+ vs v4++ Format Differences

`ass-core` supports both v4+ and v4++ formats with full backward compatibility. Note that v4++ format is represented by the `ScriptVersion::AssV4Plus` enum value. The key differences are:

### Style Definitions

**V4+ Format:**
```
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
```

**V4++ Format:**
```
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginT, MarginB, Encoding, RelativeTo
```

### Event Definitions

**V4+ Format:**
```
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
```

**V4++ Format:**
```
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginT, MarginB, Effect, Text
```

### New v4++ Features

-   **Separate Top/Bottom Margins**: `MarginT` and `MarginB` replace the single `MarginV` field
-   **RelativeTo Positioning**: Defines positioning context (`0`=window, `1`=video, `2`=script/default)
-   **Enhanced Karaoke**: `\kt` tag for absolute karaoke timing in centiseconds
-   **Improved Precision**: Better handling of timing and positioning values

### Example: Parsing Both Formats

```rust
use ass_core::{Script, ScriptVersion, Section, parser::ast::SectionType};

// v4+ script
let v4plus_script = r#"
[Script Info]
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&HFFFFFF,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,15,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,10,,Hello World!
"#;

// v4++ script
let v4plusplus_script = r#"
[Script Info]
ScriptType: v4.00++

[V4++ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginT, MarginB, Encoding, RelativeTo
Style: Default,Arial,20,&HFFFFFF,&H0,&H0,&H0,0,0,0,0,100,100,0,0,1,2,0,2,10,10,15,25,1,0

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginT, MarginB, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,5,5,,{\kt500}Hello v4++!
"#;

// Both parse successfully
let script_v4plus = Script::parse(v4plus_script).unwrap();
let script_v4plusplus = Script::parse(v4plusplus_script).unwrap();

// Check script versions - v4++ uses ScriptVersion::AssV4Plus enum value
assert_eq!(script_v4plus.version(), ScriptVersion::AssV4);
assert_eq!(script_v4plusplus.version(), ScriptVersion::AssV4Plus);

// Access v4++ specific fields
if let Some(Section::Styles(styles)) = script_v4plusplus.find_section(SectionType::Styles) {
    let style = &styles[0];
    assert_eq!(style.margin_t, Some("15")); // Top margin
    assert_eq!(style.margin_b, Some("25")); // Bottom margin
    assert_eq!(style.relative_to, Some("0")); // Window positioning
}
```

---

This documentation provides a starting point for using the `ass-core` crate. For more in-depth information, please refer to the source code and the specific documentation for each module, struct, and function.