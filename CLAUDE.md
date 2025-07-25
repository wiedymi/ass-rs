# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Test/Lint Commands
- Build: `cargo build [--release] [--features="simd,arena"]`
- Test all: `cargo test --all-features`
- Test single: `cargo test test_name`
- Test file: `cargo test --test file_name`
- Format: `cargo fmt --all`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Benchmarks: `cargo bench --features="benches"`
- WASM tests: `wasm-pack test --chrome`
- Fuzzing: `cargo +nightly fuzz run tokenizer`
- Coverage: `cargo tarpaulin --all-features`

## Code Style Guidelines
- **Safety**: No unsafe code allowed - absolutely forbidden
- **Imports**: Group by scope (std → external → internal); specific imports for frequent items
- **Formatting**: 4-space indentation; standard Rust formatting; rustfmt defaults
- **Types**: Use zero-copy spans (`&str`) for performance; custom Result types; feature-gated optimizations
- **Naming**: CamelCase for types; snake_case for functions/files; SCREAMING_SNAKE_CASE for constants
- **Error Handling**: Use `thiserror` for enums (no `anyhow`); prefer Result over panics
- **Documentation**: Use only Rustdoc (`///` for public, `//!` for modules); no inline comments; example-heavy
- **Testing**: >90% test coverage required; fuzz hot paths; WASM tests mandatory
- **Dependencies**: Minimal deps (<50KB); pin versions in workspace; feature-gate heavy ones
- **Performance**: <5ms/operation, <1.1x input memory; use zero-copy, arenas, SIMD where beneficial
- **Modularity**: Submodules per concern; traits for extensibility; file size <200 LOC
- **Features**: Consistent across crates; default minimal; gate extras; maintain no_std compatibility
- **Workarounds**: Never use workarounds, bypass, or skip logic; never use `allow(clippy)` to fix something

## Architecture Overview

### Project Structure
ASS-RS is designed as a modular ASS (Advanced SubStation Alpha) subtitle parser surpassing libass in performance and safety. The current workspace contains:
- **`crates/ass-core`**: Zero-copy parser with trait-based plugin system
- **Future crates**: renderer, editor, CLI, WASM, benchmarks (planned)

### Core Design Principles
- **Zero-Copy**: Uses `&'a str` spans throughout AST (target: ~1.1x input memory)
- **Plugin System**: `TagHandler` and `SectionProcessor` traits with runtime `ExtensionRegistry`
- **Thread Safety**: Immutable `Script` design with `Send + Sync`
- **Performance Targets**: <5ms parsing, <2ms incremental updates
- **Safety First**: Zero unsafe code, comprehensive error handling with `thiserror`

### Key Features
- **Format Support**: ASS v4+, SSA v4, libass 0.17.4+ compatibility
- **SIMD Acceleration**: Feature-gated with `simd` and `simd-full` features
- **Arena Allocation**: Optional `bumpalo` integration for reduced allocations
- **Analysis Engine**: Linting, style resolution, performance analysis
- **Incremental Parsing**: Editor-friendly partial re-parsing

### Feature Flags
- **Default**: `std`, `analysis`, `plugins`
- **Performance**: `simd`, `simd-full`, `arena`
- **Platform**: `nostd` for embedded/WASM
- **Development**: `serde`, `benches`, `stream`

## Post-Iteration Checklist
Always run: fmt → clippy → test → bench → coverage → build release