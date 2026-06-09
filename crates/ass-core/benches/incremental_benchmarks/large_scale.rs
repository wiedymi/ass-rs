//! Large-scale anime subtitle benchmark group (30-50MB range).
//!
//! Generates representative large releases and measures incremental edits,
//! full parses, and memory ratios against those files.

#[cfg(not(feature = "std"))]
use alloc::format;
use ass_core::{parser::Script, utils::ScriptGenerator};
use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use std::{hint::black_box as std_black_box, time::Instant};

use crate::helpers::{apply_incremental_change, create_section_change};

/// Benchmark large-scale anime subtitle files (30-50MB range)
pub fn bench_large_scale_anime(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_scale_anime");
    group.sample_size(10); // Fewer samples due to large file sizes
    group.measurement_time(std::time::Duration::from_secs(30)); // More time for large files

    // Test different large file sizes representative of real anime releases
    let large_sizes = [
        ("24min_episode", 10_000), // ~5-10MB typical TV episode
        ("movie_2hr", 30_000),     // ~15-30MB typical movie
        ("ova_complex", 50_000),   // ~25-50MB OVA with heavy effects
        ("bdmv_full", 100_000),    // ~50MB+ BD release with all tracks
    ];

    for (profile_name, event_count) in &large_sizes {
        // Generate large anime subtitle file
        let script_text = ScriptGenerator::anime_realistic(*event_count).generate();
        let file_size_mb = script_text.len() / 1_048_576;

        println!("Generated {profile_name}: {file_size_mb}MB ({event_count} events)");

        // Test incremental changes on large files
        let small_change = create_section_change(&script_text, 50);
        let medium_change = create_section_change(&script_text, 500);
        let large_change = create_section_change(&script_text, 5000);

        // Benchmark small edit (typical dialogue correction)
        group.throughput(Throughput::Bytes(script_text.len() as u64));
        group.bench_with_input(
            BenchmarkId::new(format!("{profile_name}_small_edit"), event_count),
            &(script_text.as_str(), &small_change),
            |b, (text, change)| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = apply_incremental_change(black_box(text), black_box(change));
                    let duration = start.elapsed();
                    std_black_box((result, duration))
                });
            },
        );

        // Benchmark medium edit (scene timing adjustment)
        group.bench_with_input(
            BenchmarkId::new(format!("{profile_name}_medium_edit"), event_count),
            &(script_text.as_str(), &medium_change),
            |b, (text, change)| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = apply_incremental_change(black_box(text), black_box(change));
                    let duration = start.elapsed();
                    std_black_box((result, duration))
                });
            },
        );

        // Benchmark large edit (karaoke template insertion)
        group.bench_with_input(
            BenchmarkId::new(format!("{profile_name}_large_edit"), event_count),
            &(script_text.as_str(), &large_change),
            |b, (text, change)| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = apply_incremental_change(black_box(text), black_box(change));
                    let duration = start.elapsed();
                    std_black_box((result, duration))
                });
            },
        );

        // Benchmark full parse for comparison
        group.bench_with_input(
            BenchmarkId::new(format!("{profile_name}_full_parse"), event_count),
            &script_text,
            |b, text| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = Script::parse(black_box(text));
                    let duration = start.elapsed();
                    std_black_box((result, duration))
                });
            },
        );

        // Memory efficiency check
        group.bench_with_input(
            BenchmarkId::new(format!("{profile_name}_memory_ratio"), event_count),
            &(script_text.as_str(), &medium_change),
            |b, (text, change)| {
                b.iter(|| {
                    let input_size = text.len();
                    let start = Instant::now();
                    let result = apply_incremental_change(black_box(text), black_box(change));
                    let duration = start.elapsed();

                    // Estimate memory usage (in real implementation would use actual measurements)
                    let estimated_memory = input_size + change.new_text.len();
                    let memory_ratio = (estimated_memory * 100)
                        .checked_div(input_size)
                        .unwrap_or(0);

                    std_black_box((result, duration, memory_ratio))
                });
            },
        );
    }

    group.finish();
}
