//! Comprehensive benchmarks for ASS parsing and analysis
//!
//! Tests parsing performance against project targets:
//! - <5ms for typical 1KB scripts
//! - <10MB peak memory usage
//! - <1.1x input memory ratio
//!
//! Generates synthetic ASS data programmatically to test various
//! complexity scenarios without external file dependencies.

use criterion::{criterion_group, criterion_main};

#[path = "parser_benchmarks/parsing.rs"]
mod parsing;

#[path = "parser_benchmarks/analysis.rs"]
mod analysis;

#[path = "parser_benchmarks/memory.rs"]
mod memory;

#[path = "parser_benchmarks/styles.rs"]
mod styles;

criterion_group!(
    benches,
    parsing::bench_parsing,
    parsing::bench_streaming,
    analysis::bench_text_analysis,
    analysis::bench_dialogue_analysis,
    analysis::bench_linting,
    memory::bench_memory_usage,
    memory::bench_uu_decoding,
    styles::bench_style_resolution,
    styles::bench_overlap_detection
);
criterion_main!(benches);
