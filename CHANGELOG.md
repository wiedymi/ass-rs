# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - ReleaseDate

### Added
- Initial release of ass-core and ass-editor crates
- Zero-copy ASS/SSA subtitle parsing with full v4+ support
- Plugin system for extensible tag and section handling
- Incremental parsing support for editor integration
- Thread-safe immutable Script design
- Feature-gated SIMD acceleration for performance
- no_std support with minimal feature set
- Comprehensive error handling and validation
- Rich analysis and linting capabilities
- Full editor functionality with undo/redo support
- Format conversion support (SRT, WebVTT)
- FST-based search indexing for large documents
- ass-core: JSON export for the AST via the `serde` feature (`Serialize` only;
  borrowed zero-copy deserialization is intentionally unsupported)
- ass-core: Unicode line-break opportunities (UAX #14) via the `unicode-wrap`
  feature, exposed as `analysis::events::unicode_wrap` (libass
  `ASS_FEATURE_WRAP_UNICODE` parity, backed by the `unicode-linebreak` crate)

### Changed
- ass-core: `serde` is now no_std-aware (`alloc`-only by default; `std`
  propagates through the crate's `std` feature)

### Fixed
- ass-editor: no_std document ID generation now uses `AtomicU32` instead of
  `unsafe static mut`, restoring the project's zero-unsafe guarantee

### Removed
- ass-editor: dead `extensions/mod_backup.rs` scaffolding file

### ass-core Features
- Complete ASS v4+ and SSA v4 format support
- libass 0.17.4+ compatibility
- Zero-allocation parsing with lifetime-based AST
- Streaming/chunked input support
- Comprehensive test coverage (>90%)
- Performance: <5ms parsing, <1.1x memory overhead

### ass-editor Features
- Rope-based text editing for efficiency
- Arena-allocated history for performance
- Plugin system with syntax highlighting
- Multi-threaded and async support
- Import/export for multiple subtitle formats
- Incremental parsing integration

## Version History

<!-- Releases will be added here -->

[Unreleased]: https://github.com/wiedymi/ass-rs/compare/v0.1.0...HEAD