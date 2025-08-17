# ASS-RS

[![Crates.io](https://img.shields.io/crates/v/ass-core.svg)](https://crates.io/crates/ass-core)
[![Documentation](https://docs.rs/ass-core/badge.svg)](https://docs.rs/ass-core)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/wiedymi/ass-rs/workflows/CI/badge.svg)](https://github.com/wiedymi/ass-rs/actions)

A modular, high-performance Rust implementation of the ASS (Advanced SubStation Alpha) subtitle format.

## 🚀 Key Advantages

- **Memory Safety**: 100% safe Rust with zero unsafe code
- **Modularity**: Trait-based plugin system vs. monolithic C codebase
- **Performance**: <5ms parsing with zero-copy spans, SIMD optimizations
- **Thread Safety**: Immutable `Script` design with `Send + Sync`
- **Extensibility**: Runtime plugin registry for custom tags/sections
- **Modern Standards**: Full libass 0.17.4+ compatibility with Unicode wrapping
- **Cross-Platform**: Native WASM support, nostd compatibility

## 📖 Specifications

This implementation adheres to official ASS/SSA specifications:

- **[TCax ASS Specification](http://www.tcax.org/docs/ass-specs.htm)** - Official ASS format documentation
- **[Aegisub ASS Tags](https://aegisub.org/docs/latest/ass_tags/)** -  Tag reference
- **[libass ASS Guide](https://github.com/libass/libass/wiki/ASS-File-Format-Guide)** - Extensions and implementation notes
- **[SSA v4.00 Original](http://www.eswat.demon.co.uk/)** - Legacy SSA compatibility

## 🏗️ Architecture

The ASS-RS ecosystem consists of modular, interoperable crates:

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│  ass-core   │────│ ass-renderer │    │ ass-editor  │
│   (parser)  │    │  (rendering) │    │ (editing)   │
└─────────────┘    └──────────────┘    └─────────────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
              ┌─────────────────────────┐
              │      ass-wasm           │
              │   (web bindings)        │
              └─────────────────────────┘
```

- **`ass-core`**: Zero-copy parsing, analysis, and AST manipulation
- **`ass-renderer`**: Multiple rendering backends (software, GPU, web)
- **`ass-editor`**: Interactive editing APIs with incremental updates
- **`ass-cli`**: Command-line tools for processing and conversion
- **`ass-wasm`**: WebAssembly bindings for browser integration
- **`ass-benchmarks`**: Performance testing and libass comparisons

## ⚡ Performance Targets

- **Parsing**: <5ms for typical scripts (1KB-10KB)
- **Incremental Updates**: <2ms for single-event modifications
- **Memory Usage**: ~1.1x input size via zero-copy spans
- **SIMD Acceleration**: 20-30% faster with portable SIMD
- **Streaming**: <10ms/MB for chunked inputs


## 🎯 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
ass-core = "0.1.1"
```

Basic usage:

```rust
use ass_core::Script;

let script_text = r#"
[Script Info]
Title: Example Karaoke
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\k50}Ka{\k50}ra{\k100}oke
"#;

// Zero-copy parsing
let script = Script::parse(script_text)?;

// Analysis and linting
let analysis = script.analyze()?;
for issue in analysis.lint_issues() {
    println!("Warning: {}", issue);
}

// Access parsed data with zero-copy spans
for section in script.sections() {
    match section {
        Section::Events(events) => {
            for event in events.dialogues() {
                println!("Text: {}", event.text());
                println!("Start: {}", event.start_time());
            }
        }
        _ => {}
    }
}

// For complex nested transforms, use the fixed parser
use ass_core::analysis::events::parse_override_block_fixed;

let mut tags = Vec::new();
let mut diagnostics = Vec::new();
let complex_transform = r"\t(0,1000,\fs50\1c&HFF0000&)";
parse_override_block_fixed(complex_transform, 0, &mut tags, &mut diagnostics);
```

## 🔧 Features

Enable features as needed:

```toml
[dependencies]
ass-core = { version = "0.1", features = ["simd", "arena", "serde"] }
```

- **`analysis`** (default): Deep analysis and linting capabilities
- **`plugins`** (default): Extension registry for custom handlers
- **`simd`**: SIMD-accelerated parsing and processing
- **`arena`**: Arena allocation for reduced memory overhead
- **`nostd`**: Embedded and WASM-optimized builds
- **`stream`**: Chunked processing for large files
- **`serde`**: JSON serialization support

## 🧪 Testing and Benchmarks

Run the full test suite:

```bash
# Unit and integration tests
cargo test --all-features

# Performance benchmarks vs libass
cargo bench --features="benches"

# WASM compatibility
wasm-pack test --chrome

# Fuzzing (requires nightly)
cargo +nightly fuzz run tokenizer
```

### Development Setup

```bash
# Clone repository
git clone https://github.com/wiedymi/ass-rs.git
cd ass-rs

# Run tests
cargo test --all-features

# Check code quality
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings

# Run benchmarks
cargo bench
```

### Code Quality Standards

- **No unsafe code** - 100% memory safe Rust
- **>90% test coverage** - Comprehensive testing required
- **Strict linting** - All Clippy warnings must be resolved
- **Performance validation** - No >10% regressions allowed
- **Documentation** - All public APIs documented with examples

## 📋 Roadmap

### v0.1.0 - Core Foundation ✅
- [x] Zero-copy ASS parser
- [x] Comprehensive AST with span support
- [x] Plugin system architecture
- [x] SIMD-optimized tokenization
- [x] Full spec compliance testing

### v0.2.0 - Rendering Pipeline (In Progress)
- [ ] Software rasterizer backend
- [ ] WebGPU rendering support
- [ ] Advanced typography (shaping, kerning)
- [ ] Animation timeline evaluation

### v0.3.0 - Editor Integration ✅
- [x] Incremental parsing for editors (<1ms edits, <5ms re-parses)
- [x] Real-time style preview and validation
- [x] Multi-document session management  
- [x] Undo/redo with efficient deltas and arena pooling

### v1.0.0 - Production Ready
- [ ] Complete libass API parity
- [ ] Browser runtime optimization
- [ ] Production battle-testing
- [ ] Comprehensive documentation

## 📄 License

Licensed under the [MIT license](LICENSE).
