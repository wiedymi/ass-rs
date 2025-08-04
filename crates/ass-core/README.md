# ass-core

[![Crates.io](https://img.shields.io/crates/v/ass-core.svg)](https://crates.io/crates/ass-core)
[![Documentation](https://docs.rs/ass-core/badge.svg)](https://docs.rs/ass-core)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

High-performance ASS (Advanced SubStation Alpha) subtitle format parser and analyzer for Rust.

## Features

- **Zero-copy parsing**: Efficient lifetime-generic AST with minimal allocations
- **Full ASS v4+ support**: Complete compatibility with libass and Aegisub
- **Advanced analysis**: Script linting, style resolution, and performance optimization
- **SIMD acceleration**: Optional SIMD-optimized parsing for maximum performance
- **nostd compatible**: Works in embedded and WASM environments
- **Streaming support**: Parse large files incrementally with bounded memory

## Performance Targets

- **Parse speed**: <5ms for typical 1KB scripts
- **Memory usage**: <1.1x input size via zero-copy design
- **Peak memory**: <10MB for large subtitle files
- **Incremental parsing**: <5ms for typical edit operations (infrastructure ready)

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
- `nostd`: nostd compatibility
- `analysis`: Enable script analysis and linting
- `plugins`: Extension registry support
- `simd`: SIMD-accelerated parsing
- `arena`: Arena allocation for improved performance
- `stream`: Streaming/chunked input support
- `serde`: Serialization support
- `benches`: Enable benchmarking infrastructure

## Architecture

The crate is organized into several modules:

- `parser`: Core ASS parsing with zero-copy AST
- `analysis`: Deep script analysis and optimization detection
- `linting`: Comprehensive rule-based script validation
- `utils`: Common utilities and error types

## Benchmarking

Performance benchmarks can be run with:

```bash
# Run all benchmarks
cargo bench --features=benches

# Run specific benchmarks
cargo bench --features=benches parser_benchmarks
cargo bench --features=benches incremental_benchmarks
```

The benchmarking suite includes:
- **Parser benchmarks**: Full parsing performance across complexity levels
- **Incremental benchmarks**: Infrastructure for validating incremental parsing
- **Real-world scenarios**: Testing against realistic subtitle patterns (anime, movies, karaoke, signs, educational content)
- **Large-scale anime**: Benchmarks for 30-50MB+ files with 10k-100k events (BD releases, complex OVAs)
- **Editor simulation**: Realistic editing patterns (typing, backspace, copy-paste)
- **Memory benchmarks**: Zero-copy efficiency and memory ratio validation
- **Memory profiling**: Simple tool to measure actual memory usage

### Memory Profiling

To analyze memory usage without external tools:

```bash
# Run memory benchmarks
cargo bench --features=benches memory_benchmarks

# Run simple memory profiler
cargo run --features=benches --bin memory-profile
```

The memory profiler shows:
- Input size vs actual memory usage
- Memory ratio (target: <1.1x)
- Parse time and section counts
- Pass/fail status for memory targets

### Benchmark Configuration

For faster benchmark runs (e.g., in CI):

```bash
# Quick mode - fewer samples, shorter measurement time
QUICK_BENCH=1 cargo bench --features=benches

# Run specific benchmark group
cargo bench --features=benches incremental_parsing

# Increase time limit for complex benchmarks
cargo bench --features=benches -- --measurement-time 15
```

If you see warnings about unable to complete samples:
- This is normal for complex benchmarks
- The results are still valid
- Use `QUICK_BENCH=1` for faster iteration
- Or increase measurement time as suggested

## Compatibility

- **ASS v4.00+**: Full support including advanced features
- **SSA v4**: Legacy compatibility mode
- **libass extensions**: Support for renderer-specific features
- **Aegisub compatibility**: Full support for Aegisub-specific features

## License

Licensed under the [MIT license](../../LICENSE).

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../../CONTRIBUTING.md) for guidelines.
