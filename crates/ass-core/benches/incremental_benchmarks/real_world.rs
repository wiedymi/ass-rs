//! Realistic-profile and stress-scenario benchmark groups.
//!
//! Exercises representative subtitle profiles and pathological edits such as
//! section-boundary shifts, malformed recovery, and very large changes.

use ass_core::{parser::Script, utils::ScriptGenerator};
use criterion::{black_box, BenchmarkId, Criterion};
use std::{hint::black_box as std_black_box, time::Instant};

use crate::helpers::{
    apply_incremental_change, create_large_change, create_malformed_change,
    create_section_boundary_change, create_section_change,
};

/// Benchmark real-world files with realistic complexity profiles
pub fn bench_real_world_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_files");
    group.sample_size(100);

    let size = 1000;
    let realistic_profiles = [
        ("anime", ScriptGenerator::anime_realistic(size).generate()),
        ("movie", ScriptGenerator::movie_realistic(size).generate()),
        (
            "karaoke",
            ScriptGenerator::karaoke_realistic(size).generate(),
        ),
        ("sign", ScriptGenerator::sign_realistic(size).generate()),
        (
            "educational",
            ScriptGenerator::educational_realistic(size).generate(),
        ),
    ];

    for (profile_name, script_text) in &realistic_profiles {
        let change = create_section_change(script_text, 100);

        group.bench_with_input(
            BenchmarkId::new("realistic_incremental", profile_name),
            &(script_text.as_str(), &change),
            |b, (text, change)| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = apply_incremental_change(black_box(text), black_box(change));
                    let duration = start.elapsed();
                    std_black_box((result, duration))
                });
            },
        );

        // Also benchmark full parse for comparison
        group.bench_with_input(
            BenchmarkId::new("realistic_full_parse", profile_name),
            script_text,
            |b, text| {
                b.iter(|| {
                    let start = Instant::now();
                    let result = Script::parse(black_box(text));
                    let duration = start.elapsed();
                    std_black_box((result, duration))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark stress scenarios and edge cases
pub fn bench_stress_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_scenarios");
    group.sample_size(50);
    group.measurement_time(std::time::Duration::from_secs(10)); // More time for stress tests

    // Test section boundary changes
    let script_text = ScriptGenerator::complex(1000).generate();
    let boundary_change = create_section_boundary_change(&script_text);

    group.bench_with_input(
        BenchmarkId::new("section_boundary", "change"),
        &(script_text.as_str(), &boundary_change),
        |b, (text, change)| {
            b.iter(|| {
                let start = Instant::now();
                let result = apply_incremental_change(black_box(text), black_box(change));
                let duration = start.elapsed();
                std_black_box((result, duration))
            });
        },
    );

    // Test malformed input recovery
    let malformed_change = create_malformed_change(&script_text);
    group.bench_with_input(
        BenchmarkId::new("malformed_recovery", "change"),
        &(script_text.as_str(), &malformed_change),
        |b, (text, change)| {
            b.iter(|| {
                let start = Instant::now();
                let result = apply_incremental_change(black_box(text), black_box(change));
                let duration = start.elapsed();
                std_black_box((result, duration))
            });
        },
    );

    // Test extremely large single change
    let large_change = create_large_change(&script_text, 10000);
    group.bench_with_input(
        BenchmarkId::new("large_single_change", "10k_chars"),
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

    group.finish();
}
