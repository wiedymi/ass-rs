//! Comprehensive incremental parsing benchmarks
//!
//! Tests incremental parsing performance against project targets:
//! - <5ms incremental parsing for typical operations
//! - <1.1x input memory ratio
//! - Correctness validation vs full reparse
//!
//! Includes real-world editing scenarios and stress testing.

#[cfg(not(feature = "std"))]
extern crate alloc;

use criterion::{criterion_group, criterion_main};

#[path = "incremental_benchmarks/helpers.rs"]
mod helpers;

#[path = "incremental_benchmarks/simulations.rs"]
mod simulations;

#[path = "incremental_benchmarks/parsing.rs"]
mod parsing;

#[path = "incremental_benchmarks/editing.rs"]
mod editing;

#[path = "incremental_benchmarks/real_world.rs"]
mod real_world;

#[path = "incremental_benchmarks/large_scale.rs"]
mod large_scale;

criterion_group!(
    incremental_benches,
    parsing::bench_incremental_parsing,
    parsing::bench_incremental_vs_full,
    editing::bench_editor_simulation,
    editing::bench_memory_usage,
    real_world::bench_real_world_files,
    large_scale::bench_large_scale_anime,
    real_world::bench_stress_scenarios
);
criterion_main!(incremental_benches);
