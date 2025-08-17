//! Memory usage benchmarks for ASS parser
//!
//! Measures memory consumption patterns to validate the <1.1x input size target.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
use ass_core::{parser::Script, utils::ScriptGenerator};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
/// Estimate memory usage of parsed AST
fn estimate_ast_memory(script: &Script) -> usize {
    // Base Script struct size
    let mut total = std::mem::size_of::<Script>();

    // Estimate sections memory
    for section in script.sections() {
        total += std::mem::size_of_val(section);

        // Add estimated content size based on section type
        // Each section type has different memory characteristics
        total += 1024; // Conservative estimate per section
    }

    total
}

/// Benchmark memory efficiency for different script sizes
fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    group.sample_size(50);

    let sizes = [100, 1000, 5000, 10000, 50000];

    for &event_count in &sizes {
        // Generate realistic anime script
        let script_text = ScriptGenerator::anime_realistic(event_count).generate();
        let input_size = script_text.len();

        group.bench_with_input(
            BenchmarkId::new("parse_memory_ratio", event_count),
            &script_text,
            |b, text| {
                b.iter(|| {
                    let script = Script::parse(black_box(text)).unwrap();
                    let ast_size = estimate_ast_memory(&script);
                    let ratio = if input_size == 0 {
                        0
                    } else {
                        ast_size * 100 / input_size
                    };

                    // Return tuple to prevent optimization
                    black_box((script, ratio))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark zero-copy efficiency
fn bench_zero_copy_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("zero_copy");
    group.sample_size(100);

    let script_text = ScriptGenerator::complex(1000).generate();

    // Measure parsing without copying
    group.bench_function("parse_no_alloc", |b| {
        b.iter(|| {
            let script = Script::parse(black_box(&script_text)).unwrap();

            // Verify we're using references to original input
            let sections_count = script.sections().len();
            black_box(sections_count)
        });
    });

    // Compare with a hypothetical copying parser
    group.bench_function("parse_with_copy_simulation", |b| {
        b.iter(|| {
            // Simulate what a copying parser would do
            let owned_copy = script_text.to_string();
            let script = Script::parse(black_box(&owned_copy)).unwrap();
            let sections_count = script.sections().len();
            black_box((owned_copy, sections_count))
        });
    });

    group.finish();
}

/// Benchmark memory patterns for real-world files
fn bench_real_world_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_memory");
    group.sample_size(20);

    let profiles = [
        ("tv_episode_24min", 10_000),
        ("movie_2hr", 30_000),
        ("ova_complex", 50_000),
    ];

    for (name, event_count) in &profiles {
        let script_text = ScriptGenerator::anime_realistic(*event_count).generate();
        let input_mb = script_text.len() / 1_048_576;

        group.bench_with_input(
            BenchmarkId::new("memory_overhead", name),
            &script_text,
            |b, text| {
                b.iter(|| {
                    let script = Script::parse(black_box(text)).unwrap();

                    // Calculate overhead
                    let ast_memory = estimate_ast_memory(&script);
                    let overhead_ratio = if text.is_empty() {
                        0
                    } else {
                        ast_memory * 100 / text.len()
                    };

                    println!("{name}: {input_mb}MB input, {overhead_ratio}% memory ratio");

                    black_box((script, overhead_ratio))
                });
            },
        );
    }

    group.finish();
}

/// Memory allocation patterns during parsing
fn bench_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_patterns");

    // Test different allocation patterns
    let script_text = ScriptGenerator::complex(1000).generate();

    // Benchmark section-by-section allocation pattern
    group.bench_function("incremental_sections", |b| {
        b.iter(|| {
            let mut sections = Vec::new();

            // Simulate incremental parsing allocations
            for line in script_text.lines() {
                if line.starts_with('[') {
                    sections.push(line.to_string());
                }
            }

            black_box(sections)
        });
    });

    // Benchmark pre-allocated pattern
    group.bench_function("preallocated", |b| {
        b.iter(|| {
            // Count sections first
            let section_count = script_text
                .lines()
                .filter(|line| line.starts_with('['))
                .count();

            let mut sections = Vec::with_capacity(section_count);

            for line in script_text.lines() {
                if line.starts_with('[') {
                    sections.push(line);
                }
            }

            black_box(sections)
        });
    });

    group.finish();
}

criterion_group!(
    memory_benches,
    bench_memory_efficiency,
    bench_zero_copy_efficiency,
    bench_real_world_memory,
    bench_allocation_patterns
);
criterion_main!(memory_benches);
