//! Parsing and streaming benchmark functions for `parser_benchmarks`.
//!
//! Measures basic parse throughput across script sizes/complexities and a
//! placeholder streaming scenario.

use ass_core::{parser::Script, utils::ScriptGenerator};
use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use std::hint::black_box as std_black_box;

/// Benchmark basic parsing performance
pub fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    // Test different script sizes
    let sizes = [10, 100, 1000, 5000];

    for &size in &sizes {
        let simple_script = ScriptGenerator::simple(size).generate();
        let moderate_script = ScriptGenerator::moderate(size).generate();
        let complex_script = ScriptGenerator::complex(size).generate();
        let extreme_script = ScriptGenerator::extreme(size).generate();

        group.throughput(Throughput::Bytes(simple_script.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("simple", size),
            &simple_script,
            |b, script| {
                b.iter(|| {
                    let result = Script::parse(black_box(script));
                    std_black_box(result)
                });
            },
        );

        group.throughput(Throughput::Bytes(moderate_script.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("moderate", size),
            &moderate_script,
            |b, script| {
                b.iter(|| {
                    let result = Script::parse(black_box(script));
                    std_black_box(result)
                });
            },
        );

        group.throughput(Throughput::Bytes(complex_script.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("complex", size),
            &complex_script,
            |b, script| {
                b.iter(|| {
                    let parsed = Script::parse(black_box(script));
                    black_box(parsed)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("extreme", size),
            &extreme_script,
            |b, script| {
                b.iter(|| {
                    let parsed = Script::parse(black_box(script));
                    black_box(parsed)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark streaming parser performance (placeholder - requires stream feature)
pub fn bench_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming");

    // Placeholder benchmark - streaming parser requires 'stream' feature
    let script = ScriptGenerator::moderate(100).generate();
    group.bench_function("streaming_placeholder", |b| {
        b.iter(|| {
            // Simulate streaming by parsing in chunks
            let result = Script::parse(black_box(&script));
            std_black_box(result)
        });
    });

    group.finish();
}
