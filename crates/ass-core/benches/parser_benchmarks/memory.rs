//! Memory usage and UU-decoding benchmark functions for `parser_benchmarks`.

use ass_core::{
    analysis::events::dialogue_info::DialogueInfo,
    parser::{Script, Section},
    utils::ScriptGenerator,
};
use criterion::{black_box, BenchmarkId, Criterion};
use std::hint::black_box as std_black_box;

/// Benchmark memory usage patterns
pub fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    let sizes = [100, 1000, 5000, 10000];

    for &size in &sizes {
        let complex_script = ScriptGenerator::complex(size).generate();
        let extreme_script = ScriptGenerator::extreme(size).generate();

        group.bench_with_input(
            BenchmarkId::new("parse_and_analyze_complex", size),
            &complex_script,
            |b, script| {
                b.iter(|| {
                    // Parse script
                    let parsed = Script::parse(black_box(script)).unwrap();

                    // Analyze all events
                    if let Some(Section::Events(events)) = parsed
                        .sections()
                        .iter()
                        .find(|s| matches!(s, Section::Events(_)))
                    {
                        for event in events {
                            let dialogue_info = DialogueInfo::analyze(event);
                            let _ = std_black_box(dialogue_info);
                        }
                    }

                    std_black_box(parsed)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parse_and_analyze_extreme", size),
            &extreme_script,
            |b, script| {
                b.iter(|| {
                    // Parse script
                    let parsed = Script::parse(black_box(script)).unwrap();

                    // Analyze all events
                    if let Some(Section::Events(events)) = parsed
                        .sections()
                        .iter()
                        .find(|s| matches!(s, Section::Events(_)))
                    {
                        for event in events {
                            let dialogue_info = DialogueInfo::analyze(event);
                            let _ = std_black_box(dialogue_info);
                        }
                    }

                    std_black_box(parsed)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark UU-decoding performance for embedded media
pub fn bench_uu_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("uu_decoding");
    group.sample_size(1000);

    // Generate test UU-encoded data of various sizes
    let small_data = [
        "#0V%T", // "Cat" - 3 bytes
        "`",
    ];

    let medium_data = [
        "M5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O", // ~50 bytes
        "M5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O",
        "M5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O",
        "`",
    ];

    let large_data: Vec<&str> = (0..100)
        .map(|_| "M9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F")
        .chain(std::iter::once("`"))
        .collect();

    group.bench_function("small_uu_decode", |b| {
        b.iter(|| black_box(ass_core::utils::decode_uu_data(small_data.iter().copied())).unwrap());
    });

    group.bench_function("medium_uu_decode", |b| {
        b.iter(|| black_box(ass_core::utils::decode_uu_data(medium_data.iter().copied())).unwrap());
    });

    group.bench_function("large_uu_decode", |b| {
        b.iter(|| black_box(ass_core::utils::decode_uu_data(large_data.iter().copied())).unwrap());
    });

    group.finish();
}
