# ass-core

[![Crates.io](https://img.shields.io/crates/v/ass-core.svg)](https://crates.io/crates/ass-core)
[![Documentation](https://docs.rs/ass-core/badge.svg)](https://docs.rs/ass-core)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](../../../LICENSE-MIT)

High-performance ASS (Advanced SubStation Alpha) subtitle format parser and analyzer for Rust.

## Features

- **Zero-copy parsing**: Efficient lifetime-generic AST with minimal allocations
- **Full ASS v4+ support**: Complete compatibility with libass and Aegisub
- **Advanced analysis**: Script linting, style resolution, and performance optimization
- **SIMD acceleration**: Optional SIMD-optimized parsing for maximum performance
- **no_std compatible**: Works in embedded and WASM environments
- **Streaming support**: Parse large files incrementally with bounded memory

## Performance Targets

- **Parse speed**: <5ms for typical 1KB scripts
- **Memory usage**: <1.1x input size via zero-copy design
- **Peak memory**: <10MB for large subtitle files

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
ass-core = { version = "0.1", features = ["analysis"] }
```

Basic usage:

```rust
use ass_core::parser::Script;

let script_text = r#"
[Script Info]
Title: Example
ScriptType: v4.00+

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
"#;

let script = Script::parse(script_text)?;
println!("Parsed {} sections", script.sections().len());
```

## Advanced Usage

### Script Analysis

```rust
use ass_core::analysis::ScriptAnalysis;

let analysis = ScriptAnalysis::analyze(&script)?;
println!("Found {} dialogue events", analysis.dialogue_info().len());
```

### Linting

```rust
use ass_core::analysis::linting::{lint_script, LintConfig};

let config = LintConfig::default();
let issues = lint_script(&script, &config)?;
for issue in issues {
    println!("{}: {}", issue.severity(), issue.message());
}
```

### Streaming Parser

```rust
use ass_core::parser::streaming::StreamingParser;

let mut parser = StreamingParser::new();
for chunk in file_chunks {
    parser.feed_chunk(chunk)?;
}
let result = parser.finalize()?;
```

## Feature Flags

- `std` (default): Standard library support
- `no_std`: no_std compatibility
- `analysis`: Enable script analysis and linting
- `plugins`: Extension registry support
- `simd`: SIMD-accelerated parsing
- `arena`: Arena allocation for improved performance
- `stream`: Streaming/chunked input support
- `serde`: Serialization support

## Architecture

The crate is organized into several modules:

- `parser`: Core ASS parsing with zero-copy AST
- `analysis`: Deep script analysis and optimization detection
- `linting`: Comprehensive rule-based script validation
- `utils`: Common utilities and error types

## Compatibility

- **ASS v4.00+**: Full support including advanced features
- **SSA v4**: Legacy compatibility mode
- **libass extensions**: Support for renderer-specific features
- **Aegisub compatibility**: Full support for Aegisub-specific features

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../../CONTRIBUTING.md) for guidelines.